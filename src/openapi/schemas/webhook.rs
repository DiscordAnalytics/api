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
    pub id: String,
    pub username: String,
    pub discriminator: Option<String>,
    pub avatar: Option<String>,
    pub admin: Option<bool>,
    pub stripe: Option<bool>,
    pub roblox: Option<bool>,
    pub promotable_bot: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DiscordListPayload {
    pub bot_id: String,
    pub user_id: String,
    pub query: Option<String>,
    pub is_test: bool,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct DiscordPlacePayload {
    pub bot: String,
    pub user: String,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct DiscordsComQuery {
    pub cast: String,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct DiscordsComPayload {
    pub user: String,
    pub bot: String,
    pub query: Option<DiscordsComQuery>,
    pub engine: String,
    #[serde(rename = "type")]
    pub type_: String,
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
