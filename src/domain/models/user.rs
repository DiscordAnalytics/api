use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct User {
    pub avatar: String,
    pub avatar_decoration: Option<String>,
    pub banned: bool,
    #[serde(rename = "botsLimit")]
    pub bots_limit: i32,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime,
    #[serde(rename = "joinedAt")]
    pub joined_at: DateTime,
    pub mail: String,
    pub(crate) token: String,
    pub username: String,
    #[serde(rename = "userId")]
    pub user_id: String,
}

impl User {
    pub fn with_avatar(mut self, avatar: String) -> Self {
        self.avatar = avatar;
        self
    }

    pub fn with_avatar_decoration(mut self, avatar_decoration: Option<String>) -> Self {
        self.avatar_decoration = avatar_decoration;
        self
    }

    pub fn with_banned(mut self, banned: bool) -> Self {
        self.banned = banned;
        self
    }

    pub fn with_bots_limit(mut self, bots_limit: i32) -> Self {
        self.bots_limit = bots_limit;
        self
    }

    pub fn with_created_at(mut self, created_at: DateTime) -> Self {
        self.created_at = created_at;
        self
    }

    pub fn with_joined_at(mut self, joined_at: DateTime) -> Self {
        self.joined_at = joined_at;
        self
    }

    pub fn with_mail(mut self, mail: String) -> Self {
        self.mail = mail;
        self
    }

    pub fn with_token(mut self, token: String) -> Self {
        self.token = token;
        self
    }

    pub fn with_username(mut self, username: String) -> Self {
        self.username = username;
        self
    }

    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = user_id;
        self
    }
}
