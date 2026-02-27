mod refresh;
mod sessions;

use actix_web::{HttpRequest, web};
use apistos::{
    api_operation,
    web::{Redirect, ServiceConfig, get, resource, scope},
};
use mongodb::bson::{DateTime, Uuid};
use tracing::{error, info, warn};

use crate::{
    app_env,
    domain::{
        auth::{generate_access_token, generate_refresh_token, hash_refresh_token},
        models::{GlobalStats, Session, User},
    },
    openapi::schemas::AuthCallbackQuery,
    repository::{GlobalStatsUpdate, Repositories, UserUpdate},
    services::Services,
    utils::{discord::get_user_creation_date, logger::LogCode},
};

#[api_operation(
    summary = "OAuth callback endpoint",
    description = "Handles Discord OAuth callback and processes the authorization code.",
    tag = "Auth",
    skip
)]
async fn oauth_callback(
    req: HttpRequest,
    services: web::Data<Services>,
    repos: web::Data<Repositories>,
    query: web::Query<AuthCallbackQuery>,
) -> Redirect {
    info!(
        code = %LogCode::Auth,
        "OAuth callback received"
    );

    let token_response = match services
        .discord
        .exchange_code(&query.code, &query.redirection, &query.scopes)
        .await
    {
        Ok(token) => token,
        Err(e) => {
            error!(
                code = %LogCode::Auth,
                error = %e,
                "Failed to exchange OAuth code"
            );
            return Redirect::to(format!(
                "{}/auth?error=token_exchange_failed",
                app_env!().client_url
            ))
            .temporary();
        }
    };

    let discord_user = match services
        .discord
        .get_user(&token_response.token_type, &token_response.access_token)
        .await
    {
        Ok(user) => user,
        Err(e) => {
            error!(
                code = %LogCode::Auth,
                error = %e,
                "Failed to fetch Discord user info"
            );
            return Redirect::to(format!(
                "{}/auth?error=fetch_user_failed",
                app_env!().client_url
            ))
            .temporary();
        }
    };

    if discord_user.discriminator != "0" {
        warn!(
            code = %LogCode::Auth,
            user_id = %discord_user.id,
            "User with old username system attempted login"
        );
        return Redirect::to(format!(
            "{}/auth?error=unsupported_user",
            app_env!().client_url
        ));
    }

    let existing_user = repos
        .users
        .find_by_id(&discord_user.id)
        .await
        .ok()
        .flatten();

    let user_id = discord_user.id.clone();

    match existing_user {
        Some(db_user) => {
            if db_user.banned {
                warn!(
                    code = %LogCode::Auth,
                    user_id = %user_id,
                    "Banned user attempted login"
                );
                return Redirect::to(format!("{}/auth?error=banned_user", app_env!().client_url))
                    .permanent();
            }

            let mut user_update = UserUpdate::new();
            if let Some(avatar) = discord_user.avatar.as_deref() {
                user_update = user_update.with_avatar(avatar.to_string());
            }
            if let Some(decoration) = discord_user
                .avatar_decoration_data
                .as_ref()
                .and_then(|data| data.asset.as_deref())
            {
                user_update = user_update.with_avatar_decoration(decoration.to_string());
            }
            if let Some(mail) = discord_user.email.as_deref() {
                user_update = user_update.with_mail(mail.to_string());
            }
            user_update = user_update.with_username(discord_user.username.clone());

            if let Err(e) = repos.users.update(&user_id, user_update).await {
                error!(
                    code = %LogCode::Auth,
                    user_id = %user_id,
                    error = %e,
                    "Failed to update existing user"
                );
            }

            info!(
                code = %LogCode::Auth,
                user_id = %user_id,
                "Existing user logged in successfully"
            );
        }
        None => {
            if !app_env!().enable_registrations {
                warn!(
                    code = %LogCode::Auth,
                    user_id = %user_id,
                    "Registration attempt while registrations are disabled"
                );
                return Redirect::to(format!(
                    "{}/auth?error=registrations_disabled",
                    app_env!().client_url
                ))
                .temporary();
            }

            let user_created_at = get_user_creation_date(&user_id).unwrap_or_else(DateTime::now);

            let new_user = User {
                avatar: discord_user.avatar.unwrap_or_default(),
                avatar_decoration: discord_user
                    .avatar_decoration_data
                    .and_then(|data| data.asset),
                banned: false,
                bots_limit: 3,
                created_at: user_created_at,
                joined_at: DateTime::now(),
                mail: discord_user.email.unwrap_or_default(),
                username: discord_user.username.clone(),
                user_id: user_id.clone(),
            };

            if let Err(e) = repos.users.insert(&new_user).await {
                error!(
                    code = %LogCode::Auth,
                    user_id = %user_id,
                    error = %e,
                    "Failed to create new user"
                );
                return Redirect::to(format!(
                    "{}/auth?error=registration_failed",
                    app_env!().client_url
                ))
                .temporary();
            }

            let current_date = DateTime::now();
            let start_of_hour = DateTime::from_millis(
                current_date.timestamp_millis() - (current_date.timestamp_millis() % 3600000),
            );

            match repos.global_stats.find_one(&start_of_hour).await {
                Ok(Some(stats)) => {
                    let updated_stats =
                        GlobalStatsUpdate::new().with_user_count(stats.user_count + 1);
                    if let Err(e) = repos
                        .global_stats
                        .update(&start_of_hour, updated_stats)
                        .await
                    {
                        error!(
                            code = %LogCode::Auth,
                            error = %e,
                            "Failed to update global stats for new user registration"
                        );
                    }
                }
                _ => {
                    let total_bots = repos.bots.count_bots().await.unwrap_or(0) as i32;
                    let total_users = repos.users.count_users().await.unwrap_or(0) as i32;
                    let new_stats = GlobalStats {
                        bot_count: total_bots,
                        date: start_of_hour,
                        registered_bots: 0,
                        user_count: total_users,
                    };
                    if let Err(e) = repos.global_stats.insert(&new_stats).await {
                        error!(
                            code = %LogCode::Auth,
                            error = %e,
                            "Failed to create global stats for new user registration"
                        );
                    }
                }
            }

            info!(
               code = %LogCode::Auth,
               user_id = %user_id,
               "New user registered successfully"
            );
        }
    }

    let refresh_token = match generate_refresh_token(&user_id, &Uuid::new().to_string()) {
        Ok(token) => token,
        Err(e) => {
            error!(
              code = %LogCode::Auth,
              user_id = %user_id,
              error = %e,
              "Failed to generate refresh token"
            );
            return Redirect::to(format!(
                "{}/auth?error=token_generation_failed",
                app_env!().client_url
            ))
            .temporary();
        }
    };
    let refresh_token_hash = hash_refresh_token(&refresh_token);

    let mut session = Session::new(user_id.clone(), refresh_token_hash);

    if let Some(user_agent) = req.headers().get("User-Agent") {
        if let Ok(ua) = user_agent.to_str() {
            session = session.with_user_agent(ua.to_string());
        }
    }
    if let Some(ip) = req.peer_addr() {
        session = session.with_ip(ip.ip().to_string());
    }

    if let Err(e) = repos.sessions.insert(&session).await {
        error!(
            code = %LogCode::Auth,
            user_id = %user_id,
            error = %e,
            "Failed to create session for user"
        );
        return Redirect::to(format!(
            "{}/auth?error=session_creation_failed",
            app_env!().client_url
        ))
        .temporary();
    }

    let access_token = match generate_access_token(&user_id, &session.session_id) {
        Ok(token) => token,
        Err(e) => {
            error!(
                code = %LogCode::Auth,
                user_id = %user_id,
                error = %e,
                "Failed to generate access token"
            );
            return Redirect::to(format!(
                "{}/auth?error=token_generation_failed",
                app_env!().client_url
            ))
            .temporary();
        }
    };

    info!(
        code = %LogCode::Auth,
        user_id = %user_id,
        "User authenticated successfully, redirecting with tokens"
    );

    Redirect::to(format!(
        "{}/auth?code=ok&accessToken={}&refreshToken={}&id={}",
        app_env!().client_url,
        access_token,
        refresh_token,
        user_id,
    ))
    .temporary()
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/auth")
            .service(resource("").route(get().to(oauth_callback)))
            .configure(refresh::configure)
            .configure(sessions::configure),
        // .configure(linkedroles::configure),
    );
}
