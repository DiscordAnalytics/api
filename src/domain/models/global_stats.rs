use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct GlobalStats {
    #[serde(rename = "botCount")]
    pub bot_count: i32,
    pub date: DateTime,
    #[serde(rename = "logsEntryCount")]
    pub logs_entry_count: i32,
    #[serde(rename = "registeredBots")]
    pub registered_bots: i32,
    #[serde(rename = "userCount")]
    pub user_count: i32,
}

impl GlobalStats {
    pub fn with_bot_count(mut self, bot_count: i32) -> Self {
        self.bot_count = bot_count;
        self
    }

    pub fn with_date(mut self, date: DateTime) -> Self {
        self.date = date;
        self
    }

    pub fn with_logs_entry_count(mut self, logs_entry_count: i32) -> Self {
        self.logs_entry_count = logs_entry_count;
        self
    }

    pub fn with_registered_bots(mut self, registered_bots: i32) -> Self {
        self.registered_bots = registered_bots;
        self
    }

    pub fn with_user_count(mut self, user_count: i32) -> Self {
        self.user_count = user_count;
        self
    }
}
