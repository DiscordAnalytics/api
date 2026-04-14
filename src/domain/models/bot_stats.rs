use std::collections::HashMap;

use apistos::ApiComponent;
use mongodb::bson::DateTime;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::impl_kv_iter;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BotStats {
    pub added_guilds: i32,
    pub bot_id: String,
    pub custom_events: HashMap<String, i32>,
    pub date: DateTime,
    pub guilds: Option<Vec<Guild>>,
    pub guild_count: i32,
    pub guild_locales: Vec<Locale>,
    pub guild_members: GuildMembers,
    pub interactions: Vec<Interaction>,
    pub interactions_locales: Vec<Locale>,
    pub removed_guilds: i32,
    pub user_count: i32,
    pub user_install_count: Option<i32>,
    pub users_type: Option<UserType>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Guild {
    pub guild_id: String,
    pub icon: Option<String>,
    pub interactions: i32,
    pub members: i32,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, ApiComponent, JsonSchema)]
pub struct GuildMembers {
    pub little: i32,
    pub medium: i32,
    pub big: i32,
    pub huge: i32,
}

impl_kv_iter!(
    GuildMembers,
    [
        ("little", little),
        ("medium", medium),
        ("big", big),
        ("huge", huge),
    ]
);

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Interaction {
    pub command_type: Option<i32>,
    pub name: String,
    pub number: i32,
    #[serde(rename = "type")]
    pub type_: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, ApiComponent, JsonSchema)]
pub struct Locale {
    pub locale: String,
    pub number: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserType {
    pub admin: i32,
    pub moderator: i32,
    pub new_member: i32,
    pub other: i32,
    pub private_message: i32,
}

impl_kv_iter!(
    UserType,
    [
        ("admin", admin),
        ("moderator", moderator),
        ("newMember", new_member),
        ("other", other),
        ("privateMessage", private_message),
    ]
);
