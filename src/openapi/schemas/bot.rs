use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::models::Bot;

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct BotResponse {
    #[serde(rename = "advancedStats")]
    pub advanced_stats: Option<bool>,
    pub avatar: Option<String>,
    pub banned: Option<bool>,
    #[serde(rename = "botId")]
    pub bot_id: String,
    pub framework: Option<String>,
    #[serde(rename = "goalsLimit")]
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

impl TryFrom<Bot> for BotResponse {
    type Error = anyhow::Error;

    fn try_from(bot: Bot) -> Result<Self, Self::Error> {
        Ok(Self {
            advanced_stats: Some(bot.advanced_stats),
            avatar: bot.avatar,
            banned: Some(bot.suspended),
            bot_id: bot.bot_id,
            framework: bot.framework,
            goals_limit: bot.goals_limit,
            language: bot.language,
            last_push: bot
                .last_push
                .map(|dt| dt.try_to_rfc3339_string())
                .transpose()?,
            owner_id: Some(bot.owner_id),
            team: Some(bot.team),
            token: None,
            username: Some(bot.username),
            version: bot.version,
            votes_webhook_url: bot.votes_webhook_url,
            watched_since: bot
                .watched_since
                .map(|dt| dt.try_to_rfc3339_string())
                .transpose()?,
        })
    }
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct BotCreationBody {
    #[serde(rename = "userId")]
    pub user_id: String,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct BotUpdateBody {
    pub avatar: Option<String>,
    pub framework: Option<String>,
    pub team: Option<Vec<String>>,
    pub username: Option<String>,
    pub version: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct BotDeletionResponse {
    pub message: String,
}
