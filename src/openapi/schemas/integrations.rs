use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationPayload {
    pub bot_id: String,
    pub user_id: String,
    pub webhook_secret: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct TopGGIntegrationPayload {
    pub data: TopGGIntegrationData,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct TopGGIntegrationData {
    pub connection_id: String,
    pub project: Option<TopGGIntegrationProject>,
    pub user: Option<TopGGIntegrationUser>,
    pub webhook_secret: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct TopGGIntegrationProject {
    pub id: String,
    pub platform: String,
    pub platform_id: String,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct TopGGIntegrationUser {
    pub avatar_url: String,
    pub id: String,
    pub name: String,
    pub platform_id: String,
}
