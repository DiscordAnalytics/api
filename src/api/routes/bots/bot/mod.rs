mod achievements;
mod events;
mod settings;
mod stats;
mod suspend;
mod team;
mod token;

use actix_web::web::{Data, Json};
use apistos::{
    api_operation,
    web::{ServiceConfig, delete, get, patch, post, resource, scope},
};
use mongodb::bson::DateTime;
use tracing::{error, info, warn};

use crate::{
    api::middleware::{Authenticated, Snowflake},
    domain::{
        auth::generate_bot_token,
        error::{ApiError, ApiResult},
        models::{Achievement, AchievementType, Bot},
    },
    openapi::schemas::{
        BotCreationBody, BotDeletionPayload, BotResponse, BotUpdateBody, MessageResponse,
    },
    repository::{BotUpdate, Repositories},
    services::Services,
    utils::{
        discord::{DiscordNotification, NotificationType},
        logger::LogCode,
    },
};

#[api_operation(
    summary = "Get bot details",
    description = "Fetch detailed information about a specific bot registered in the Discord Analytics API",
    tag = "Bots"
)]
async fn get_bot(
    auth: Authenticated,
    services: Data<Services>,
    repos: Data<Repositories>,
    id: Snowflake,
) -> ApiResult<Json<BotResponse>> {
    let bot_id = id.0;

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Fetching details for bot",
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
            "Admin access granted for bot details",
        );
    } else if ctx.is_bot() && ctx.token.as_deref() != Some(&bot.token) {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Bot attempting to access details of another bot",
        );
        return Err(ApiError::Forbidden);
    } else if ctx.is_user() {
        let user_id = ctx.user_id.as_deref().ok_or(ApiError::Unauthorized)?;
        if !services.auth.user_has_bot_access(user_id, &bot_id).await? {
            warn!(
                code = %LogCode::Forbidden,
                bot_id = %bot_id,
                user_id = %user_id,
                "User does not have access to bot details",
            );
            return Err(ApiError::Forbidden);
        }
    } else {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Access denied for bot details",
        );
        return Err(ApiError::Forbidden);
    }

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Bot details fetched successfully",
    );

    Ok(Json(BotResponse::try_from(bot)?))
}

#[api_operation(
    summary = "Create a new bot",
    description = "Register a new bot in the Discord Analytics API. This endpoint generates a unique token for the bot, which is required for authentication in future requests.",
    tag = "Bots"
)]
async fn post_bot(
    auth: Authenticated,
    services: Data<Services>,
    repos: Data<Repositories>,
    body: Json<BotCreationBody>,
    id: Snowflake,
) -> ApiResult<Json<BotResponse>> {
    let bot_id = id.0;

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Attempting to create bot",
    );

    let ctx = &auth.0;

    if !ctx.is_admin() && !ctx.is_user() {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            auth_type = ?ctx.auth_type,
            "Unauthorized bot creation attempt",
        );
        return Err(ApiError::Forbidden);
    }

    let body_data = body.into_inner();

    if ctx.is_user() {
        let user_id = ctx.user_id.as_deref().ok_or(ApiError::Unauthorized)?;
        if user_id != body_data.user_id {
            warn!(
                code = %LogCode::Forbidden,
                bot_id = %bot_id,
                user_id = %user_id,
                "User ID in request does not match authenticated user",
            );
            return Err(ApiError::Forbidden);
        }
    }

    if repos.bots.find_by_id(&bot_id).await?.is_some() {
        warn!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            "Bot with this ID already exists",
        );
        return Err(ApiError::AlreadyExists(format!(
            "Bot with ID {} already exists",
            bot_id
        )));
    }

    if services
        .users
        .has_reached_bots_limit(&body_data.user_id)
        .await?
    {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            user_id = %body_data.user_id,
            "User has reached bots limit and cannot create more",
        );
        return Err(ApiError::Forbidden);
    }

    let bot_details = services.discord.get_bot(&bot_id).await?;
    if !bot_details.bot.unwrap_or(false) {
        warn!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            "User ID provided is not a bot according to Discord API",
        );
        return Err(ApiError::InvalidInput(format!(
            "User ID {} is not a bot according to Discord API",
            bot_id
        )));
    }

    let token = generate_bot_token(&bot_id)?;
    let bot = Bot::new(
        &bot_id,
        &body_data.user_id,
        token,
        &bot_details.username,
        bot_details.avatar.as_deref(),
    );

    repos.bots.insert(&bot).await?;

    let default_achievements = Achievement::defaults(&bot_id);
    repos
        .achievements
        .insert_many(&default_achievements)
        .await?;

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Bot created successfully",
    );

    Ok(Json(BotResponse::try_from(bot)?))
}

#[api_operation(
    summary = "Update bot details",
    description = "Update specific details of a bot registered in the Discord Analytics API. Only certain fields can be updated.",
    tag = "Bots"
)]
async fn patch_bot(
    auth: Authenticated,
    repos: Data<Repositories>,
    body: Json<BotUpdateBody>,
    id: Snowflake,
) -> ApiResult<Json<BotResponse>> {
    let bot_id = id.0;

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Attempting to update bot",
    );

    let ctx = &auth.0;

    if !(ctx.is_admin() || ctx.is_bot() && ctx.bot_id.as_deref() == Some(bot_id.as_str())) {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            auth_type = ?ctx.auth_type,
            "Unauthorized bot update attempt",
        );
        return Err(ApiError::Forbidden);
    }

    let bot = repos.bots.find_by_id(&bot_id).await?.ok_or_else(|| {
        info!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            "Bot not found for update",
        );
        ApiError::NotFound(format!("Bot with ID {} not found", bot_id))
    })?;

    if bot.suspended {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Access denied for suspended bot update",
        );
        return Err(ApiError::BotSuspended);
    }

    if ctx.is_bot() {
        let auth_token = ctx.token.as_deref().ok_or(ApiError::InvalidToken)?;
        if bot.token() != auth_token {
            warn!(
                code = %LogCode::InvalidToken,
                bot_id = %bot_id,
                "Bot token mismatch during update",
            );
            return Err(ApiError::InvalidToken);
        }
    }

    let update_data = body.into_inner();

    let mut update = BotUpdate::new();
    if let Some(avatar) = update_data.avatar {
        update = update.with_avatar(avatar);
    }
    if let Some(framework) = update_data.framework {
        update = update.with_framework(framework);
    }
    if let Some(team) = update_data.team {
        update = update.with_team(team);
    }
    if let Some(username) = update_data.username {
        update = update.with_username(username);
    }
    if let Some(version) = update_data.version {
        update = update.with_version(version);
    }

    let update_result = repos.bots.update(&bot_id, update).await?;

    let updated_bot = update_result.ok_or_else(|| {
        warn!(
            code = %LogCode::DbError,
            bot_id = %bot_id,
            "Bot not found after update",
        );
        ApiError::DatabaseError(format!("Bot with ID {} not found after update", bot_id))
    })?;

    let achievements = repos.achievements.find_unachieved_by_bot(&bot_id).await?;
    for mut achievement in achievements {
        if achievement.objective.achievement_type == AchievementType::BotConfigured {
            achievement.current = Some(achievement.objective.value);
            achievement.achieved_on = Some(DateTime::now());
            repos
                .achievements
                .update_progress(
                    &bot_id,
                    &achievement.id.to_string(),
                    achievement.current,
                    achievement.achieved_on,
                )
                .await?;
        }
    }

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Bot update successful",
    );

    Ok(Json(BotResponse::try_from(updated_bot)?))
}

#[api_operation(
    summary = "Delete a bot",
    description = "Delete a specific bot from the Discord Analytics API. This action is irreversible.",
    tag = "Bots"
)]
async fn delete_bot(
    auth: Authenticated,
    services: Data<Services>,
    repos: Data<Repositories>,
    id: Snowflake,
    payload: Option<Json<BotDeletionPayload>>,
) -> ApiResult<Json<MessageResponse>> {
    let bot_id = id.0;

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Attempting to delete bot",
    );

    #[cfg_attr(not(feature = "mails"), allow(unused_variables))]
    let bot = repos.bots.find_by_id(&bot_id).await?.ok_or_else(|| {
        info!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            "Bot not found for deletion",
        );
        ApiError::NotFound(format!("Bot with ID {} not found", bot_id))
    })?;

    let ctx = &auth.0;

    if ctx.is_admin() {
        info!(
            code = %LogCode::AdminAction,
            bot_id = %bot_id,
            "Admin access granted for bot deletion",
        );
    } else if ctx.is_bot() {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Bot attempting to delete a bot",
        );
        return Err(ApiError::Forbidden);
    } else if ctx.is_user() {
        let user_id = ctx.user_id.as_deref().ok_or(ApiError::Unauthorized)?;
        if !services.auth.user_owns_bot(user_id, &bot_id).await? {
            warn!(
                code = %LogCode::Forbidden,
                bot_id = %bot_id,
                user_id = %user_id,
                "User does not own bot and cannot delete",
            );
            return Err(ApiError::Forbidden);
        }
    } else {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Access denied for bot deletion",
        );
        return Err(ApiError::Forbidden);
    }

    repos.bots.find_by_id(&bot_id).await?.ok_or_else(|| {
        info!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            "Bot not found for deletion",
        );
        ApiError::NotFound(format!("Bot with ID {} not found", bot_id))
    })?;

    services.bots.delete_bot(&bot_id).await?;

    if ctx.is_admin() {
        let owner = repos
            .users
            .find_by_id(&bot.owner_id)
            .await?
            .ok_or_else(|| {
                warn!(
                    code = %LogCode::DbError,
                    bot_id = %bot_id,
                    user_id = %bot.owner_id,
                    "Bot owner not found for deletion email",
                );
                ApiError::DatabaseError(format!(
                    "Owner with ID {} not found for deletion email",
                    bot.owner_id
                ))
            })?;

        if let Some(payload) = payload {
            let payload_data = payload.into_inner();
            let reason = payload_data
                .reason
                .unwrap_or_else(|| "Deleted by admin".to_string());

            if let Err(e) = services
                .discord
                .send_dm(
                    &owner.user_id,
                    None,
                    Some(DiscordNotification::create(
                        NotificationType::BotDeletedByAdmin {
                            bot_username: bot.username.clone(),
                            bot_id: bot_id.clone(),
                            reason: reason.clone(),
                        },
                    )),
                )
                .await
            {
                error!(
                    code = %LogCode::Mail,
                    bot_id = %bot_id,
                    user_id = %owner.user_id,
                    error = ?e,
                    "Failed to send bot deletion DM to owner",
                );
            }

            #[cfg(feature = "mails")]
            if let Err(e) = services
                .mail
                .send_bot_deleted_by_admin(&owner, &bot, &reason)
            {
                error!(
                    code = %LogCode::Mail,
                    bot_id = %bot_id,
                    user_id = %owner.user_id,
                    error = ?e,
                    "Failed to send bot deletion email to owner",
                );
            }
        }
    }

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Bot successfully deleted",
    );

    Ok(Json(MessageResponse {
        message: format!("Bot with ID {} has been deleted", bot_id),
    }))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/{id}")
            .service(
                resource("")
                    .route(get().to(get_bot))
                    .route(post().to(post_bot))
                    .route(patch().to(patch_bot))
                    .route(delete().to(delete_bot)),
            )
            .configure(achievements::configure)
            .configure(events::configure)
            .configure(settings::configure)
            .configure(stats::configure)
            .configure(suspend::configure)
            .configure(team::configure)
            .configure(token::configure),
    );
}
