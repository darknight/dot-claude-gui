use claude_types::{CommandStream, Settings, WsValidationError};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigChangedPayload {
    pub settings: Settings,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationErrorPayload {
    pub errors: Vec<WsValidationError>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandOutputPayload {
    pub command_id: String,
    pub line: String,
    pub stream: CommandStream,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandCompletedPayload {
    pub command_id: String,
    pub exit_code: i32,
}

pub const EVT_CONFIG_CHANGED: &str = "config-changed";
pub const EVT_VALIDATION_ERROR: &str = "validation-error";
pub const EVT_COMMAND_OUTPUT: &str = "command-output";
pub const EVT_COMMAND_COMPLETED: &str = "command-completed";
