use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Memory API types
// ---------------------------------------------------------------------------

/// A project that has associated memory files.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryProject {
    pub id: String,
    pub project_path: String,
    pub file_count: usize,
}

/// Lightweight summary of a memory file (no content).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryFile {
    pub filename: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_type: Option<String>,
}

/// Full detail of a memory file including its content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryFileDetail {
    pub filename: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_type: Option<String>,
}

/// Request body for updating a memory file's content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMemoryRequest {
    pub content: String,
}
