mod bots;
mod suspend;

use actix_web::web::{Data, Json};
use apistos::{
    api_operation,
    web::{ServiceConfig, delete, get, patch, resource, scope},
};
use tracing::{error, info, warn};

use crate::{
    api::middleware::{Authenticated, RequireAdmin, Snowflake},
    domain::error::{ApiError, ApiResult},
    openapi::schemas::{MessageResponse, UserResponse, UserUpdateRequest},
    repository::{Repositories, UserUpdate},
    services::Services,
    utils::{
        discord::{DiscordNotification, NotificationType},
        logger::LogCode,
    },
};

#[api_operation(
    summary = "Get user details",
    description = "Fetch detailed information about a specific user registered in the Discord Analytics API",
    tag = "Users"
)]
async fn get_user(
    auth: Authenticated,
    repos: Data<Repositories>,
    id: Snowflake,
) -> ApiResult<Json<UserResponse>> {
    let user_id = id.0;

    info!(
        code = %LogCode::Request,
        user_id = %user_id,
        "Received request to fetch user details"
    );

    let ctx = &auth.0;

    if ctx.is_admin() {
        info!(
            code = %LogCode::AdminAction,
            user_id = %user_id,
            "Admin access granted for user details"
        );
    } else if ctx.is_user() && ctx.user_id.as_deref() != Some(&user_id) {
        warn!(
            code = %LogCode::Forbidden,
            user_id = %user_id,
            "User attempted to access another user's details"
        );
        return Err(ApiError::Forbidden);
    } else if !ctx.is_user() {
        warn!(
            code = %LogCode::Forbidden,
            user_id = %user_id,
            "Unauthenticated access attempt to user details"
        );
        return Err(ApiError::Forbidden);
    }

    let user = repos.users.find_by_id(&user_id).await?.ok_or_else(|| {
        info!(
            code = %LogCode::Request,
            user_id = %user_id,
            "User not found"
        );
        ApiError::NotFound(format!("User with ID {} not found", user_id))
    })?;

    info!(
        code = %LogCode::Request,
        user_id = %user_id,
        "Fetched details for user"
    );

    Ok(Json(UserResponse::try_from(user)?))
}

#[api_operation(
    summary = "Update user details",
    description = "Update information for a specific user registered in the Discord Analytics API",
    tag = "Users",
    skip
)]
async fn update_user(
    _admin: RequireAdmin,
    repos: Data<Repositories>,
    body: Json<UserUpdateRequest>,
    id: Snowflake,
) -> ApiResult<Json<UserResponse>> {
    let user_id = id.0;

    info!(
        code = %LogCode::Request,
        user_id = %user_id,
        "Received request to update user details"
    );

    let bots_limit = body.bots_limit;

    let user_update = UserUpdate::new().with_bots_limit(bots_limit);

    let update_result = repos.users.update(&user_id, user_update).await?;

    let updated_user = update_result.ok_or_else(|| {
        warn!(
            code = %LogCode::Request,
            user_id = %user_id,
            "User not found after update"
        );
        ApiError::DatabaseError(format!("User with ID {} not found after update", user_id))
    })?;

    info!(
        code = %LogCode::Request,
        user_id = %user_id,
        "User details updated"
    );

    Ok(Json(UserResponse::try_from(updated_user)?))
}

#[api_operation(
    summary = "Delete a user",
    description = "Delete a specific user from the Discord Analytics API",
    tag = "Users"
)]
async fn delete_user(
    auth: Authenticated,
    services: Data<Services>,
    repos: Data<Repositories>,
    id: Snowflake,
) -> ApiResult<Json<MessageResponse>> {
    let user_id = id.0;

    info!(
        code = %LogCode::Request,
        user_id = %user_id,
        "Received request to delete user account"
    );

    #[cfg_attr(not(feature = "mails"), allow(unused_variables))]
    let user = repos.users.find_by_id(&user_id).await?.ok_or_else(|| {
        info!(
            code = %LogCode::Request,
            user_id = %user_id,
            "User not found for deletion"
        );
        ApiError::NotFound(format!("User with ID {} not found", user_id))
    })?;

    let ctx = &auth.0;

    if ctx.is_admin() {
        info!(
            code = %LogCode::AdminAction,
            user_id = %user_id,
            "Admin access granted for user deletion"
        );
    } else if ctx.is_user() && ctx.user_id.as_deref() != Some(&user_id) {
        warn!(
            code = %LogCode::Forbidden,
            user_id = %user_id,
            "User attempted to delete another user's account"
        );
        return Err(ApiError::Forbidden);
    } else if !ctx.is_user() {
        warn!(
            code = %LogCode::Forbidden,
            user_id = %user_id,
            "Unauthenticated access attempt to delete user account"
        );
        return Err(ApiError::Forbidden);
    }

    let result = repos.users.delete_by_id(&user_id).await?;

    if result.deleted_count == 0 {
        info!(
            code = %LogCode::Request,
            user_id = %user_id,
            "User not found for deletion"
        );
        return Err(ApiError::NotFound(format!(
            "User with ID {} not found",
            user_id
        )));
    }

    if ctx.is_admin() {
        if let Err(e) = services
            .discord
            .send_dm(
                &user.user_id,
                None,
                Some(DiscordNotification::create(
                    NotificationType::UserDeletedByAdmin {
                        username: user.username.clone(),
                        user_id: user_id.clone(),
                    },
                )),
            )
            .await
        {
            error!(
                code = %LogCode::Mail,
                user_id = %user_id,
                error = ?e,
                "Failed to send account deletion DM to user"
            );
        }

        #[cfg(feature = "mails")]
        if let Err(e) = services.mail.send_user_deleted_by_admin(&user) {
            error!(
                code = %LogCode::Mail,
                user_id = %user_id,
                error = ?e,
                "Failed to send account deletion email to user"
            );
        }
    }

    info!(
        code = %LogCode::Request,
        user_id = %user_id,
        "User account deleted"
    );

    Ok(Json(MessageResponse {
        message: format!("User with ID {} has been deleted", user_id),
    }))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/{id}")
            .service(
                resource("")
                    .route(get().to(get_user))
                    .route(patch().to(update_user))
                    .route(delete().to(delete_user)),
            )
            .configure(bots::configure)
            .configure(suspend::configure),
    );
}
