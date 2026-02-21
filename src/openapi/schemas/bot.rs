use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, ApiComponent, JsonSchema)]
pub struct BotResponse {
    #[serde(rename = "advancedStats")]
    pub advanced_stats: Option<bool>,
    pub avatar: Option<String>,
    pub banned: Option<bool>,
    #[serde(rename = "botId")]
    pub bot_id: String,
    pub framework: Option<String>,
    pub goals_limit: Option<i32>,
    pub language: Option<String>,
    #[serde(rename = "lastPush")]
    pub last_push: Option<String>,
    #[serde(rename = "ownerId")]
    pub owner_id: Option<String>,
    pub team: Option<Vec<String>>,
    pub token: Option<String>,
    pub username: Option<String>,
    pub version: Option<String>,
    #[serde(rename = "votesWebhookUrl")]
    pub votes_webhook_url: Option<String>,
    #[serde(rename = "watchedSince")]
    pub watched_since: Option<String>,
}
