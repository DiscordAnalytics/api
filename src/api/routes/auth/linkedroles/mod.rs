use actix_web::{
    HttpRequest, Responder,
    cookie::{Cookie, time::Duration},
    web::{Data, Query, Redirect},
};
use apistos::{
    api_operation,
    web::{ServiceConfig, get},
};
use mongodb::bson::Uuid;
use tracing::{error, info};

use crate::{
    app_env, openapi::schemas::LinkedRolesQuery, repository::Repositories, services::Services,
    utils::logger::LogCode,
};

fn connection_url(state: &str) -> String {
    format!(
        "https://discord.com/api/oauth2/authorize?client_id={}&redirect_uri={}/api/auth/linkedroles&response_type=code&state={}&scope=role_connections.write%20identify&prompt=consent",
        app_env!().client_id,
        app_env!().api_url,
        state
    )
}

#[api_operation(
    summary = "Get linked roles for the authenticated user",
    description = "Retrieves the linked roles associated with the authenticated user based on their OAuth credentials.",
    tag = "Auth",
    skip
)]
async fn oauth_callback(
    req: HttpRequest,
    services: Data<Services>,
    repos: Data<Repositories>,
    query: Query<LinkedRolesQuery>,
) -> impl Responder {
    info!(
        code = %LogCode::Auth,
        message = "Getting linked roles for the authenticated user."
    );

    if query.code.is_none() || req.cookie("clientState").is_none() {
        let state = Uuid::new().to_string();
        let cookie = Cookie::build("clientState", state.clone())
            .max_age(Duration::minutes(5))
            .http_only(true)
            .finish();

        return Redirect::to(connection_url(&state))
            .temporary()
            .customize()
            .append_header(("Set-Cookie", cookie.to_string()));
    }

    let code = query.code.as_deref().unwrap();
    let discord_state = query.state.as_deref().unwrap_or_default();
    let client_state = req.cookie("clientState").unwrap();

    if discord_state != client_state.value() {
        error!(
            code = %LogCode::Auth,
            message = "State mismatch in linked roles OAuth callback.",
            discord_state = %discord_state,
            client_state = %client_state.value()
        );
        return Redirect::to(format!(
            "{}/auth?error=invalide_state",
            app_env!().client_url
        ))
        .temporary()
        .customize();
    }

    let token_response = match services.discord.exchange_linked_roles_code(code).await {
        Ok(token) => token,
        Err(e) => {
            error!(
                code = %LogCode::Auth,
                error = %e,
                message = "Failed to exchange linked roles OAuth code."
            );
            return Redirect::to(format!(
                "{}/auth?error=token_exchange_failed",
                app_env!().client_url
            ))
            .temporary()
            .customize();
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
                message = "Failed to fetch Discord user after token exchange."
            );
            return Redirect::to(format!(
                "{}/auth?error=fetch_user_failed",
                app_env!().client_url
            ))
            .temporary()
            .customize();
        }
    };

    let bot_count = match repos.bots.count_by_user_id(&discord_user.id).await {
        Ok(count) => count as i32,
        Err(e) => {
            error!(
                code = %LogCode::Auth,
                error = %e,
                message = "Failed to count bots for user after fetching Discord user."
            );
            return Redirect::to(format!(
                "{}/auth?error=bot_count_failed",
                app_env!().client_url
            ))
            .temporary()
            .customize();
        }
    };

    if let Err(e) = services
        .discord
        .update_role_connection(
            &token_response.token_type,
            &token_response.access_token,
            bot_count,
        )
        .await
    {
        error!(
            code = %LogCode::Auth,
            error = %e,
            message = "Failed to update Discord role connection with bot count."
        );
        return Redirect::to(format!(
            "{}/auth?error=update_role_connection_failed",
            app_env!().client_url
        ))
        .temporary()
        .customize();
    }

    info!(
        code = %LogCode::Auth,
        message = "Successfully updated Discord role connection with bot count.",
        user_id = %discord_user.id,
        bot_count = bot_count
    );

    Redirect::to(format!("{}/auth/linked", app_env!().client_url))
        .temporary()
        .customize()
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.route("/linkedroles", get().to(oauth_callback));
}
