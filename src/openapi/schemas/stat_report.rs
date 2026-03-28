use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::models::{StatsReport, StatsReportFrequency};

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StatsReportResponse {
    pub bot_id: String,
    pub frequency: StatsReportFrequency,
    pub user_id: String,
}

impl TryFrom<StatsReport> for StatsReportResponse {
    type Error = anyhow::Error;

    fn try_from(value: StatsReport) -> Result<Self, Self::Error> {
        Ok(Self {
            bot_id: value.bot_id,
            frequency: value.frequency,
            user_id: value.user_id,
        })
    }
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StatsReportSubPayload {
    pub user_id: String,
    pub frequency: StatsReportFrequency,
}
