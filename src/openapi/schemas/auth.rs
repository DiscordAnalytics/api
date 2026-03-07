use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthConfigResponse {
    pub client_id: String,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, ApiComponent, JsonSchema)]
pub struct LinkedRolesQuery {
    pub code: Option<String>,
    pub state: Option<String>,
}

#[derive(Debug, Clone, Deserialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthCallbackQuery {
    pub code: String,
    pub redirection: String,
    pub scopes: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiscordOAuthUser {
    pub avatar: Option<String>,
    pub avatar_decoration_data: Option<AvatarDecoration>,
    pub discriminator: String,
    pub email: Option<String>,
    pub id: String,
    pub username: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiscordBot {
    pub avatar: Option<String>,
    pub bot: Option<bool>,
    pub username: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AvatarDecoration {
    pub asset: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiscordTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: String,
    pub scope: String,
}
