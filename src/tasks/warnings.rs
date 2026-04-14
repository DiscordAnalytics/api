use chrono::{Duration as ChronoDuration, Utc};
use mongodb::bson::DateTime;
use tokio::{
    spawn,
    time::{Duration, interval},
};
use tracing::{error, info};

use crate::{
    repository::{BotUpdate, Repositories},
    services::Services,
    utils::{
        discord::{DiscordNotification, NotificationType},
        logger::LogCode,
    },
};

pub fn warnings_task(repos: Repositories, services: Services) {
    spawn(async move {
        let mut interval = interval(Duration::from_secs(24 * 60 * 60));

        loop {
            interval.tick().await;

            if cfg!(debug_assertions) {
                continue;
            }

            handle_not_configured(&repos, &services).await;
            handle_inactive(&repos, &services).await;
        }
    });
}

async fn handle_not_configured(repos: &Repositories, services: &Services) {
    let not_configured = match repos.bots.find_not_configured().await {
        Ok(bots) => bots,
        Err(e) => {
            error!(
                code = %LogCode::System,
                error = %e,
                "Failed to find not configured bots"
            );
            return;
        }
    };

    let six_days_ago = Utc::now() - ChronoDuration::days(6);
    let six_days_ago = DateTime::from_millis(six_days_ago.timestamp_millis());
    let one_week_ago = Utc::now() - ChronoDuration::weeks(1);
    let one_week_ago = DateTime::from_millis(one_week_ago.timestamp_millis());

    for bot in not_configured {
        let owner = match repos.users.find_by_id(&bot.owner_id).await {
            Ok(Some(user)) => user,
            _ => continue,
        };

        let watched_since = bot.watched_since;
        let warn_level = bot.warn_level;

        if watched_since < six_days_ago && warn_level == 0 {
            if let Err(e) = services
                .discord
                .send_dm(
                    &owner.user_id,
                    Some(DiscordNotification::create(
                        NotificationType::BotConfigurationWarning {
                            bot_username: bot.username.clone(),
                            bot_id: bot.bot_id.clone(),
                        },
                    )),
                )
                .await
            {
                error!(
                    code = %LogCode::BotExpiration,
                    owner_id = %owner.user_id,
                    bot_id = %bot.bot_id,
                    error = %e,
                    "Failed to warn owner for not configuring the bot",
                );
            }

            #[cfg(feature = "mails")]
            if let Err(e) = services.mail.send_bot_configuration_warning(&owner, &bot) {
                error!(
                    code = %LogCode::Mail,
                    bot_id = %bot.bot_id,
                    user_id = %owner.user_id,
                    error = %e,
                    "Failed to send bot configuration warning email to user",
                );
            }

            let update = BotUpdate::default().with_warn_level(1);

            match repos.bots.update(&bot.bot_id, update).await {
                Ok(Some(_)) => info!(
                    code = %LogCode::BotExpiration,
                    bot_id = %bot.bot_id,
                    "Bot has been warned for not being configured",
                ),
                _ => error!(
                    code = %LogCode::BotExpiration,
                    bot_id = %bot.bot_id,
                    "Failed to warn bot for not being configured",
                ),
            }
        } else if watched_since < one_week_ago && warn_level == 1 {
            if let Err(e) = services
                .discord
                .send_dm(
                    &owner.user_id,
                    Some(DiscordNotification::create(
                        NotificationType::BotConfigurationDeletion {
                            bot_username: bot.username.clone(),
                            bot_id: bot.bot_id.clone(),
                        },
                    )),
                )
                .await
            {
                error!(
                    code = %LogCode::BotExpiration,
                    error = %e,
                    "Failed to send non-configured bot deletion DM"
                );
            }

            #[cfg(feature = "mails")]
            if let Err(e) = services.mail.send_bot_configuration_deletion(&owner, &bot) {
                error!(
                    code = %LogCode::BotExpiration,
                    error = %e,
                    "Failed to send non-configured bot deletion email"
                );
            }

            match services.bots.delete_bot(&bot.bot_id).await {
                Ok(_) => info!(
                    code = %LogCode::BotExpiration,
                    bot_id = %bot.bot_id,
                    "Deleted non-configured bot"
                ),
                Err(e) => {
                    error!(
                        code = %LogCode::BotExpiration,
                        bot_id = %bot.bot_id,
                        error = %e,
                        "Failed to delete non-configured bot"
                    );
                }
            }
        }
    }
}

async fn handle_inactive(repos: &Repositories, services: &Services) {
    let inactive = match repos.bots.find_inactive().await {
        Ok(bots) => bots,
        Err(e) => {
            error!(
                code = %LogCode::System,
                error = %e,
                "Failed to find inactive bots"
            );
            return;
        }
    };

    let five_months_ago = Utc::now() - ChronoDuration::days(5 * 30);
    let five_months_ago = DateTime::from_millis(five_months_ago.timestamp_millis());
    let six_months_ago = Utc::now() - ChronoDuration::days(6 * 30);
    let six_months_ago = DateTime::from_millis(six_months_ago.timestamp_millis());

    for bot in inactive {
        if let Some(last_push) = bot.last_push {
            let owner = match repos.users.find_by_id(&bot.owner_id).await {
                Ok(Some(user)) => user,
                _ => continue,
            };

            let warn_level = bot.warn_level;

            if last_push < five_months_ago && warn_level != 2 {
                if let Err(e) = services
                    .discord
                    .send_dm(
                        &owner.user_id,
                        Some(DiscordNotification::create(
                            NotificationType::BotInactiveWarning {
                                bot_username: bot.username.clone(),
                                bot_id: bot.bot_id.clone(),
                            },
                        )),
                    )
                    .await
                {
                    error!(
                        code = %LogCode::BotExpiration,
                        owner_id = %owner.user_id,
                        bot_id = %bot.bot_id,
                        error = %e,
                        "Failed to warn owner for bot inactivity",
                    );
                };

                #[cfg(feature = "mails")]
                if let Err(e) = services.mail.send_bot_inactive_warning(&owner, &bot) {
                    error!(
                        code = %LogCode::Mail,
                        bot_id = %bot.bot_id,
                        user_id = %owner.user_id,
                        error = %e,
                        "Failed to send bot inactivity warning email to user",
                    );
                }

                let update = BotUpdate::default().with_warn_level(2);

                match repos.bots.update(&bot.bot_id, update).await {
                    Ok(Some(_)) => info!(
                        code = %LogCode::BotExpiration,
                        bot_id = %bot.bot_id,
                        "Bot has been warned for being inactive",
                    ),
                    _ => error!(
                        code = %LogCode::BotExpiration,
                        bot_id = %bot.bot_id,
                        "Failed to warn bot for being inactive",
                    ),
                }
            } else if last_push < six_months_ago && warn_level == 2 {
                if let Err(e) = services
                    .discord
                    .send_dm(
                        &owner.user_id,
                        Some(DiscordNotification::create(
                            NotificationType::BotInactiveDeletion {
                                bot_username: bot.username.clone(),
                                bot_id: bot.bot_id.clone(),
                            },
                        )),
                    )
                    .await
                {
                    error!(
                        code = %LogCode::BotExpiration,
                        error = %e,
                        "Failed to send inactive bot deletion DM"
                    );
                }

                #[cfg(feature = "mails")]
                if let Err(e) = services.mail.send_bot_inactive_deletion(&owner, &bot) {
                    error!(
                        code = %LogCode::BotExpiration,
                        error = %e,
                        "Failed to send inactive bot deletion email"
                    );
                }

                match services.bots.delete_bot(&bot.bot_id).await {
                    Ok(_) => info!(
                        code = %LogCode::BotExpiration,
                        bot_id = %bot.bot_id,
                        "Deleted inactive bot"
                    ),
                    Err(e) => {
                        error!(
                            code = %LogCode::BotExpiration,
                            bot_id = %bot.bot_id,
                            error = %e,
                            "Failed to delete inactive bot"
                        );
                    }
                }
            }
        }
    }
}
