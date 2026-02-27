use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthCallbackQuery {
    pub code: String,
    pub redirection: String,
    pub scopes: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiscordOAuthUser {
    pub id: String,
    pub username: String,
    pub avatar: Option<String>,
    pub avatar_decoration_data: Option<AvatarDecoration>,
    pub email: Option<String>,
    pub discriminator: String,
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
