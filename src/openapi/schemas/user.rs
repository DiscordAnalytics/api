use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    domain::{auth::is_admin, models::User},
    openapi::schemas::BotResponse,
};

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub admin: bool,
    pub avatar: Option<String>,
    pub avatar_decoration: Option<String>,
    pub bots_limit: i32,
    pub created_at: String,
    pub joined_at: String,
    pub suspended: bool,
    pub username: String,
    pub user_id: String,
}

impl TryFrom<User> for UserResponse {
    type Error = anyhow::Error;

    fn try_from(user: User) -> Result<Self, Self::Error> {
        Ok(Self {
            admin: is_admin(&user.user_id),
            avatar: user.avatar,
            avatar_decoration: user.avatar_decoration,
            bots_limit: user.bots_limit,
            created_at: user.created_at.try_to_rfc3339_string()?,
            joined_at: user.joined_at.try_to_rfc3339_string()?,
            suspended: user.suspended,
            username: user.username,
            user_id: user.user_id,
        })
    }
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserUpdateRequest {
    pub bots_limit: i32,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserBotsResponse {
    pub owned_bots: Vec<BotResponse>,
    pub team_bots: Vec<BotResponse>,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct UserSuspendRequest {
    pub reason: String,
}
