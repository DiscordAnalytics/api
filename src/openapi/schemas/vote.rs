use std::collections::HashMap;

use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::models::Vote;

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct VoteResponse {
    pub bot_id: String,
    pub date: String,
    #[serde(flatten)]
    pub votes: HashMap<String, u32>,
}

impl TryFrom<Vote> for VoteResponse {
    type Error = anyhow::Error;

    fn try_from(vote: Vote) -> Result<Self, Self::Error> {
        Ok(Self {
            bot_id: vote.bot_id,
            date: vote.date.try_to_rfc3339_string()?,
            votes: vote.votes,
        })
    }
}
