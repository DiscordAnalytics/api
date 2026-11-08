use apistos::ApiComponent;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::models::BotDeploymentMode;

#[derive(Deserialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBotStatusConfigRequest {
    pub deployment_mode: BotDeploymentMode,
    pub total_shards: u32,
}

#[derive(Serialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BotStatusConfigResponse {
    pub bot_id: String,
    pub deployment_mode: BotDeploymentMode,
    pub total_shards: u32,
    pub updated_at: DateTime<Utc>,
}
