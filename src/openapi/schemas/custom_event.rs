use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::models::CustomEvent;

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CustomEventPayload {
    pub default_value: Option<i32>,
    pub event_key: String,
    pub graph_name: String,
}

impl From<CustomEvent> for CustomEventPayload {
    fn from(event: CustomEvent) -> Self {
        Self {
            default_value: event.default_value,
            event_key: event.event_key,
            graph_name: event.graph_name,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CustomEventUpdatePayload {
    pub graph_name: String,
}
