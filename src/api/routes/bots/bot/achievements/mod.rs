mod reset;

use actix_web::web::{Data, Json, Path};
use apistos::{
    api_operation,
    web::{ServiceConfig, delete, get, patch, post, resource, scope},
};
use tracing::{info, warn};

use crate::{
    api::middleware::Authenticated,
    domain::{
        error::{ApiError, ApiResult},
        models::Achievement,
    },
    openapi::schemas::{
        AchievementCreationPayload, AchievementResponse, AchievementUpdatePayload,
        DeleteAchievementQuery, MessageResponse,
    },
    repository::{AchievementUpdate, Repositories},
    services::Services,
    utils::{discord::Snowflake, logger::LogCode},
};

#[api_operation(
    summary = "Get bot achievements",
    description = "Retrieve a list of achievements for a specific bot",
    tag = "Achievements"
)]
async fn get_bot_achievements(
    auth: Authenticated,
    services: Data<Services>,
    repos: Data<Repositories>,
    id: Path<String>,
) -> ApiResult<Json<Vec<AchievementResponse>>> {
    let bot_id = Snowflake::try_from(id.into_inner())?.into_inner();

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Fetching achievements for bot",
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
            "Admin access granted for bot achievements",
        );
    } else if ctx.is_bot() && ctx.token.as_deref() != Some(&bot.token) {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Bot attempting to access achievements of another bot",
        );
        return Err(ApiError::Forbidden);
    } else if ctx.is_user() {
        let user_id = ctx.user_id.as_deref().ok_or(ApiError::Unauthorized)?;
        if !services.auth.user_has_bot_access(user_id, &bot_id).await? {
            warn!(
                code = %LogCode::Forbidden,
                bot_id = %bot_id,
                user_id = %user_id,
                "User does not have access to bot achievements",
            );
            return Err(ApiError::Forbidden);
        }
    } else {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Access denied for bot achievements",
        );
        return Err(ApiError::Forbidden);
    }

    if bot.suspended {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Attempt to access achievements of suspended bot",
        );
        return Err(ApiError::BotSuspended);
    }

    let achievements = repos.achievements.find_by_bot_id(&bot_id).await?;

    let achievement_reponses = achievements
        .into_iter()
        .map(AchievementResponse::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "All bot achievements fetched successfully",
    );

    Ok(Json(achievement_reponses))
}

#[api_operation(
    summary = "Create an achievement",
    description = "Create a new achievement for a bot",
    tag = "Achievements"
)]
async fn create_achievement(
    auth: Authenticated,
    repos: Data<Repositories>,
    id: Path<String>,
    payload: Json<AchievementCreationPayload>,
) -> ApiResult<Json<AchievementResponse>> {
    let bot_id = Snowflake::try_from(id.into_inner())?.into_inner();

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Attempting to create achievement",
    );

    let bot = repos.bots.find_by_id(&bot_id).await?.ok_or_else(|| {
        info!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            "Bot not found for achievement creation",
        );
        ApiError::NotFound(format!("Bot with ID {} not found", bot_id))
    })?;

    let ctx = &auth;

    if ctx.is_admin() {
        info!(
            code = %LogCode::AdminAction,
            bot_id = %bot_id,
            "Admin access granted for creating achievement",
        );
    } else if ctx.is_user() {
        let user_id = ctx.user_id.as_deref().ok_or(ApiError::Unauthorized)?;
        if !bot.is_owner(user_id) {
            warn!(
                code = %LogCode::Forbidden,
                bot_id = %bot_id,
                user_id = %user_id,
                "User does not have access to create bot achievement",
            );
            return Err(ApiError::Forbidden);
        }
    } else if !ctx.is_user() {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Access denied for creating bot achievement",
        );
        return Err(ApiError::Forbidden);
    }

    if bot.suspended {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Attempt to create achievement for suspended bot",
        );
        return Err(ApiError::BotSuspended);
    }

    let payload = payload.into_inner();
    let mut achievement = Achievement::new(
        &bot_id,
        &payload.description,
        &payload.title,
        payload.objective,
    );
    if let Some(description_i18n) = payload.description_i18n {
        achievement = achievement.with_description_i18n(&description_i18n);
    }
    if let Some(from) = payload.from {
        repos.achievements.find_by_id(&from).await?.ok_or_else(|| {
            warn!(
                code = %LogCode::Request,
                bot_id = %bot_id,
                from_id = %from,
                "Referenced 'from' achievement not found for creation",
            );
            ApiError::NotFound(format!(
                "Referenced 'from' achievement with ID {} not found",
                from
            ))
        })?;

        if let Some(_) = repos
            .achievements
            .find_existing_by_bot(&bot_id, &from)
            .await?
        {
            warn!(
                code = %LogCode::Request,
                bot_id = %bot_id,
                from_id = %from,
                "Referenced 'from' achievement already exists for bot",
            );
            return Err(ApiError::AlreadyExists(format!(
                "Referenced 'from' achievement with ID {} already exists for bot",
                from
            )));
        }

        achievement = achievement.with_from(&from);

        let used_by_count = repos
            .achievements
            .count_used_by(&from)
            .await?
            .saturating_add(1);
        let update = AchievementUpdate::new().with_used_by(used_by_count as i64);
        repos.achievements.update(&from, update).await?;
    }
    if let Some(title_i18n) = payload.title_i18n {
        achievement = achievement.with_title_i18n(&title_i18n);
    }
    let insert_result = repos.achievements.insert(&achievement).await?;

    let inserted_oid = insert_result.inserted_id.as_object_id().ok_or_else(|| {
        ApiError::DatabaseError(format!(
            "Failed to get inserted achievement ID {}",
            insert_result.inserted_id
        ))
    })?;

    let result = repos
        .achievements
        .find_by_id(&inserted_oid.to_hex())
        .await?
        .ok_or_else(|| {
            info!(
                code = %LogCode::Request,
                bot_id = %bot_id,
                achievement_id = %inserted_oid,
                "Achievement not found after creation",
            );
            ApiError::DatabaseError(format!(
                "Achievement with ID {} not found after creation",
                insert_result.inserted_id
            ))
        })?;

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        achievement_id = %inserted_oid,
        "Achievement created successfully",
    );

    Ok(Json(AchievementResponse::try_from(result)?))
}

#[api_operation(
    summary = "Update an achievement",
    description = "Update an existing achievement for a bot",
    tag = "Achievements"
)]
async fn update_achievement(
    auth: Authenticated,
    repos: Data<Repositories>,
    id: Path<String>,
    payload: Json<AchievementUpdatePayload>,
) -> ApiResult<Json<AchievementResponse>> {
    let bot_id = Snowflake::try_from(id.into_inner())?.into_inner();

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        achievement_id = %payload.id,
        "Attempting to update achievement",
    );

    let bot = repos.bots.find_by_id(&bot_id).await?.ok_or_else(|| {
        info!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            achievement_id = %payload.id,
            "Bot not found for achievement update",
        );
        ApiError::NotFound(format!("Bot with ID {} not found", bot_id))
    })?;

    let ctx = &auth;

    if ctx.is_admin() {
        info!(
            code = %LogCode::AdminAction,
            bot_id = %bot_id,
            achievement_id = %payload.id,
            "Admin access granted for updating achievement",
        );
    } else if ctx.is_user() {
        let user_id = ctx.user_id.as_deref().ok_or(ApiError::Unauthorized)?;
        if !bot.is_owner(user_id) {
            warn!(
                code = %LogCode::Forbidden,
                bot_id = %bot_id,
                achievement_id = %payload.id,
                user_id = %user_id,
                "User does not have access to update bot achievement",
            );
            return Err(ApiError::Forbidden);
        }
    } else if !ctx.is_user() {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            achievement_id = %payload.id,
            "Access denied for updating bot achievement",
        );
        return Err(ApiError::Forbidden);
    }

    if bot.suspended {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            achievement_id = %payload.id,
            "Attempt to update achievement of suspended bot",
        );
        return Err(ApiError::BotSuspended);
    }

    let existing = repos
        .achievements
        .find_by_id(&payload.id)
        .await?
        .ok_or_else(|| {
            info!(
                code = %LogCode::Request,
                bot_id = %bot_id,
                achievement_id = %payload.id,
                "Achievement not found for update",
            );
            ApiError::NotFound(format!("Achievement with ID {} not found", payload.id))
        })?;

    if existing.from.is_some() {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            achievement_id = %payload.id,
            "Attempt to update achievement with from field set",
        );
        return Err(ApiError::Forbidden);
    }

    let payload = payload.into_inner();

    if let Some(shared) = payload.shared {
        if shared && !existing.shared {
            warn!(
                code = %LogCode::Forbidden,
                bot_id = %bot_id,
                achievement_id = %payload.id,
                "Attempt to update shared achievement to non-shared",
            );
            return Err(ApiError::Forbidden);
        }
    }

    let mut updates = AchievementUpdate::new();
    if let Some(description) = payload.description {
        updates = updates.with_description(description);
    }
    if let Some(lang) = payload.lang {
        updates = updates.with_lang(lang);
    }
    if let Some(title) = payload.title {
        updates = updates.with_title(title);
    }
    if let Some(shared) = payload.shared {
        updates = updates.with_shared(shared);
    }
    let update_result = repos
        .achievements
        .update(&payload.id, updates)
        .await?
        .ok_or_else(|| {
            info!(
                code = %LogCode::Request,
                bot_id = %bot_id,
                achievement_id = %payload.id,
                "Achievement not found for update after update attempt",
            );
            ApiError::DatabaseError(format!("Achievement with ID {} not found", payload.id))
        })?;

    Ok(Json(AchievementResponse::try_from(update_result)?))
}

#[api_operation(
    summary = "Delete an achievement",
    description = "Delete an existing achievement for a bot",
    tag = "Achievements"
)]
async fn delete_achievement(
    auth: Authenticated,
    repos: Data<Repositories>,
    id: Path<String>,
    query: Json<DeleteAchievementQuery>,
) -> ApiResult<Json<MessageResponse>> {
    let bot_id = Snowflake::try_from(id.into_inner())?.into_inner();

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        achievement_id = %query.id,
        "Attempting to delete achievement",
    );

    let bot = repos.bots.find_by_id(&bot_id).await?.ok_or_else(|| {
        info!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            "Bot not found for achievement deletion",
        );
        ApiError::NotFound(format!("Bot with ID {} not found", bot_id))
    })?;

    let ctx = &auth;

    if ctx.is_admin() {
        info!(
            code = %LogCode::AdminAction,
            bot_id = %bot_id,
            achievement_id = %query.id,
            "Admin access granted for deleting achievement",
        );
    } else if ctx.is_user() {
        let user_id = ctx.user_id.as_deref().ok_or(ApiError::Unauthorized)?;
        if !bot.is_owner(user_id) {
            warn!(
                code = %LogCode::Forbidden,
                bot_id = %bot_id,
                achievement_id = %query.id,
                user_id = %user_id,
                "User does not have access to delete bot achievement",
            );
            return Err(ApiError::Forbidden);
        }
    } else if !ctx.is_user() {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            achievement_id = %query.id,
            "Access denied for deleting bot achievement",
        );
        return Err(ApiError::Forbidden);
    }

    if bot.suspended {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            achievement_id = %query.id,
            "Attempt to delete achievement of suspended bot",
        );
        return Err(ApiError::BotSuspended);
    }

    let achievement = repos
        .achievements
        .find_by_id(&query.id)
        .await?
        .ok_or_else(|| {
            info!(
                code = %LogCode::Request,
                bot_id = %bot_id,
                achievement_id = %query.id,
                "Achievement not found for deletion",
            );
            ApiError::NotFound(format!("Achievement with ID {} not found", query.id))
        })?;

    if achievement.shared {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            achievement_id = %query.id,
            "Attempt to delete non-editable achievement",
        );
        return Err(ApiError::Forbidden);
    }

    if let Some(from) = achievement.from {
        let used_by_count = repos
            .achievements
            .count_used_by(&query.id)
            .await?
            .saturating_sub(1);
        let update = AchievementUpdate::new().with_used_by(used_by_count as i64);
        repos.achievements.update(&from, update).await?;
    }

    repos.achievements.delete_by_id(&query.id).await?;
    repos
        .achievements
        .update_many(&query.id, AchievementUpdate::new().with_from(None))
        .await?;

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        achievement_id = %query.id,
        "Achievement deleted successfully",
    );

    Ok(Json(MessageResponse {
        message: format!("Achievement with ID {} deleted successfully", query.id),
    }))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/achievements")
            .service(
                resource("")
                    .route(get().to(get_bot_achievements))
                    .route(post().to(create_achievement))
                    .route(patch().to(update_achievement))
                    .route(delete().to(delete_achievement)),
            )
            .configure(reset::configure),
    );
}
