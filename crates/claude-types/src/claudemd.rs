use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeMdFile {
    pub id: String,
    pub scope: String,
    pub filename: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_name: Option<String>,
    pub exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeMdFileDetail {
    pub id: String,
    pub scope: String,
    pub filename: String,
    pub path: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateClaudeMdRequest {
    pub content: String,
}
