use apistos::ApiComponent;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    domain::models::{AnnouncementStatus, AnnouncementType, IncidentSeverity, StatusIncident},
    openapi::schemas::PaginationResponse,
};

use super::incidents::IncidentUpdateResponse;

#[derive(Serialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ManagedIncidentResponse {
    pub incident_id: String,
    pub title: String,
    pub description: String,
    #[serde(rename = "type")]
    pub incident_type: AnnouncementType,
    pub status: AnnouncementStatus,
    pub severity: Option<IncidentSeverity>,
    pub affected_shard_ids: Vec<u32>,
    pub created_by: String,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub scheduled_start_at: Option<DateTime<Utc>>,
    pub scheduled_end_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub updates: Vec<IncidentUpdateResponse>,
}

impl From<(String, StatusIncident)> for ManagedIncidentResponse {
    fn from((incident_id, incident): (String, StatusIncident)) -> Self {
        Self {
            incident_id,
            title: incident.title,
            description: incident.description,
            incident_type: incident.incident_type,
            status: incident.status,
            severity: incident.severity,
            affected_shard_ids: incident.affected_shard_ids,
            created_by: incident.created_by,
            is_public: incident.is_public,
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
pub struct ManagedIncidentListResponse {
    pub bot_id: String,
    pub data: Vec<ManagedIncidentResponse>,
    pub pagination: PaginationResponse,
}

#[derive(Deserialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateIncidentRequest {
    pub title: String,
    pub description: String,
    #[serde(rename = "type")]
    pub incident_type: AnnouncementType,
    pub status: AnnouncementStatus,
    pub severity: Option<IncidentSeverity>,
    pub affected_shard_ids: Vec<u32>,
    pub is_public: bool,
    pub scheduled_start_at: Option<DateTime<Utc>>,
    pub scheduled_end_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateIncidentRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub severity: Option<IncidentSeverity>, // None when type is maintenance
    pub affected_shard_ids: Option<Vec<u32>>,
    pub is_public: Option<bool>,
    pub scheduled_start_at: Option<DateTime<Utc>>, // None when type is incident
    pub scheduled_end_at: Option<DateTime<Utc>>,   // None when type is incident
}

#[derive(Deserialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateIncidentUpdateRequest {
    pub status: AnnouncementStatus,
    pub message: String,
}

#[derive(Serialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateIncidentUpdateResponse {
    pub success: bool,
    pub update: IncidentUpdateResponse,
}
