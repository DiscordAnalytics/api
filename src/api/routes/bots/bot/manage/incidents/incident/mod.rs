mod updates;

use actix_web::web::{Data, Json, Path};
use apistos::{
    api_operation,
    web::{ServiceConfig, get, patch, scope},
};
use mongodb::bson::{Bson, DateTime as BsonDateTime, Document, oid::ObjectId, serialize_to_bson};

use crate::{
    api::middleware::Authenticated,
    domain::{
        error::{ApiError, ApiResult},
        models::{AnnouncementType, StatusIncident},
    },
    openapi::schemas::{ManagedIncidentResponse, UpdateIncidentRequest},
    repository::Repositories,
    utils::{discord::Snowflake, logger::LogCode},
};

#[api_operation(
    summary = "Get incident",
    description = "Get the details of an incident for a bot.",
    tag = "Status"
)]
async fn get_incident(
    auth: Authenticated,
    repos: Data<Repositories>,
    path: Path<(String, String)>,
) -> ApiResult<Json<ManagedIncidentResponse>> {
    let (bot_id, incident_id) = path.into_inner();
    let bot_id = Snowflake::try_from(bot_id)?.into_inner();

    let incident_id = ObjectId::parse_str(&incident_id)
        .map_err(|_| ApiError::InvalidInput("Invalid incident ID".into()))?;

    info!(
        code = %LogCode::Status,
        bot_id = %bot_id,
        incident_id = %incident_id,
        "Attempting to retrieve incident",
    );

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
            "Admin retrieving incident",
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

    let incident = repos
        .status_incidents
        .find_by_id(&bot_id, incident_id, false)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Incident with ID {} not found", incident_id)))?;

    Ok(Json(ManagedIncidentResponse::from((
        incident_id.to_hex(),
        incident,
    ))))
}

fn validate_update_incident(
    request: &UpdateIncidentRequest,
    incident: &StatusIncident,
) -> ApiResult<()> {
    if let Some(title) = &request.title
        && title.trim().is_empty()
    {
        return Err(ApiError::InvalidInput("title cannot be empty".to_string()));
    }

    if let Some(description) = &request.description
        && description.trim().is_empty()
    {
        return Err(ApiError::InvalidInput(
            "description cannot be empty".to_string(),
        ));
    }

    match incident.incident_type {
        AnnouncementType::Incident => {
            if request.scheduled_start_at.is_some() || request.scheduled_end_at.is_some() {
                return Err(ApiError::InvalidInput(
                    "Scheduled dates are not allowed for incidents".to_string(),
                ));
            }
        }
        AnnouncementType::Maintenance => {}
    }

    let start = request
        .scheduled_start_at
        .map(mongodb::bson::DateTime::from_chrono)
        .or(incident.scheduled_start_at);

    let end = request
        .scheduled_end_at
        .map(mongodb::bson::DateTime::from_chrono)
        .or(incident.scheduled_end_at);

    if let (Some(start), Some(end)) = (start, end)
        && start > end
    {
        return Err(ApiError::InvalidInput(
            "scheduledStartAt must be less than or equal to scheduledEndAt".to_string(),
        ));
    }

    if let Some(shard_ids) = &request.affected_shard_ids {
        let mut unique_ids = shard_ids.clone();
        unique_ids.sort_unstable();
        unique_ids.dedup();

        if unique_ids.len() != shard_ids.len() {
            return Err(ApiError::InvalidInput(
                "affectedShardIds must not contain duplicates".to_string(),
            ));
        }
    }

    Ok(())
}

#[api_operation(
    summary = "Update incident",
    description = "Update an incident for a bot.",
    tag = "Status"
)]
async fn update_incident(
    auth: Authenticated,
    repos: Data<Repositories>,
    path: Path<(String, String)>,
    Json(payload): Json<UpdateIncidentRequest>,
) -> ApiResult<Json<ManagedIncidentResponse>> {
    let (bot_id, incident_id) = path.into_inner();
    let bot_id = Snowflake::try_from(bot_id)?.into_inner();

    let incident_id = ObjectId::parse_str(&incident_id)
        .map_err(|_| ApiError::InvalidInput("Invalid incident ID".to_string()))?;

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
            "Admin updating incident",
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

    let existing = repos
        .status_incidents
        .find_by_id(&bot_id, incident_id, false)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Incident with ID {} not found", incident_id)))?;

    validate_update_incident(&payload, &existing)?;

    let has_updates = payload.title.is_some()
        || payload.description.is_some()
        || payload.severity.is_some()
        || payload.affected_shard_ids.is_some()
        || payload.is_public.is_some()
        || payload.scheduled_start_at.is_some()
        || payload.scheduled_end_at.is_some();

    if !has_updates {
        return Err(ApiError::InvalidInput(
            "At least one field must be provided".to_string(),
        ));
    }

    let mut update = Document::new();

    if let Some(title) = payload.title {
        update.insert("title", title.trim());
    }

    if let Some(description) = payload.description {
        update.insert("description", description.trim());
    }

    match existing.incident_type {
        AnnouncementType::Incident => {
            if let Some(severity) = payload.severity {
                update.insert("severity", serialize_to_bson(&severity)?);
            }
        }
        AnnouncementType::Maintenance => {
            update.insert("severity", Bson::Null);
        }
    }

    if let Some(affected_shard_ids) = payload.affected_shard_ids {
        update.insert("affectedShardIds", affected_shard_ids);
    }

    if let Some(is_public) = payload.is_public {
        update.insert("isPublic", is_public);
    }

    if let Some(scheduled_start_at) = payload.scheduled_start_at {
        update.insert(
            "scheduledStartAt",
            BsonDateTime::from_chrono(scheduled_start_at),
        );
    }

    if let Some(scheduled_end_at) = payload.scheduled_end_at {
        update.insert(
            "scheduledEndAt",
            BsonDateTime::from_chrono(scheduled_end_at),
        );
    }

    if update.is_empty() {
        return Err(ApiError::InvalidInput(
            "At least one field must be provided".to_string(),
        ));
    }

    update.insert("updatedAt", BsonDateTime::now());

    let incident = repos
        .status_incidents
        .update_incident(
            &bot_id,
            incident_id,
            update,
            existing.scheduled_start_at,
            existing.scheduled_end_at,
        )
        .await?
        .ok_or(ApiError::Conflict(format!(
            "Incident {} was modified by another request. Please retry",
            incident_id
        )))?;

    info!(
        code = %LogCode::Status,
        bot_id = %bot_id,
        incident_id = %incident_id,
        "Incident updated successfully",
    );

    Ok(Json(ManagedIncidentResponse::from((
        incident_id.to_hex(),
        incident,
    ))))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/{incidentId}")
            .route("", get().to(get_incident))
            .route("", patch().to(update_incident))
            .configure(updates::configure),
    );
}
