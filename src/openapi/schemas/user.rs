use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::models::User;

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub avatar: String,
    pub avatar_decoration: Option<String>,
    pub banned: bool,
    pub bots_limit: i32,
    pub created_at: String,
    pub joined_at: String,
    pub username: String,
    pub user_id: String,
}

impl TryFrom<User> for UserResponse {
    type Error = anyhow::Error;

    fn try_from(user: User) -> Result<Self, Self::Error> {
        Ok(Self {
            avatar: user.avatar,
            avatar_decoration: user.avatar_decoration,
            banned: user.banned,
            bots_limit: user.bots_limit,
            created_at: user
                .created_at
                .try_to_rfc3339_string()?,
            joined_at: user
                .joined_at
                .try_to_rfc3339_string()?,
            username: user.username,
            user_id: user.user_id,
        })
    }
}
