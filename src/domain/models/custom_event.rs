use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomEvent {
    pub bot_id: String,
    pub event_key: String,
    pub graph_name: String,
}
