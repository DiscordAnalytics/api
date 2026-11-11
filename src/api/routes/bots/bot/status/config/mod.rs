use actix_web::web::{Data, Json, Path};
use apistos::{
    api_operation,
    web::{ServiceConfig, patch},
};

use crate::{
    api::middleware::Authenticated,
    domain::{
        error::{ApiError, ApiResult},
        models::BotDeploymentMode,
    },
    openapi::schemas::{BotStatusConfigResponse, UpdateBotStatusConfigRequest},
    repository::Repositories,
    utils::{discord::Snowflake, logger::LogCode},
};

#[api_operation(
    summary = "Update bot status config",
    description = "Update the status config for a bot",
    tag = "Status"
)]
async fn update_config(
    auth: Authenticated,
    repos: Data<Repositories>,
    path: Path<String>,
    Json(payload): Json<UpdateBotStatusConfigRequest>,
) -> ApiResult<Json<BotStatusConfigResponse>> {
    let bot_id = Snowflake::try_from(path.into_inner())?.into_inner();

    info!(
        code = %LogCode::Status,
        bot_id = %bot_id,
        "Attempting to update status config for bot"
    );

    let bot = repos.bots.find_by_id(&bot_id).await?.ok_or_else(|| {
        info!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            "Bot not found for status config update",
        );
        ApiError::NotFound(format!("Bot with ID {} not found", bot_id))
    })?;

    let ctx = &auth;

    if ctx.is_admin() {
        info!(
            code = %LogCode::AdminAction,
            bot_id = %bot_id,
            "Admin access granted to update bot status config",
        );
    } else if ctx.is_bot() {
        if ctx.token.as_deref() != Some(&bot.token) {
            warn!(
                code = %LogCode::Forbidden,
                bot_id = %bot_id,
                auth_type = ?ctx.auth_type,
                "Unauthorized bot status config update attempt",
            );
            return Err(ApiError::Forbidden);
        }
    } else {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            auth_type = ?ctx.auth_type,
            "Unauthorized bot status config update attempt",
        );
        return Err(ApiError::Forbidden);
    }

    if bot.suspended && !ctx.is_admin() {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Access denied for suspended bot update",
        );
        return Err(ApiError::BotSuspended);
    }

    let current_status = repos.status_current.find_by_bot_id(&bot_id).await?;

    let is_sharded = payload.deployment_mode == BotDeploymentMode::Sharded;

    let total_shards = if is_sharded { payload.total_shards } else { 1 };

    if is_sharded && total_shards == 0 {
        return Err(ApiError::BadRequest(
            "A sharded deployment must have at least one shard".into(),
        ));
    }

    let updated_at = match current_status {
        None => {
            info!(
                code = %LogCode::Status,
                bot_id = %bot_id,
                "Creating initial bot status document",
            );

            repos
                .status_current
                .create_status(&bot_id, payload.deployment_mode, total_shards)
                .await?
        }
        Some(current) => {
            let config_changed = current.deployment_mode != payload.deployment_mode
                || current.total_shards != total_shards;

            if config_changed {
                info!(
                    code = %LogCode::Status,
                    bot_id = %bot_id,
                    "Updating bot status deployment configuration",
                );

                repos
                    .status_current
                    .update_status_config(&bot_id, payload.deployment_mode, total_shards)
                    .await?
            } else {
                current.updated_at
            }
        }
    };

    Ok(Json(BotStatusConfigResponse {
        bot_id: bot_id.to_string(),
        deployment_mode: payload.deployment_mode,
        total_shards,
        updated_at: updated_at.to_chrono(),
    }))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.route("/config", patch().to(update_config));
}
