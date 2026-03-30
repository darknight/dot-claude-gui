use serde::{Deserialize, Serialize};
use crate::settings::Settings;

// ---------------------------------------------------------------------------
// WebSocket event types (server → client)
// ---------------------------------------------------------------------------

/// Events emitted by the daemon over the WebSocket connection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum WsEvent {
    /// The settings file changed on disk; carries the new settings.
    ConfigChanged {
        settings: Settings,
        /// Source that triggered the change (e.g. "file-watcher", "api").
        source: Option<String>,
    },

    /// One or more validation errors were detected in the settings file.
    ValidationError {
        errors: Vec<WsValidationError>,
    },

    /// A line of output from a background command (e.g. a hook check).
    CommandOutput {
        command_id: String,
        line: String,
        stream: CommandStream,
    },

    /// A background command finished.
    CommandCompleted {
        command_id: String,
        exit_code: i32,
    },

    /// Sent immediately after the WebSocket handshake succeeds.
    Connected {
        daemon_version: String,
    },
}

/// Which output stream the `CommandOutput` line came from.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CommandStream {
    Stdout,
    Stderr,
}

/// Field-level validation error carried inside `WsEvent::ValidationError`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WsValidationError {
    pub field: String,
    pub message: String,
}

// ---------------------------------------------------------------------------
// WebSocket message types (client → server)
// ---------------------------------------------------------------------------

/// Messages sent by the GUI client to the daemon over WebSocket.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum WsClientMessage {
    /// Subscribe to one or more event topics.
    Subscribe {
        topics: Vec<String>,
    },

    /// Unsubscribe from one or more event topics.
    Unsubscribe {
        topics: Vec<String>,
    },
}
