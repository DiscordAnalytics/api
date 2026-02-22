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
    pub fn token(self) -> String {
        self.token
    }
}
