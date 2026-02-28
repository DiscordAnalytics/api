use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bot {
    pub advanced_stats: bool,
    pub avatar: Option<String>,
    pub bot_id: String,
    pub framework: Option<String>,
    pub goals_limit: i32,
    pub language: Option<String>,
    pub last_push: Option<DateTime>,
    pub owner_id: String,
    pub suspended: bool,
    pub team: Vec<String>,
    pub(crate) token: String,
    pub username: String,
    pub version: Option<String>,
    pub votes_webhook_url: Option<String>,
    pub warn_level: i32,
    pub watched_since: Option<DateTime>,
}

impl Bot {
    pub fn new(
        bot_id: &str,
        owner_id: &str,
        token: String,
        username: &str,
        avatar: Option<&str>,
    ) -> Self {
        Bot {
            advanced_stats: false,
            avatar: avatar.map(|s| s.to_string()),
            bot_id: bot_id.to_string(),
            framework: None,
            goals_limit: 30,
            language: None,
            last_push: None,
            owner_id: owner_id.to_string(),
            suspended: false,
            team: Vec::new(),
            token,
            username: username.to_string(),
            version: None,
            votes_webhook_url: None,
            warn_level: 0,
            watched_since: None,
        }
    }

    pub fn token(self) -> String {
        self.token
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
