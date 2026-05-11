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
    let now_iso = chrono_like_now_iso();

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

fn chrono_like_now_iso() -> String {
    // Avoid pulling chrono in just for this; use seconds-since-epoch as a
    // sortable ISO-ish stamp. (Real ISO 8601 is unnecessary for our purpose.)
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("1970-01-01T00:00:00Z+{secs}")  // monotonic stamp; frontend doesn't parse
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
}
