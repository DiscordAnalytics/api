use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TeamResponse {
    pub avatar: Option<String>,
    pub invitation_id: Option<String>,
    pub pending_invitation: bool,
    pub registered: bool,
    pub user_id: String,
    pub username: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TeamRequestBody {
    pub user_id: String,
}
