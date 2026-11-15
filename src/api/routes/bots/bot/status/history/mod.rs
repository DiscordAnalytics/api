use actix_web::web::{Data, Json, Path, Query};
use apistos::{
    api_operation,
    web::{ServiceConfig, get},
};
use chrono::{Duration, Utc};
use mongodb::bson::DateTime as BsonDateTime;

use crate::{
    domain::error::{ApiError, ApiResult},
    openapi::schemas::{
        HistoryCursor, HistoryResolution, HourlyHistoryCursor, PaginationResponse,
        PublicHourlyHistoryResponse, PublicHourlySnapshot, PublicStatusHistoryResponse,
        PublicStatusSnapshot, StatusHistoryQuery, StatusHistoryResponse,
    },
    repository::Repositories,
    utils::{
        discord::Snowflake,
        status::{decode_status_cursor, encode_status_cursor},
    },
};

#[api_operation(
    summary = "Get status history",
    description = "Get the status history of the bot. Use `resolution=hourly` for wide date ranges to retrieve pre-aggregated hourly data in far fewer requests.",
    tag = "Status"
)]
async fn get_public_history(
    repos: Data<Repositories>,
    path: Path<String>,
    query: Query<StatusHistoryQuery>,
) -> ApiResult<Json<StatusHistoryResponse>> {
    let bot_id = Snowflake::try_from(path.into_inner())?.into_inner();

    let StatusHistoryQuery {
        from,
        to,
        limit,
        cursor,
        resolution,
    } = query.into_inner();

    let now = Utc::now();
    let from = from.unwrap_or(now - Duration::days(30));
    let to = to.unwrap_or(now);

    if from > to {
        return Err(ApiError::BadRequest(
            "from must be less than or equal to to".to_string(),
        ));
    }

    let limit = limit.unwrap_or(100);
    if !matches!(limit, 1..=500) {
        return Err(ApiError::BadRequest(
            "limit must be between 1 and 500".to_string(),
        ));
    }

    let bot = repos
        .bots
        .find_by_id(&bot_id)
        .await?
        .ok_or(ApiError::NotFound(format!(
            "Bot with ID {} not found",
            bot_id
        )))?;

    if bot.suspended {
        return Err(ApiError::NotFound(format!(
            "Bot with ID {} not found",
            bot_id
        )));
    }

    let resolution = resolution.unwrap_or(HistoryResolution::Raw);

    match resolution {
        HistoryResolution::Raw => {
            let cursor: Option<HistoryCursor> =
                cursor.as_deref().map(decode_status_cursor).transpose()?;

            let history = repos
                .status_check
                .find_history(
                    &bot_id,
                    BsonDateTime::from_chrono(from),
                    BsonDateTime::from_chrono(to),
                    limit,
                    cursor,
                )
                .await?;

            let next_cursor = if history.has_more {
                history
                    .checks
                    .last()
                    .map(|last| encode_status_cursor(last.check.timestamp, last.id))
            } else {
                None
            };

            let data: Vec<PublicStatusSnapshot> = history
                .checks
                .into_iter()
                .map(|stored| PublicStatusSnapshot::from(stored.check))
                .collect();

            Ok(Json(StatusHistoryResponse::Raw(
                PublicStatusHistoryResponse {
                    bot_id,
                    data,
                    pagination: PaginationResponse { limit, next_cursor },
                },
            )))
        }

        HistoryResolution::Hourly => {
            let cursor: Option<HourlyHistoryCursor> =
                cursor.as_deref().map(decode_status_cursor).transpose()?;

            let history = repos
                .status_check_hourly
                .find_history(
                    &bot_id,
                    BsonDateTime::from_chrono(from),
                    BsonDateTime::from_chrono(to),
                    limit,
                    cursor,
                )
                .await?;

            let next_cursor = if history.has_more {
                history
                    .checks
                    .last()
                    .map(|last| encode_status_cursor(last.check.hour_timestamp, last.id))
            } else {
                None
            };

            let data: Vec<PublicHourlySnapshot> = history
                .checks
                .into_iter()
                .map(|stored| PublicHourlySnapshot::from(stored.check))
                .collect();

            Ok(Json(StatusHistoryResponse::Hourly(
                PublicHourlyHistoryResponse {
                    bot_id,
                    data,
                    pagination: PaginationResponse { limit, next_cursor },
                },
            )))
        }
    }
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.route("/history", get().to(get_public_history));
}
