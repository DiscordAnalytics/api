use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct CustomEvent {
    pub bot_id: String,
    pub event_key: String,
    pub graph_name: String,
}

impl CustomEvent {
    pub fn with_bot_id(mut self, bot_id: String) -> Self {
        self.bot_id = bot_id;
        self
    }

    pub fn with_event_key(mut self, event_key: String) -> Self {
        self.event_key = event_key;
        self
    }

    pub fn with_graph_name(mut self, graph_name: String) -> Self {
        self.graph_name = graph_name;
        self
    }
}
