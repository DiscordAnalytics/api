use actix_web::web::{Data, Json, Query};
use anyhow::Result;
use apistos::{
    api_operation,
    web::{ServiceConfig, get, resource, scope},
};
use mongodb::bson::DateTime;
use tracing::info;

use crate::{
    api::middleware::RequireAdmin,
    domain::error::ApiResult,
    openapi::schemas::{StatResponse, StatsQuery},
    repository::Repositories,
    utils::logger::LogCode,
};

#[api_operation(
    summary = "Get global stats for a date range",
    description = "Fetch global statistics for all bots and users in the Discord Analytics API for a specified date range.",
    tag = "Stats",
    skip
)]
async fn get_stats(
    _admin: RequireAdmin,
    repos: Data<Repositories>,
    query: Query<StatsQuery>,
) -> ApiResult<Json<Vec<StatResponse>>> {
    info!(
        code = %LogCode::Request,
        start = %query.start,
        end = %query.end,
        "Fetching global stats for date range",
    );

    let start = DateTime::parse_rfc3339_str(format!("{}T00:00:00Z", query.start).as_str())?;
    let end = DateTime::parse_rfc3339_str(format!("{}T23:59:59Z", query.end).as_str())?;

    let stats = repos
        .global_stats
        .find_from_date_range(&start, &end)
        .await?;

    let stat_responses = stats
        .into_iter()
        .map(StatResponse::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    info!(
        code = %LogCode::Request,
        count = stat_responses.len(),
        "Fetched global stats for date range",
    );

    Ok(Json(stat_responses))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(scope("/stats").service(resource("").route(get().to(get_stats))));
}
