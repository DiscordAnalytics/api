use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::models::Vote;

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct BotListMePayload {
    pub bot: String,
    pub user: String,
    #[serde(rename = "type")]
    pub vote_type: String,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct DBListPayload {
    pub admin: Option<bool>,
    pub avatar: Option<String>,
    pub bot_id: String,
    pub discriminator: Option<String>,
    pub id: String,
    pub promotable_bot: Option<String>,
    pub roblox: Option<bool>,
    pub stripe: Option<bool>,
    pub username: String,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DiscordListPayload {
    pub bot_id: String,
    pub is_test: bool,
    pub query: Option<String>,
    pub user_id: String,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct DiscordPlacePayload {
    pub bot: String,
    pub test: bool,
    pub user: String,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct DiscordsComQuery {
    pub cast: String,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct DiscordsComPayload {
    pub bot: String,
    pub engine: String,
    pub query: Option<DiscordsComQuery>,
    #[serde(rename = "type")]
    pub type_: String,
    pub user: String,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct TopGGPayload {
    pub data: TopGGData,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct TopGGData {
    pub created_at: Option<String>,
    pub expires_at: Option<String>,
    pub id: Option<String>,
    pub project: TopGGProject,
    pub user: TopGGUser,
    pub weight: Option<i32>,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct TopGGProject {
    pub id: String,
    pub platform: String,
    pub platform_id: String,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct TopGGUser {
    pub avatar_url: String,
    pub id: String,
    pub name: String,
    pub platform_id: String,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct VoteWebhookResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WebhookVoteResponse {
    pub bot_id: String,
    pub provider: String,
    pub timestamp: String,
    pub count: i32,
}

impl TryFrom<Vote> for WebhookVoteResponse {
    type Error = anyhow::Error;

    fn try_from(vote: Vote) -> Result<Self, Self::Error> {
        Ok(Self {
            bot_id: vote.bot_id,
            provider: vote.provider,
            timestamp: vote.date.try_to_rfc3339_string()?,
            count: vote.count,
        })
    }
}
