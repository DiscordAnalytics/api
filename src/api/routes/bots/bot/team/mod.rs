use actix_web::web::{Data, Json};
use apistos::{
    api_operation,
    web::{ServiceConfig, delete, get, post, resource, scope},
};
use tracing::{info, warn};

use crate::{
    api::middleware::{Authenticated, Snowflake},
    domain::{
        error::{ApiError, ApiResult},
        models::TeamInvitation,
    },
    openapi::schemas::{MessageResponse, TeamRequestBody, TeamResponse},
    repository::{BotUpdate, Repositories},
    services::Services,
    utils::logger::LogCode,
};

#[api_operation(
    summary = "Get the team of a bot",
    description = "Get the team of a bot",
    tag = "Bots"
)]
async fn get_team(
    auth: Authenticated,
    repos: Data<Repositories>,
    id: Snowflake,
) -> ApiResult<Json<Vec<TeamResponse>>> {
    let bot_id = id.0;

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Fetching team for bot"
    );

    let ctx = &auth.0;

    let bot = repos.bots.find_by_id(&bot_id).await?.ok_or_else(|| {
        info!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            "Bot not found",
        );
        ApiError::NotFound(format!("Bot with ID {} not found", bot_id))
    })?;

    if bot.suspended {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Access denied for suspended bot team",
        );
        return Err(ApiError::BotSuspended);
    }

    if ctx.is_admin() {
        info!(
            code = %LogCode::AdminAction,
            bot_id = %bot_id,
            "Admin access granted to fetch team for bot",
        );
    } else if ctx.is_bot() && ctx.bot_id.as_deref() != Some(&bot_id) {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Bot access denied to fetch team for another bot",
        );
        return Err(ApiError::Forbidden);
    } else if ctx.is_user() {
        let user_id = ctx.user_id.as_deref().ok_or(ApiError::Unauthorized)?;
        if !bot.is_owner(user_id) {
            warn!(
                code = %LogCode::Forbidden,
                bot_id = %bot_id,
                user_id = %user_id,
                "User does not have access to bot team",
            );
            return Err(ApiError::Forbidden);
        }
    } else {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Access denied for bot team",
        );
        return Err(ApiError::Forbidden);
    }

    let mut team = Vec::new();

    for user_id in bot.team {
        let mut response = TeamResponse {
            avatar: None,
            invitation_id: None,
            pending_invitation: false,
            registered: false,
            user_id: user_id.clone(),
            username: None,
        };

        if let Some(user) = repos.users.find_by_id(&user_id).await? {
            response.avatar = user.avatar;
            response.username = Some(user.username);
            response.registered = true;
        }

        if let Some(invitation) = repos
            .team_invitations
            .find_by_bot_and_user(&bot_id, &user_id)
            .await?
        {
            response.invitation_id = Some(invitation.invitation_id);
            response.pending_invitation = !invitation.accepted;
        }

        team.push(response);
    }

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        team_size = team.len(),
        "Fetched team for bot",
    );

    Ok(Json(team))
}

#[api_operation(
    summary = "Add a user to the team of a bot",
    description = "Add a user to the team of a bot",
    tag = "Bots"
)]
async fn add_to_team(
    auth: Authenticated,
    services: Data<Services>,
    repos: Data<Repositories>,
    body: Json<TeamRequestBody>,
    id: Snowflake,
) -> ApiResult<Json<TeamResponse>> {
    let bot_id = id.0;

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        user_id = %body.user_id,
        "Adding user to bot team"
    );

    let bot = repos.bots.find_by_id(&bot_id).await?.ok_or_else(|| {
        info!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            "Bot not found",
        );
        ApiError::NotFound(format!("Bot with ID {} not found", bot_id))
    })?;

    if bot.suspended {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            user_id = %body.user_id,
            "Access denied to add user to suspended bot team",
        );
        return Err(ApiError::BotSuspended);
    }

    let ctx = &auth.0;

    if ctx.is_admin() {
        info!(
            code = %LogCode::AdminAction,
            bot_id = %bot_id,
            user_id = %body.user_id,
            "Admin access granted to add user to bot team",
        );
    } else if ctx.is_bot() && ctx.bot_id.as_deref() != Some(&bot_id) {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            user_id = %body.user_id,
            "Bot access denied to add user to another bot team",
        );
        return Err(ApiError::Forbidden);
    } else if ctx.is_user() {
        let user_id = ctx.user_id.as_deref().ok_or(ApiError::Unauthorized)?;
        if !bot.is_owner(user_id) {
            warn!(
                code = %LogCode::Forbidden,
                bot_id = %bot_id,
                user_id = %user_id,
                "User does not have access to add user to bot team",
            );
            return Err(ApiError::Forbidden);
        }
    } else {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            user_id = %body.user_id,
            "Access denied to add user to bot team",
        );
        return Err(ApiError::Forbidden);
    }

    if services
        .auth
        .user_has_bot_access(&body.user_id, &bot_id)
        .await?
    {
        warn!(
            code = %LogCode::Conflict,
            bot_id = %bot_id,
            user_id = %body.user_id,
            "User is already a member of the bot team",
        );
        return Err(ApiError::Conflict(format!(
            "User with ID {} is already a member of the bot team",
            body.user_id
        )));
    }

    let update = BotUpdate::new().with_team_member(&body.user_id);
    repos.bots.update(&bot_id, update).await?;

    let invitation = TeamInvitation::new(&bot_id, &body.user_id);
    repos.team_invitations.insert(&invitation).await?;

    let response = TeamResponse {
        avatar: None,
        invitation_id: Some(invitation.invitation_id),
        pending_invitation: true,
        registered: false,
        user_id: body.user_id.clone(),
        username: None,
    };

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        user_id = %body.user_id,
        "Added user to bot team",
    );

    Ok(Json(response))
}

#[api_operation(
    summary = "Remove a user from the team of a bot",
    description = "Remove a user from the team of a bot",
    tag = "Bots"
)]
async fn delete_from_team(
    auth: Authenticated,
    services: Data<Services>,
    repos: Data<Repositories>,
    body: Json<TeamRequestBody>,
    id: Snowflake,
) -> ApiResult<Json<MessageResponse>> {
    let bot_id = id.0;

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        user_id = %body.user_id,
        "Removing user from bot team"
    );

    let bot = repos.bots.find_by_id(&bot_id).await?.ok_or_else(|| {
        info!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            "Bot not found",
        );
        ApiError::NotFound(format!("Bot with ID {} not found", bot_id))
    })?;

    let ctx = &auth.0;

    if ctx.is_admin() {
        info!(
            code = %LogCode::AdminAction,
            bot_id = %bot_id,
            user_id = %body.user_id,
            "Admin access granted to remove user from bot team",
        );
    } else if ctx.is_bot() && ctx.bot_id.as_deref() != Some(&bot_id) {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            user_id = %body.user_id,
            "Bot access denied to remove user from another bot team",
        );
        return Err(ApiError::Forbidden);
    } else if ctx.is_user() {
        let user_id = ctx.user_id.as_deref().ok_or(ApiError::Unauthorized)?;
        if !bot.is_owner(user_id) {
            warn!(
                code = %LogCode::Forbidden,
                bot_id = %bot_id,
                user_id = %user_id,
                "User does not have access to remove user from bot team",
            );
            return Err(ApiError::Forbidden);
        }
    } else {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            user_id = %body.user_id,
            "Access denied to remove user from bot team",
        );
        return Err(ApiError::Forbidden);
    }

    if !services
        .auth
        .user_has_bot_access(&body.user_id, &bot_id)
        .await?
    {
        warn!(
            code = %LogCode::Conflict,
            bot_id = %bot_id,
            user_id = %body.user_id,
            "User is not a member of the bot team",
        );
        return Err(ApiError::Conflict(format!(
            "User with ID {} is not a member of the bot team",
            body.user_id
        )));
    }

    repos
        .bots
        .remove_user_from_team(&bot_id, &body.user_id)
        .await?;

    repos
        .team_invitations
        .delete_by_bot_and_user(&bot_id, &body.user_id)
        .await?;

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        user_id = %body.user_id,
        "Removed user from bot team",
    );

    Ok(Json(MessageResponse {
        message: format!(
            "User with ID {} has been removed from the bot team",
            body.user_id
        ),
    }))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/team").service(
            resource("")
                .route(get().to(get_team))
                .route(post().to(add_to_team))
                .route(delete().to(delete_from_team)),
        ),
    );
}
