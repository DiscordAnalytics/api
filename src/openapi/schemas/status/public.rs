use apistos::ApiComponent;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::Serialize;

use crate::domain::models::{
    BotDeploymentMode, OverallStatus, ShardCurrentStatus, ShardStatus, StatusCurrent,
};

#[derive(Serialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PublicBotStatusResponse {
    pub bot_id: String,
    pub deployment_mode: BotDeploymentMode,
    pub status: OverallStatus,
    pub total_shards: u32,
    pub latency_ms: Option<u32>,
    pub guild_count: Option<i64>,
    pub last_heartbeat_at: Option<DateTime<Utc>>,
    pub shards: Vec<PublicShardStatus>,
    pub updated_at: DateTime<Utc>,
}

impl From<StatusCurrent> for PublicBotStatusResponse {
    fn from(value: StatusCurrent) -> Self {
        Self {
            bot_id: value.bot_id,
            deployment_mode: value.deployment_mode,
            status: value.status,
            total_shards: value.total_shards,
            latency_ms: value.latency_ms,
            guild_count: value.guild_count,
            last_heartbeat_at: value.last_heartbeat_at.map(|t| t.to_chrono()),
            shards: value.shards.into_iter().map(|s| s.into()).collect(),
            updated_at: value.updated_at.to_chrono(),
        }
    }
}

#[derive(Serialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PublicShardStatus {
    pub shard_id: u32,
    pub status: ShardStatus,
    pub latency_ms: Option<u32>,
    pub guild_count: Option<i64>,
    pub last_heartbeat_at: Option<DateTime<Utc>>,
}

impl From<ShardCurrentStatus> for PublicShardStatus {
    fn from(value: ShardCurrentStatus) -> Self {
        Self {
            shard_id: value.shard_id,
            status: value.status,
            latency_ms: value.latency_ms,
            guild_count: value.guild_count,
            last_heartbeat_at: value.last_heartbeat_at.map(|t| t.to_chrono()),
        }
    }
}
