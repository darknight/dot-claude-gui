use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Skills API types
// ---------------------------------------------------------------------------

/// Information about a user-created skill (SKILL.md with YAML front-matter).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillInfo {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub source: String,
    pub path: String,
    pub valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_error: Option<String>,
}

/// Response for GET /api/v1/skills/{id}/content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillContentResponse {
    pub id: String,
    pub content: String,
}
