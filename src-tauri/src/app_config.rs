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

pub const DEFAULT_ACCOUNT_NAME: &str = "default";

/// Given an in-memory `AppConfig` and a `project_path`, return the absolute
/// account directory that the project is bound to.
///
/// - Project not in `cfg.projects` → `Err("Unbound project: …")`
/// - Project bound to an account name not in `cfg.accounts` → `Err("Unknown account: …")`
/// - `account == "default"` → `<home>/.claude/`
/// - any other name → `<home>/.dot-claude-gui/accounts/<name>/`
///
/// Pure function: no disk I/O, easily unit-testable.
pub fn resolve_account_dir_for_project(
    home: &Path,
    cfg: &AppConfig,
    project_path: &str,
) -> Result<PathBuf, String> {
    let binding = cfg
        .projects
        .get(project_path)
        .ok_or_else(|| format!("Unbound project: {project_path}"))?;
    if !cfg.accounts.iter().any(|a| a.name == binding.account) {
        return Err(format!("Unknown account: {}", binding.account));
    }
    Ok(account_dir(home, &binding.account))
}

/// Resolve the on-disk directory for an account name relative to `home`.
///
/// - `"default"` (or empty) → `<home>/.claude/` (the native Claude dir)
/// - any other name → `<home>/.dot-claude-gui/accounts/<name>/`
///
/// This is the inverse of the convention used by `commands::launcher` when it
/// injects `CLAUDE_CONFIG_DIR`. Pure function: does NOT check that the
/// resulting path exists.
pub fn account_dir(home: &Path, name: &str) -> PathBuf {
    if name.is_empty() || name == DEFAULT_ACCOUNT_NAME {
        home.join(".claude")
    } else {
        home.join(".dot-claude-gui").join("accounts").join(name)
    }
}

/// If `native_exists` and no `default` account is in `cfg.accounts`, inserts
/// one at index 0 with `isNative: true` and the given `created_at`.
pub fn ensure_default_account(cfg: &mut AppConfig, native_exists: bool, created_at: &str) {
    if !native_exists { return; }
    if cfg.accounts.iter().any(|a| a.name == DEFAULT_ACCOUNT_NAME) { return; }
    cfg.accounts.insert(0, Account {
        name: DEFAULT_ACCOUNT_NAME.to_string(),
        display_name: "Native ~/.claude/".to_string(),
        is_native: true,
        created_at: created_at.to_string(),
    });
}

use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize)]
pub struct MigrationReport {
    pub migrated: bool,
    pub bak_path: Option<PathBuf>,
    pub default_injected: bool,
}

/// Read the config at `path`; if pre-v2, migrate and back up; then ensure the
/// default account exists when `native_exists`. Writes the result back to `path`.
pub fn migrate_at_startup(path: &Path, native_exists: bool) -> Result<MigrationReport, String> {
    let now_iso = now_iso8601();

    // Case 1: file missing → create a fresh default config (with default account if applicable).
    if !path.exists() {
        let mut cfg = AppConfig::default();
        ensure_default_account(&mut cfg, native_exists, &now_iso);
        write_config(path, &cfg)?;
        return Ok(MigrationReport {
            migrated: false,
            bak_path: None,
            default_injected: native_exists,
        });
    }

    let raw_bytes = std::fs::read(path).map_err(|e| format!("read config: {e}"))?;
    let raw_json: serde_json::Value = serde_json::from_slice(&raw_bytes)
        .map_err(|e| format!("parse config: {e}"))?;

    let is_v2 = raw_json.get("schemaVersion")
        .and_then(|v| v.as_u64())
        .map(|n| n as u32) == Some(SCHEMA_VERSION);

    if is_v2 {
        // No migration needed; still ensure default account is present.
        let mut cfg: AppConfig = serde_json::from_value(raw_json)
            .map_err(|e| format!("parse v2 config: {e}"))?;
        let had_default = cfg.accounts.iter().any(|a| a.name == DEFAULT_ACCOUNT_NAME);
        ensure_default_account(&mut cfg, native_exists, &now_iso);
        let injected = native_exists && !had_default;
        if injected {
            write_config(path, &cfg)?;
        }
        return Ok(MigrationReport {
            migrated: false,
            bak_path: None,
            default_injected: injected,
        });
    }

    // Pre-v2: snapshot the original, migrate, write new.
    let bak = bak_path_for(path);
    std::fs::copy(path, &bak).map_err(|e| format!("write bak: {e}"))?;

    let mut cfg = migrate_from_v1(raw_json)?;
    let had_default = cfg.accounts.iter().any(|a| a.name == DEFAULT_ACCOUNT_NAME);
    ensure_default_account(&mut cfg, native_exists, &now_iso);
    let injected = native_exists && !had_default;
    write_config(path, &cfg)?;

    Ok(MigrationReport {
        migrated: true,
        bak_path: Some(bak),
        default_injected: injected,
    })
}

fn bak_path_for(path: &Path) -> PathBuf {
    let unix = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let file = path.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let bak_name = format!("{file}.bak.{unix}");
    path.with_file_name(bak_name)
}

fn now_iso8601() -> String {
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    unix_secs_to_iso8601(secs)
}

/// Convert unix epoch seconds (UTC) to RFC 3339 / ISO 8601 of the form
/// `2026-05-11T17:42:33Z`. Pure arithmetic; valid for years 1970..9999.
/// Uses Howard Hinnant's days-to-Gregorian algorithm (public domain).
fn unix_secs_to_iso8601(secs: u64) -> String {
    const SECS_PER_DAY: u64 = 86_400;
    let days = secs / SECS_PER_DAY;
    let s_in_day = secs % SECS_PER_DAY;
    let hh = s_in_day / 3600;
    let mm = (s_in_day % 3600) / 60;
    let ss = s_in_day % 60;

    // Days since 1970-01-01 → year/month/day (Hinnant).
    let z = days as i64 + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };

    format!("{year:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}Z")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_v2() {
        assert_eq!(AppConfig::default().schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn unix_secs_to_iso8601_known_values() {
        assert_eq!(unix_secs_to_iso8601(0), "1970-01-01T00:00:00Z");
        assert_eq!(unix_secs_to_iso8601(86_400), "1970-01-02T00:00:00Z");
        // 2023-11-14T22:13:20Z (a common reference value)
        assert_eq!(unix_secs_to_iso8601(1_700_000_000), "2023-11-14T22:13:20Z");
        // Leap year boundary: 2024-02-29 00:00:00 UTC = 1_709_164_800
        assert_eq!(unix_secs_to_iso8601(1_709_164_800), "2024-02-29T00:00:00Z");
        // 2026-05-11T00:00:00Z = 1_778_457_600
        assert_eq!(unix_secs_to_iso8601(1_778_457_600), "2026-05-11T00:00:00Z");
    }

    #[test]
    fn now_iso8601_parses_with_chrono_like_format() {
        let s = now_iso8601();
        // Shape: YYYY-MM-DDTHH:MM:SSZ — 20 chars exactly
        assert_eq!(s.len(), 20, "iso 8601 wall-time should be 20 chars: {s}");
        assert!(s.ends_with('Z'));
        assert_eq!(&s[4..5], "-");
        assert_eq!(&s[7..8], "-");
        assert_eq!(&s[10..11], "T");
        assert_eq!(&s[13..14], ":");
        assert_eq!(&s[16..17], ":");
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

    #[test]
    fn inject_default_adds_native_when_missing() {
        let mut cfg = AppConfig::default();
        ensure_default_account(&mut cfg, /* native_exists */ true, "2026-05-11T00:00:00Z");
        assert_eq!(cfg.accounts.len(), 1);
        assert_eq!(cfg.accounts[0].name, "default");
        assert!(cfg.accounts[0].is_native);
        assert_eq!(cfg.accounts[0].display_name, "Native ~/.claude/");
        assert_eq!(cfg.accounts[0].created_at, "2026-05-11T00:00:00Z");
    }

    #[test]
    fn inject_default_idempotent() {
        let mut cfg = AppConfig::default();
        ensure_default_account(&mut cfg, true, "ts1");
        ensure_default_account(&mut cfg, true, "ts2");
        assert_eq!(cfg.accounts.len(), 1);
        assert_eq!(cfg.accounts[0].created_at, "ts1"); // first one wins
    }

    #[test]
    fn inject_default_skipped_when_native_missing() {
        let mut cfg = AppConfig::default();
        ensure_default_account(&mut cfg, false, "ts");
        assert!(cfg.accounts.iter().all(|a| a.name != "default"));
    }

    #[test]
    fn inject_default_does_not_disturb_other_accounts() {
        let mut cfg = AppConfig::default();
        cfg.accounts.push(Account {
            name: "work".into(),
            display_name: "work".into(),
            is_native: false,
            created_at: "x".into(),
        });
        ensure_default_account(&mut cfg, true, "ts");
        assert_eq!(cfg.accounts.len(), 2);
        assert_eq!(cfg.accounts[0].name, "default"); // default inserted at index 0
        assert_eq!(cfg.accounts[1].name, "work");
    }

    use std::fs;

    fn write_raw(path: &std::path::Path, val: serde_json::Value) {
        fs::write(path, serde_json::to_string_pretty(&val).unwrap()).unwrap();
    }

    #[test]
    fn migrate_at_startup_backs_up_old_v1() {
        let dir = tempfile::tempdir().unwrap();
        let cfg_path = dir.path().join("config.json");
        write_raw(&cfg_path, serde_json::json!({ "theme": "dark", "subpanelWidth": 200 }));

        let report = migrate_at_startup(&cfg_path, /* native_exists */ false).unwrap();
        assert!(report.migrated);
        assert!(report.bak_path.is_some());

        // .bak.<timestamp> exists alongside config.json
        let bak = report.bak_path.unwrap();
        assert!(bak.exists(), "expected bak file at {:?}", bak);
        let bak_name = bak.file_name().unwrap().to_string_lossy();
        assert!(bak_name.starts_with("config.json.bak."), "got {}", bak_name);

        // New config is v2
        let new_cfg = read_config(&cfg_path).unwrap();
        assert_eq!(new_cfg.schema_version, SCHEMA_VERSION);
        assert_eq!(new_cfg.theme, "dark");
    }

    #[test]
    fn migrate_at_startup_no_op_for_v2() {
        let dir = tempfile::tempdir().unwrap();
        let cfg_path = dir.path().join("config.json");
        write_config(&cfg_path, &AppConfig::default()).unwrap();

        let report = migrate_at_startup(&cfg_path, false).unwrap();
        assert!(!report.migrated);
        assert!(report.bak_path.is_none());
    }

    #[test]
    fn migrate_at_startup_creates_default_when_native_exists() {
        let dir = tempfile::tempdir().unwrap();
        let cfg_path = dir.path().join("config.json");
        // start with v1, no accounts
        write_raw(&cfg_path, serde_json::json!({ "theme": "dark" }));

        migrate_at_startup(&cfg_path, /* native_exists */ true).unwrap();
        let new_cfg = read_config(&cfg_path).unwrap();
        assert!(new_cfg.accounts.iter().any(|a| a.name == "default" && a.is_native));
    }

    #[test]
    fn migrate_at_startup_handles_missing_config() {
        let dir = tempfile::tempdir().unwrap();
        let cfg_path = dir.path().join("config.json");
        let report = migrate_at_startup(&cfg_path, true).unwrap();
        assert!(!report.migrated); // nothing to migrate
        assert!(cfg_path.exists());
        let cfg = read_config(&cfg_path).unwrap();
        assert!(cfg.accounts.iter().any(|a| a.name == "default"));
    }

    /// One-off smoke test: copies the live user config to a tempdir and runs
    /// migrate_at_startup against the copy. Run with:
    ///   cargo test -p dot-claude-gui migration_real_config_smoke -- --ignored --nocapture
    /// Does not touch the live config.
    #[test]
    #[ignore]
    fn migration_real_config_smoke() {
        let home = std::env::var("HOME").expect("HOME");
        let live = std::path::Path::new(&home)
            .join(".dot-claude-gui")
            .join("config.json");
        if !live.exists() {
            eprintln!("no live config at {live:?} — skipping");
            return;
        }

        let dir = tempfile::tempdir().unwrap();
        let copy = dir.path().join("config.json");
        std::fs::copy(&live, &copy).unwrap();

        let native_exists = std::path::Path::new(&home).join(".claude").exists();
        eprintln!("native_exists = {native_exists}");

        let report = migrate_at_startup(&copy, native_exists).unwrap();
        eprintln!("report: {report:?}");

        let migrated = read_config(&copy).unwrap();
        let pretty = serde_json::to_string_pretty(&migrated).unwrap();
        eprintln!("migrated config:\n{pretty}");

        // Assertions on shape
        assert_eq!(migrated.schema_version, SCHEMA_VERSION, "must be v2");
        assert!(report.migrated, "expected migration from v1");
        assert!(report.bak_path.is_some(), "expected .bak file");
        assert!(report.bak_path.as_ref().unwrap().exists(), "bak file must exist on disk");

        // Default account presence (native exists on this machine)
        if native_exists {
            assert!(
                migrated.accounts.iter().any(|a| a.name == DEFAULT_ACCOUNT_NAME && a.is_native),
                "default native account should be injected"
            );
            assert!(
                report.default_injected,
                "default_injected should be true (had no default in v1)"
            );
        }

        // No more subpanelWidth in the serialized v2 form
        let value: serde_json::Value = serde_json::from_str(&pretty).unwrap();
        assert!(
            !value.as_object().unwrap().contains_key("subpanelWidth"),
            "subpanelWidth must be dropped"
        );

        // launcherProjectEnv must have been lifted into projects + knownProjects
        assert!(
            !value.as_object().unwrap().contains_key("launcherProjectEnv"),
            "launcherProjectEnv must be dropped"
        );

        // Re-run migration on the migrated file: should be idempotent
        let second_report = migrate_at_startup(&copy, native_exists).unwrap();
        assert!(!second_report.migrated, "second run should be no-op for migration");
        assert!(second_report.bak_path.is_none(), "no new bak on idempotent run");
    }

    #[test]
    fn account_dir_default_returns_native_home() {
        let home = std::path::Path::new("/u/eric");
        assert_eq!(account_dir(home, "default"), home.join(".claude"));
    }

    #[test]
    fn account_dir_named_returns_gui_account_subdir() {
        let home = std::path::Path::new("/u/eric");
        assert_eq!(
            account_dir(home, "work"),
            home.join(".dot-claude-gui").join("accounts").join("work")
        );
    }

    #[test]
    fn account_dir_treats_empty_name_as_default() {
        let home = std::path::Path::new("/u/eric");
        assert_eq!(account_dir(home, ""), home.join(".claude"));
    }

    // ── resolve_account_dir_for_project tests ───────────────────────────────

    fn make_cfg_with_project(project_path: &str, account_name: &str) -> AppConfig {
        let mut cfg = AppConfig::default();
        cfg.accounts.push(Account {
            name: account_name.to_string(),
            display_name: account_name.to_string(),
            is_native: account_name == DEFAULT_ACCOUNT_NAME,
            created_at: "2026-01-01T00:00:00Z".to_string(),
        });
        cfg.projects.insert(
            project_path.to_string(),
            ProjectBinding {
                account: account_name.to_string(),
                launch: LaunchConfig::default(),
            },
        );
        cfg
    }

    #[test]
    fn resolve_account_dir_for_project_bound_non_default() {
        let home = std::path::Path::new("/u/eric");
        let cfg = make_cfg_with_project("/Users/eric/myproject", "work");
        let result = resolve_account_dir_for_project(home, &cfg, "/Users/eric/myproject").unwrap();
        assert_eq!(
            result,
            home.join(".dot-claude-gui").join("accounts").join("work")
        );
    }

    #[test]
    fn resolve_account_dir_for_project_bound_default() {
        let home = std::path::Path::new("/u/eric");
        let cfg = make_cfg_with_project("/Users/eric/myproject", DEFAULT_ACCOUNT_NAME);
        let result = resolve_account_dir_for_project(home, &cfg, "/Users/eric/myproject").unwrap();
        assert_eq!(result, home.join(".claude"));
    }

    #[test]
    fn resolve_account_dir_for_project_unbound_returns_error() {
        let home = std::path::Path::new("/u/eric");
        let cfg = AppConfig::default(); // no projects
        let err = resolve_account_dir_for_project(home, &cfg, "/Users/eric/unknown")
            .unwrap_err();
        let lower = err.to_lowercase();
        assert!(
            lower.contains("unbound"),
            "expected error containing 'unbound', got: {err}"
        );
    }

    #[test]
    fn resolve_account_dir_for_project_orphaned_binding_returns_error() {
        // make_cfg_with_project inserts both account "ghost" and the binding;
        // remove "ghost" from cfg.accounts to simulate a dangling reference.
        let cfg = make_cfg_with_project("/p", "ghost");
        let cfg_no_ghost = AppConfig {
            accounts: cfg.accounts.iter().filter(|a| a.name != "ghost").cloned().collect(),
            ..cfg
        };
        let err = resolve_account_dir_for_project(
            std::path::Path::new("/u/eric"),
            &cfg_no_ghost,
            "/p",
        )
        .unwrap_err();
        assert!(
            err.to_lowercase().contains("unknown account"),
            "expected error containing 'unknown account', got: {err}"
        );
    }
}
