mod session;

use actix_web::web::{Data, Json};
use apistos::{
    api_operation,
    web::{ServiceConfig, delete, get, resource, scope},
};
use serde_json::{Value, json};
use tracing::info;

use crate::{
    api::middleware::Authenticated,
    domain::error::{ApiError, ApiResult},
    openapi::schemas::SessionResponse,
    repository::Repositories,
    utils::logger::LogCode,
};

#[api_operation(
    summary = "List active sessions",
    description = "Fetch a list of all active sessions for the authenticated user.",
    tag = "Auth",
    skip
)]
async fn list_sessions(
    auth: Authenticated,
    repos: Data<Repositories>,
) -> ApiResult<Json<Vec<SessionResponse>>> {
    let user_id = auth.0.user_id.ok_or(ApiError::Unauthorized)?;

    info!(
        code = %LogCode::Request,
        user_id = %user_id,
        "Listing sessions",
    );

    let sessions = repos.sessions.find_by_user_id(&user_id).await?;

    let session_responses = sessions
        .into_iter()
        .map(SessionResponse::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    info!(
        code = %LogCode::Request,
        user_id = %user_id,
        "Listed sessions",
    );

    Ok(Json(session_responses))
}

#[api_operation(
    summary = "Revoke all sessions",
    description = "Revoke all active sessions for the authenticated user, except the current session.",
    tag = "Auth",
    skip
)]
async fn revoke_all_sessions(
    auth: Authenticated,
    repos: Data<Repositories>,
) -> ApiResult<Json<Value>> {
    let user_id = auth.0.user_id.ok_or(ApiError::Unauthorized)?;
    let current_session_id = auth.0.session_id.ok_or(ApiError::Unauthorized)?;

    info!(
        code = %LogCode::Request,
        user_id = %user_id,
        "Revoking all sessions",
    );

    let sessions = repos.sessions.find_by_user_id(&user_id).await?;
    for session in sessions {
        if session.session_id != current_session_id {
            repos.sessions.revoke(&session.session_id).await?;
        }
    }

    info!(
        code = %LogCode::Request,
        user_id = %user_id,
        "Revoked all sessions",
    );

    Ok(Json(json!({
      "message": "All other sessions revoked successfully"
    })))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/sessions")
            .service(
                resource("")
                    .route(get().to(list_sessions))
                    .route(delete().to(revoke_all_sessions)),
            )
            .configure(session::configure),
    );
}
