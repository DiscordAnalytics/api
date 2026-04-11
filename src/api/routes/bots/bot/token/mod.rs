use actix_web::web::{Data, Json, Path};
use apistos::{
    api_operation,
    web::{ServiceConfig, get, patch, resource, scope},
};
use tracing::{error, info, warn};

use crate::{
    api::middleware::Authenticated,
    domain::{
        auth::generate_bot_token,
        error::{ApiError, ApiResult},
    },
    openapi::schemas::BotTokenResponse,
    repository::{BotUpdate, Repositories},
    services::Services,
    utils::{
        discord::{DiscordNotification, NotificationType, Snowflake},
        logger::LogCode,
    },
};

#[api_operation(
    summary = "Get Bot Token",
    description = "Retrieve the current authentication token for a bot. This endpoint is intended for testing purposes and should not be used in production environments. The token returned by this endpoint is the same as the one generated during bot creation or last refresh. For security reasons, it is recommended to use the token provided at bot creation time and to rotate it using the refresh endpoint if needed.",
    tag = "Bots"
)]
async fn get_token(
    auth: Authenticated,
    repos: Data<Repositories>,
    id: Path<String>,
) -> ApiResult<Json<BotTokenResponse>> {
    let bot_id = Snowflake::try_from(id.into_inner())?.into_inner();

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Retrieving bot token"
    );

    let bot = repos.bots.find_by_id(&bot_id).await?.ok_or_else(|| {
        warn!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            "Bot not found for token retrieval",
        );
        ApiError::NotFound(format!("Bot with ID {} not found", bot_id))
    })?;

    let ctx = &auth;

    if ctx.is_admin() {
        info!(
            code = %LogCode::AdminAction,
            bot_id = %bot_id,
            "Admin access granted for token retrieval",
        );
    } else if ctx.is_bot() && ctx.bot_id.as_deref() != Some(&bot_id) {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Bot attempting to retrieve token for a different bot",
        );
        return Err(ApiError::Forbidden);
    } else if ctx.is_user() {
        let user_id = ctx.user_id.as_deref().ok_or(ApiError::Unauthorized)?;
        if !bot.is_owner(user_id) {
            warn!(
                code = %LogCode::Forbidden,
                bot_id = %bot_id,
                user_id = %user_id,
                "User tried to retrieve token for a bot they don't have access to",
            );
            return Err(ApiError::Forbidden);
        }
    } else {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Access denied for token retrieval.",
        );
        return Err(ApiError::Forbidden);
    }

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Bot token retrieved successfully",
    );

    Ok(Json(BotTokenResponse { token: bot.token }))
}

#[api_operation(
    summary = "Refresh Bot Token",
    description = "Refresh the authentication token for a bot. This endpoint is used when a bot's token has been compromised or needs to be rotated for security reasons. The old token will be invalidated, and a new token will be generated and returned in the response. Ensure to update your bot's configuration with the new token to maintain uninterrupted service.",
    tag = "Bots"
)]
async fn refresh_token(
    auth: Authenticated,
    services: Data<Services>,
    repos: Data<Repositories>,
    id: Path<String>,
) -> ApiResult<Json<BotTokenResponse>> {
    let bot_id = Snowflake::try_from(id.into_inner())?.into_inner();

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Refreshing bot token"
    );

    let bot = repos.bots.find_by_id(&bot_id).await?.ok_or_else(|| {
        warn!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            "Bot not found for token refresh",
        );
        ApiError::NotFound(format!("Bot with ID {} not found", bot_id))
    })?;

    let ctx = &auth;

    if ctx.is_admin() {
        info!(
            code = %LogCode::AdminAction,
            bot_id = %bot_id,
            "Admin access granted for token refresh",
        );
    } else if ctx.is_bot() && ctx.bot_id.as_deref() != Some(&bot_id) {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Bot attempting to refresh token for a different bot",
        );
        return Err(ApiError::Forbidden);
    } else if ctx.is_user() {
        let user_id = ctx.user_id.as_deref().ok_or(ApiError::Unauthorized)?;
        if !bot.is_owner(user_id) {
            warn!(
                code = %LogCode::Forbidden,
                bot_id = %bot_id,
                user_id = %user_id,
                "User tried to refresh token for a bot they don't have access to",
            );
            return Err(ApiError::Forbidden);
        }
    } else {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Access denied for token refresh.",
        );
        return Err(ApiError::Forbidden);
    }

    if bot.suspended && !ctx.is_admin() {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Access denied for token refresh of suspended bot",
        );
        return Err(ApiError::BotSuspended);
    }

    let new_token = generate_bot_token(&bot_id)?;
    let bot_update = BotUpdate::new().with_token(new_token.clone());

    let update_result = repos
        .bots
        .update(&bot_id, bot_update)
        .await?
        .ok_or_else(|| {
            warn!(
                code = %LogCode::Request,
                bot_id = %bot_id,
                "Bot not found during token refresh update",
            );
            ApiError::NotFound(format!("Bot with ID {} not found during update", bot_id))
        })?;

    if ctx.is_admin() || ctx.is_bot() {
        let owner = repos
            .users
            .find_by_id(&bot.owner_id)
            .await?
            .ok_or_else(|| {
                warn!(
                    code = %LogCode::Request,
                    bot_id = %bot_id,
                    "Bot owner not found for token refresh notification",
                );
                ApiError::NotFound(format!("Owner with ID {} not found", bot.owner_id))
            })?;

        if let Err(e) = services
            .discord
            .send_dm(
                &owner.user_id,
                Some(DiscordNotification::create(
                    NotificationType::BotTokenRegen {
                        bot_username: bot.username,
                        bot_id: bot.bot_id,
                    },
                )),
            )
            .await
        {
            error!(
                code = %LogCode::Mail,
                bot_id = %bot_id,
                user_id = %owner.user_id,
                error = %e,
                "Failed to send bot token regeneration DM",
            );
        }

        #[cfg(feature = "mails")]
        if let Err(e) = services.mail.send_bot_token_regen(&owner, &update_result) {
            error!(
                code = %LogCode::Mail,
                bot_id = %bot_id,
                user_id = %owner.user_id,
                error = %e,
                "Failed to send bot token regeneration email",
            );
        };
    };

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Bot token refreshed successfully",
    );

    Ok(Json(BotTokenResponse {
        token: update_result.token,
    }))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/token").service(
            resource("")
                .route(get().to(get_token))
                .route(patch().to(refresh_token)),
        ),
    );
}
