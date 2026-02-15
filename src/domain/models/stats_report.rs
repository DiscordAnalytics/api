use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct StatsReport {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub bot_id: String,
    pub frequency: String,
    pub user_id: String,
}

impl StatsReport {
    pub fn with_id(mut self, id: ObjectId) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_bot_id(mut self, bot_id: String) -> Self {
        self.bot_id = bot_id;
        self
    }

    pub fn with_frequency(mut self, frequency: StatsReportFrequency) -> Self {
        self.frequency = frequency.into();
        self
    }

    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = user_id;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StatsReportFrequency {
    Weekly,
    Monthly,
}

impl StatsReportFrequency {
    pub fn as_str(&self) -> &'static str {
        match self {
            StatsReportFrequency::Weekly => "weekly",
            StatsReportFrequency::Monthly => "monthly",
        }
    }
}

impl From<StatsReportFrequency> for String {
    fn from(frequency: StatsReportFrequency) -> Self {
        frequency.as_str().to_string()
    }
}
