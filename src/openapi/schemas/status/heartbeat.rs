use apistos::ApiComponent;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatRequest {
    pub shard_id: Option<u32>,
    pub total_shards: Option<u32>,
    pub latency_ms: Option<u32>,
    pub guild_count: Option<i64>,
}

#[derive(Serialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatResponse {
    pub success: bool,
    pub message: String,
    pub received_at: DateTime<Utc>,
}
