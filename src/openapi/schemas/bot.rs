use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::models::{Bot, WebhooksConfig};

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BotResponse {
    pub advanced_stats: bool,
    pub avatar: Option<String>,
    pub bot_id: String,
    pub custom_events_limit: i32,
    pub framework: Option<String>,
    pub goals_limit: i32,
    pub language: Option<String>,
    pub last_push: Option<String>,
    pub owner_id: String,
    pub suspended: bool,
    pub team: Vec<String>,
    pub teammates_limit: i32,
    pub username: String,
    pub version: Option<String>,
    pub watched_since: String,
    pub webhooks_config: Option<WebhooksConfig>,
}

impl TryFrom<Bot> for BotResponse {
    type Error = anyhow::Error;

    fn try_from(bot: Bot) -> Result<Self, Self::Error> {
        Ok(Self {
            advanced_stats: bot.advanced_stats,
            avatar: bot.avatar,
            bot_id: bot.bot_id,
            custom_events_limit: bot.custom_events_limit,
            framework: bot.framework,
            goals_limit: bot.goals_limit,
            language: bot.language,
            last_push: bot
                .last_push
                .map(|dt| dt.try_to_rfc3339_string())
                .transpose()?,
            owner_id: bot.owner_id,
            suspended: bot.suspended,
            team: bot.team,
            teammates_limit: bot.teammates_limit,
            username: bot.username,
            version: bot.version,
            watched_since: bot.watched_since.try_to_rfc3339_string()?,
            webhooks_config: Some(bot.webhooks_config),
        })
    }
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BotCreationBody {
    pub user_id: String,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct BotUpdateBody {
    pub avatar: Option<String>,
    pub framework: Option<String>,
    pub username: Option<String>,
    pub version: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct BotDeletionPayload {
    pub reason: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct BotSuspendRequest {
    pub reason: String,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct BotTokenResponse {
    pub token: String,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BotSettingsPayload {
    pub advanced_stats: Option<bool>,
    pub custom_events_limit: Option<i32>,
    pub goals_limit: Option<i32>,
    pub teammates_limit: Option<i32>,
    pub webhook_url: Option<String>,
}
