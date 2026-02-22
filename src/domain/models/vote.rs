use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Vote {
    pub bot_id: String,
    pub count: i32,
    pub date: DateTime,
    pub provider: String,
}
