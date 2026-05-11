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

use std::path::Path;
use std::io::Write;

/// Read AppConfig from `path`. Returns `AppConfig::default()` if the file
/// doesn't exist; errors only on IO failure or unparseable JSON.
pub fn read_config(path: &Path) -> Result<AppConfig, String> {
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let bytes = std::fs::read(path)
        .map_err(|e| format!("read config: {e}"))?;
    serde_json::from_slice(&bytes)
        .map_err(|e| format!("parse config: {e}"))
}

/// Atomically write `cfg` to `path` via tempfile + rename.
pub fn write_config(path: &Path, cfg: &AppConfig) -> Result<(), String> {
    let dir = path.parent()
        .ok_or_else(|| "config path has no parent".to_string())?;
    std::fs::create_dir_all(dir)
        .map_err(|e| format!("mkdir config dir: {e}"))?;

    let json = serde_json::to_vec_pretty(cfg)
        .map_err(|e| format!("serialize config: {e}"))?;

    let mut tmp = tempfile::NamedTempFile::new_in(dir)
        .map_err(|e| format!("create tempfile: {e}"))?;
    tmp.write_all(&json)
        .map_err(|e| format!("write tempfile: {e}"))?;
    tmp.persist(path)
        .map_err(|e| format!("rename tempfile: {e}"))?;
    Ok(())
}

/// Migrate any older JSON shape to the current `AppConfig`. Idempotent for v2.
///
/// Strategy: deserialize known v1 keys, drop subpanelWidth, lift
/// launcherProjectEnv → projects + knownProjects, expand accounts.
pub fn migrate_from_v1(raw: serde_json::Value) -> Result<AppConfig, String> {
    // Fast path: already v2.
    if raw.get("schemaVersion")
        .and_then(|v| v.as_u64())
        .map(|n| n as u32)
        == Some(SCHEMA_VERSION)
    {
        return serde_json::from_value(raw)
            .map_err(|e| format!("parse v2 config: {e}"));
    }

    let mut out = AppConfig::default();

    // ── Preferences ──────────────────────────────────────────────
    if let Some(s) = raw.get("theme").and_then(|v| v.as_str()) {
        out.theme = s.to_string();
    }
    if let Some(s) = raw.get("language").and_then(|v| v.as_str()) {
        out.language = s.to_string();
    }
    if let Some(n) = raw.get("fontSize").and_then(|v| v.as_u64()) {
        out.font_size = n as u32;
    }
    if let Some(n) = raw.get("sidebarWidth").and_then(|v| v.as_u64()) {
        out.sidebar_width = n as u32;
    }
    if let Some(s) = raw.get("preferredTerminal").and_then(|v| v.as_str()) {
        out.preferred_terminal = s.to_string();
    }

    // ── Accounts ─────────────────────────────────────────────────
    if let Some(arr) = raw.get("accounts").and_then(|v| v.as_array()) {
        for entry in arr {
            let name = match entry.get("name").and_then(|v| v.as_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            let created_at = entry.get("createdAt")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            out.accounts.push(Account {
                display_name: name.clone(),
                name,
                is_native: false,
                created_at,
            });
        }
    }

    // ── launcherProjectEnv → projects + knownProjects ────────────
    if let Some(obj) = raw.get("launcherProjectEnv").and_then(|v| v.as_object()) {
        for (path, entry) in obj {
            out.known_projects.push(path.clone());

            let account = entry.get("accountName")
                .and_then(|v| v.as_str())
                .unwrap_or("default")
                .to_string();

            let mut env = BTreeMap::new();
            if let Some(arr) = entry.get("customEnv").and_then(|v| v.as_array()) {
                for kv in arr {
                    let enabled = kv.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
                    if !enabled { continue; }
                    let key = kv.get("key").and_then(|v| v.as_str()).unwrap_or("");
                    if key.is_empty() { continue; }
                    let value = kv.get("value").and_then(|v| v.as_str()).unwrap_or("");
                    env.insert(key.to_string(), value.to_string());
                }
            }

            let mut args = Vec::new();
            if let Some(arr) = entry.get("customArgs").and_then(|v| v.as_array()) {
                for ka in arr {
                    let enabled = ka.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
                    if !enabled { continue; }
                    let flag = ka.get("flag").and_then(|v| v.as_str()).unwrap_or("");
                    if flag.is_empty() { continue; }
                    args.push(flag.to_string());
                    // value is Option<String>; null/missing => bare flag.
                    if let Some(v) = ka.get("value").and_then(|v| v.as_str()) {
                        if !v.is_empty() {
                            args.push(v.to_string());
                        }
                    }
                }
            }

            out.projects.insert(path.clone(), ProjectBinding {
                account,
                launch: LaunchConfig { env, args },
            });
        }
    }

    Ok(out)
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

    #[test]
    fn write_then_read_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");

        let mut cfg = AppConfig::default();
        cfg.theme = "dark".to_string();
        write_config(&path, &cfg).unwrap();

        let loaded = read_config(&path).unwrap();
        assert_eq!(loaded.theme, "dark");
    }

    #[test]
    fn read_missing_file_returns_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("does-not-exist.json");
        let loaded = read_config(&path).unwrap();
        assert_eq!(loaded, AppConfig::default());
    }

    #[test]
    fn write_is_atomic_via_tempfile() {
        // After write, no temp leftovers in the parent dir.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        write_config(&path, &AppConfig::default()).unwrap();

        let entries: Vec<_> = std::fs::read_dir(dir.path()).unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        assert_eq!(entries, vec!["config.json"]);
    }

    #[test]
    fn migrate_v1_preserves_preferences() {
        let v1 = serde_json::json!({
            "theme": "dark",
            "language": "en-US",
            "fontSize": 16,
            "sidebarWidth": 200,
            "subpanelWidth": 240,
            "preferredTerminal": "iterm2"
        });
        let cfg = migrate_from_v1(v1).unwrap();
        assert_eq!(cfg.schema_version, SCHEMA_VERSION);
        assert_eq!(cfg.theme, "dark");
        assert_eq!(cfg.language, "en-US");
        assert_eq!(cfg.font_size, 16);
        assert_eq!(cfg.sidebar_width, 200);
        assert_eq!(cfg.preferred_terminal, "iterm2");
    }

    #[test]
    fn migrate_v1_drops_subpanel_width() {
        let v1 = serde_json::json!({ "subpanelWidth": 999 });
        let cfg = migrate_from_v1(v1).unwrap();
        let json = serde_json::to_value(&cfg).unwrap();
        assert!(!json.as_object().unwrap().contains_key("subpanelWidth"));
    }

    #[test]
    fn migrate_v1_lifts_accounts_with_defaults() {
        let v1 = serde_json::json!({
            "accounts": [
                { "name": "work", "createdAt": "2026-01-01T00:00:00Z" }
            ]
        });
        let cfg = migrate_from_v1(v1).unwrap();
        assert_eq!(cfg.accounts.len(), 1);
        assert_eq!(cfg.accounts[0].name, "work");
        assert_eq!(cfg.accounts[0].display_name, "work"); // fallback to name
        assert!(!cfg.accounts[0].is_native);
        assert_eq!(cfg.accounts[0].created_at, "2026-01-01T00:00:00Z");
    }

    #[test]
    fn migrate_v1_lifts_launcher_project_env_to_projects_and_known() {
        let v1 = serde_json::json!({
            "launcherProjectEnv": {
                "/Users/x/p1": {
                    "accountName": "work",
                    "customEnv": [
                        { "key": "FOO", "value": "bar", "enabled": true },
                        { "key": "OFF", "value": "x",   "enabled": false }
                    ],
                    "customArgs": [
                        { "flag": "--effort", "value": "high",  "enabled": true },
                        { "flag": "--brief",  "value": null,    "enabled": true },
                        { "flag": "--skip",   "value": "x",     "enabled": false }
                    ]
                },
                "/Users/x/p2": {}
            }
        });
        let cfg = migrate_from_v1(v1).unwrap();

        assert!(cfg.known_projects.contains(&"/Users/x/p1".to_string()));
        assert!(cfg.known_projects.contains(&"/Users/x/p2".to_string()));

        let p1 = cfg.projects.get("/Users/x/p1").expect("p1 present");
        assert_eq!(p1.account, "work");
        assert_eq!(p1.launch.env.get("FOO"), Some(&"bar".to_string()));
        assert!(!p1.launch.env.contains_key("OFF"));     // disabled dropped
        assert_eq!(p1.launch.args, vec!["--effort", "high", "--brief"]);

        // p2 has no accountName → defaults to "default"
        let p2 = cfg.projects.get("/Users/x/p2").expect("p2 present");
        assert_eq!(p2.account, "default");
        assert!(p2.launch.env.is_empty());
        assert!(p2.launch.args.is_empty());
    }

    #[test]
    fn migrate_v2_is_passthrough() {
        let v2 = serde_json::to_value(AppConfig::default()).unwrap();
        let cfg = migrate_from_v1(v2.clone()).unwrap();
        let v2_again = serde_json::to_value(&cfg).unwrap();
        assert_eq!(v2, v2_again);
    }
}
