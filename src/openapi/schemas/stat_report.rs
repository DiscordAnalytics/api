use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::models::StatsReport;

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StatsReportResponse {
    pub id: String,
    pub bot_id: String,
    pub frequency: String,
    pub user_id: String,
}

impl TryFrom<StatsReport> for StatsReportResponse {
    type Error = anyhow::Error;

    fn try_from(value: StatsReport) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.to_string(),
            bot_id: value.bot_id,
            frequency: value.frequency,
            user_id: value.user_id,
        })
    }
}
