use actix_web::web::{Data, Json, Query};
use apistos::{
    api_operation,
    web::{ServiceConfig, get, post, resource, scope},
};
use tracing::info;

use crate::{
    api::middleware::Authenticated,
    domain::error::{ApiError, ApiResult},
    openapi::schemas::{
        InvitationAcceptBody, InvitationAcceptResponse, InvitationQuery, InvitationResponse,
    },
    repository::Repositories,
    services::Services,
    utils::logger::LogCode,
};

#[api_operation(
    summary = "Get an invitation",
    description = "Fetch details of a specific invitation using its ID",
    tag = "Invitations"
)]
async fn get_invitation(
    repos: Data<Repositories>,
    query: Query<InvitationQuery>,
) -> ApiResult<Json<InvitationResponse>> {
    let invitation_id = &query.invitation_id;

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
        owner_avatar: owner.avatar.into(),
    }))
}

#[api_operation(
    summary = "Accept or reject an invitation",
    description = "Accept or reject a team invitation using its ID",
    tag = "Invitations"
)]
async fn post_invitation(
    auth: Authenticated,
    services: Data<Services>,
    repos: Data<Repositories>,
    body: Json<InvitationAcceptBody>,
) -> ApiResult<Json<InvitationAcceptResponse>> {
    let invitation_id = &body.invitation_id;

    info!(
        code = %LogCode::Request,
        invitation_id = %invitation_id,
        "Processing invitation acceptance/rejection",
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

    let bot = repos
        .bots
        .find_by_id(&invitation.bot_id)
        .await?
        .ok_or_else(|| {
            info!(
                code = %LogCode::Request,
                bot_id = %invitation.bot_id,
                "Bot not found for",
            );
            ApiError::NotFound(format!("Bot with ID {} not found", invitation.bot_id))
        })?;

    let ctx = &auth.0;

    if ctx.is_admin() {
        info!(
            code = %LogCode::AdminAction,
            invitation_id = %invitation_id,
            "Admin user processing invitation",
        );
    } else if ctx.is_user() {
        let user_id = ctx.user_id.as_deref().ok_or(ApiError::Unauthorized)?;
        if !bot.is_team_member(user_id) {
            info!(
                code = %LogCode::Forbidden,
                invitation_id = %invitation_id,
                user_id = %user_id,
                "User does not have access to process invitation",
            );
            return Err(ApiError::Forbidden);
        }
        info!(
            code = %LogCode::Request,
            invitation_id = %invitation_id,
            user_id = %user_id,
            "User processing invitation",
        );
    } else if !ctx.is_user() {
        info!(
            code = %LogCode::Forbidden,
            invitation_id = %invitation_id,
            "Unauthorized context attempting to process invitation",
        );
        return Err(ApiError::Forbidden);
    }

    if invitation.accepted {
        info!(
            code = %LogCode::Request,
            invitation_id = %invitation_id,
            "Invitation already accepted, cannot process",
        );
        return Err(ApiError::InvitationAlreadyAccepted);
    }

    if invitation.is_expired() {
        info!(
            code = %LogCode::Request,
            invitation_id = %invitation_id,
            "Invitation expired, cannot process",
        );
        return Err(ApiError::InvitationExpired);
    }

    if body.accept {
        repos
            .team_invitations
            .accept_invitation(invitation_id)
            .await?;
        info!(
            code = %LogCode::Request,
            invitation_id = %invitation_id,
            "Invitation accepted successfully",
        );
    } else {
        services
            .invitations
            .reject_invitation(invitation_id, &invitation.bot_id, &invitation.user_id)
            .await?;
        info!(
            code = %LogCode::Request,
            invitation_id = %invitation_id,
            "Invitation rejected successfully",
        );
    }

    Ok(Json(InvitationAcceptResponse {
        accepted: body.accept,
    }))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/invitations").service(
            resource("")
                .route(get().to(get_invitation))
                .route(post().to(post_invitation)),
        ),
    );
}
