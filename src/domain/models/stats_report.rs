use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsReport {
    pub bot_id: String,
    pub frequency: String,
    pub user_id: String,
}

impl StatsReport {
    pub fn new(bot_id: &str, user_id: &str, frequency: StatsReportFrequency) -> Self {
        Self {
            bot_id: bot_id.to_string(),
            frequency: frequency.into(),
            user_id: user_id.to_string(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize, ApiComponent, JsonSchema)]
#[serde(rename_all = "lowercase")]
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
