use apistos::ApiComponent;
use chrono::{DateTime, Utc};
use mongodb::bson::{DateTime as BsonDateTime, oid::ObjectId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    domain::models::{
        BotDeploymentMode, OverallStatus, ShardStatus, ShardStatusSnapshot, StatusCheck,
        StoredStatusCheck,
    },
    openapi::schemas::PaginationResponse,
};

#[derive(Deserialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StatusHistoryQuery {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
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
