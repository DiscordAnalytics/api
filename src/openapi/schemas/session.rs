use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::models::Session;

#[derive(Debug, Clone, Serialize, Deserialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: i64,
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionResponse {
    pub active: bool,
    pub created_at: String,
    pub device_info: Option<String>,
    pub ip_address: Option<String>,
    pub last_used_at: String,
    pub session_id: String,
    pub user_agent: Option<String>,
}

impl TryFrom<Session> for SessionResponse {
    type Error = anyhow::Error;

    fn try_from(session: Session) -> Result<Self, Self::Error> {
        Ok(Self {
            active: session.active,
            created_at: session.created_at.try_to_rfc3339_string()?,
            device_info: session.device_info,
            ip_address: session.ip_address,
            last_used_at: session.last_used_at.try_to_rfc3339_string()?,
            session_id: session.session_id,
            user_agent: session.user_agent,
        })
    }
}
