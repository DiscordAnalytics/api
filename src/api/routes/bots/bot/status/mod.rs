mod config;
mod heartbeat;
mod history;

use actix_web::web::{Data, Json, Path};
use apistos::{
    api_operation,
    web::{ServiceConfig, get, scope},
};

use crate::{
    domain::error::{ApiError, ApiResult},
    openapi::schemas::PublicBotStatusResponse,
    repository::Repositories,
    utils::{discord::Snowflake, logger::LogCode},
};

#[api_operation(
    summary = "Get the public status of the bot",
    description = "Returns the current public status of a bot, including its deployment mode, overall status, latency, guild count, and shard statuses.",
    tag = "Status"
)]
async fn get_public_status(
    repos: Data<Repositories>,
    path: Path<String>,
) -> ApiResult<Json<PublicBotStatusResponse>> {
    let bot_id = Snowflake::try_from(path.into_inner())?.into_inner();

    info!(
        code = %LogCode::Status,
        bot_id = %bot_id,
        "Attempting to get public status for bot"
    );

    let bot = repos.bots.find_by_id(&bot_id).await?.ok_or_else(|| {
        info!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            "Bot not found for public status request",
        );
        ApiError::NotFound(format!("Bot with ID {} not found", bot_id))
    })?;

    if bot.suspended {
        warn!(
            code = %LogCode::Request,
            bot_id = %bot.bot_id,
            "Bot suspended for public status request",
        );
        return Err(ApiError::NotFound(format!(
            "Bot with ID {} not found",
            bot.bot_id
        )));
    }

    let current_status = repos
        .status_current
        .find_by_bot_id(&bot.bot_id)
        .await?
        .ok_or_else(|| {
            info!(
                code = %LogCode::Request,
                bot_id = %bot.bot_id,
                "No status found for public status request",
            );
            ApiError::NotFound(format!("No status found for bot with ID {}", bot.bot_id))
        })?;

    Ok(Json(current_status.into()))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/status")
            .route("", get().to(get_public_status))
            .configure(config::configure)
            .configure(heartbeat::configure)
            .configure(history::configure),
    );
}
