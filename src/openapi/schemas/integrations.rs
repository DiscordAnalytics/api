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
pub struct PlatformIntegrationPayload {
    pub data: PlatformIntegrationData,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct PlatformIntegrationData {
    pub connection_id: Option<String>,
    pub project: Option<PlatformIntegrationProject>,
    pub user: Option<PlatformIntegrationUser>,
    pub webhook_secret: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct PlatformIntegrationProject {
    pub platform: String,
    pub platform_id: String,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct PlatformIntegrationUser {
    pub platform_id: String,
}
