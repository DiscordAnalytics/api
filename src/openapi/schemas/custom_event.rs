use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::models::CustomEvent;

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CustomEventResponse {
    pub bot_id: String,
    pub event_key: String,
    pub graph_name: String,
}

impl From<CustomEvent> for CustomEventResponse {
    fn from(event: CustomEvent) -> Self {
        Self {
            bot_id: event.bot_id,
            event_key: event.event_key,
            graph_name: event.graph_name,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CustomEventBody {
    pub event_key: String,
    pub graph_name: String,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct CustomEventDeleteResponse {
    pub message: String,
}
