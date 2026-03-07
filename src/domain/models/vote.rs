use std::collections::HashMap;

use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Vote {
    pub bot_id: String,
    pub date: DateTime,
    #[serde(flatten)]
    pub votes: HashMap<String, u32>,
}

impl Vote {
    pub fn new(bot_id: String, date: DateTime) -> Self {
        Self {
            bot_id,
            date,
            votes: HashMap::new(),
        }
    }
}
