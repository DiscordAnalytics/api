use actix_web::web::{self, Json};
use apistos::{
    api_operation,
    web::{ServiceConfig, get, resource, scope},
};
use tracing::info;

use crate::{
    api::middleware::Authenticated,
    domain::error::{ApiError, ApiResult},
    openapi::schemas::BotResponse,
    repository::Repositories,
    services::Services,
    utils::logger::LogCode,
};

#[api_operation(
    summary = "Get bot details",
    description = "Fetch detailed information about a specific bot registered in the Discord Analytics API",
    tag = "Bots"
)]
async fn get_bot(
    auth: Authenticated,
    services: web::Data<Services>,
    repos: web::Data<Repositories>,
    id: web::Path<String>,
) -> ApiResult<Json<BotResponse>> {
    let bot_id = id.into_inner();

    info!(
      code = %LogCode::Request,
      "Fetching details for bot with ID: {}",
      bot_id,
    );

    if auth.0.is_user()
        && !services
            .auth
            .user_has_bot_access(&auth.0.user_id.clone().unwrap_or_default(), &bot_id)
            .await?
    {
        return Err(ApiError::Forbidden);
    }

    if auth.0.is_bot() && auth.0.bot_id.as_deref() != Some(&bot_id) {
        return Err(ApiError::Forbidden);
    }

    let bot = repos.bots.find_by_id(&bot_id).await?;
    if bot.is_none() {
        info!(
          code = %LogCode::Request,
          "Bot with ID {} not found",
          bot_id,
        );
        return Err(ApiError::NotFound(format!(
            "Bot with ID {} not found",
            bot_id
        )));
    }
    let bot = bot.unwrap();

    Ok(Json(BotResponse {
        advanced_stats: bot.advanced_stats.into(),
        avatar: bot.avatar,
        banned: bot.suspended.into(),
        bot_id: bot.bot_id,
        framework: bot.framework,
        goals_limit: bot.goals_limit,
        language: bot.language,
        last_push: bot
            .last_push
            .map(|dt| dt.try_to_rfc3339_string())
            .transpose()?,
        owner_id: bot.owner_id.into(),
        team: bot.team.into(),
        token: bot.token.into(),
        username: bot.username.into(),
        version: bot.version,
        votes_webhook_url: bot.votes_webhook_url,
        watched_since: bot
            .watched_since
            .map(|dt| dt.try_to_rfc3339_string())
            .transpose()?,
    }))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(scope("/{id}").service(resource("").route(get().to(get_bot))));
}
