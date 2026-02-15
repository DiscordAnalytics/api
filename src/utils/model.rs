use std::collections::HashMap;

use mongodb::bson::{DateTime, oid::ObjectId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct BotStats {
    #[serde(rename = "botId")]
    pub bot_id: String,
    pub date: DateTime,
    #[serde(rename = "guildCount")]
    pub guild_count: i32,
    #[serde(rename = "guildLocales")]
    pub guild_locales: Vec<Locale>,
    #[serde(rename = "guildMembers")]
    pub guild_members: GuildMembers,
    pub interactions: Vec<Interaction>,
    #[serde(rename = "interactionsLocales")]
    pub interactions_locales: Vec<Locale>,
    pub guilds: Option<Vec<Guild>>,
    #[serde(rename = "userCount")]
    pub user_count: i32,
    #[serde(rename = "addedGuilds")]
    pub added_guilds: i32,
    #[serde(rename = "removedGuilds")]
    pub removed_guilds: i32,
    pub users_type: Option<UserType>,
    pub custom_events: Option<HashMap<String, i32>>,
    pub user_install_count: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct CustomEvents {
    pub bot_id: String,
    pub event_key: String,
    pub graph_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct GlobalStats {
    pub date: DateTime,
    #[serde(rename = "botCount")]
    pub bot_count: i32,
    #[serde(rename = "userCount")]
    pub user_count: i32,
    #[serde(rename = "registeredBots")]
    pub registered_bots: i32,
    #[serde(rename = "logsEntryCount")]
    pub logs_entry_count: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Guild {
    #[serde(rename = "guildId")]
    pub guild_id: String,
    pub name: String,
    pub icon: Option<String>,
    pub interactions: i32,
    pub members: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct GuildMembers {
    pub little: i32,
    pub medium: i32,
    pub big: i32,
    pub huge: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Interaction {
    pub name: String,
    pub number: i32,
    #[serde(rename = "type")]
    pub type_: i32,
    pub command_type: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Locale {
    pub locale: String,
    pub number: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct StatsReport {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub bot_id: String,
    pub frequency: String,
    pub user_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct TeamInvitation {
    pub invitation_id: String,
    pub user_id: String,
    pub bot_id: String,
    pub expiration: DateTime,
    pub accepted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct User {
    #[serde(rename = "userId")]
    pub user_id: String,
    pub username: String,
    pub mail: String,
    pub banned: bool,
    pub avatar: String,
    pub token: String,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime,
    #[serde(rename = "joinedAt")]
    pub joined_at: DateTime,
    #[serde(rename = "botsLimit")]
    pub bots_limit: i32,
    pub avatar_decoration: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct UserType {
    pub admin: i32,
    pub moderator: i32,
    pub new_member: i32,
    pub other: i32,
    pub private_message: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Vote {
    pub date: DateTime,
    pub provider: String,
    #[serde(rename = "botId")]
    pub bot_id: String,
    pub count: i32,
}
