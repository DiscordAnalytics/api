use actix_web::web::{Data, Json};
use apistos::{
    api_operation,
    web::{ServiceConfig, post},
};
use tracing::info;

use crate::{
    domain::{
        auth::{decode_refresh_token, generate_access_token, hash_refresh_token},
        error::{ApiError, ApiResult},
    },
    openapi::schemas::{RefreshTokenRequest, TokenResponse},
    repository::Repositories,
    utils::{constants::ACCESS_TOKEN_LIFETIME, logger::LogCode},
};

#[api_operation(
    summary = "Refresh access token",
    description = "Refreshes the access token using a valid refresh token.",
    tag = "Auth",
    skip
)]
async fn refresh_token(
    repos: Data<Repositories>,
    body: Json<RefreshTokenRequest>,
) -> ApiResult<Json<TokenResponse>> {
    info!(
        code = %LogCode::Auth,
        "Refreshing access token",
    );

    let refresh_claims =
        decode_refresh_token(&body.refresh_token).map_err(|_| ApiError::InvalidToken)?;

    let token_hash = hash_refresh_token(&body.refresh_token);

    let session = repos
        .sessions
        .find_by_id(&refresh_claims.sid)
        .await?
        .ok_or(ApiError::InvalidToken)?;

    if !session.active || session.is_expired() || session.refresh_token_hash != token_hash {
        return Err(ApiError::InvalidToken);
    }

    repos.sessions.update_last_used(&session.session_id).await?;

    let access_token = generate_access_token(&session.user_id, &session.session_id)
        .map_err(|_| ApiError::TokenGenerationFailed)?;

    info!(
        code = %LogCode::Auth,
        session_id = %session.session_id,
        user_id = %session.user_id,
        "Access token refreshed successfully",
    );

    Ok(Json(TokenResponse {
        access_token,
        expires_in: ACCESS_TOKEN_LIFETIME,
        refresh_token: body.refresh_token.clone(),
    }))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.route("/refresh", post().to(refresh_token));
}
