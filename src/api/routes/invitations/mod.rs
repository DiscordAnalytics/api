mod invitation;

use actix_web::web::{Data, Json};
use apistos::{
    api_operation,
    web::{ServiceConfig, get, post, resource, scope},
};
use tracing::info;

use crate::{
    api::middleware::{Authenticated, RequireAdmin},
    domain::error::{ApiError, ApiResult},
    openapi::schemas::{InvitationAcceptBody, InvitationAcceptResponse, TeamInvitationResponse},
    repository::Repositories,
    services::Services,
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
) -> ApiResult<Json<Vec<TeamInvitationResponse>>> {
    info!(
        code = %LogCode::Request,
        "Fetching all team invitations",
    );

    let invitations = repos.team_invitations.find_all().await?;

    let invitation_responses = invitations
        .into_iter()
        .map(TeamInvitationResponse::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    info!(
        code = %LogCode::Request,
        "All team invitations fetched successfully",
    );

    Ok(Json(invitation_responses))
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
    } else {
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
        scope("/invitations")
            .service(
                resource("")
                    .route(get().to(get_invitations))
                    .route(post().to(post_invitation)),
            )
            .configure(invitation::configure),
    );
}
