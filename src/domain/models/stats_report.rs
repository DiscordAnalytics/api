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
