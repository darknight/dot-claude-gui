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
    /// Real filesystem path if this skill's directory (or the file itself)
    /// is reached through a symlink — i.e. the canonical target is not under
    /// the account's own dir. UI uses this to badge shared/external skills
    /// (e.g. ccs-shared) so users know the source isn't account-local.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_target: Option<String>,
}

/// Response for GET /api/v1/skills/{id}/content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillContentResponse {
    pub id: String,
    pub content: String,
}
