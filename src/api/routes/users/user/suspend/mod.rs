use actix_web::web::{Data, Json, Path};
use apistos::{
    api_operation,
    web::{ServiceConfig, delete, post, scope},
};
use tracing::info;

use crate::{
    api::middleware::RequireAdmin,
    domain::error::{ApiError, ApiResult},
    openapi::schemas::{UserSuspendRequest, UserSuspendResponse},
    repository::{Repositories, UserUpdate},
    utils::{discord::is_valid_snowflake, logger::LogCode},
};

#[api_operation(
    summary = "Suspend a user",
    description = "Suspends a user, preventing them from accessing their account and using the API. Only administrators can perform this action.",
    tag = "Users",
    skip
)]
async fn suspend_user(
    _admin: RequireAdmin,
    repos: Data<Repositories>,
    body: Json<UserSuspendRequest>,
    id: Path<String>,
) -> ApiResult<Json<UserSuspendResponse>> {
    let user_id = id.into_inner();

    if !is_valid_snowflake(user_id.as_str()) {
        return Err(ApiError::InvalidId);
    }

    info!(
        code = %LogCode::Request,
        user_id = %user_id,
        reason = %body.reason,
        "Received request to suspend user"
    );

    let reason = body.reason.trim();

    let user_update = UserUpdate::new().with_suspended(true);

    repos.users.update(&user_id, user_update).await?;
    repos.sessions.revoke_all_for_user(&user_id).await?;

    info!(
        code = %LogCode::AdminAction,
        user_id = %user_id,
        reason = %reason,
        "User has been suspended"
    );

    Ok(Json(UserSuspendResponse {
        message: format!("User {} has been suspended for reason: {}", user_id, reason),
    }))
}

#[api_operation(
    summary = "Unsuspend a user",
    description = "Unsuspends a user, restoring their access to their account and the API. Only administrators can perform this action.",
    tag = "Users",
    skip
)]
async fn unsuspend_user(
    _admin: RequireAdmin,
    repos: Data<Repositories>,
    id: Path<String>,
) -> ApiResult<Json<UserSuspendResponse>> {
    let user_id = id.into_inner();

    if !is_valid_snowflake(user_id.as_str()) {
        return Err(ApiError::InvalidId);
    }

    info!(
        code = %LogCode::Request,
        user_id = %user_id,
        "Received request to unsuspend user"
    );

    let user_update = UserUpdate::new().with_suspended(false);

    repos.users.update(&user_id, user_update).await?;

    info!(
        code = %LogCode::AdminAction,
        user_id = %user_id,
        "User has been unsuspended"
    );

    Ok(Json(UserSuspendResponse {
        message: format!("User {} has been unsuspended", user_id),
    }))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/suspend")
            .route("", post().to(suspend_user))
            .route("", delete().to(unsuspend_user)),
    );
}
