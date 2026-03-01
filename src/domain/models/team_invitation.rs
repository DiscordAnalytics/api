use chrono::{Duration, Utc};
use mongodb::bson::{DateTime, Uuid};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamInvitation {
    pub accepted: bool,
    pub bot_id: String,
    pub expiration: DateTime,
    pub invitation_id: String,
    pub user_id: String,
}

impl TeamInvitation {
    pub fn new(bot_id: &str, user_id: &str) -> Self {
        let expiration = Utc::now() + Duration::days(7);
        let datetime_expiration = DateTime::from_millis(expiration.timestamp_millis());

        Self {
            accepted: false,
            bot_id: bot_id.to_string(),
            expiration: datetime_expiration,
            invitation_id: Uuid::new().to_string(),
            user_id: user_id.to_string(),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expiration < DateTime::now()
    }
}
