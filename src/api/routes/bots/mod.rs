use actix_web::web::{self, Json};
use anyhow::Result;
use apistos::{
    api_operation,
    web::{ServiceConfig, get},
};

use crate::{
    api::middleware::RequireAdmin, domain::error::ApiResult, openapi::schemas::BotResponse,
    repository::Repositories,
};

#[api_operation(
    summary = "Get all bots",
    description = "Fetch a list of all bots registered in the Discord Analytics API",
    tag = "Bots"
)]
async fn get_all_bots(
    _admin: RequireAdmin,
    repos: web::Data<Repositories>,
) -> ApiResult<Json<Vec<BotResponse>>> {
    let bots = repos.bots.find_all().await?;

    let bot_responses: Vec<BotResponse> = bots
        .into_iter()
        .map(|bot| -> Result<BotResponse> {
            Ok(BotResponse {
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
            })
        })
        .collect::<Result<Vec<BotResponse>>>()?;
    Ok(Json(bot_responses))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.route("/bots", get().to(get_all_bots));
}
