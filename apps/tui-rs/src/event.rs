use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct HookEvent {
    #[serde(default)]
    pub source_app: String,
    #[serde(default)]
    pub session_id: String,
    #[serde(default)]
    pub hook_event_type: String,
    #[serde(default)]
    pub payload: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub timestamp: i64,
    #[serde(default)]
    pub model_name: String,
    #[serde(default)]
    pub id: Option<i64>,
}

impl HookEvent {
    pub fn agent_id(&self) -> String {
        let short = if self.session_id.len() >= 8 {
            &self.session_id[..8]
        } else if self.session_id.is_empty() {
            "????????"
        } else {
            &self.session_id
        };
        format!("{}:{}", self.source_app, short)
    }

    pub fn tool_name(&self) -> Option<&str> {
        self.payload
            .get("tool_name")
            .and_then(|v| v.as_str())
    }
}
