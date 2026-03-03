use actix_web::web::{Data, Json, Path};
use apistos::{
    api_operation,
    web::{ServiceConfig, get},
};
use tracing::info;

use crate::{
    domain::error::{ApiError, ApiResult},
    openapi::schemas::InvitationResponse,
    repository::Repositories,
    utils::logger::LogCode,
};

#[api_operation(
    summary = "Get an invitation",
    description = "Fetch details of a specific invitation using its ID",
    tag = "Invitations"
)]
async fn get_invitation(
    repos: Data<Repositories>,
    id: Path<String>,
) -> ApiResult<Json<InvitationResponse>> {
    let invitation_id = &id.into_inner();

    info!(
        code = %LogCode::Request,
        invitation_id = %invitation_id,
        "Fetching details for invitation",
    );

    let invitation = repos
        .team_invitations
        .find_by_id(invitation_id)
        .await?
        .ok_or_else(|| {
            info!(
                code = %LogCode::Request,
                invitation_id = %invitation_id,
                "Invitation not found",
            );
            ApiError::NotFound(format!("Invitation with ID {} not found", invitation_id))
        })?;

    if invitation.accepted {
        info!(
            code = %LogCode::Request,
            invitation_id = %invitation_id,
            "Invitation already accepted",
        );
        return Err(ApiError::InvitationAlreadyAccepted);
    }

    if invitation.is_expired() {
        info!(
            code = %LogCode::Request,
            invitation_id = %invitation_id,
            "Invitation expired",
        );
        return Err(ApiError::InvitationExpired);
    }

    let bot = repos
        .bots
        .find_by_id(&invitation.bot_id)
        .await?
        .ok_or_else(|| {
            info!(
                code = %LogCode::Request,
                bot_id = %invitation.bot_id,
                "Bot not found for invitation",
            );
            ApiError::NotFound(format!("Bot with ID {} not found", invitation.bot_id))
        })?;

    let owner = repos
        .users
        .find_by_id(&bot.owner_id)
        .await?
        .ok_or_else(|| {
            info!(
                code = %LogCode::Request,
                owner_id = %bot.owner_id,
                "Owner not found for bot in invitation",
            );
            ApiError::NotFound(format!("Owner with ID {} not found", bot.owner_id))
        })?;

    info!(
        code = %LogCode::Request,
        invitation_id = %invitation_id,
        bot_id = %bot.bot_id,
        owner_id = %owner.user_id,
        "Successfully fetched invitation details",
    );

    Ok(Json(InvitationResponse {
        invitation: invitation.try_into()?,
        bot_username: bot.username,
        bot_avatar: bot.avatar,
        owner_username: owner.username,
        owner_avatar: owner.avatar,
    }))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.route("/{invitation_id}", get().to(get_invitation));
}
