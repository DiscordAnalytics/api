use apistos::ApiComponent;
use bson::oid::ObjectId;
use mongodb::bson::DateTime;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Collections

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusCurrent {
    pub bot_id: String,
    pub deployment_mode: BotDeploymentMode,
    pub status: OverallStatus,
    pub total_shards: u32,
    pub shards: Vec<ShardCurrentStatus>,
    pub latency_ms: Option<u32>,
    pub guild_count: Option<i64>,
    pub last_heartbeat_at: Option<DateTime>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub last_evaluated_at: DateTime,
}

#[derive(Deserialize)]
pub struct StoredStatusCheck {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub check: StatusCheck,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusCheck {
    pub bot_id: String,
    pub deployment_mode: BotDeploymentMode,
    pub timestamp: DateTime,
    pub status: OverallStatus,
    pub latency_ms: Option<u32>,
    pub guild_count: Option<i64>,
    pub total_shards: u32,
    pub shards: Vec<ShardStatusSnapshot>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusIncident {
    pub bot_id: String,
    pub title: String,
    pub description: String,
    #[serde(rename = "type")]
    pub incident_type: AnnouncementType,
    pub status: AnnouncementStatus,
    pub severity: Option<IncidentSeverity>,
    pub affected_shard_ids: Vec<u32>,
    pub created_by: String,
    pub is_public: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub started_at: Option<DateTime>,
    pub scheduled_start_at: Option<DateTime>,
    pub scheduled_end_at: Option<DateTime>,
    pub resolved_at: Option<DateTime>,
    pub updates: Vec<IncidentUpdate>,
}
// Data objects

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShardCurrentStatus {
    pub shard_id: u32,
    pub status: ShardStatus,
    pub latency_ms: Option<u32>,
    pub guild_count: Option<i64>,
    pub last_heartbeat_at: Option<DateTime>,
    pub updated_at: DateTime,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShardStatusSnapshot {
    pub shard_id: u32,
    pub status: ShardStatus,
    pub latency_ms: Option<u32>,
    pub guild_count: Option<i64>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncidentUpdate {
    pub status: AnnouncementStatus,
    pub message: String,
    pub created_by: String,
    pub created_at: DateTime,
}

// Enums

#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Eq, ApiComponent, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub enum BotDeploymentMode {
    Single,
    Sharded,
}

#[derive(Deserialize, Serialize, Clone, Copy, ApiComponent, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub enum OverallStatus {
    Operational,
    Degraded,
    PartialOutage,
    MajorOutage,
    Unknown,
}

#[derive(Deserialize, Serialize, Clone, Copy, ApiComponent, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub enum ShardStatus {
    Operational,
    Degraded,
    Offline,
    Unknown,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum AnnouncementType {
    Incident,
    Maintenance,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum AnnouncementStatus {
    Investigating,
    Identified,
    Monitoring,
    Resolved,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum IncidentSeverity {
    Minor,
    Major,
    Critical,
}
