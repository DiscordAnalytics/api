use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::models::Vote;

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct VoteResponse {
    pub bot_id: String,
    pub count: i32,
    pub date: String,
    pub provider: String,
}

impl TryFrom<Vote> for VoteResponse {
    type Error = anyhow::Error;

    fn try_from(vote: Vote) -> Result<Self, Self::Error> {
        Ok(Self {
            bot_id: vote.bot_id,
            count: vote.count,
            date: vote.date.try_to_rfc3339_string()?,
            provider: vote.provider,
        })
    }
}
