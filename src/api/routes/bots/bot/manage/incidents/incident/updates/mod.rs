use actix_web::web::{Data, Json, Path};
use apistos::{
    actix::CreatedJson,
    api_operation,
    web::{ServiceConfig, post},
};
use bson::{DateTime as BsonDateTime, oid::ObjectId};

use crate::{
    api::middleware::Authenticated,
    domain::{
        error::{ApiError, ApiResult},
        models::{AnnouncementStatus, AnnouncementType, IncidentUpdate},
    },
    openapi::schemas::{
        CreateIncidentUpdateRequest, CreateIncidentUpdateResponse, IncidentUpdateResponse,
    },
    repository::Repositories,
    utils::{discord::Snowflake, logger::LogCode},
};

fn validate_status_transition(
    incident_type: AnnouncementType,
    current: AnnouncementStatus,
    next: AnnouncementStatus,
) -> ApiResult<()> {
    use AnnouncementStatus::*;

    let valid = match incident_type {
        AnnouncementType::Incident => matches!(
            (current, next),
            (Investigating, Identified)
                | (Investigating, Monitoring)
                | (Investigating, Resolved)
                | (Identified, Monitoring)
                | (Identified, Resolved)
                | (Monitoring, Identified)
                | (Monitoring, Resolved)
        ),
        AnnouncementType::Maintenance => matches!(
            (current, next),
            (Scheduled, InProgress)
                | (Scheduled, Cancelled)
                | (InProgress, Completed)
                | (InProgress, Cancelled)
        ),
    };

    if !valid {
        return Err(ApiError::InvalidInput(
            "Invalid incident status transition".to_string(),
        ));
    }

    Ok(())
}

fn validate_incident_update(message: &str) -> ApiResult<()> {
    if message.trim().is_empty() {
        return Err(ApiError::InvalidInput(
            "message cannot be empty".to_string(),
        ));
    }

    if message.len() > 2000 {
        return Err(ApiError::InvalidInput(
            "message cannot be longer than 2000 characters".to_string(),
        ));
    }

    Ok(())
}

#[api_operation(summary = "", description = "", tag = "Status")]
async fn create_update(
    auth: Authenticated,
    repos: Data<Repositories>,
    path: Path<(String, String)>,
    Json(payload): Json<CreateIncidentUpdateRequest>,
) -> ApiResult<CreatedJson<CreateIncidentUpdateResponse>> {
    let (bot_id, incident_id) = path.into_inner();
    let bot_id = Snowflake::try_from(bot_id)?.into_inner();

    let incident_id = ObjectId::parse_str(&incident_id)
        .map_err(|_| ApiError::InvalidInput("Invalid incident ID".to_string()))?;

    validate_incident_update(&payload.message)?;

    info!(
        code = %LogCode::Status,
        bot_id = %bot_id,
        incident_id = %incident_id,
        "Attempting to create incident update",
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
            "Admin creating incident update",
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
        .ok_or(ApiError::NotFound(format!(
            "Incident with ID {} not found",
            incident_id
        )))?;

    validate_status_transition(incident.incident_type, incident.status, payload.status)?;

    let created_by = if ctx.is_bot() {
        bot_id.clone()
    } else {
        ctx.user_id.clone().ok_or(ApiError::Unauthorized)?
    };

    let update = IncidentUpdate {
        status: payload.status,
        message: payload.message.trim().to_string(),
        created_by,
        created_at: BsonDateTime::now(),
    };

    let updated_incident = repos
        .status_incidents
        .create_update(
            &bot_id,
            incident_id,
            incident.status,
            update,
            payload.status,
        )
        .await?
        .ok_or(ApiError::Conflict(
            "Incident status changed concurrently. Please retry.".to_string(),
        ))?;

    let update = updated_incident
        .updates
        .last()
        .ok_or(ApiError::DatabaseError(
            "Incident update was not persisted".to_string(),
        ))?;

    info!(
        code = %LogCode::Status,
        bot_id = %bot_id,
        incident_id = %incident_id,
        "Incident update created successfully",
    );

    Ok(CreatedJson(CreateIncidentUpdateResponse {
        success: true,
        update: IncidentUpdateResponse {
            status: update.status,
            message: update.message.clone(),
            created_by: update.created_by.clone(),
            created_at: update.created_at.to_chrono(),
        },
    }))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.route("/updates", post().to(create_update));
}
