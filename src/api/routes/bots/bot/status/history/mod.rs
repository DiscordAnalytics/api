use actix_web::web::{Data, Json, Path, Query};
use apistos::{
    api_operation,
    web::{ServiceConfig, get},
};
use bson::DateTime as BsonDateTime;
use chrono::{Duration, Utc};

use crate::{
    domain::error::{ApiError, ApiResult},
    openapi::schemas::{
        HistoryCursor, PaginationResponse, PublicStatusHistoryResponse, PublicStatusSnapshot,
        StatusHistoryQuery,
    },
    repository::Repositories,
    utils::{
        discord::Snowflake,
        status::{decode_status_cursor, encode_status_cursor},
    },
};

#[api_operation(
    summary = "Get status history",
    description = "Get the status history of the bot.",
    tag = "Status"
)]
async fn get_public_history(
    repos: Data<Repositories>,
    path: Path<String>,
    query: Query<StatusHistoryQuery>,
) -> ApiResult<Json<PublicStatusHistoryResponse>> {
    let bot_id = Snowflake::try_from(path.into_inner())?.into_inner();

    let StatusHistoryQuery {
        from,
        to,
        limit,
        cursor,
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

    if !(1..=500).contains(&limit) {
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

    let cursor: Option<HistoryCursor> = cursor.as_deref().map(decode_status_cursor).transpose()?;

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

    let data = history
        .checks
        .into_iter()
        .map(|stored| PublicStatusSnapshot::from(stored.check))
        .collect();

    Ok(Json(PublicStatusHistoryResponse {
        bot_id,
        data,
        pagination: PaginationResponse { limit, next_cursor },
    }))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.route("/history", get().to(get_public_history));
}
