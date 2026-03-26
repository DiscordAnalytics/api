use actix_web::web::{Data, Json, Path};
use anyhow::Result;
use apistos::{
    api_operation,
    web::{ServiceConfig, get},
};
use tracing::{info, warn};

use crate::{
    api::middleware::Authenticated,
    domain::error::{ApiError, ApiResult},
    openapi::schemas::TeamInvitationResponse,
    repository::Repositories,
    utils::{discord::Snowflake, logger::LogCode},
};

#[api_operation(
    summary = "Get user's invitations",
    description = "Fetch a list of invitations destined for the authenticated user",
    tag = "Users"
)]
async fn get_user_invitations(
    auth: Authenticated,
    repos: Data<Repositories>,
    id: Path<String>,
) -> ApiResult<Json<Vec<TeamInvitationResponse>>> {
    let user_id = Snowflake::try_from(id.into_inner())?.into_inner();

    info!(
        code = %LogCode::Request,
        user_id = %user_id,
        "Received request to fetch user's invitations"
    );

    let ctx = &auth;

    if ctx.is_admin() {
        info!(
            code = %LogCode::AdminAction,
            user_id = %user_id,
            "Admin access granted for user invitations"
        );
    } else if ctx.is_user() && ctx.user_id.as_deref() != Some(&user_id) {
        warn!(
            code = %LogCode::Forbidden,
            user_id = %user_id,
            "User attempted to access another user's invitations"
        );
        return Err(ApiError::Forbidden);
    } else if !ctx.is_user() {
        warn!(
            code = %LogCode::Forbidden,
            user_id = %user_id,
            "Unauthenticated access attempt to user invitations"
        );
        return Err(ApiError::Forbidden);
    }

    let user_invitations = repos.team_invitations.find_by_user(&user_id).await?;

    let invitations_response = user_invitations
        .into_iter()
        .map(TeamInvitationResponse::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    info!(
        code = %LogCode::Request,
        user_id = %user_id,
        invitations = invitations_response.len(),
        "Fetched user's invitations"
    );

    Ok(Json(invitations_response))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.route("/invitations", get().to(get_user_invitations));
}
