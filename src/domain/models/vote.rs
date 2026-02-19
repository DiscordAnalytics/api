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

impl Vote {
    pub fn with_bot_id(mut self, bot_id: String) -> Self {
        self.bot_id = bot_id;
        self
    }

    pub fn with_count(mut self, count: i32) -> Self {
        self.count = count;
        self
    }

    pub fn with_date(mut self, date: DateTime) -> Self {
        self.date = date;
        self
    }

    pub fn with_provider(mut self, provider: String) -> Self {
        self.provider = provider;
        self
    }
}
