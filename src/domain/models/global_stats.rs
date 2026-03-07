use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalStats {
    pub bot_count: i32,
    pub date: DateTime,
    pub registered_bots: i32,
    pub user_count: i32,
}
