use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub avatar: String,
    pub avatar_decoration: Option<String>,
    pub bots_limit: i32,
    pub created_at: DateTime,
    pub joined_at: DateTime,
    pub mail: String,
    pub suspended: bool,
    pub username: String,
    pub user_id: String,
}
