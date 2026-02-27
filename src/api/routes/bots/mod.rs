mod bot;

use actix_web::web::{Data, Json};
use anyhow::Result;
use apistos::{
    api_operation,
    web::{ServiceConfig, get, resource, scope},
};
use tracing::info;

use crate::{
    api::middleware::RequireAdmin, domain::error::ApiResult, openapi::schemas::BotResponse,
    repository::Repositories, utils::logger::LogCode,
};

#[api_operation(
    summary = "Get all bots",
    description = "Fetch a list of all bots registered in the Discord Analytics API",
    tag = "Bots",
    skip
)]
async fn get_all_bots(
    _admin: RequireAdmin,
    repos: Data<Repositories>,
) -> ApiResult<Json<Vec<BotResponse>>> {
    info!(
        code = %LogCode::Request,
        "Fetching all bots",
    );

    let bots = repos.bots.find_all().await?;

    let bot_responses = bots
        .into_iter()
        .map(BotResponse::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    info!(
        code = %LogCode::Request,
        "All bots fetched successfully",
    );

    Ok(Json(bot_responses))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/bots")
            .service(resource("").route(get().to(get_all_bots)))
            .configure(bot::configure),
    );
}
