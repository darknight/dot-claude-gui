// src-tauri/src/app_config.rs
//
// AppConfig v2 — single source of truth for GUI preferences, account
// registry, project bindings, and known-project list.
//
// IO: atomic write via tempfile + rename. Migration runs at startup.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    /// Schema version. Bump on breaking changes. Absent => v1.
    #[serde(default)]
    pub schema_version: u32,

    // ── App preferences ──────────────────────────────────────────
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_font_size")]
    pub font_size: u32,
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: u32,
    #[serde(default = "default_preferred_terminal")]
    pub preferred_terminal: String,

    // ── Account registry ────────────────────────────────────────
    #[serde(default)]
    pub accounts: Vec<Account>,

    // ── Project bindings: path → { account, launch } ────────────
    #[serde(default)]
    pub projects: BTreeMap<String, ProjectBinding>,

    // ── All project paths the user has registered ───────────────
    #[serde(default)]
    pub known_projects: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub name: String,
    pub display_name: String,
    #[serde(default)]
    pub is_native: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectBinding {
    /// Account `name` from `accounts` (or `"default"` for native).
    pub account: String,
    #[serde(default)]
    pub launch: LaunchConfig,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchConfig {
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub args: Vec<String>,
}

// ── Defaults ────────────────────────────────────────────────────
fn default_theme() -> String { "system".to_string() }
fn default_language() -> String { "zh-CN".to_string() }
fn default_font_size() -> u32 { 14 }
fn default_sidebar_width() -> u32 { 140 }
fn default_preferred_terminal() -> String { "terminal".to_string() }

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            theme: default_theme(),
            language: default_language(),
            font_size: default_font_size(),
            sidebar_width: default_sidebar_width(),
            preferred_terminal: default_preferred_terminal(),
            accounts: vec![],
            projects: BTreeMap::new(),
            known_projects: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_v2() {
        assert_eq!(AppConfig::default().schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn round_trips_through_json() {
        let cfg = AppConfig::default();
        let s = serde_json::to_string(&cfg).unwrap();
        let back: AppConfig = serde_json::from_str(&s).unwrap();
        assert_eq!(cfg, back);
    }

    #[test]
    fn camel_case_json_field_names() {
        let json = serde_json::to_value(AppConfig::default()).unwrap();
        let obj = json.as_object().unwrap();
        assert!(obj.contains_key("schemaVersion"));
        assert!(obj.contains_key("fontSize"));
        assert!(obj.contains_key("sidebarWidth"));
        assert!(obj.contains_key("preferredTerminal"));
        assert!(obj.contains_key("knownProjects"));
    }
}
