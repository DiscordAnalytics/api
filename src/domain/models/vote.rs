use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Vote {
    #[serde(rename = "botId")]
    pub bot_id: String,
    pub count: i32,
    pub date: DateTime,
    pub provider: String,
}
