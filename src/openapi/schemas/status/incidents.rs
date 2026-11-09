use apistos::ApiComponent;
use bson::{DateTime as BsonDateTime, oid::ObjectId};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    domain::models::{
        AnnouncementStatus, AnnouncementType, IncidentSeverity, IncidentUpdate, StatusIncident,
        StoredStatusIncident,
    },
    openapi::schemas::PaginationResponse,
};

#[derive(Serialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PublicIncidentResponse {
    pub incident_id: String,
    pub title: String,
    pub description: String,
    pub incident_type: AnnouncementType,
    pub status: AnnouncementStatus,
    pub severity: Option<IncidentSeverity>,
    pub affected_shard_ids: Vec<u32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub scheduled_start_at: Option<DateTime<Utc>>,
    pub scheduled_end_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub updates: Vec<IncidentUpdateResponse>,
}

impl From<(String, StatusIncident)> for PublicIncidentResponse {
    fn from((incident_id, incident): (String, StatusIncident)) -> Self {
        Self {
            incident_id,
            title: incident.title,
            description: incident.description,
            incident_type: incident.incident_type,
            status: incident.status,
            severity: incident.severity,
            affected_shard_ids: incident.affected_shard_ids,
            created_at: incident.created_at.to_chrono(),
            updated_at: incident.updated_at.to_chrono(),
            started_at: incident.started_at.map(|date| date.to_chrono()),
            scheduled_start_at: incident.scheduled_start_at.map(|date| date.to_chrono()),
            scheduled_end_at: incident.scheduled_end_at.map(|date| date.to_chrono()),
            resolved_at: incident.resolved_at.map(|date| date.to_chrono()),
            updates: incident
                .updates
                .into_iter()
                .map(IncidentUpdateResponse::from)
                .collect(),
        }
    }
}

#[derive(Serialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IncidentUpdateResponse {
    pub status: AnnouncementStatus,
    pub message: String,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
}

impl From<IncidentUpdate> for IncidentUpdateResponse {
    fn from(update: IncidentUpdate) -> Self {
        Self {
            status: update.status,
            message: update.message,
            created_by: update.created_by,
            created_at: update.created_at.to_chrono(),
        }
    }
}

#[derive(Serialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PublicIncidentListResponse {
    pub bot_id: String,
    pub data: Vec<PublicIncidentResponse>,
    pub pagination: PaginationResponse,
}

#[derive(Deserialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IncidentListQuery {
    #[serde(rename = "type")]
    pub incident_type: Option<AnnouncementType>,
    pub status: Option<AnnouncementStatus>,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

pub struct IncidentCursor {
    pub created_at: BsonDateTime,
    pub id: ObjectId,
}

pub struct IncidentPage {
    pub incidents: Vec<StoredStatusIncident>,
    pub has_more: bool,
}
