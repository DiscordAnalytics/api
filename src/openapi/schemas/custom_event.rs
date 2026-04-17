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

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CustomEventResponse {
    pub current_value: Option<i32>,
    pub default_value: Option<i32>,
    pub event_key: String,
    pub graph_name: String,
    #[deprecated]
    #[serde(rename = "today_value")]
    pub today_value: Option<i32>,
}

impl CustomEventResponse {
    pub fn new(event: CustomEvent, current_value: Option<i32>) -> Self {
        Self {
            current_value,
            today_value: current_value,
            ..Self::from(event)
        }
    }
}

impl From<CustomEvent> for CustomEventResponse {
    fn from(event: CustomEvent) -> Self {
        Self {
            current_value: None,
            default_value: event.default_value,
            event_key: event.event_key,
            graph_name: event.graph_name,
            today_value: None,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CustomEventUpdatePayload {
    pub graph_name: String,
}
