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
    pub custom_events: Option<HashMap<String, i32>>,
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
#[serde(rename_all = "camelCase")]
pub struct BotStatsBodyOld {
    pub added_guilds: i32,
    pub custom_events: Option<HashMap<String, i32>>,
    pub guilds: i32,
    pub guilds_locales: Vec<Locale>,
    pub guild_members: GuildMembers,
    pub guilds_stats: Option<Vec<Guild>>,
    pub interactions: Vec<Interaction>,
    pub locales: Vec<Locale>,
    pub removed_guilds: i32,
    pub users: i32,
    pub user_install_count: Option<i32>,
    pub users_type: Option<UserType>,
}

impl BotStatsBodyOld {
    pub fn into(self, bot_id: &str, date: &DateTime) -> BotStats {
        BotStats {
            added_guilds: self.added_guilds,
            bot_id: bot_id.to_string(),
            custom_events: self.custom_events,
            date: date.to_owned(),
            guilds: self.guilds_stats,
            guild_count: self.guilds,
            guild_locales: self.guilds_locales,
            guild_members: self.guild_members,
            interactions: self.interactions,
            interactions_locales: self.locales,
            removed_guilds: self.removed_guilds,
            user_count: self.users,
            user_install_count: self.user_install_count,
            users_type: self.users_type,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BotStatsBodyNew {
    pub added_guilds: i32,
    pub custom_events: Option<HashMap<String, i32>>,
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

impl BotStatsBodyNew {
    pub fn into(self, bot_id: &str, date: &DateTime) -> BotStats {
        BotStats {
            added_guilds: self.added_guilds,
            bot_id: bot_id.to_string(),
            custom_events: self.custom_events,
            date: date.to_owned(),
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

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct BotStatsUpdateResponse {
    pub message: String,
}
