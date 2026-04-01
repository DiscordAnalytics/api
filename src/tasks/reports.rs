use std::{collections::HashSet, time::Duration as StdDuration};

use anyhow::{Result, anyhow};
use chrono::{
    DateTime as ChronoDateTime, Datelike, Duration as ChronoDuration, Local, NaiveDate, TimeZone,
    Weekday,
};
use mongodb::bson::DateTime as MongoDateTime;
use tokio::{spawn, time::sleep};
use tracing::{error, info};

use crate::{
    domain::models::StatsReportFrequency,
    repository::Repositories,
    services::Services,
    utils::{
        logger::LogCode,
        reports::{compute_stats, draw_chart},
    },
};

fn time_to_sleep_until(now: ChronoDateTime<Local>) -> Result<StdDuration> {
    let today_9 = Local
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 9, 0, 0)
        .earliest()
        .ok_or_else(|| anyhow!("invalid local time (DST issue)"))?;

    let target = if now >= today_9 {
        let tomorrow = now
            .date_naive()
            .succ_opt()
            .ok_or_else(|| anyhow!("date overflow"))?;

        Local
            .with_ymd_and_hms(tomorrow.year(), tomorrow.month(), tomorrow.day(), 9, 0, 0)
            .earliest()
            .ok_or_else(|| anyhow!("invalid local time (DST issue)"))?
    } else {
        today_9
    };

    (target - now)
        .to_std()
        .map_err(|_| anyhow!("negative duration"))
}

fn is_last_day_of_month(date: NaiveDate) -> bool {
    date.succ_opt().is_none_or(|d| d.month() != date.month())
}

pub fn reports_task(repos: Repositories, services: Services) {
    spawn(async move {
        loop {
            let now = Local::now();
            let sleep_until = match time_to_sleep_until(now) {
                Ok(dur) => dur,
                Err(e) => {
                    error!(
                        code = %LogCode::System,
                        error = ?e,
                        "Sleep calculation failed",
                    );
                    StdDuration::from_secs(60)
                }
            };

            sleep(sleep_until).await;

            if cfg!(debug_assertions) {
                continue;
            }

            let scheduled_time = Local::now();
            if scheduled_time.weekday() == Weekday::Sun {
                handle_reports(&repos, &services, StatsReportFrequency::Weekly).await;
            }

            let today = scheduled_time.date_naive();
            if is_last_day_of_month(today) {
                handle_reports(&repos, &services, StatsReportFrequency::Monthly).await;
            }

            info!(
                code = %LogCode::Task,
                "Reports task executed",
            );
        }
    });
}

async fn handle_reports(
    repos: &Repositories,
    services: &Services,
    frequency: StatsReportFrequency,
) {
    let subscriptions = match repos
        .stats_reports
        .find_by_frequency(frequency.as_str())
        .await
    {
        Ok(subscriptions) => subscriptions,
        Err(e) => {
            error!(
                code = %LogCode::System,
                error = ?e,
                frequency = %frequency.as_str(),
                "Failed to find subscriptions"
            );
            return;
        }
    };

    let bot_ids: HashSet<String> = subscriptions
        .iter()
        .map(|subscription| subscription.bot_id.clone())
        .collect();
    let user_ids: HashSet<String> = subscriptions
        .iter()
        .map(|subscription| subscription.user_id.clone())
        .collect();

    let bots = match repos.bots.find_many_by_ids(&bot_ids).await {
        Ok(bots) => bots,
        Err(e) => {
            error!(
                code = %LogCode::System,
                error = ?e,
                "Failed to find bots"
            );
            return;
        }
    };
    let users = match repos.users.find_many_by_ids(&user_ids).await {
        Ok(users) => users,
        Err(e) => {
            error!(
                code = %LogCode::System,
                error = ?e,
                "Failed to find users"
            );
            return;
        }
    };

    let now = Local::now();
    let from = match frequency {
        StatsReportFrequency::Weekly => now - ChronoDuration::days(7 * 2),
        StatsReportFrequency::Monthly => now - ChronoDuration::days(30 * 2),
    };
    let to = match frequency {
        StatsReportFrequency::Weekly => now - ChronoDuration::days(7),
        StatsReportFrequency::Monthly => now - ChronoDuration::days(30),
    };

    let mongo_from = MongoDateTime::from_millis(from.timestamp_millis());
    let mongo_to = MongoDateTime::from_millis(to.timestamp_millis());
    let mongo_now = MongoDateTime::now();

    for subscription in subscriptions {
        let previous_stats = match repos
            .bot_stats
            .find_from_date_range(&subscription.bot_id, &mongo_from, &mongo_to)
            .await
        {
            Ok(stats) => stats,
            Err(e) => {
                error!(
                    code = %LogCode::Database,
                    error = ?e,
                    "Failed to find previous bot stats"
                );
                continue;
            }
        };

        let current_stats = match repos
            .bot_stats
            .find_from_date_range(&subscription.bot_id, &mongo_to, &mongo_now)
            .await
        {
            Ok(stats) => stats,
            Err(e) => {
                error!(
                    code = %LogCode::Database,
                    error = ?e,
                    "Failed to find current bot stats"
                );
                continue;
            }
        };

        let days_count = (to - from).num_days();
        let (previous_interactions, previous_guilds, previous_users) =
            compute_stats(previous_stats, mongo_from, days_count as usize);
        let (current_interactions, current_guilds, current_users) =
            compute_stats(current_stats, mongo_to, days_count as usize);

        let interactions_chart = match draw_chart(
            previous_interactions,
            current_interactions,
            days_count as usize,
        ) {
            Ok(chart) => chart,
            Err(e) => {
                error!(
                    code = %LogCode::Report,
                    error = ?e,
                    "Failed to draw interactions chart"
                );
                continue;
            }
        };
        let guilds_chart = match draw_chart(previous_guilds, current_guilds, days_count as usize) {
            Ok(chart) => chart,
            Err(e) => {
                error!(
                    code = %LogCode::Report,
                    error = ?e,
                    "Failed to draw guilds chart"
                );
                continue;
            }
        };
        let users_chart = match draw_chart(previous_users, current_users, days_count as usize) {
            Ok(chart) => chart,
            Err(e) => {
                error!(
                    code = %LogCode::Report,
                    error = ?e,
                    "Failed to draw users chart"
                );
                continue;
            }
        };

        match repos
            .r2
            .put_png(&interactions_chart.0, &interactions_chart.1)
            .await
        {
            Ok(()) => {}
            Err(e) => {
                error!(
                    code = %LogCode::Report,
                    error = ?e,
                    "Failed to upload interactions chart"
                );
            }
        }
        match repos.r2.put_png(&guilds_chart.0, &guilds_chart.1).await {
            Ok(()) => {}
            Err(e) => {
                error!(
                    code = %LogCode::Report,
                    error = ?e,
                    "Failed to upload guilds chart"
                );
            }
        }
        match repos.r2.put_png(&users_chart.0, &users_chart.1).await {
            Ok(()) => {}
            Err(e) => {
                error!(
                    code = %LogCode::Report,
                    error = ?e,
                    "Failed to upload users chart"
                );
            }
        }

        let user = match users.get(&subscription.user_id) {
            Some(user) => user,
            None => {
                error!(
                    code = %LogCode::Report,
                    "Failed to get user for stats report"
                );
                return;
            }
        };
        let bot = match bots.get(&subscription.bot_id) {
            Some(bot) => bot,
            None => {
                error!(
                    code = %LogCode::Report,
                    "Failed to get bot for stats report"
                );
                return;
            }
        };

        match services.mail.send_stats_reports(
            user,
            bot,
            &frequency,
            &interactions_chart.0,
            &guilds_chart.0,
            &users_chart.0,
        ) {
            Ok(_) => {
                info!(
                    code = %LogCode::Report,
                    user_id = %subscription.user_id,
                    bot_id = %subscription.bot_id,
                    frequency = %frequency.as_str(),
                    "Sent stats report successfully"
                );
            }
            Err(e) => {
                error!(
                    code = %LogCode::Report,
                    error = ?e,
                    user_id = %subscription.user_id,
                    bot_id = %subscription.bot_id,
                    frequency = %frequency.as_str(),
                    "Failed to send stats report"
                );
            }
        }
    }
}
