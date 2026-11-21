use apistos::ApiComponent;
use chrono::{DateTime, Utc};
use mongodb::bson::{DateTime as BsonDateTime, oid::ObjectId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    domain::models::{
        BotDeploymentMode, OverallStatus, ShardHourlySnapshot, ShardStatus, ShardStatusSnapshot,
        StatusCheck, StatusCheckHourly, StoredStatusCheck, StoredStatusCheckHourly,
    },
    openapi::schemas::PaginationResponse,
};

// Union response (returned by the history endpoint)

/// Either a raw or an hourly history response, tagged by `resolution`.
#[derive(Serialize, ApiComponent, JsonSchema)]
#[serde(tag = "resolution", rename_all = "camelCase")]
pub enum StatusHistoryResponse {
    Raw(PublicStatusHistoryResponse),
    Hourly(PublicHourlyHistoryResponse),
}

/// Resolution of history data to return.
#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Eq, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum HistoryResolution {
    /// Raw 5-min snapshots (available for last 7 days).
    Raw,
    /// Pre-aggregated hourly buckets (available for last 90 days).
    Hourly,
}

#[derive(Deserialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StatusHistoryQuery {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
    /// Defaults to `raw`. Use `hourly` for wide date ranges to avoid pagination.
    pub resolution: Option<HistoryResolution>,
}

#[derive(Serialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PublicStatusHistoryResponse {
    pub bot_id: String,
    pub data: Vec<PublicStatusSnapshot>,
    pub pagination: PaginationResponse,
}

#[derive(Serialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PublicStatusSnapshot {
    pub timestamp: DateTime<Utc>,
    pub status: OverallStatus,
    pub deployment_mode: BotDeploymentMode,
    pub total_shards: u32,
    pub shards: Vec<PublicShardSnapshot>,
}

impl From<StatusCheck> for PublicStatusSnapshot {
    fn from(check: StatusCheck) -> Self {
        PublicStatusSnapshot {
            timestamp: check.timestamp.to_chrono(),
            status: check.status,
            deployment_mode: check.deployment_mode,
            total_shards: check.total_shards,
            shards: check.shards.into_iter().map(|s| s.into()).collect(),
        }
    }
}

#[derive(Serialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PublicShardSnapshot {
    pub shard_id: u32,
    pub status: ShardStatus,
    pub latency_ms: Option<u32>,
    pub guild_count: Option<i64>,
}

impl From<ShardStatusSnapshot> for PublicShardSnapshot {
    fn from(status: ShardStatusSnapshot) -> Self {
        Self {
            shard_id: status.shard_id,
            status: status.status,
            latency_ms: status.latency_ms,
            guild_count: status.guild_count,
        }
    }
}

pub struct HistoryCursor {
    pub timestamp: BsonDateTime,
    pub id: ObjectId,
}

pub struct HistoryPage {
    pub checks: Vec<StoredStatusCheck>,
    pub has_more: bool,
}

// Hourly history

#[derive(Serialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PublicHourlyHistoryResponse {
    pub bot_id: String,
    pub data: Vec<PublicHourlySnapshot>,
    pub pagination: PaginationResponse,
}

#[derive(Serialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PublicHourlySnapshot {
    pub hour_timestamp: DateTime<Utc>,
    pub status: OverallStatus,
    pub deployment_mode: BotDeploymentMode,
    pub total_shards: u32,
    pub shards: Vec<PublicShardHourlySummary>,
}

impl From<StatusCheckHourly> for PublicHourlySnapshot {
    fn from(c: StatusCheckHourly) -> Self {
        Self {
            hour_timestamp: c.hour_timestamp.to_chrono(),
            status: c.status,
            deployment_mode: c.deployment_mode,
            total_shards: c.total_shards,
            shards: c.shards.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PublicShardHourlySummary {
    pub shard_id: u32,
    pub worst_status: ShardStatus,
    pub avg_latency_ms: Option<u32>,
    pub last_guild_count: Option<i64>,
    pub sample_count: u32,
}

impl From<ShardHourlySnapshot> for PublicShardHourlySummary {
    fn from(s: ShardHourlySnapshot) -> Self {
        Self {
            shard_id: s.shard_id,
            worst_status: s.worst_status,
            avg_latency_ms: s
                .cumulative_latency_ms
                .map(|v| v / s.latency_sample_count.unwrap_or(1)),
            last_guild_count: s.last_guild_count,
            sample_count: s.sample_count,
        }
    }
}

pub struct HourlyHistoryCursor {
    pub timestamp: BsonDateTime,
    pub id: ObjectId,
}

pub struct HourlyHistoryPage {
    pub checks: Vec<StoredStatusCheckHourly>,
    pub has_more: bool,
}
