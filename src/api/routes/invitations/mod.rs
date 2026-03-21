mod invitation;

use actix_web::web::{Data, Json};
use apistos::{
    api_operation,
    web::{ServiceConfig, get, scope},
};
use tracing::info;

use crate::{
    api::middleware::RequireAdmin, domain::error::ApiResult,
    openapi::schemas::TeamInvitationResponse, repository::Repositories, utils::logger::LogCode,
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

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/invitations")
            .route("", get().to(get_invitations))
            .configure(invitation::configure),
    );
}
