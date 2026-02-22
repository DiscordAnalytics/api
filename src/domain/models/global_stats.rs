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
