use actix_web::web::{Data, Json, Path, Query};
use apistos::{
    api_operation,
    web::{ServiceConfig, get, post, resource, scope},
};
use mongodb::bson::DateTime;
use tracing::{info, warn};

use crate::{
    api::middleware::Authenticated,
    domain::error::{ApiError, ApiResult},
    openapi::schemas::{
        BotStatsBodyNew, BotStatsBodyOld, BotStatsContent, BotStatsQuery, BotStatsResponse,
        BotStatsUpdateResponse, VoteResponse,
    },
    repository::{BotStatsUpdate, Repositories},
    services::Services,
    utils::{discord::is_valid_snowflake, logger::LogCode},
};

#[api_operation(
    summary = "Get bot stats",
    description = "Get the stats of a bot within a specified date range.",
    tag = "Stats"
)]
async fn get_stats(
    auth: Authenticated,
    services: Data<Services>,
    repos: Data<Repositories>,
    query: Query<BotStatsQuery>,
    id: Path<String>,
) -> ApiResult<Json<BotStatsResponse>> {
    let bot_id = id.into_inner();

    if !is_valid_snowflake(&bot_id) {
        return Err(ApiError::InvalidId);
    }

    let from = DateTime::parse_rfc3339_str(format!("{}T00:00:00Z", query.from).as_str())?;
    let to = DateTime::parse_rfc3339_str(format!("{}T23:59:59Z", query.to).as_str())?;

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        from = %query.from,
        to = %query.to,
        "Fetching bot stats for date range",
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
            "Access denied for suspended bot team",
        );
        return Err(ApiError::BotSuspended);
    }

    let ctx = &auth.0;

    if ctx.is_admin() {
        info!(
            code = %LogCode::AdminAction,
            bot_id = %bot_id,
            "Admin access granted for bot stats",
        );
    } else if ctx.is_bot() && ctx.bot_id.as_deref() != Some(&bot_id) {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Bot attempting to access stats of another bot",
        );
        return Err(ApiError::Forbidden);
    } else if ctx.is_user() {
        let user_id = ctx.user_id.as_deref().ok_or(ApiError::Unauthorized)?;
        if !services.auth.user_has_bot_access(user_id, &bot_id).await? {
            warn!(
                code = %LogCode::Forbidden,
                bot_id = %bot_id,
                user_id = %user_id,
                "User does not have access to bot stats",
            );
            return Err(ApiError::Forbidden);
        }
    } else {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Access denied for bot stats",
        );
        return Err(ApiError::Forbidden);
    }

    let stats = repos
        .bot_stats
        .find_from_date_range(&bot_id, &from, &to)
        .await?;

    let stat_responses = stats
        .into_iter()
        .map(BotStatsContent::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    let votes = repos
        .votes
        .find_from_date_range(&bot_id, &from, &to)
        .await?;

    let vote_responses = votes
        .into_iter()
        .map(VoteResponse::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        count = stat_responses.len(),
        "Fetched bot stats for date range",
    );

    Ok(Json(BotStatsResponse {
        stats: stat_responses,
        votes: vote_responses,
    }))
}

#[api_operation(
    summary = "Post bot stats",
    description = "Submit bot stats for a specific date.",
    tag = "Stats"
)]
async fn post_stats_old(
    auth: Authenticated,
    repos: Data<Repositories>,
    body: Json<BotStatsBodyOld>,
    id: Path<String>,
) -> ApiResult<Json<BotStatsUpdateResponse>> {
    let bot_id = id.into_inner();

    if !is_valid_snowflake(&bot_id) {
        return Err(ApiError::InvalidId);
    }

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Posting bot stats",
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
            "Access denied for suspended bot team",
        );
        return Err(ApiError::BotSuspended);
    }

    let ctx = &auth.0;

    if ctx.is_admin() {
        info!(
            code = %LogCode::AdminAction,
            bot_id = %bot_id,
            "Admin access granted for posting bot stats",
        );
    } else if ctx.is_bot() && ctx.bot_id.as_deref() != Some(&bot_id) {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Bot attempting to post stats for another bot",
        );
        return Err(ApiError::Forbidden);
    } else if !ctx.is_admin() && !ctx.is_bot() {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Access denied for posting bot stats",
        );
        return Err(ApiError::Forbidden);
    }

    let body = body.into_inner();

    let current_date = DateTime::now();
    let start_of_hour = DateTime::from_millis(
        current_date.timestamp_millis() - (current_date.timestamp_millis() % 3600000),
    );

    match repos
        .bot_stats
        .find_by_date(&bot_id, &start_of_hour)
        .await?
    {
        Some(_) => {
            let mut updates = BotStatsUpdate::new()
                .with_guild_count(body.guilds)
                .with_user_count(body.users);

            if body.added_guilds != 0 {
                updates = updates.with_added_guilds(body.added_guilds);
            }

            if let Some(custom_events) = body.custom_events {
                updates = custom_events
                    .into_iter()
                    .fold(updates, |u, (event_name, count)| {
                        u.with_custom_event(&event_name, count)
                    });
            }

            if let Some(guilds) = body.guilds_stats {
                updates = updates.with_guilds(&guilds);
            }

            let guilds_locales: Vec<(&str, i32)> = body
                .guilds_locales
                .iter()
                .map(|locale_stat| (locale_stat.locale.as_str(), locale_stat.number))
                .collect();
            updates = updates.with_guild_locales(&guilds_locales);

            updates = body
                .guild_members
                .into_iter()
                .fold(updates, |u, (bucket, count)| {
                    u.with_guild_member(&bucket, count)
                });

            updates = updates.with_interactions(&body.interactions);

            let interactions_locales: Vec<(&str, i32)> = body
                .locales
                .iter()
                .map(|locale_stat| (locale_stat.locale.as_str(), locale_stat.number))
                .collect();
            updates = updates.with_interactions_locales(&interactions_locales);

            if body.removed_guilds != 0 {
                updates = updates.with_removed_guilds(body.removed_guilds);
            }

            if let Some(user_install_count) = body.user_install_count {
                updates = updates.with_user_install_count(user_install_count);
            }

            if let Some(users_types) = body.users_type {
                updates = users_types
                    .into_iter()
                    .fold(updates, |u, (user_type, count)| {
                        u.with_user_type(&user_type, count)
                    });
            }

            repos
                .bot_stats
                .update(&bot_id, &start_of_hour, updates)
                .await?;
        }
        None => {
            let new_stats = BotStatsBodyOld::into(body, &bot_id, &start_of_hour);
            repos.bot_stats.insert(&new_stats).await?;
        }
    };

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Posted bot stats",
    );

    Ok(Json(BotStatsUpdateResponse {
        message: "Bot stats updated successfully".to_string(),
    }))
}

#[api_operation(
    summary = "Post bot stats",
    description = "Submit bot stats for a specific date.",
    tag = "Stats"
)]
#[allow(dead_code)]
async fn post_stats_new(
    auth: Authenticated,
    repos: Data<Repositories>,
    body: Json<BotStatsBodyNew>,
    id: Path<String>,
) -> ApiResult<Json<BotStatsUpdateResponse>> {
    let bot_id = id.into_inner();

    if !is_valid_snowflake(&bot_id) {
        return Err(ApiError::InvalidId);
    }

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Posting bot stats",
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
            "Access denied for suspended bot team",
        );
        return Err(ApiError::BotSuspended);
    }

    let ctx = &auth.0;

    if ctx.is_admin() {
        info!(
            code = %LogCode::AdminAction,
            bot_id = %bot_id,
            "Admin access granted for posting bot stats",
        );
    } else if ctx.is_bot() && ctx.bot_id.as_deref() != Some(&bot_id) {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Bot attempting to post stats for another bot",
        );
        return Err(ApiError::Forbidden);
    } else if !ctx.is_admin() && !ctx.is_bot() {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Access denied for posting bot stats",
        );
        return Err(ApiError::Forbidden);
    }

    let body = body.into_inner();

    let current_date = DateTime::now();
    let start_of_hour = DateTime::from_millis(
        current_date.timestamp_millis() - (current_date.timestamp_millis() % 3600000),
    );

    match repos
        .bot_stats
        .find_by_date(&bot_id, &start_of_hour)
        .await?
    {
        Some(_) => {
            let mut updates = BotStatsUpdate::new()
                .with_guild_count(body.guild_count)
                .with_user_count(body.user_count);

            if body.added_guilds != 0 {
                updates = updates.with_added_guilds(body.added_guilds);
            }

            if let Some(custom_events) = body.custom_events {
                updates = custom_events
                    .into_iter()
                    .fold(updates, |u, (event_name, count)| {
                        u.with_custom_event(&event_name, count)
                    });
            }

            if let Some(guilds) = body.guilds {
                updates = updates.with_guilds(&guilds);
            }

            let guilds_locales: Vec<(&str, i32)> = body
                .guild_locales
                .iter()
                .map(|locale_stat| (locale_stat.locale.as_str(), locale_stat.number))
                .collect();
            updates = updates.with_guild_locales(&guilds_locales);

            updates = body
                .guild_members
                .into_iter()
                .fold(updates, |u, (bucket, count)| {
                    u.with_guild_member(&bucket, count)
                });

            updates = updates.with_interactions(&body.interactions);

            let interactions_locales: Vec<(&str, i32)> = body
                .interactions_locales
                .iter()
                .map(|locale_stat| (locale_stat.locale.as_str(), locale_stat.number))
                .collect();
            updates = updates.with_interactions_locales(&interactions_locales);

            if body.removed_guilds != 0 {
                updates = updates.with_removed_guilds(body.removed_guilds);
            }

            if let Some(user_install_count) = body.user_install_count {
                updates = updates.with_user_install_count(user_install_count);
            }

            if let Some(users_types) = body.users_type {
                updates = users_types
                    .into_iter()
                    .fold(updates, |u, (user_type, count)| {
                        u.with_user_type(&user_type, count)
                    });
            }

            repos
                .bot_stats
                .update(&bot_id, &start_of_hour, updates)
                .await?;
        }
        None => {
            let new_stats = BotStatsBodyNew::into(body, &bot_id, &start_of_hour);
            repos.bot_stats.insert(&new_stats).await?;
        }
    };

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Posted bot stats",
    );

    Ok(Json(BotStatsUpdateResponse {
        message: "Bot stats updated successfully".to_string(),
    }))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/stats").service(
            resource("")
                .route(get().to(get_stats))
                .route(post().to(post_stats_old)),
        ),
    );
}
