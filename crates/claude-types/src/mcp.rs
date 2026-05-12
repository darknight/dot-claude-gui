use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP server info for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerInfo {
    pub name: String,
    pub scope: String,
    pub transport: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// Request to add an MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddMcpServerRequest {
    pub name: String,
    pub transport: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_or_url: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

/// Request to launch claude in a project
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchRequest {
    /// Working directory to `cd` into before running `claude`. None means run in
    /// the terminal's default cwd (~), used for non-project flows like OAuth login.
    #[serde(default)]
    pub project_path: Option<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub args: Vec<String>,
    /// Preferred terminal app: "terminal" (default), "iterm2", or None for default.
    #[serde(default)]
    pub preferred_terminal: Option<String>,
    /// Account name from binding. None or "default" → no CLAUDE_CONFIG_DIR
    /// injection (claude uses native ~/.claude/). Any other name → inject
    /// CLAUDE_CONFIG_DIR=~/.dot-claude-gui/accounts/<name> unless user env
    /// already sets it.
    #[serde(default)]
    pub account: Option<String>,
}
