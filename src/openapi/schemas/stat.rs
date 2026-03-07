use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::models::GlobalStats;

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StatResponse {
    pub bot_count: i32,
    pub date: String,
    pub registered_bots: i32,
    pub user_count: i32,
}

impl TryFrom<GlobalStats> for StatResponse {
    type Error = anyhow::Error;

    fn try_from(user: GlobalStats) -> Result<Self, Self::Error> {
        Ok(Self {
            bot_count: user.bot_count,
            date: user.date.try_to_rfc3339_string()?,
            registered_bots: user.registered_bots,
            user_count: user.user_count,
        })
    }
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct StatsQuery {
    pub start: String,
    pub end: String,
}
