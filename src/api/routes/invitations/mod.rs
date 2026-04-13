mod invitation;

use actix_web::web::{Data, Json};
use apistos::{
    api_operation,
    web::{ServiceConfig, get, scope},
};
use tracing::info;

use crate::{
    api::middleware::RequireAdmin,
    domain::error::{ApiError, ApiResult},
    openapi::schemas::InvitationResponse,
    repository::Repositories,
    utils::logger::LogCode,
};

#[api_operation(
    summary = "Get all invitations",
    description = "Retrieve a list of team invitations",
    tag = "Invitations",
    skip
)]
async fn get_invitations(
    _admin: RequireAdmin,
    repos: Data<Repositories>,
) -> ApiResult<Json<Vec<InvitationResponse>>> {
    info!(
        code = %LogCode::Request,
        "Fetching all team invitations",
    );

    let invitations = repos.team_invitations.find_all().await?;

    let bot_ids = invitations.iter().map(|i| i.bot_id.clone()).collect();
    let user_ids = invitations.iter().map(|i| i.user_id.clone()).collect();

    let bots = repos.bots.find_many_by_ids(&bot_ids).await?;
    let users = repos.users.find_many_by_ids(&user_ids).await?;

    let invitation_responses = invitations
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
                user_username: user.username.clone(),
                user_avatar: user.avatar.clone(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    info!(
        code = %LogCode::Request,
        "All team invitations fetched successfully",
    );

    Ok(Json(invitation_responses))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/invitations")
            .route("", get().to(get_invitations))
            .configure(invitation::configure),
    );
}
