use std::collections::HashMap;

use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct BotStats {
    #[serde(rename = "addedGuilds")]
    pub added_guilds: i32,
    #[serde(rename = "botId")]
    pub bot_id: String,
    pub custom_events: Option<HashMap<String, i32>>,
    pub date: DateTime,
    pub guilds: Option<Vec<Guild>>,
    #[serde(rename = "guildCount")]
    pub guild_count: i32,
    #[serde(rename = "guildLocales")]
    pub guild_locales: Vec<Locale>,
    #[serde(rename = "guildMembers")]
    pub guild_members: GuildMembers,
    pub interactions: Vec<Interaction>,
    #[serde(rename = "interactionsLocales")]
    pub interactions_locales: Vec<Locale>,
    #[serde(rename = "removedGuilds")]
    pub removed_guilds: i32,
    #[serde(rename = "userCount")]
    pub user_count: i32,
    pub user_install_count: Option<i32>,
    pub users_type: Option<UserType>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Guild {
    #[serde(rename = "guildId")]
    pub guild_id: String,
    pub icon: Option<String>,
    pub interactions: i32,
    pub members: i32,
    pub name: String,
}

impl Guild {
    pub fn with_guild_id(mut self, guild_id: String) -> Self {
        self.guild_id = guild_id;
        self
    }

    pub fn with_icon(mut self, icon: Option<String>) -> Self {
        self.icon = icon;
        self
    }

    pub fn with_interactions(mut self, interactions: i32) -> Self {
        self.interactions = interactions;
        self
    }

    pub fn with_members(mut self, members: i32) -> Self {
        self.members = members;
        self
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct GuildMembers {
    pub little: i32,
    pub medium: i32,
    pub big: i32,
    pub huge: i32,
}

impl GuildMembers {
    pub fn with_little(mut self, little: i32) -> Self {
        self.little = little;
        self
    }

    pub fn with_medium(mut self, medium: i32) -> Self {
        self.medium = medium;
        self
    }

    pub fn with_big(mut self, big: i32) -> Self {
        self.big = big;
        self
    }

    pub fn with_huge(mut self, huge: i32) -> Self {
        self.huge = huge;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Interaction {
    pub command_type: Option<i32>,
    pub name: String,
    pub number: i32,
    #[serde(rename = "type")]
    pub type_: i32,
}

impl Interaction {
    pub fn with_command_type(mut self, command_type: Option<i32>) -> Self {
        self.command_type = command_type;
        self
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn with_number(mut self, number: i32) -> Self {
        self.number = number;
        self
    }

    pub fn with_type(mut self, type_: i32) -> Self {
        self.type_ = type_;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Locale {
    pub locale: String,
    pub number: i32,
}

impl Locale {
    pub fn with_locale(mut self, locale: String) -> Self {
        self.locale = locale;
        self
    }

    pub fn with_number(mut self, number: i32) -> Self {
        self.number = number;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct UserType {
    pub admin: i32,
    pub moderator: i32,
    pub new_member: i32,
    pub other: i32,
    pub private_message: i32,
}

impl UserType {
    pub fn with_admin(mut self, admin: i32) -> Self {
        self.admin = admin;
        self
    }

    pub fn with_moderator(mut self, moderator: i32) -> Self {
        self.moderator = moderator;
        self
    }

    pub fn with_new_member(mut self, new_member: i32) -> Self {
        self.new_member = new_member;
        self
    }

    pub fn with_other(mut self, other: i32) -> Self {
        self.other = other;
        self
    }

    pub fn with_private_message(mut self, private_message: i32) -> Self {
        self.private_message = private_message;
        self
    }
}
