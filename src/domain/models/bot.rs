use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Bot {
    #[serde(rename = "advancedStats")]
    pub advanced_stats: bool,
    pub avatar: Option<String>,
    #[serde(rename = "botId")]
    pub bot_id: String,
    pub framework: Option<String>,
    pub goals_limit: Option<i32>,
    pub language: Option<String>,
    #[serde(rename = "lastPush")]
    pub last_push: Option<DateTime>,
    #[serde(rename = "ownerId")]
    pub owner_id: String,
    pub suspended: bool,
    pub team: Vec<String>,
    pub(crate) token: String,
    pub username: String,
    pub version: Option<String>,
    #[serde(rename = "votesWebhookUrl")]
    pub votes_webhook_url: Option<String>,
    #[serde(rename = "warnLevel")]
    pub warn_level: Option<i32>,
    #[serde(rename = "watchedSince")]
    pub watched_since: Option<DateTime>,
}

impl Bot {
    pub fn with_advanced_stats(mut self, advanced_stats: bool) -> Self {
        self.advanced_stats = advanced_stats;
        self
    }

    pub fn with_avatar(mut self, avatar: Option<String>) -> Self {
        self.avatar = avatar;
        self
    }

    pub fn with_bot_id(mut self, bot_id: String) -> Self {
        self.bot_id = bot_id;
        self
    }

    pub fn with_framework(mut self, framework: Framework) -> Self {
        self.framework = Some(framework.into());
        self
    }

    pub fn with_goals_limit(mut self, goals_limit: Option<i32>) -> Self {
        self.goals_limit = goals_limit;
        self
    }

    pub fn with_language(mut self, language: Language) -> Self {
        self.language = Some(language.into());
        self
    }

    pub fn with_last_push(mut self, last_push: Option<DateTime>) -> Self {
        self.last_push = last_push;
        self
    }

    pub fn with_owner_id(mut self, owner_id: String) -> Self {
        self.owner_id = owner_id;
        self
    }

    pub fn with_suspended(mut self, suspended: bool) -> Self {
        self.suspended = suspended;
        self
    }

    pub fn with_team(mut self, team: Vec<String>) -> Self {
        self.team = team;
        self
    }

    pub fn with_token(mut self, token: String) -> Self {
        self.token = token;
        self
    }

    pub fn with_username(mut self, username: String) -> Self {
        self.username = username;
        self
    }

    pub fn with_version(mut self, version: Option<String>) -> Self {
        self.version = version;
        self
    }

    pub fn with_votes_webhook_url(mut self, votes_webhook_url: Option<String>) -> Self {
        self.votes_webhook_url = votes_webhook_url;
        self
    }

    pub fn with_warn_level(mut self, warn_level: Option<i32>) -> Self {
        self.warn_level = warn_level;
        self
    }

    pub fn with_watched_since(mut self, watched_since: Option<DateTime>) -> Self {
        self.watched_since = watched_since;
        self
    }

    pub fn is_owner(&self, user_id: &str) -> bool {
        self.owner_id == user_id
    }

    pub fn is_team_member(&self, user_id: &str) -> bool {
        self.team.contains(&user_id.to_string())
    }

    pub fn has_access(&self, user_id: &str) -> bool {
        self.is_owner(user_id) || self.is_team_member(user_id)
    }

    pub fn add_team_member(mut self, user_id: String) -> Self {
        if !self.team.contains(&user_id) {
            self.team.push(user_id);
        }
        self
    }

    pub fn remove_team_member(mut self, user_id: &str) -> Self {
        self.team.retain(|id| id != user_id);
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Framework {
    DiscordPy,
    PyCord,
    DiscordJs,
    Eris,
    Oceanic,
    DiscordJsLite,
    Custom,
}

impl Framework {
    pub fn from_str(lib: &str) -> Framework {
        match lib {
            "discord.py" => Framework::DiscordPy,
            "pycord" => Framework::PyCord,
            "discord.js" => Framework::DiscordJs,
            "eris" => Framework::Eris,
            "oceanic" => Framework::Oceanic,
            "discord.js-light" => Framework::DiscordJsLite,
            _ => Framework::Custom,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Framework::DiscordPy => "discord.py",
            Framework::PyCord => "pycord",
            Framework::DiscordJs => "discord.js",
            Framework::Eris => "eris",
            Framework::Oceanic => "oceanic",
            Framework::DiscordJsLite => "discord.js-light",
            Framework::Custom => "custom",
        }
    }
}

impl From<Framework> for String {
    fn from(framework: Framework) -> Self {
        framework.as_str().to_string()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Language {
    Python,
    JavaScript,
    Other,
}

impl Language {
    pub fn from_str(lang: &str) -> Language {
        match lang {
            "python" => Language::Python,
            "javascript" => Language::JavaScript,
            _ => Language::Other,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Language::Python => "python",
            Language::JavaScript => "javascript",
            Language::Other => "other",
        }
    }
}

impl From<Language> for String {
    fn from(language: Language) -> Self {
        language.as_str().to_string()
    }
}
