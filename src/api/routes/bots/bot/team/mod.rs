use std::collections::{HashMap, HashSet};

use actix_web::web::{Data, Json, Path};
use apistos::{
    api_operation,
    web::{ServiceConfig, delete, get, post, resource, scope},
};
use tracing::{error, info, warn};

use crate::{
    api::middleware::Authenticated,
    domain::{
        error::{ApiError, ApiResult},
        models::TeamInvitation,
    },
    openapi::schemas::{MessageResponse, NewInvitationResponse, TeamRequestBody, TeamResponse},
    repository::Repositories,
    services::Services,
    utils::{
        discord::{DiscordNotification, NotificationType, Snowflake},
        logger::LogCode,
    },
};

#[api_operation(
    summary = "Get the team of a bot",
    description = "Get the team of a bot",
    tag = "Bots"
)]
async fn get_team(
    auth: Authenticated,
    repos: Data<Repositories>,
    id: Path<String>,
) -> ApiResult<Json<Vec<TeamResponse>>> {
    let bot_id = Snowflake::try_from(id.into_inner())?.into_inner();

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Fetching team for bot"
    );

    let ctx = &auth;

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

    let invitations = repos.team_invitations.find_by_bot(&bot_id).await?;

    let mut user_ids: HashSet<String> = invitations.iter().map(|i| i.user_id.clone()).collect();

    user_ids.extend(bot.team.iter().cloned());

    let users_map = repos.users.find_many_by_ids(&user_ids).await?;

    let invitations_map: HashMap<String, TeamInvitation> = invitations
        .into_iter()
        .map(|i| (i.user_id.clone(), i))
        .collect();

    let team = user_ids
        .into_iter()
        .map(|user_id| {
            let user = users_map.get(&user_id);
            let invitation = invitations_map.get(&user_id);

            TeamResponse {
                avatar: user.and_then(|u| u.avatar.clone()),
                invitation_id: invitation.map(|i| i.invitation_id.clone()),
                pending_invitation: invitation.map(|i| !i.accepted).unwrap_or(false),
                registered: user.is_some(),
                user_id: user_id.clone(),
                username: user.map(|u| u.username.clone()),
            }
        })
        .collect::<Vec<_>>();

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
    id: Path<String>,
) -> ApiResult<Json<NewInvitationResponse>> {
    let bot_id = Snowflake::try_from(id.into_inner())?.into_inner();

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

    let ctx = &auth;

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

    if bot.team.len() as i32 == bot.teammates_limit {
        warn!(
            code = %LogCode::Conflict,
            bot_id = %bot_id,
            user_id = %body.user_id,
            "Team limit reached",
        );
        return Err(ApiError::LimitExceeded("Team limit reached".to_string()));
    }

    let discord_user = services.discord.get_bot(&body.user_id).await?;
    if let Some(is_bot) = discord_user.bot
        && is_bot
    {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            user_id = %body.user_id,
            "Cannot add a bot to a bot team",
        );
        return Err(ApiError::BadRequest(
            "Cannot add a bot to a bot team".to_string(),
        ));
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

    let invitation = TeamInvitation::new(&bot_id, &body.user_id);
    repos.team_invitations.insert(&invitation).await?;

    let response = TeamResponse {
        avatar: None,
        invitation_id: Some(invitation.invitation_id.clone()),
        pending_invitation: true,
        registered: false,
        user_id: body.user_id.clone(),
        username: None,
    };

    let owner = repos
        .users
        .find_by_id(&bot.owner_id)
        .await?
        .ok_or_else(|| {
            warn!(
                code = %LogCode::Mail,
                bot_id = %bot_id,
                user_id = %body.user_id,
                "Bot owner not found for sending team invitation email",
            );
            ApiError::NotFound(format!("Bot owner with ID {} not found", bot.owner_id))
        })?;

    let mut invitation_sent = true;

    match repos.users.find_by_id(&body.user_id).await? {
        Some(user) => {
            if let Err(e) = services
                .discord
                .send_dm(
                    &user.user_id,
                    Some(DiscordNotification::create(NotificationType::TeamInvite {
                        bot_username: bot.username.clone(),
                        owner_username: owner.username.clone(),
                        invitation_id: invitation.invitation_id.clone(),
                    })),
                )
                .await
            {
                error!(
                    code = %LogCode::Mail,
                    bot_id = %bot_id,
                    user_id = %body.user_id,
                    "Failed to send team invitation DM: {}",
                    e
                );

                invitation_sent = false;
            }

            #[cfg(feature = "mails")]
            if let Err(e) =
                services
                    .mail
                    .send_team_invite(&user, &owner, &bot, invitation.invitation_id)
            {
                error!(
                    code = %LogCode::Mail,
                    bot_id = %bot_id,
                    user_id = %body.user_id,
                    "Failed to send team invitation email: {}",
                    e
                );

                invitation_sent = false;
            }
        }
        None => {
            warn!(
                code = %LogCode::Mail,
                bot_id = %bot_id,
                user_id = %body.user_id,
                "User not found for sending team invitation email",
            );

            invitation_sent = false;
        }
    }

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        user_id = %body.user_id,
        "Added user to bot team",
    );

    Ok(Json(NewInvitationResponse {
        sent: invitation_sent,
        details: response,
    }))
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
    id: Path<String>,
) -> ApiResult<Json<MessageResponse>> {
    let bot_id = Snowflake::try_from(id.into_inner())?.into_inner();

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

    let ctx = &auth;

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
