use apistos::ApiComponent;
use mongodb::bson::{DateTime, oid::ObjectId};
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
    pub last_heartbeat_at: Option<DateTime>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub last_evaluated_at: DateTime,
}

#[derive(Deserialize)]
pub struct StoredStatusCheck {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    #[serde(flatten)]
    pub check: StatusCheck,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusCheck {
    pub bot_id: String,
    pub deployment_mode: BotDeploymentMode,
    pub timestamp: DateTime,
    pub status: OverallStatus,
    pub total_shards: u32,
    pub shards: Vec<ShardStatusSnapshot>,
}

/// One document per (botId, bucket_timestamp) - transient, sealed by background task.
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusPendingBucket {
    pub bot_id: String,
    pub deployment_mode: BotDeploymentMode,
    pub bucket_timestamp: DateTime,
    pub status: OverallStatus,
    pub total_shards: u32,
    /// Sparse map: shard_id -> snapshot. Stored as array for MongoDB compat.
    pub shards: Vec<ShardStatusSnapshot>,
    pub sealed: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

impl From<StatusPendingBucket> for StatusCheck {
    fn from(bucket: StatusPendingBucket) -> Self {
        Self {
            bot_id: bucket.bot_id,
            deployment_mode: bucket.deployment_mode,
            timestamp: bucket.bucket_timestamp,
            status: bucket.status,
            total_shards: bucket.total_shards,
            shards: bucket.shards,
        }
    }
}

/// One document per (botId, hour) - long-lived downsampled history.
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusCheckHourly {
    pub bot_id: String,
    pub hour_timestamp: DateTime,
    pub deployment_mode: BotDeploymentMode,
    pub total_shards: u32,
    /// Worst-case overall status observed in this hour.
    pub status: OverallStatus,
    /// Per-shard aggregated data for the hour.
    pub shards: Vec<ShardHourlySnapshot>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Deserialize)]
pub struct StoredStatusCheckHourly {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    #[serde(flatten)]
    pub check: StatusCheckHourly,
}

#[derive(Deserialize)]
pub struct StoredStatusIncident {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    #[serde(flatten)]
    pub incident: StatusIncident,
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

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShardHourlySnapshot {
    pub shard_id: u32,
    /// Worst-case status seen during the hour.
    pub worst_status: ShardStatus,
    /// Cumulative latency across all samples in the hour (ms).
    pub cumulative_latency_ms: Option<u32>,
    /// Number of latency samples contributing to this shard's data.
    pub latency_sample_count: Option<u32>,
    /// Last known guild count in the hour.
    pub last_guild_count: Option<i64>,
    /// Number of samples contributing to this shard's data.
    pub sample_count: u32,
}

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

#[derive(Clone, Deserialize, Serialize)]
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

#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Eq, ApiComponent, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub enum OverallStatus {
    Operational,
    Degraded,
    PartialOutage,
    MajorOutage,
    Unknown,
}

#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Eq, ApiComponent, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub enum ShardStatus {
    Operational,
    Degraded,
    Offline,
    Unknown,
}

#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Eq, ApiComponent, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub enum AnnouncementType {
    Incident,
    Maintenance,
}

#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Eq, ApiComponent, JsonSchema)]
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

#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Eq, ApiComponent, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub enum IncidentSeverity {
    Minor,
    Major,
    Critical,
}
