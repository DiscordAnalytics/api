use actix_web::web::{Data, Json, Path};
use apistos::{
    api_operation,
    web::{ServiceConfig, get, patch, resource, scope},
};
use tracing::{info, warn};

use crate::{
    api::middleware::Authenticated,
    domain::{
        auth::generate_bot_token,
        error::{ApiError, ApiResult},
    },
    openapi::schemas::BotTokenResponse,
    repository::{BotUpdate, Repositories},
    services::Services,
    utils::{discord::is_valid_snowflake, logger::LogCode},
};

#[api_operation(
    summary = "Get Bot Token",
    description = "Retrieve the current authentication token for a bot. This endpoint is intended for testing purposes and should not be used in production environments. The token returned by this endpoint is the same as the one generated during bot creation or last refresh. For security reasons, it is recommended to use the token provided at bot creation time and to rotate it using the refresh endpoint if needed.",
    tag = "Bots"
)]
async fn get_token(
    auth: Authenticated,
    services: Data<Services>,
    repos: Data<Repositories>,
    id: Path<String>,
) -> ApiResult<Json<BotTokenResponse>> {
    let bot_id = id.into_inner();

    if !is_valid_snowflake(&bot_id) {
        return Err(ApiError::InvalidId);
    }

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Retrieving bot token"
    );

    let ctx = &auth.0;

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
        if !services.auth.user_has_bot_access(user_id, &bot_id).await? {
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

    let bot = repos.bots.find_by_id(&bot_id).await?.ok_or_else(|| {
        warn!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            "Bot not found for token retrieval",
        );
        ApiError::NotFound(format!("Bot with ID {} not found", bot_id))
    })?;

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
    let bot_id = id.into_inner();

    if !is_valid_snowflake(&bot_id) {
        return Err(ApiError::InvalidId);
    }

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Refreshing bot token"
    );

    let ctx = &auth.0;

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
        if !services.auth.user_has_bot_access(user_id, &bot_id).await? {
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

    let new_token = generate_bot_token(&bot_id)?;
    let bot_update = BotUpdate::new().with_token(new_token.clone());

    repos.bots.update(&bot_id, bot_update).await?;

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Bot token refreshed successfully",
    );

    Ok(Json(BotTokenResponse { token: new_token }))
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
