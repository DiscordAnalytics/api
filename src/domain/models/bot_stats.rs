use std::{collections::HashMap, vec::IntoIter};

use apistos::ApiComponent;
use mongodb::bson::DateTime;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BotStats {
    pub added_guilds: i32,
    pub bot_id: String,
    pub custom_events: Option<HashMap<String, i32>>,
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

impl IntoIterator for GuildMembers {
    type Item = (String, i32);
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        vec![
            ("little".to_string(), self.little),
            ("medium".to_string(), self.medium),
            ("big".to_string(), self.big),
            ("huge".to_string(), self.huge),
        ]
        .into_iter()
    }
}

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

impl IntoIterator for UserType {
    type Item = (String, i32);
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        vec![
            ("admin".to_string(), self.admin),
            ("moderator".to_string(), self.moderator),
            ("newMember".to_string(), self.new_member),
            ("other".to_string(), self.other),
            ("privateMessage".to_string(), self.private_message),
        ]
        .into_iter()
    }
}
