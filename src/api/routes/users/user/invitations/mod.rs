use actix_web::web::{Data, Json, Path};
use apistos::{
    api_operation,
    web::{ServiceConfig, get},
};
use tracing::{info, warn};

use crate::{
    api::middleware::Authenticated,
    domain::error::{ApiError, ApiResult},
    openapi::schemas::InvitationResponse,
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
) -> ApiResult<Json<Vec<InvitationResponse>>> {
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
    } else if ctx.is_user() {
        if ctx.user_id.as_deref() != Some(&user_id) {
            warn!(
                code = %LogCode::Forbidden,
                user_id = %user_id,
                "User attempted to access another user's invitations"
            );
            return Err(ApiError::Forbidden);
        }
    } else {
        warn!(
            code = %LogCode::Forbidden,
            user_id = %user_id,
            "Unauthenticated access attempt to user invitations"
        );
        return Err(ApiError::Forbidden);
    }

    let user_invitations = repos.team_invitations.find_by_user(&user_id).await?;

    let bot_ids = user_invitations.iter().map(|i| i.bot_id.clone()).collect();
    let user_ids = user_invitations.iter().map(|i| i.user_id.clone()).collect();

    let bots = repos.bots.find_many_by_ids(&bot_ids).await?;
    let users = repos.users.find_many_by_ids(&user_ids).await?;

    let invitations = user_invitations
        .into_iter()
        .map(|invitation| -> ApiResult<_> {
            let bot = bots.get(&invitation.bot_id).ok_or_else(|| {
                ApiError::NotFound(format!("Bot not found: {}", invitation.bot_id))
            })?;
            let user = users.get(&invitation.user_id).ok_or_else(|| {
                ApiError::NotFound(format!("User not found: {}", invitation.user_id))
            })?;

            Ok(InvitationResponse {
                invitation: invitation.try_into()?,
                bot_username: bot.username.clone(),
                bot_avatar: bot.avatar.clone(),
                user_username: None,
                user_avatar: None,
                owner_username: Some(user.username.clone()),
                owner_avatar: user.avatar.clone(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    info!(
        code = %LogCode::Request,
        user_id = %user_id,
        invitations = invitations.len(),
        "Fetched user's invitations"
    );

    Ok(Json(invitations))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.route("/invitations", get().to(get_user_invitations));
}
