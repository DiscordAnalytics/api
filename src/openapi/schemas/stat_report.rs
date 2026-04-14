use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::models::StatsReportFrequency;

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StatsReportSubPayload {
    pub user_id: String,
    pub frequency: StatsReportFrequency,
}
