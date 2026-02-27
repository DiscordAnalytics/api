use actix_web::web::{Data, Json, Path};
use apistos::{
    api_operation,
    web::{ServiceConfig, delete, post, scope},
};
use tracing::info;

use crate::{
    api::middleware::RequireAdmin,
    domain::error::{ApiError, ApiResult},
    openapi::schemas::{BotSuspendRequest, BotSuspendResponse},
    repository::{BotUpdate, Repositories},
    utils::{discord::is_valid_snowflake, logger::LogCode},
};

#[api_operation(
    summary = "Suspend a bot",
    description = "Suspends a bot, preventing it from accessing the API. The bot's owner will still be able to access their account, but the bot itself will be disabled. Only administrators can perform this action.",
    tag = "Bots",
    skip
)]
async fn suspend_bot(
    _admin: RequireAdmin,
    repos: Data<Repositories>,
    body: Json<BotSuspendRequest>,
    id: Path<String>,
) -> ApiResult<Json<BotSuspendResponse>> {
    let bot_id = id.into_inner();

    if !is_valid_snowflake(bot_id.as_str()) {
        return Err(ApiError::InvalidId);
    }

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        reason = %body.reason,
        "Received request to suspend bot"
    );

    let reason = body.reason.trim();

    let bot_update = BotUpdate::new().with_suspended(true);

    repos.bots.update(&bot_id, bot_update).await?;

    info!(
        code = %LogCode::AdminAction,
        bot_id = %bot_id,
        reason = %reason,
        "Bot has been suspended"
    );

    Ok(Json(BotSuspendResponse {
        message: format!("Bot {} has been suspended for reason: {}", bot_id, reason),
    }))
}

#[api_operation(
    summary = "Unsuspend a bot",
    description = "Unsuspends a bot, allowing it to access the API again. Only administrators can perform this action.",
    tag = "Bots",
    skip
)]
async fn unsuspend_bot(
    _admin: RequireAdmin,
    repos: Data<Repositories>,
    id: Path<String>,
) -> ApiResult<Json<BotSuspendResponse>> {
    let bot_id = id.into_inner();

    if !is_valid_snowflake(bot_id.as_str()) {
        return Err(ApiError::InvalidId);
    }

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Received request to unsuspend bot"
    );

    let bot_update = BotUpdate::new().with_suspended(false);

    repos.bots.update(&bot_id, bot_update).await?;

    info!(
        code = %LogCode::AdminAction,
        bot_id = %bot_id,
        "Bot has been unsuspended"
    );

    Ok(Json(BotSuspendResponse {
        message: format!("Bot {} has been unsuspended", bot_id),
    }))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/suspend")
            .route("", post().to(suspend_bot))
            .route("", delete().to(unsuspend_bot)),
    );
}
