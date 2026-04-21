mod config;
mod linkedroles;
mod refresh;
mod sessions;

use actix_web::{
    HttpRequest,
    web::{Data, Query},
};
use apistos::{
    api_operation,
    web::{Redirect, ServiceConfig, get, resource, scope},
};
use mongodb::bson::{DateTime, Uuid};
use tracing::{error, info, warn};

use crate::{
    app_env,
    domain::{
        auth::{generate_token, hash_refresh_token},
        models::{Session, User},
    },
    openapi::schemas::AuthCallbackQuery,
    repository::{Repositories, UserUpdate},
    services::Services,
    utils::{
        constants::{ACCESS_TOKEN_LIFETIME, MAX_BOTS_PER_USER, REFRESH_TOKEN_LIFETIME},
        discord::get_user_creation_date,
        logger::LogCode,
    },
};

#[api_operation(
    summary = "OAuth callback endpoint",
    description = "Handles Discord OAuth callback and processes the authorization code.",
    tag = "Auth",
    skip
)]
async fn oauth_callback(
    req: HttpRequest,
    services: Data<Services>,
    repos: Data<Repositories>,
    query: Query<AuthCallbackQuery>,
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
            if db_user.suspended {
                warn!(
                    code = %LogCode::Auth,
                    user_id = %user_id,
                    "Banned user attempted login"
                );
                return Redirect::to(format!(
                    "{}/auth?error=suspended_user",
                    app_env!().client_url
                ))
                .permanent();
            }

            let decoration = discord_user
                .avatar_decoration_data
                .as_ref()
                .and_then(|data| data.asset.clone());
            let mut user_update = UserUpdate::default()
                .with_avatar(discord_user.avatar)
                .with_username(discord_user.username.clone())
                .with_avatar_decoration(decoration);
            if let Some(mail) = discord_user.email.as_deref() {
                user_update = user_update.with_mail(mail.to_string());
            }

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
                avatar: discord_user.avatar.clone(),
                avatar_decoration: discord_user
                    .avatar_decoration_data
                    .and_then(|data| data.asset),
                suspended: false,
                bots_limit: MAX_BOTS_PER_USER,
                created_at: user_created_at,
                joined_at: DateTime::now(),
                mail: if cfg!(feature = "mails") {
                    discord_user.email
                } else {
                    None
                },
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

            info!(
                code = %LogCode::Auth,
                user_id = %user_id,
                "New user registered successfully"
            );
        }
    }

    let session_id = Uuid::new().to_string();
    let refresh_token = match generate_token(&user_id, &session_id, REFRESH_TOKEN_LIFETIME) {
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

    let mut session = Session::new(user_id.clone(), refresh_token_hash, session_id);

    if let Some(user_agent) = req.headers().get("User-Agent")
        && let Ok(ua) = user_agent.to_str()
    {
        session = session.with_user_agent(ua.to_string());
    }
    if let Some(ip) = req.connection_info().realip_remote_addr() {
        session = session.with_ip(ip.to_string());
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

    let access_token = match generate_token(&user_id, &session.session_id, ACCESS_TOKEN_LIFETIME) {
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
        "{}/auth?code=ok&accessToken={}&refreshToken={}&expiresIn={}&id={}",
        app_env!().client_url,
        access_token,
        refresh_token,
        ACCESS_TOKEN_LIFETIME,
        user_id,
    ))
    .temporary()
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/auth")
            .service(resource("").route(get().to(oauth_callback)))
            .configure(config::configure)
            .configure(linkedroles::configure)
            .configure(refresh::configure)
            .configure(sessions::configure),
    );
}
