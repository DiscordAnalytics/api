use std::collections::HashMap;

use apistos::ApiComponent;
use mongodb::bson::DateTime;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    domain::models::{BotStats, Guild, GuildMembers, Interaction, Locale, UserType},
    openapi::schemas::VoteResponse,
};

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct BotStatsResponse {
    pub stats: Vec<BotStatsContent>,
    pub votes: Vec<VoteResponse>,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BotStatsContent {
    pub added_guilds: i32,
    pub custom_events: HashMap<String, i32>,
    pub date: String,
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

impl TryFrom<BotStats> for BotStatsContent {
    type Error = anyhow::Error;

    fn try_from(stats: BotStats) -> Result<Self, Self::Error> {
        Ok(Self {
            added_guilds: stats.added_guilds,
            custom_events: stats.custom_events,
            date: stats.date.try_to_rfc3339_string()?,
            guilds: stats.guilds,
            guild_count: stats.guild_count,
            guild_locales: stats.guild_locales,
            guild_members: stats.guild_members,
            interactions: stats.interactions,
            interactions_locales: stats.interactions_locales,
            removed_guilds: stats.removed_guilds,
            user_count: stats.user_count,
            user_install_count: stats.user_install_count,
            users_type: stats.users_type,
        })
    }
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct BotStatsQuery {
    pub from: String,
    pub to: String,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(untagged)]
pub enum BotStatsBody {
    New(BotStatsBodyNew),
    Old(BotStatsBodyOld),
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct OldInteraction {
    pub command_type: Option<i32>,
    pub name: String,
    pub number: i32,
    #[serde(rename = "type")]
    pub type_: i32,
}

impl From<OldInteraction> for Interaction {
    fn from(value: OldInteraction) -> Self {
        Interaction {
            command_type: value.command_type,
            name: value.name,
            number: value.number,
            type_: value.type_,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct OldUserType {
    pub admin: i32,
    pub moderator: i32,
    pub new_member: i32,
    pub other: i32,
    pub private_message: i32,
}

impl From<OldUserType> for UserType {
    fn from(value: OldUserType) -> Self {
        UserType {
            admin: value.admin,
            moderator: value.moderator,
            new_member: value.new_member,
            other: value.other,
            private_message: value.private_message,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct BotStatsBodyOld {
    #[serde(rename = "addedGuilds")]
    pub added_guilds: i32,
    pub custom_events: Option<HashMap<String, i32>>,
    pub date: String,
    pub guilds: i32,
    #[serde(rename = "guildsLocales")]
    pub guilds_locales: Vec<Locale>,
    #[serde(rename = "guildMembers")]
    pub guild_members: GuildMembers,
    #[serde(rename = "guildsStats")]
    pub guilds_stats: Option<Vec<Guild>>,
    pub interactions: Vec<OldInteraction>,
    pub locales: Vec<Locale>,
    #[serde(rename = "removedGuilds")]
    pub removed_guilds: i32,
    pub users: i32,
    pub user_install_count: Option<i32>,
    pub users_type: Option<OldUserType>,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BotStatsBodyNew {
    pub added_guilds: i32,
    pub custom_events: HashMap<String, i32>,
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

#[derive(Clone, Debug)]
pub struct NormalizedStatsBody {
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

impl NormalizedStatsBody {
    pub fn from_old(old: BotStatsBodyOld, bot_id: &str, date: &DateTime) -> Self {
        Self {
            added_guilds: old.added_guilds,
            bot_id: bot_id.to_string(),
            custom_events: old.custom_events.unwrap_or_default(),
            date: *date,
            guilds: old.guilds_stats,
            guild_count: old.guilds,
            guild_locales: old.guilds_locales,
            guild_members: old.guild_members,
            interactions: old.interactions.into_iter().map(|i| i.into()).collect(),
            interactions_locales: old.locales,
            removed_guilds: old.removed_guilds,
            user_count: old.users,
            user_install_count: old.user_install_count,
            users_type: old.users_type.map(|u| u.into()),
        }
    }

    pub fn from_new(new: BotStatsBodyNew, bot_id: &str, date: &DateTime) -> Self {
        Self {
            added_guilds: new.added_guilds,
            bot_id: bot_id.to_string(),
            custom_events: new.custom_events,
            date: *date,
            guilds: new.guilds,
            guild_count: new.guild_count,
            guild_locales: new.guild_locales,
            guild_members: new.guild_members,
            interactions: new.interactions,
            interactions_locales: new.interactions_locales,
            removed_guilds: new.removed_guilds,
            user_count: new.user_count,
            user_install_count: new.user_install_count,
            users_type: new.users_type,
        }
    }

    pub fn into_stats(self) -> BotStats {
        BotStats {
            added_guilds: self.added_guilds,
            bot_id: self.bot_id,
            custom_events: self.custom_events,
            date: self.date,
            guilds: self.guilds,
            guild_count: self.guild_count,
            guild_locales: self.guild_locales,
            guild_members: self.guild_members,
            interactions: self.interactions,
            interactions_locales: self.interactions_locales,
            removed_guilds: self.removed_guilds,
            user_count: self.user_count,
            user_install_count: self.user_install_count,
            users_type: self.users_type,
        }
    }
}
