use serde::{Deserialize, Serialize};
use crate::settings::Settings;

// ---------------------------------------------------------------------------
// REST API types
// ---------------------------------------------------------------------------

/// Response returned when fetching the current configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConfigResponse {
    pub settings: Settings,
    /// ISO-8601 timestamp of the last modification.
    pub last_modified: Option<String>,
    /// ETag / version token for optimistic concurrency.
    pub version: Option<String>,
}

/// Request body for updating the configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateConfigRequest {
    pub settings: Settings,
    /// Optional ETag from a previous `ConfigResponse` for optimistic locking.
    pub if_match: Option<String>,
}

/// A project entry in the project registry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectEntry {
    pub id: String,
    pub name: String,
    pub path: String,
    pub registered_at: Option<String>,
}

/// Request body for registering a new project.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RegisterProjectRequest {
    pub name: String,
    pub path: String,
}

/// Response from the daemon health endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: Option<u64>,
}

/// Generic error response body.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub validation_errors: Vec<ValidationError>,
}

/// A single field-level validation error.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}
