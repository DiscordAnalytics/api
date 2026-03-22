mod api;
mod config;
mod domain;
mod managers;
mod openapi;
mod repository;
mod services;
mod utils;

use std::{net::Ipv4Addr, sync::Arc};

use actix_cors::Cors;
use actix_web::{App, HttpServer, http, rt, web::Data};
use anyhow::Result;
use apistos::app::OpenApiWrapper;
use chrono::{Duration as ChronoDuration, Utc};
use mongodb::bson::DateTime;
use tokio::{
    spawn,
    sync::Mutex,
    time::{Duration, interval},
    try_join,
};
use tracing::{Level, error, info};
use tracing_actix_web::TracingLogger;

use crate::{
    api::{middleware::AuthMiddleware, routes},
    config::env::init_env,
    managers::{ChatServer, VotesWebhooksManager},
    openapi::build_spec,
    repository::{BotUpdate, Repositories},
    services::Services,
    utils::{
        discord::{DiscordNotification, NotificationType},
        logger::{LogCode, Logger},
    },
};

#[actix_web::main]
async fn main() -> Result<()> {
    let dev_mode = cfg!(debug_assertions);

    init_env().expect("Failed to initialize environment variables");

    Logger::new()
        .with_level(if dev_mode { Level::DEBUG } else { Level::INFO })
        .init()
        .expect("Failed to initialize logger");

    info!(
        code = %LogCode::Server,
        "Starting app",
    );
    info!(
        code = %LogCode::Server,
        "Running in {} mode",
        if dev_mode {
            "development"
        } else {
            "production"
        }
    );

    let repos = Repositories::init().await?;
    info!(
        code = %LogCode::Server,
        "Repositories initialized",
    );

    let services = Services::new(repos.clone());
    info!(
        code = %LogCode::Server,
        "Services initialized",
    );

    let repos_clone = repos.clone();
    rt::spawn(async move {
        let repos_clone = repos_clone.clone();
        let mut interval = interval(Duration::from_secs(60 * 60));

        loop {
            interval.tick().await;

            match repos_clone.sessions.delete_expired().await {
                Ok(deleted_count) => info!(
                    code = %LogCode::Server,
                    deleted_count = %deleted_count,
                    "Deleted expired sessions",
                ),
                Err(e) => error!(
                    code = %LogCode::Server,
                    error = %e,
                    "Failed to delete expired sessions"
                ),
            }

            match repos_clone
                .team_invitations
                .delete_expired_invitations()
                .await
            {
                Ok(deleted_count) => info!(
                    code = %LogCode::Server,
                    deleted_count = %deleted_count,
                    "Deleted expired team invitations",
                ),
                Err(e) => error!(
                    code = %LogCode::Server,
                    error = %e,
                    "Failed to delete expired team invitations"
                ),
            }
        }
    });

    let repos_clone = repos.clone();
    let services_clone = services.clone();
    rt::spawn(async move {
        let repos_clone = repos_clone.clone();
        let services_clone = services_clone.clone();
        let mut interval = interval(Duration::from_secs(24 * 60 * 60));

        loop {
            interval.tick().await;

            if dev_mode {
                continue;
            }

            let not_configured = match repos_clone.bots.find_not_configured().await {
                Ok(bots) => bots,
                Err(e) => {
                    error!(
                        code = %LogCode::Server,
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
                let owner = match repos_clone.users.find_by_id(&bot.owner_id).await {
                    Ok(Some(user)) => user,
                    _ => continue,
                };

                let watched_since = bot.watched_since;
                let warn_level = bot.warn_level;

                if watched_since < six_days_ago && warn_level == 0 {
                    if let Err(e) = services_clone
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
                    if let Err(e) = services_clone
                        .mail
                        .send_bot_configuration_warning(&owner, &bot)
                    {
                        error!(
                            code = %LogCode::Mail,
                            bot_id = %bot.bot_id,
                            user_id = %owner.user_id,
                            error = %e,
                            "Failed to send bot configuration warning email to user",
                        );
                    }

                    let update = BotUpdate::new().with_warn_level(1);

                    match repos_clone.bots.update(&bot.bot_id, update).await {
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
                    if let Err(e) = services_clone
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
                    if let Err(e) = services_clone
                        .mail
                        .send_bot_configuration_deletion(&owner, &bot)
                    {
                        error!(
                            code = %LogCode::BotExpiration,
                            error = %e,
                            "Failed to send non-configured bot deletion email"
                        );
                    }

                    match services_clone.bots.delete_bot(&bot.bot_id).await {
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

            let inactive = match repos_clone.bots.find_inactive().await {
                Ok(bots) => bots,
                Err(e) => {
                    error!(
                        code = %LogCode::Server,
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
                let owner = match repos_clone.users.find_by_id(&bot.owner_id).await {
                    Ok(Some(user)) => user,
                    _ => continue,
                };

                let watched_since = bot.watched_since;
                let warn_level = bot.warn_level;

                if watched_since < five_months_ago && warn_level != 2 {
                    if let Err(e) = services_clone
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
                    if let Err(e) = services_clone.mail.send_bot_inactive_warning(&owner, &bot) {
                        error!(
                            code = %LogCode::Mail,
                            bot_id = %bot.bot_id,
                            user_id = %owner.user_id,
                            error = %e,
                            "Failed to send bot inactivity warning email to user",
                        );
                    }

                    let update = BotUpdate::new().with_warn_level(2);

                    match repos_clone.bots.update(&bot.bot_id, update).await {
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
                } else if watched_since < six_months_ago && warn_level == 2 {
                    if let Err(e) = services_clone
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
                    if let Err(e) = services_clone.mail.send_bot_inactive_deletion(&owner, &bot) {
                        error!(
                            code = %LogCode::BotExpiration,
                            error = %e,
                            "Failed to send inactive bot deletion email"
                        );
                    }

                    match services_clone.bots.delete_bot(&bot.bot_id).await {
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
    });

    let votes_webhooks_manager = Data::new(Arc::new(Mutex::new(VotesWebhooksManager::new())));
    info!(
        code = %LogCode::Server,
        "VotesWebhooksManager initialized",
    );

    let (chat_server, chat_server_handle) = ChatServer::new();
    let chat_server = spawn(chat_server.run());
    let chat_server_handle = Data::new(chat_server_handle);
    info!(
        code = %LogCode::Server,
        "ChatServer initialized",
    );

    let http_server = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&app_env!().client_url)
            .allowed_methods(vec!["GET", "POST", "DELETE", "PATCH"])
            .allowed_headers(vec![
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
                http::header::CONTENT_TYPE,
            ])
            .supports_credentials()
            .max_age(3600);

        let spec = build_spec();

        App::new()
            .document(spec)
            .app_data(Data::new(repos.clone()))
            .app_data(Data::new(services.clone()))
            .app_data(votes_webhooks_manager.clone())
            .app_data(chat_server_handle.clone())
            .wrap(TracingLogger::default())
            .wrap(cors)
            .wrap(AuthMiddleware)
            .configure(routes::configure)
            .build("/openapi.json")
    })
    .bind((Ipv4Addr::UNSPECIFIED, app_env!().port))?
    .run();

    info!(
        code = %LogCode::Server,
        "App started",
    );
    info!(
        code = %LogCode::Server,
        "Listening on port {}",
        app_env!().port,
    );
    info!(
        code = %LogCode::Server,
        "Access the API at {}",
        app_env!().api_url,
    );
    info!(
        code = %LogCode::Server,
        "Access the client at {}",
        app_env!().client_url,
    );

    try_join!(http_server, async { chat_server.await? })?;

    Ok(())
}
