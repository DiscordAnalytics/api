use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct BotListMePayload {
    pub bot: String,
    pub user: String,
    #[serde(rename = "type")]
    pub vote_type: String,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct DBListPayload {
    pub bot_id: String,
    pub id: String,
    pub promotable_bot: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DiscordListPayload {
    pub bot_id: String,
    pub is_test: bool,
    pub user_id: String,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct DiscordPlacePayload {
    pub bot: String,
    pub test: bool,
    pub user: String,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct DiscordsComPayload {
    pub bot: String,
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
    pub project: TopGGProject,
    pub user: TopGGUser,
    pub weight: Option<i32>,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct TopGGProject {
    pub platform: String,
    pub platform_id: String,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct TopGGUser {
    pub name: String,
    pub platform_id: String,
}
