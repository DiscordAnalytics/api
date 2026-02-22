use mongodb::bson::DateTime;
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
    pub fn is_expired(&self) -> bool {
        self.expiration < DateTime::now()
    }
}
