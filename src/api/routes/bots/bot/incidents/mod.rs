mod incident;

use actix_web::web::{Data, Json, Path, Query};
use apistos::{
    api_operation,
    web::{ServiceConfig, get, scope},
};

use crate::{
    domain::error::{ApiError, ApiResult},
    openapi::schemas::{
        IncidentCursor, IncidentListQuery, PaginationResponse, PublicIncidentListResponse,
        PublicIncidentResponse,
    },
    repository::Repositories,
    utils::{
        discord::Snowflake,
        logger::LogCode,
        status::{decode_status_cursor, encode_status_cursor},
    },
};

#[api_operation(summary = "", description = "", tag = "Status")]
async fn get_incidents(
    repos: Data<Repositories>,
    path: Path<String>,
    query: Query<IncidentListQuery>,
) -> ApiResult<Json<PublicIncidentListResponse>> {
    let bot_id = Snowflake::try_from(path.into_inner())?.into_inner();

    info!(
        code = %LogCode::Status,
        bot_id = %bot_id,
        "Attempting to get public incidents for bot"
    );

    let IncidentListQuery {
        incident_type,
        status,
        limit,
        cursor,
    } = query.into_inner();

    let limit = limit.unwrap_or(10);

    if !(1..=100).contains(&limit) {
        return Err(ApiError::InvalidInput(
            "limit must be between 1 and 100".to_string(),
        ));
    }

    let bot = repos.bots.find_by_id(&bot_id).await?.ok_or_else(|| {
        info!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            "Bot not found for public incidents request",
        );
        ApiError::NotFound(format!("Bot with ID {} not found", bot_id))
    })?;

    if bot.suspended {
        warn!(
            code = %LogCode::Request,
            bot_id = %bot.bot_id,
            "Bot suspended for public incidents request",
        );
        return Err(ApiError::NotFound(format!(
            "Bot with ID {} not found",
            bot.bot_id
        )));
    }

    let cursor: Option<IncidentCursor> = cursor.as_deref().map(decode_status_cursor).transpose()?;

    let page = repos
        .status_incidents
        .find_public_incidents(&bot_id, incident_type, status, limit, cursor)
        .await?;

    let next_cursor = if page.has_more {
        page.incidents
            .last()
            .map(|last| encode_status_cursor(last.incident.created_at, last.id))
    } else {
        None
    };

    let data = page
        .incidents
        .into_iter()
        .map(|stored| PublicIncidentResponse::from((stored.id.to_hex(), stored.incident)))
        .collect();

    Ok(Json(PublicIncidentListResponse {
        bot_id,
        data,
        pagination: PaginationResponse { limit, next_cursor },
    }))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/incidents")
            .route("", get().to(get_incidents))
            .configure(incident::configure),
    );
}
