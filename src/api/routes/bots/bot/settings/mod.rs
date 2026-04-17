use actix_web::web::{Data, Json, Path};
use apistos::{
    api_operation,
    web::{ServiceConfig, patch},
};
use tracing::{info, warn};

use crate::{
    api::middleware::Authenticated,
    domain::error::{ApiError, ApiResult},
    openapi::schemas::{BotSettingsPayload, MessageResponse},
    repository::{BotUpdate, Repositories},
    utils::{discord::Snowflake, logger::LogCode},
};

#[api_operation(
    summary = "Update bot settings",
    description = "Update the settings of a bot. Only the owner of the bot or team members with the appropriate permissions can perform this action.",
    tag = "Bots"
)]
async fn update_settings(
    auth: Authenticated,
    repos: Data<Repositories>,
    body: Json<BotSettingsPayload>,
    id: Path<String>,
) -> ApiResult<Json<MessageResponse>> {
    let bot_id = Snowflake::try_from(id.into_inner())?.into_inner();

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Attempting to update bot settings",
    );

    let bot = repos.bots.find_by_id(&bot_id).await?.ok_or_else(|| {
        info!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            "Bot not found",
        );
        ApiError::NotFound(format!("Bot with ID {} not found", bot_id))
    })?;

    let ctx = &auth;

    if ctx.is_admin() {
        info!(
            code = %LogCode::AdminAction,
            bot_id = %bot_id,
            "User is admin, proceeding with settings update",
        );
    } else if ctx.is_bot() {
        if ctx.token.as_deref() != Some(&bot.token) {
            warn!(
                code = %LogCode::Forbidden,
                bot_id = %bot_id,
                "Bot attempted to update settings of a different bot",
            );
            return Err(ApiError::Forbidden);
        }
    } else if ctx.is_user() {
        let user_id = ctx.user_id.as_deref().ok_or(ApiError::Unauthorized)?;
        if !bot.is_owner(user_id) {
            warn!(
                code = %LogCode::Forbidden,
                bot_id = %bot_id,
                user_id = %user_id,
                "User does not have access to bot details",
            );
            return Err(ApiError::Forbidden);
        }
    } else {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Access denied for bot details",
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

    let body = body.into_inner();

    let mut update = BotUpdate::default()
        .with_advanced_stats(body.advanced_stats)
        .with_webhook_url(body.webhook_url);

    if auth.is_admin() {
        update = update
            .with_custom_events_limit(body.custom_events_limit)
            .with_goals_limit(body.goals_limit)
            .with_teammates_limit(body.teammates_limit);
    }

    repos.bots.update(&bot_id, update).await?.ok_or_else(|| {
        info!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            "Bot not found during update",
        );
        ApiError::DatabaseError(format!("Bot with ID {} not found after update", bot_id))
    })?;

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Bot settings updated successfully",
    );

    Ok(Json(MessageResponse {
        message: "Bot settings updated successfully".to_string(),
    }))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.route("/settings", patch().to(update_settings));
}
