use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct TeamInvitation {
    pub accepted: bool,
    pub bot_id: String,
    pub expiration: DateTime,
    pub invitation_id: String,
    pub user_id: String,
}

impl TeamInvitation {
    pub fn with_accepted(mut self, accepted: bool) -> Self {
        self.accepted = accepted;
        self
    }

    pub fn with_bot_id(mut self, bot_id: String) -> Self {
        self.bot_id = bot_id;
        self
    }

    pub fn with_expiration(mut self, expiration: DateTime) -> Self {
        self.expiration = expiration;
        self
    }

    pub fn with_invitation_id(mut self, invitation_id: String) -> Self {
        self.invitation_id = invitation_id;
        self
    }

    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = user_id;
        self
    }
}
