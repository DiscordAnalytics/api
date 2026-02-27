use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::models::Bot;

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BotResponse {
    pub advanced_stats: Option<bool>,
    pub avatar: Option<String>,
    pub bot_id: String,
    pub framework: Option<String>,
    pub goals_limit: Option<i32>,
    pub language: Option<String>,
    pub last_push: Option<String>,
    pub owner_id: Option<String>,
    pub suspended: Option<bool>,
    pub team: Option<Vec<String>>,
    pub username: Option<String>,
    pub version: Option<String>,
    pub votes_webhook_url: Option<String>,
    pub watched_since: Option<String>,
}

impl TryFrom<Bot> for BotResponse {
    type Error = anyhow::Error;

    fn try_from(bot: Bot) -> Result<Self, Self::Error> {
        Ok(Self {
            advanced_stats: Some(bot.advanced_stats),
            avatar: bot.avatar,
            bot_id: bot.bot_id,
            framework: bot.framework,
            goals_limit: bot.goals_limit,
            language: bot.language,
            last_push: bot
                .last_push
                .map(|dt| dt.try_to_rfc3339_string())
                .transpose()?,
            owner_id: Some(bot.owner_id),
            suspended: Some(bot.suspended),
            team: Some(bot.team),
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
#[serde(rename_all = "camelCase")]
pub struct BotCreationBody {
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

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct BotSuspendRequest {
    pub reason: String,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct BotSuspendResponse {
    pub message: String,
}
