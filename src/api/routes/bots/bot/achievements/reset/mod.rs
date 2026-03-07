use actix_web::web::{Data, Json};
use apistos::{
    api_operation,
    web::{ServiceConfig, post},
};
use tracing::{info, warn};

use crate::{
    api::middleware::{Authenticated, Snowflake},
    domain::{
        error::{ApiError, ApiResult},
        models::Achievement,
    },
    openapi::schemas::MessageResponse,
    repository::Repositories,
    services::Services,
    utils::logger::LogCode,
};

#[api_operation(
    summary = "Reset Bot Achievements",
    description = "Reset all achievements for a specific bot. This will remove all existing achievements and allow you to start fresh. This operation is irreversible, so use with caution.",
    tag = "Achievements"
)]
async fn reset_achievements(
    auth: Authenticated,
    services: Data<Services>,
    repos: Data<Repositories>,
    id: Snowflake,
) -> ApiResult<Json<MessageResponse>> {
    let bot_id = id.0;

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Attempting to reset achievements for bot",
    );

    let bot = repos.bots.find_by_id(&bot_id).await?.ok_or_else(|| {
        info!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            "Bot not found for achievements reset",
        );
        ApiError::NotFound(format!("Bot with ID {} not found", bot_id))
    })?;

    let ctx = &auth.0;

    if ctx.is_admin() {
        info!(
            code = %LogCode::AdminAction,
            bot_id = %bot_id,
            "Admin access granted for achievements reset",
        );
    } else if ctx.is_user() {
        let user_id = ctx.user_id.as_deref().ok_or(ApiError::Unauthorized)?;
        if !services.auth.user_has_bot_access(user_id, &bot_id).await? {
            warn!(
                code = %LogCode::Forbidden,
                bot_id = %bot_id,
                user_id = %user_id,
                "User does not have access to reset achievements for this bot",
            );
            return Err(ApiError::Forbidden);
        }
    } else {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Access denied for achievements reset",
        );
        return Err(ApiError::Forbidden);
    }

    if bot.suspended {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Attempt to reset achievements for suspended bot",
        );
        return Err(ApiError::BotSuspended);
    }

    repos.achievements.delete_by_bot_id(&bot_id).await?;
    let default_achievements = Achievement::defaults(&bot_id);
    repos
        .achievements
        .insert_many(&default_achievements)
        .await?;

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Successfully reset achievements for bot",
    );

    Ok(Json(MessageResponse {
        message: "Achievements reset successfully".to_string(),
    }))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.route("/reset", post().to(reset_achievements));
}
