use std::collections::HashMap;

use apistos::ApiComponent;
use mongodb::bson::DateTime;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::utils::constants::{
    MAX_CUSTOM_EVENTS_PER_BOT, MAX_GOALS_PER_BOT, MAX_TEAMMATES_PER_BOT,
};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bot {
    pub advanced_stats: bool,
    pub avatar: Option<String>,
    pub bot_id: String,
    pub custom_events_limit: i32,
    pub framework: Option<String>,
    pub goals_limit: i32,
    pub language: Option<String>,
    pub last_push: Option<DateTime>,
    pub owner_id: String,
    pub suspended: bool,
    pub team: Vec<String>,
    pub teammates_limit: i32,
    pub(crate) token: String,
    pub username: String,
    pub version: Option<String>,
    pub warn_level: i32,
    pub watched_since: DateTime,
    pub webhooks_config: WebhooksConfig,
}

impl Bot {
    pub fn new(
        bot_id: &str,
        owner_id: &str,
        token: String,
        username: &str,
        avatar: Option<&str>,
    ) -> Self {
        Self {
            advanced_stats: false,
            avatar: avatar.map(|s| s.to_string()),
            bot_id: bot_id.to_string(),
            custom_events_limit: MAX_CUSTOM_EVENTS_PER_BOT,
            framework: None,
            goals_limit: MAX_GOALS_PER_BOT,
            language: None,
            last_push: None,
            owner_id: owner_id.to_string(),
            suspended: false,
            team: Vec::new(),
            teammates_limit: MAX_TEAMMATES_PER_BOT,
            token,
            username: username.to_string(),
            version: None,
            warn_level: 0,
            watched_since: DateTime::now(),
            webhooks_config: WebhooksConfig::default(),
        }
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

#[derive(
    Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, ApiComponent, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct WebhooksConfig {
    pub webhook_url: Option<String>,
    #[serde(flatten)]
    pub webhooks: HashMap<String, WebhookConfig>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WebhookConfig {
    pub connection_id: Option<String>,
    pub webhook_secret: Option<String>,
}
