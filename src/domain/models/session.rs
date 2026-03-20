use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};

use crate::utils::constants::REFRESH_TOKEN_LIFETIME;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub active: bool,
    pub created_at: DateTime,
    pub device_info: Option<String>,
    pub expires_at: DateTime,
    pub ip_address: Option<String>,
    pub last_used_at: DateTime,
    pub refresh_token_hash: String,
    pub session_id: String,
    pub user_agent: Option<String>,
    pub user_id: String,
}

impl Session {
    pub fn new(user_id: String, refresh_token_hash: String, session_id: String) -> Self {
        let now = DateTime::now();
        let expires_at =
            DateTime::from_millis(now.timestamp_millis() + (REFRESH_TOKEN_LIFETIME * 1000));

        Self {
            active: true,
            created_at: now,
            device_info: None,
            expires_at,
            ip_address: None,
            last_used_at: now,
            refresh_token_hash,
            session_id,
            user_agent: None,
            user_id,
        }
    }

    pub fn with_ip(mut self, ip: String) -> Self {
        self.ip_address = Some(ip);
        self
    }

    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at < DateTime::now()
    }
}
