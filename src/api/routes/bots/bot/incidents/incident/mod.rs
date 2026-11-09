use actix_web::web::{Data, Json, Path};
use apistos::{
    api_operation,
    web::{ServiceConfig, get},
};
use bson::oid::ObjectId;

use crate::{
    domain::error::{ApiError, ApiResult},
    openapi::schemas::PublicIncidentResponse,
    repository::Repositories,
    utils::{discord::Snowflake, logger::LogCode},
};

#[api_operation(
    summary = "Get public incident",
    description = "Get the details of a public incident for a bot",
    tag = "Status"
)]
async fn get_incident(
    repos: Data<Repositories>,
    path: Path<(String, String)>,
) -> ApiResult<Json<PublicIncidentResponse>> {
    let (bot_id, incident_id) = path.into_inner();

    let bot_id = Snowflake::try_from(bot_id)?.into_inner();
    let incident_id = ObjectId::parse_str(&incident_id)
        .map_err(|_| ApiError::BadRequest("Invalid incident ID".to_string()))?;

    info!(
        code = %LogCode::Status,
        bot_id = %bot_id,
        incident_id = %incident_id,
        "Attempting to get public incident for bot"
    );

    let bot = repos.bots.find_by_id(&bot_id).await?.ok_or_else(|| {
        info!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            incident_id = %incident_id,
            "Bot not found for public incident request",
        );
        ApiError::NotFound(format!("Bot with ID {} not found", bot_id))
    })?;

    if bot.suspended {
        warn!(
            code = %LogCode::Request,
            bot_id = %bot.bot_id,
            incident_id = %incident_id,
            "Bot suspended for public incident request",
        );
        return Err(ApiError::NotFound(format!(
            "Bot with ID {} not found",
            bot.bot_id
        )));
    }

    let incident = repos
        .status_incidents
        .find_public_by_id(&bot_id, incident_id)
        .await?
        .ok_or_else(|| {
            info!(
                code = %LogCode::Request,
                bot_id = %bot_id,
                incident_id = %incident_id,
                "Public incident not found",
            );

            ApiError::NotFound(format!("Incident with ID {} not found", incident_id))
        })?;

    Ok(Json(PublicIncidentResponse::from((
        incident_id.to_string(),
        incident,
    ))))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.route("/{incidentId}", get().to(get_incident));
}
