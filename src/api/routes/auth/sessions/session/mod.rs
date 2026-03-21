use actix_web::web::{Data, Json, Path};
use apistos::{
    api_operation,
    web::{ServiceConfig, delete},
};
use serde_json::{Value, json};
use tracing::info;

use crate::{
    api::middleware::Authenticated,
    domain::error::{ApiError, ApiResult},
    repository::Repositories,
    utils::logger::LogCode,
};

#[api_operation(
    summary = "Revoke a session",
    description = "Revokes a specific session, effectively logging out the user from that session.",
    tag = "Auth",
    skip
)]
async fn revoke_session(
    auth: Authenticated,
    repos: Data<Repositories>,
    session_id: Path<String>,
) -> ApiResult<Json<Value>> {
    let user_id = auth.user_id.as_ref().ok_or(ApiError::Unauthorized)?;

    info!(
        code = %LogCode::Auth,
        session_id = %session_id,
        user_id = %user_id,
        "Revoking session"
    );

    let session = repos
        .sessions
        .find_by_id(&session_id)
        .await?
        .ok_or(ApiError::NotFound("Session not found".to_string()))?;

    if session.user_id != *user_id {
        return Err(ApiError::Forbidden);
    }

    repos.sessions.revoke(&session_id).await?;

    info!(
        code = %LogCode::Auth,
        session_id = %session_id,
        user_id = %user_id,
        "Session revoked successfully"
    );

    Ok(Json(json!({
      "message": "Session revoked successfully"
    })))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.route("/{session_id}", delete().to(revoke_session));
}
