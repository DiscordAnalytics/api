mod incident;

use actix_web::web::{Data, Json, Path, Query};
use apistos::{
    actix::CreatedJson,
    api_operation,
    web::{ServiceConfig, get, post, scope},
};
use mongodb::bson::DateTime as BsonDateTime;

use crate::{
    api::middleware::Authenticated,
    domain::{
        error::{ApiError, ApiResult},
        models::{AnnouncementStatus, AnnouncementType, StatusIncident},
    },
    openapi::schemas::{
        CreateIncidentRequest, IncidentCursor, IncidentListQuery, ManagedIncidentListResponse,
        ManagedIncidentResponse, PaginationResponse,
    },
    repository::Repositories,
    utils::{
        discord::Snowflake,
        logger::LogCode,
        status::{decode_status_cursor, encode_status_cursor},
    },
};

#[api_operation(
    summary = "Get bot incidents",
    description = "Retrieves all incidents for a bot, including private incidents. Supports filtering by incident type and status, with cursor-based pagination. Requires authorization to manage the bot's incidents.",
    tag = "Status"
)]
async fn get_incidents(
    auth: Authenticated,
    repos: Data<Repositories>,
    path: Path<String>,
    query: Query<IncidentListQuery>,
) -> ApiResult<Json<ManagedIncidentListResponse>> {
    let bot_id = Snowflake::try_from(path.into_inner())?.into_inner();

    info!(
        code = %LogCode::Status,
        bot_id = %bot_id,
        "Attempting to get incidents for bot"
    );

    let IncidentListQuery {
        incident_type,
        status,
        limit,
        cursor,
    } = query.into_inner();

    let limit = limit.unwrap_or(10);

    if !matches!(limit, 1..=100) {
        return Err(ApiError::InvalidInput(
            "limit must be between 1 and 100".into(),
        ));
    }

    let bot = repos.bots.find_by_id(&bot_id).await?.ok_or_else(|| {
        info!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            "Bot not found for incidents retrieval",
        );
        ApiError::NotFound(format!("Bot with ID {} not found", bot_id))
    })?;

    let ctx = &auth;

    if ctx.is_admin() {
        info!(
            code = %LogCode::AdminAction,
            bot_id = %bot_id,
            "User is admin, proceeding with incidents retrieval",
        );
    } else if ctx.is_bot() {
        if ctx.token.as_deref() != Some(&bot.token) {
            warn!(
                code = %LogCode::Forbidden,
                bot_id = %bot_id,
                "Bot attempted to retrieve incidents of a different bot",
            );
            return Err(ApiError::Forbidden);
        }
    } else if ctx.is_user() {
        let user_id = ctx.user_id.as_deref().ok_or(ApiError::Unauthorized)?;
        if !bot.has_access(user_id) {
            warn!(
                code = %LogCode::Forbidden,
                bot_id = %bot_id,
                user_id = %user_id,
                "User does not have access to bot incidents retrieval",
            );
            return Err(ApiError::Forbidden);
        }
    } else {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Access denied for bot incidents retrieval",
        );
        return Err(ApiError::Forbidden);
    }

    if bot.suspended && !ctx.is_admin() {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Access denied for suspended bot incidents retrieval",
        );
        return Err(ApiError::BotSuspended);
    }

    let cursor: Option<IncidentCursor> = cursor.as_deref().map(decode_status_cursor).transpose()?;

    let page = repos
        .status_incidents
        .find_incidents(&bot_id, incident_type, status, limit, cursor, false)
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
        .map(|stored| ManagedIncidentResponse::from((stored.id.to_hex(), stored.incident)))
        .collect();

    Ok(Json(ManagedIncidentListResponse {
        bot_id,
        data,
        pagination: PaginationResponse { limit, next_cursor },
    }))
}

fn validate_incident_request(request: &mut CreateIncidentRequest) -> ApiResult<()> {
    if request.title.trim().is_empty() {
        return Err(ApiError::InvalidInput("title cannot be empty".into()));
    }

    if request.description.trim().is_empty() {
        return Err(ApiError::InvalidInput("description cannot be empty".into()));
    }

    match request.incident_type {
        AnnouncementType::Incident => {
            if request.severity.is_none() {
                return Err(ApiError::InvalidInput(
                    "severity is required for incidents".to_string(),
                ));
            }

            if request.scheduled_start_at.is_some() || request.scheduled_end_at.is_some() {
                return Err(ApiError::InvalidInput(
                    "scheduled dates are not allowed for incidents".to_string(),
                ));
            }
        }
        AnnouncementType::Maintenance => {
            request.severity = None;
        }
    }

    if let (Some(start), Some(end)) = (request.scheduled_start_at, request.scheduled_end_at)
        && start > end
    {
        return Err(ApiError::InvalidInput(
            "scheduledStartAt must be less than or equal to scheduledEndAt".to_string(),
        ));
    }

    let mut shard_ids = request.affected_shard_ids.clone();
    shard_ids.sort_unstable();
    shard_ids.dedup();

    if shard_ids.len() != request.affected_shard_ids.len() {
        return Err(ApiError::InvalidInput(
            "affectedShardIds must not contain duplicates".to_string(),
        ));
    }

    Ok(())
}

#[api_operation(
    summary = "Create incident",
    description = "Create an incident or maintenance announcemnt for a bot.",
    tag = "Status"
)]
async fn create_incident(
    auth: Authenticated,
    repos: Data<Repositories>,
    path: Path<String>,
    Json(payload): Json<CreateIncidentRequest>,
) -> ApiResult<CreatedJson<ManagedIncidentResponse>> {
    let bot_id = Snowflake::try_from(path.into_inner())?.into_inner();

    let mut request = payload;
    validate_incident_request(&mut request)?;

    let bot = repos
        .bots
        .find_by_id(&bot_id)
        .await?
        .ok_or(ApiError::NotFound(format!(
            "Bot with ID {} not found",
            bot_id
        )))?;

    let ctx = &auth;

    if ctx.is_admin() {
        info!(
            code = %LogCode::AdminAction,
            bot_id = %bot_id,
            "Admin creating incident",
        );
    } else if ctx.is_bot() {
        if ctx.token.as_deref() != Some(&bot.token) {
            return Err(ApiError::Forbidden);
        }
    } else if ctx.is_user() {
        let user_id = ctx.user_id.as_deref().ok_or(ApiError::Unauthorized)?;

        if !bot.has_access(user_id) {
            return Err(ApiError::Forbidden);
        }
    } else {
        return Err(ApiError::Forbidden);
    }

    if bot.suspended && !ctx.is_admin() {
        return Err(ApiError::BotSuspended);
    }

    let created_by = if ctx.is_bot() {
        bot_id.clone()
    } else {
        ctx.user_id.clone().ok_or(ApiError::Unauthorized)?
    };

    let now = BsonDateTime::now();

    let started_at = match (request.incident_type, request.status) {
        (AnnouncementType::Incident, _)
        | (AnnouncementType::Maintenance, AnnouncementStatus::InProgress)
        | (AnnouncementType::Maintenance, AnnouncementStatus::Completed) => Some(now),
        _ => None,
    };

    let resolved_at = match request.status {
        AnnouncementStatus::Cancelled
        | AnnouncementStatus::Resolved
        | AnnouncementStatus::Completed => Some(now),
        _ => None,
    };

    let incident = StatusIncident {
        bot_id: bot_id.clone(),
        title: request.title.trim().to_string(),
        description: request.description.trim().to_string(),
        incident_type: request.incident_type,
        status: request.status,
        severity: request.severity,
        affected_shard_ids: request.affected_shard_ids,
        created_by,
        is_public: request.is_public,
        created_at: now,
        updated_at: now,
        started_at,
        scheduled_start_at: request.scheduled_start_at.map(BsonDateTime::from_chrono),
        scheduled_end_at: request.scheduled_end_at.map(BsonDateTime::from_chrono),
        resolved_at,
        updates: Vec::new(),
    };

    let incident_id = repos.status_incidents.create_incident(incident).await?;

    let stored = repos
        .status_incidents
        .find_by_id(&bot_id, incident_id, false)
        .await?
        .ok_or(ApiError::DatabaseError(
            "Created incident could not be retrieved".to_string(),
        ))?;

    Ok(CreatedJson(ManagedIncidentResponse::from((
        incident_id.to_hex(),
        stored,
    ))))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/incidents")
            .route("", get().to(get_incidents))
            .route("", post().to(create_incident))
            .configure(incident::configure),
    );
}
