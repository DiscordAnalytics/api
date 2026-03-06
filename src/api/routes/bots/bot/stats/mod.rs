use std::collections::HashMap;

use actix_web::web::{Data, Json, Query};
use apistos::{
    api_operation,
    web::{ServiceConfig, get, post, resource, scope},
};
use chrono::{Duration, Utc};
use mongodb::bson::DateTime;
use tracing::{info, warn};

use crate::{
    api::middleware::{Authenticated, Snowflake},
    domain::{
        error::{ApiError, ApiResult},
        models::AchievementType,
    },
    openapi::schemas::{
        BotStatsBody, BotStatsContent, BotStatsQuery, BotStatsResponse, MessageResponse,
        NormalizedStatsBody, VoteResponse,
    },
    repository::{BotStatsUpdate, Repositories},
    services::Services,
    utils::{constants::MAX_DATE_RANGE, logger::LogCode},
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
    id: Snowflake,
) -> ApiResult<Json<BotStatsResponse>> {
    let bot_id = id.0;

    let from = DateTime::parse_rfc3339_str(format!("{}T00:00:00Z", query.from).as_str()).map_err(
        |_| {
            warn!(
                code = %LogCode::Request,
                bot_id = %bot_id,
                from = %query.from,
                "Invalid 'from' date format",
            );
            ApiError::InvalidInput("Invalid 'from' date format. Expected YYYY-MM-DD.".to_string())
        },
    )?;
    let to =
        DateTime::parse_rfc3339_str(format!("{}T23:59:59Z", query.to).as_str()).map_err(|_| {
            warn!(
                code = %LogCode::Request,
                bot_id = %bot_id,
                to = %query.to,
                "Invalid 'to' date format",
            );
            ApiError::InvalidInput("Invalid 'to' date format. Expected YYYY-MM-DD.".to_string())
        })?;

    if from > to {
        warn!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            from = %query.from,
            to = %query.to,
            "Invalid date range: 'from' date is after 'to' date",
        );
        return Err(ApiError::InvalidInput(
            "'from' date must be before 'to' date".to_string(),
        ));
    }

    if to.timestamp_millis() - from.timestamp_millis() > MAX_DATE_RANGE * 1000 {
        warn!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            from = %query.from,
            to = %query.to,
            "Invalid date range: range exceeds maximum allowed",
        );
        return Err(ApiError::InvalidInput(format!(
            "Date range cannot exceed {} seconds",
            MAX_DATE_RANGE
        )));
    }

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
async fn post_stats(
    auth: Authenticated,
    repos: Data<Repositories>,
    body: Json<BotStatsBody>,
    id: Snowflake,
) -> ApiResult<Json<MessageResponse>> {
    let bot_id = id.0;

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

    let current_date = DateTime::now();
    let start_of_hour = DateTime::from_millis(
        current_date.timestamp_millis() - (current_date.timestamp_millis() % 3600000),
    );

    let body = match body.into_inner() {
        BotStatsBody::New(new_body) => {
            NormalizedStatsBody::from_new(new_body, &bot_id, &start_of_hour)
        }
        BotStatsBody::Old(old_body) => {
            NormalizedStatsBody::from_old(old_body, &bot_id, &start_of_hour)
        }
    };

    let new_stats = match repos
        .bot_stats
        .find_by_date(&bot_id, &start_of_hour)
        .await?
    {
        Some(existing_stats) => {
            let mut updates = BotStatsUpdate::new()
                .with_guild_count(body.guild_count)
                .with_user_count(body.user_count);

            if body.added_guilds != 0 {
                updates = updates.with_added_guilds(body.added_guilds);
            }

            if let Some(custom_events) = body.custom_events {
                let bot_events = repos
                    .custom_events
                    .find_by_bot_id(&bot_id)
                    .await?
                    .into_iter()
                    .map(|event| (event.event_key, event.default_value))
                    .collect::<HashMap<_, _>>();

                let existing_events = existing_stats.custom_events.unwrap_or_default();

                for (event_key, count) in custom_events {
                    if let Some(default_value) = bot_events.get(&event_key) {
                        let new_count = if existing_events.contains_key(&event_key) {
                            count
                        } else {
                            default_value.unwrap_or(count)
                        };

                        updates = updates.with_custom_event(&event_key, new_count);
                    }
                }
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
                .await?
                .ok_or_else(|| {
                    warn!(
                        code = %LogCode::Database,
                        bot_id = %bot_id,
                        "Failed to update existing bot stats",
                    );
                    ApiError::DatabaseError("Failed to update bot stats".to_string())
                })?
        }
        None => {
            let new_stats = body.into_stats();
            repos.bot_stats.insert(&new_stats).await?;
            new_stats
        }
    };

    let achievements = repos.achievements.find_unachieved_by_bot(&bot_id).await?;
    for mut achievement in achievements {
        let new_current = match achievement.objective.achievement_type {
            AchievementType::FrenchPercentage => {
                let (total, french_count) = new_stats.interactions_locales.iter().fold(
                    (0i64, 0i64),
                    |(total, french), locale_stat| {
                        let n = locale_stat.number as i64;
                        (
                            total + n,
                            french + if locale_stat.locale == "fr" { n } else { 0 },
                        )
                    },
                );
                if total > 0 {
                    let percentage = ((french_count as f64 / total as f64) * 100.0).round() as i64;
                    Some(percentage)
                } else {
                    None
                }
            }
            AchievementType::GuildCount => Some(new_stats.guild_count as i64),
            AchievementType::InteractionAverageWeek => {
                let one_month_ago = Utc::now() - Duration::days(30);
                let dt_month_ago = DateTime::from_millis(one_month_ago.timestamp_millis());
                let month_stats = repos
                    .bot_stats
                    .find_from_date_range(&bot_id, &dt_month_ago, &current_date)
                    .await?;

                let total_interactions: i64 = month_stats
                    .iter()
                    .flat_map(|stats| stats.interactions.iter())
                    .map(|interaction| interaction.number as i64)
                    .sum();

                if total_interactions > 0 {
                    const WEEKS_IN_RANGE: f64 = 30.0 / 7.0;
                    Some((total_interactions as f64 / WEEKS_IN_RANGE).round() as i64)
                } else {
                    None
                }
            }
            AchievementType::JoinedDa => {
                Some(current_date.timestamp_millis() - bot.watched_since.timestamp_millis())
            }
            AchievementType::UserCount => Some(new_stats.user_count as i64),
            AchievementType::UsersLocales => Some(new_stats.interactions_locales.len() as i64),
            _ => achievement.current,
        };

        achievement.current = new_current;
        if new_current.unwrap_or(0) >= achievement.objective.value {
            achievement.achieved_on = Some(DateTime::now());
        }

        repos
            .achievements
            .update_progress(
                &bot_id,
                &achievement
                    .id
                    .ok_or_else(|| anyhow::anyhow!("Achievement ID missing"))?
                    .to_string(),
                achievement.current,
                achievement.achieved_on,
            )
            .await?;
    }

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Posted bot stats",
    );

    Ok(Json(MessageResponse {
        message: "Bot stats updated successfully".to_string(),
    }))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/stats").service(
            resource("")
                .route(get().to(get_stats))
                .route(post().to(post_stats)),
        ),
    );
}
