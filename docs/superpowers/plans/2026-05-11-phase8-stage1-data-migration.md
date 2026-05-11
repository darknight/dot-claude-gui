# Phase 8 Stage 1 — Data Layer & Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Migrate the data layer to `config.json` v2 (explicit accounts with `displayName`/`isNative`, centralized project bindings, `knownProjects` registry) and replace the existing project-registry IPC with new typed CRUD commands. UI compiles but is non-functional — Stage 2 wires it back up.

**Architecture:** Backend owns the new schema. A one-shot migration runs at app startup, atomically transforms v1 → v2 (with `.bak.<unix>` snapshot), and auto-injects the `default` account when `~/.claude/` exists. New IPC commands (`list_projects`, `add_project`, `bind_project`, `unbind_project`, `remove_project`, `update_project_launch`) read/write the new schema directly. The frontend types and stores are updated to match; UI components are left broken at runtime (Stage 2 fixes).

**Tech Stack:** Rust 1.x (Tauri 2.0, serde, tempfile for atomic writes), Svelte 5, TypeScript strict, pnpm.

**Spec:** `docs/superpowers/specs/2026-05-11-phase8-mode-based-redesign-design.md`

---

## File map

**New:**
- `src-tauri/src/app_config.rs` — v2 schema types, IO, migration, default-account injection (with tests)
- `src-tauri/src/commands/gui_projects.rs` — new project-binding CRUD over `config.json` (separate from the existing `commands::projects`, which keeps serving `commands/config.rs` for settings reads)

**Modify:**
- `src-tauri/src/lib.rs` — drop the inline `AppConfig` struct + old `read/write_app_config` body (the migration runs at setup; IO moves to `app_config.rs`); register the new gui_projects commands
- `src-tauri/src/commands/mod.rs` — register `gui_projects`
- `src/lib/api/types.ts` — `AppConfig` schema v2, new `ProjectBinding`/`LaunchConfig`/`ProjectEntry` types, expanded `Account`
- `src/lib/stores/appsettings.svelte.ts` — default state matches v2
- `src/lib/stores/accounts.svelte.ts` — reconcile with `displayName`/`isNative`
- `src/lib/stores/projects.svelte.ts` — rewrite over new IPC + schema
- `src/lib/ipc/client.ts` — add new project commands

**Leave in place (Stage 1 doesn't touch):**
- `src-tauri/src/commands/projects.rs` and `src-tauri/src/state.rs`'s `projects` field — still used by `commands::config::get_project_config` to read per-project settings. Removing them would cascade-break the settings module. Stage 3 consolidates the two project concepts when the new UI is wired.
- `src/lib/stores/config.svelte.ts` — leave the existing references to `projectsStore.activeProjectId`. The new `projectsStore` rewrite (Task 11) **preserves** that field as a deprecated alias so config.svelte.ts compiles.

**Delete:**
- `src/lib/stores/launcher.svelte.ts` — per-project launch state now lives in `config.json.projects[<path>].launch` and is reached via the rewritten `projectsStore`.

---

### Task 1: Scaffold `app_config.rs` with v2 types

**Files:**
- Create: `src-tauri/src/app_config.rs`

- [ ] **Step 1: Create the file with the v2 schema types**

```rust
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
```

- [ ] **Step 2: Wire the module into the crate**

Edit `src-tauri/src/lib.rs` near the other `mod` declarations at the top:

```rust
mod app_config;
mod commands;
mod events;
mod executor;
mod state;
mod watcher;
```

- [ ] **Step 3: Run tests**

```bash
cargo test -p dot-claude-gui app_config::tests
```

Expected: 3 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/app_config.rs src-tauri/src/lib.rs
git commit -m "feat(app-config): add v2 schema types (accounts, projects, knownProjects)"
```

---

### Task 2: Atomic file IO for `config.json`

**Files:**
- Modify: `src-tauri/src/app_config.rs`
- Modify: `src-tauri/Cargo.toml` (add `tempfile`)

- [ ] **Step 1: Add tempfile dep (skip if already present)**

```bash
grep tempfile src-tauri/Cargo.toml
```

If absent, add to `[dependencies]`:
```toml
tempfile = "3"
```

- [ ] **Step 2: Write the failing test first**

Append to `app_config.rs` tests module:

```rust
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
```

- [ ] **Step 3: Run tests to verify they fail**

```bash
cargo test -p dot-claude-gui app_config::tests
```

Expected: 3 new tests FAIL with "no function or associated item named `read_config`" / `write_config`.

- [ ] **Step 4: Add `read_config` and `write_config`**

Append to `app_config.rs` (above the `#[cfg(test)]` block):

```rust
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
```

- [ ] **Step 5: Run tests**

```bash
cargo test -p dot-claude-gui app_config::tests
```

Expected: all 6 tests pass.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/app_config.rs src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "feat(app-config): atomic read/write with tempfile"
```

---

### Task 3: Migration v1 → v2

**Files:**
- Modify: `src-tauri/src/app_config.rs`

- [ ] **Step 1: Write failing migration tests**

Append to `app_config.rs` tests module:

```rust
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
```

- [ ] **Step 2: Run tests, verify failure**

```bash
cargo test -p dot-claude-gui app_config::tests
```

Expected: 5 new tests FAIL with "no function `migrate_from_v1`".

- [ ] **Step 3: Implement migration**

Append above the `#[cfg(test)]` block in `app_config.rs`:

```rust
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
```

- [ ] **Step 4: Run tests**

```bash
cargo test -p dot-claude-gui app_config::tests
```

Expected: all 11 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/app_config.rs
git commit -m "feat(app-config): migrate v1 → v2 with subpanelWidth drop and launcherProjectEnv lift"
```

---

### Task 4: Default account injection

**Files:**
- Modify: `src-tauri/src/app_config.rs`

- [ ] **Step 1: Write failing tests**

Append to tests module:

```rust
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
```

- [ ] **Step 2: Run tests, verify failure**

```bash
cargo test -p dot-claude-gui app_config::tests
```

Expected: 4 new tests fail.

- [ ] **Step 3: Implement injection**

Append to `app_config.rs` (above the `#[cfg(test)]` block):

```rust
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
```

- [ ] **Step 4: Run tests**

```bash
cargo test -p dot-claude-gui app_config::tests
```

Expected: all 15 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/app_config.rs
git commit -m "feat(app-config): auto-inject default account when ~/.claude/ exists"
```

---

### Task 5: One-shot startup migration

**Files:**
- Modify: `src-tauri/src/app_config.rs`

- [ ] **Step 1: Write failing tests**

Append to tests module:

```rust
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
```

- [ ] **Step 2: Run tests, verify failure**

```bash
cargo test -p dot-claude-gui app_config::tests
```

Expected: 4 new tests fail.

- [ ] **Step 3: Implement `migrate_at_startup`**

Append to `app_config.rs`:

```rust
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
```

- [ ] **Step 4: Run tests**

```bash
cargo test -p dot-claude-gui app_config::tests
```

Expected: all 19 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/app_config.rs
git commit -m "feat(app-config): one-shot startup migration with .bak snapshot"
```

---

### Task 6: Wire migration into Tauri setup

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Drop the old `AppConfig` struct and IO commands**

In `src-tauri/src/lib.rs`, replace the entire block from `// ── Types ──...` through the end of `write_app_config` (lines ~11-79 in current file) with:

```rust
mod app_config;
mod commands;
mod events;
mod executor;
mod state;
mod watcher;

use std::path::PathBuf;
use tauri::Manager;

// ── Config-dir helpers ────────────────────────────────────────────────────────

fn config_dir() -> Result<PathBuf, String> {
    let home = dirs_next::home_dir().ok_or("cannot determine home directory")?;
    Ok(home.join(".dot-claude-gui"))
}

fn ensure_config_dir() -> Result<PathBuf, String> {
    let dir = config_dir()?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("failed to create config dir: {}", e))?;
    Ok(dir)
}

// ── IPC commands ──────────────────────────────────────────────────────────────

#[tauri::command]
fn get_config_dir() -> Result<String, String> {
    config_dir().map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
fn read_app_config() -> Result<String, String> {
    let path = ensure_config_dir()?.join("config.json");
    let cfg = app_config::read_config(&path)?;
    serde_json::to_string(&cfg).map_err(|e| format!("serialize app config: {e}"))
}

#[tauri::command]
fn write_app_config(json: String) -> Result<(), String> {
    let cfg: app_config::AppConfig = serde_json::from_str(&json)
        .map_err(|e| format!("parse app config: {e}"))?;
    let path = ensure_config_dir()?.join("config.json");
    app_config::write_config(&path, &cfg)
}
```

- [ ] **Step 2: Call migration in `.setup()` before everything else**

Find the `.setup(|app| {` block and insert at the **start** of the closure (before `let claude_home = ...`):

```rust
            // One-shot migration v1 → v2 (idempotent for v2).
            // Runs before any state init so subsequent code reads the new schema.
            if let Ok(dir) = ensure_config_dir() {
                let cfg_path = dir.join("config.json");
                let native_exists = dirs_next::home_dir()
                    .map(|h| h.join(".claude").exists())
                    .unwrap_or(false);
                match app_config::migrate_at_startup(&cfg_path, native_exists) {
                    Ok(report) => tracing::info!("config migration: {report:?}"),
                    Err(e)     => tracing::error!("config migration failed: {e}"),
                }
            }

```

- [ ] **Step 3: Build**

```bash
cargo build -p dot-claude-gui
```

Expected: builds cleanly (warnings about unused `default_app_config` etc. may appear and are fine — we'll clean in Task 11).

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat(setup): run config migration before AppState init"
```

---

### Task 7: Add `gui_projects.rs` alongside existing `projects.rs`

The existing `commands::projects` keeps living — it's coupled to `state.inner.projects` and consumed by `commands/config.rs::get_project_config`. The new module is a parallel, narrower API for UI bindings.

**Files:**
- Create: `src-tauri/src/commands/gui_projects.rs`
- Modify: `src-tauri/src/commands/mod.rs` (add `gui_projects`; do NOT remove `projects`)

- [ ] **Step 1: Write the new module skeleton + tests**

Create `src-tauri/src/commands/gui_projects.rs`:

```rust
// src-tauri/src/commands/gui_projects.rs
//
// Project binding CRUD. Reads/writes ~/.dot-claude-gui/config.json directly.
// All operations are atomic at the file level (see app_config::write_config).

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use crate::app_config::{
    read_config, write_config, AppConfig, LaunchConfig, ProjectBinding, DEFAULT_ACCOUNT_NAME,
};

fn config_path() -> Result<PathBuf, String> {
    let dir = dirs_next::home_dir()
        .ok_or("cannot determine home directory")?
        .join(".dot-claude-gui");
    std::fs::create_dir_all(&dir).map_err(|e| format!("mkdir config dir: {e}"))?;
    Ok(dir.join("config.json"))
}

fn mutate<F>(f: F) -> Result<AppConfig, String>
where F: FnOnce(&mut AppConfig) -> Result<(), String>
{
    let path = config_path()?;
    let mut cfg = read_config(&path)?;
    f(&mut cfg)?;
    write_config(&path, &cfg)?;
    Ok(cfg)
}

// ── List ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectEntry {
    pub path: String,
    /// `None` => unbound; `Some(name)` => bound to that account.
    pub account: Option<String>,
    pub launch: LaunchConfig,
    /// True when `path` doesn't exist on disk.
    pub stale: bool,
}

/// Renamed to `gui_list_projects` to avoid IPC name collision with the
/// existing `commands::projects::list_projects` (which serves settings).
#[tauri::command]
pub fn gui_list_projects() -> Result<Vec<ProjectEntry>, String> {
    let cfg = read_config(&config_path()?)?;
    let entries = cfg.known_projects.iter().map(|path| {
        let stale = !std::path::Path::new(path).exists();
        let (account, launch) = match cfg.projects.get(path) {
            Some(b) => (Some(b.account.clone()), b.launch.clone()),
            None    => (None, LaunchConfig::default()),
        };
        ProjectEntry { path: path.clone(), account, launch, stale }
    }).collect();
    Ok(entries)
}

// ── Add (registers a path; no binding yet) ──────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddProjectRequest { pub path: String }

#[tauri::command]
pub fn add_project(req: AddProjectRequest) -> Result<ProjectEntry, String> {
    let p = std::path::PathBuf::from(&req.path);
    if !p.exists() {
        return Err(format!("invalid_path: {}", req.path));
    }
    let path = p.canonicalize()
        .map_err(|e| format!("canonicalize path: {e}"))?
        .to_string_lossy()
        .to_string();

    mutate(|cfg| {
        if !cfg.known_projects.contains(&path) {
            cfg.known_projects.push(path.clone());
        }
        Ok(())
    })?;

    Ok(ProjectEntry { path, account: None, launch: LaunchConfig::default(), stale: false })
}

// ── Bind / Unbind ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BindProjectRequest { pub path: String, pub account: String }

#[tauri::command]
pub fn bind_project(req: BindProjectRequest) -> Result<(), String> {
    mutate(|cfg| {
        let known_account =
            req.account == DEFAULT_ACCOUNT_NAME ||
            cfg.accounts.iter().any(|a| a.name == req.account);
        if !known_account {
            return Err(format!("unknown_account: {}", req.account));
        }
        if !cfg.known_projects.contains(&req.path) {
            cfg.known_projects.push(req.path.clone());
        }
        cfg.projects.entry(req.path.clone())
            .or_insert_with(ProjectBinding::default)
            .account = req.account.clone();
        Ok(())
    })?;
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnbindProjectRequest { pub path: String }

#[tauri::command]
pub fn unbind_project(req: UnbindProjectRequest) -> Result<(), String> {
    mutate(|cfg| { cfg.projects.remove(&req.path); Ok(()) })?;
    Ok(())
}

// ── Remove (from list entirely) ─────────────────────────────────────────

#[tauri::command]
pub fn remove_project(req: UnbindProjectRequest) -> Result<(), String> {
    mutate(|cfg| {
        cfg.projects.remove(&req.path);
        cfg.known_projects.retain(|p| p != &req.path);
        Ok(())
    })?;
    Ok(())
}

// ── Update launch ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLaunchRequest { pub path: String, pub launch: LaunchConfig }

#[tauri::command]
pub fn update_project_launch(req: UpdateLaunchRequest) -> Result<(), String> {
    mutate(|cfg| {
        let entry = cfg.projects.entry(req.path.clone())
            .or_insert_with(ProjectBinding::default);
        entry.launch = req.launch.clone();
        Ok(())
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_config::{write_config, AppConfig, Account};

    fn isolated() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        // Override HOME for this thread so `config_path()` resolves into the tempdir.
        std::env::set_var("HOME", dir.path());
        dir
    }

    #[test]
    fn add_then_list_includes_path_as_unbound() {
        let _g = isolated();
        let real = std::env::current_dir().unwrap().to_string_lossy().to_string();
        add_project(AddProjectRequest { path: real.clone() }).unwrap();
        let list = gui_list_projects().unwrap();
        assert!(list.iter().any(|p| p.path == real && p.account.is_none()));
    }

    #[test]
    fn bind_then_list_shows_binding() {
        let _g = isolated();
        // Seed with a known account so bind_project accepts it.
        let cfg_path = config_path().unwrap();
        let mut cfg = AppConfig::default();
        cfg.accounts.push(Account {
            name: "work".into(), display_name: "work".into(),
            is_native: false, created_at: "x".into(),
        });
        write_config(&cfg_path, &cfg).unwrap();

        let real = std::env::current_dir().unwrap().to_string_lossy().to_string();
        bind_project(BindProjectRequest { path: real.clone(), account: "work".into() }).unwrap();
        let list = gui_list_projects().unwrap();
        let entry = list.iter().find(|p| p.path == real).expect("bound entry present");
        assert_eq!(entry.account.as_deref(), Some("work"));
    }

    #[test]
    fn bind_rejects_unknown_account() {
        let _g = isolated();
        let res = bind_project(BindProjectRequest {
            path: "/some/path".into(),
            account: "ghost".into(),
        });
        assert!(res.is_err());
    }

    #[test]
    fn bind_accepts_default_without_explicit_seed() {
        let _g = isolated();
        // Native ~/.claude/ check uses real HOME which we've overridden; create
        // the dir so the "default" account exists conceptually.
        std::fs::create_dir_all(std::env::var("HOME").map(std::path::PathBuf::from).unwrap().join(".claude")).unwrap();
        let res = bind_project(BindProjectRequest {
            path: "/some/path".into(),
            account: "default".into(),
        });
        assert!(res.is_ok());
    }

    #[test]
    fn remove_drops_from_known_and_projects() {
        let _g = isolated();
        let real = std::env::current_dir().unwrap().to_string_lossy().to_string();
        add_project(AddProjectRequest { path: real.clone() }).unwrap();
        remove_project(UnbindProjectRequest { path: real.clone() }).unwrap();
        let list = gui_list_projects().unwrap();
        assert!(list.iter().all(|p| p.path != real));
    }

    #[test]
    fn list_marks_stale_paths() {
        let _g = isolated();
        let cfg_path = config_path().unwrap();
        let mut cfg = AppConfig::default();
        cfg.known_projects.push("/definitely/does/not/exist/12345".into());
        write_config(&cfg_path, &cfg).unwrap();

        let list = gui_list_projects().unwrap();
        let entry = list.iter().find(|p| p.path == "/definitely/does/not/exist/12345").unwrap();
        assert!(entry.stale);
    }
}
```

- [ ] **Step 2: Register the new module**

Edit `src-tauri/src/commands/mod.rs` — add (don't remove the existing `projects` line):

```rust
pub mod accounts;
pub mod claudemd;
pub mod config;
pub mod gui_projects;
pub mod health;
pub mod launcher;
pub mod mcp;
pub mod memory;
pub mod plugins;
pub mod projects;
pub mod skills;
```

- [ ] **Step 3: Run tests**

```bash
cargo test -p dot-claude-gui commands::gui_projects::tests
```

Expected: 6 tests pass.

> **Note:** The tests use `std::env::set_var("HOME", ...)` which is not thread-safe across the full `cargo test` run. If you see flakes, add `serial_test = "3"` as a dev-dependency and decorate each test with `#[serial_test::serial]`. Add `serial_test` to `[dev-dependencies]` only if this happens.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/gui_projects.rs src-tauri/src/commands/mod.rs
git commit -m "feat(gui-projects): config.json-backed CRUD for project bindings"
```

---

### Task 8: Register new commands in the invoke handler

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Extend the invoke_handler list**

In `src-tauri/src/lib.rs`, inside `tauri::generate_handler![...]`, add (do NOT remove the existing `commands::projects::*` lines — they still serve settings):

```rust
            // Existing — keep:
            commands::projects::list_projects,
            commands::projects::register_project,
            commands::projects::unregister_project,

            // New — add:
            commands::gui_projects::list_projects as gui_list_projects,
            commands::gui_projects::add_project,
            commands::gui_projects::bind_project,
            commands::gui_projects::unbind_project,
            commands::gui_projects::remove_project,
            commands::gui_projects::update_project_launch,
```

Tauri doesn't allow two commands with the same exported name, so register the new `list_projects` under a Tauri command alias. Adjust the gui_projects definition in Task 7 to use `#[tauri::command(rename_all = "snake_case")]` and a renamed handler:

```rust
// In gui_projects.rs — replace:
#[tauri::command]
pub fn list_projects() -> Result<Vec<ProjectEntry>, String> { ... }

// With:
#[tauri::command]
pub fn gui_list_projects() -> Result<Vec<ProjectEntry>, String> {
    let cfg = read_config(&config_path()?)?;
    let entries = cfg.known_projects.iter().map(|path| {
        let stale = !std::path::Path::new(path).exists();
        let (account, launch) = match cfg.projects.get(path) {
            Some(b) => (Some(b.account.clone()), b.launch.clone()),
            None    => (None, LaunchConfig::default()),
        };
        ProjectEntry { path: path.clone(), account, launch, stale }
    }).collect();
    Ok(entries)
}
```

And update the test calls accordingly (`gui_list_projects()` instead of `list_projects()`).

And in the handler list, drop the `as` alias:

```rust
            commands::gui_projects::gui_list_projects,
            commands::gui_projects::add_project,
            commands::gui_projects::bind_project,
            commands::gui_projects::unbind_project,
            commands::gui_projects::remove_project,
            commands::gui_projects::update_project_launch,
```

- [ ] **Step 2: Build**

```bash
cargo build -p dot-claude-gui
```

Expected: builds cleanly (warnings about unused `default_app_config` etc. are fine).

- [ ] **Step 3: Run all backend tests**

```bash
cargo test -p dot-claude-gui
```

Expected: all green, including 19 app_config tests and 6 gui_projects tests.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/src/commands/gui_projects.rs
git commit -m "wire(ipc): register gui_projects commands (gui_list_projects + 5 mutators)"
```

---

### Task 9: Frontend types — AppConfig v2

**Files:**
- Modify: `src/lib/api/types.ts`

- [ ] **Step 1: Replace the launcher / app-config blocks**

Find the block starting at `// Launcher` and ending after `AppConfig`. Replace with:

```ts
// ---------------------------------------------------------------------------
// Launcher (request shape used by IPC; reads project binding internally)
// ---------------------------------------------------------------------------

export interface LaunchRequest {
  projectPath?: string;
  env?: Record<string, string>;
  args?: string[];
  preferredTerminal?: PreferredTerminal;
}

export type PreferredTerminal = "terminal" | "iterm2";

// ---------------------------------------------------------------------------
// Accounts
// ---------------------------------------------------------------------------

export interface Account {
  name: string;
  displayName: string;
  isNative: boolean;
  /** ISO-ish stamp; opaque (not parsed by frontend). */
  createdAt: string;
}

export interface DiskAccount {
  name: string;
  createdAtUnix: number;
}

export interface AccountStatus {
  loggedIn: boolean;
  email?: string;
}

// ---------------------------------------------------------------------------
// Project bindings
// ---------------------------------------------------------------------------

export interface LaunchConfig {
  env: Record<string, string>;
  args: string[];
}

export interface ProjectBinding {
  account: string;
  launch: LaunchConfig;
}

export interface ProjectEntry {
  path: string;
  /** null when unbound; otherwise account name. */
  account: string | null;
  launch: LaunchConfig;
  stale: boolean;
}

// ---------------------------------------------------------------------------
// AppConfig v2 (persisted to ~/.dot-claude-gui/config.json)
// ---------------------------------------------------------------------------

export interface AppConfig {
  schemaVersion: number;
  theme: "light" | "dark" | "system";
  language: Locale;
  fontSize: number;
  sidebarWidth: number;
  preferredTerminal: PreferredTerminal;
  accounts: Account[];
  /** path → binding */
  projects: Record<string, ProjectBinding>;
  knownProjects: string[];
}
```

Also delete the `ConnectionEntry` and `ConnectionsFile` interfaces above `AppConfig` (orphaned from a removed feature).

- [ ] **Step 2: Type-check**

```bash
pnpm exec tsc --noEmit
```

Expected: errors in files that reference removed fields (`launcherProjectEnv`, `subpanelWidth`, `LauncherProjectEnv`, `LauncherEnvEntry`, `LauncherArgEntry`). These get fixed in Tasks 10–12. **Do not commit yet.**

- [ ] **Step 3: Commit (allow downstream type errors)**

```bash
git add src/lib/api/types.ts
git commit -m "types: AppConfig v2 (drop launcherProjectEnv, add projects/knownProjects)"
```

---

### Task 10: Frontend stores — appsettings and accounts

**Files:**
- Modify: `src/lib/stores/appsettings.svelte.ts`
- Modify: `src/lib/stores/accounts.svelte.ts`

- [ ] **Step 1: Update appsettings defaults**

Replace the `preferences` state in `appsettings.svelte.ts`:

```ts
import type { AppConfig } from "$lib/api/types.js";
import { invoke } from "@tauri-apps/api/core";
import { detectInitialLocale, isSupportedLocale } from "$lib/i18n";

class AppSettingsStore {
  preferences = $state<AppConfig>({
    schemaVersion: 2,
    theme: "system",
    language: "zh-CN",
    fontSize: 14,
    sidebarWidth: 140,
    preferredTerminal: "terminal",
    accounts: [],
    projects: {},
    knownProjects: [],
  });

  loaded = $state(false);

  async load(): Promise<void> {
    try {
      const json = await invoke<string>("read_app_config");
      const saved: Partial<AppConfig> = JSON.parse(json);
      this.preferences = { ...this.preferences, ...saved };
    } catch {
      // defaults
    }

    if (!isSupportedLocale(this.preferences.language)) {
      this.preferences.language = detectInitialLocale();
      await this.save();
    }
    this.loaded = true;
  }

  async save(): Promise<void> {
    try {
      await invoke("write_app_config", {
        json: JSON.stringify(this.preferences, null, 2),
      });
    } catch {}
  }

  async update(partial: Partial<AppConfig>): Promise<void> {
    this.preferences = { ...this.preferences, ...partial };
    await this.save();
  }
}

export const appSettingsStore = new AppSettingsStore();
```

(LocalStorage migration is removed — that was a one-time path that already ran on most installs.)

- [ ] **Step 2: Update accounts store reconciliation**

In `src/lib/stores/accounts.svelte.ts`, change `reconcile` to populate the new fields:

```ts
function reconcile(disk: DiskAccount[], configAccounts: Account[]): Account[] {
  const byName = new Map(configAccounts.map((a) => [a.name, a]));
  return disk.map((d) => {
    const fromConfig = byName.get(d.name);
    return {
      name: d.name,
      displayName: fromConfig?.displayName ?? d.name,
      isNative: false, // disk accounts are never native; default is virtual
      createdAt: fromConfig?.createdAt ?? unixToIso(d.createdAtUnix),
    };
  });
}
```

Also update `loadAccounts` to **merge** the disk-derived list with any `isNative: true` accounts from config (which don't appear on disk):

```ts
  async loadAccounts(): Promise<void> {
    try {
      const disk = await ipcClient.listAccounts();
      const configAccounts = appSettingsStore.preferences.accounts ?? [];
      const fromDisk = reconcile(disk, configAccounts);
      const native = configAccounts.filter((a) => a.isNative);
      // Native first, then disk-backed, sorted by name.
      const sorted = [...fromDisk].sort((a, b) => a.name.localeCompare(b.name));
      this.accounts = [...native, ...sorted];
    } catch {
      this.accounts = [];
    }
    await this.loadStatuses();
  }
```

And update `createAccount` to include the new fields:

```ts
  async createAccount(name: string): Promise<Account> {
    const disk = await ipcClient.createAccount(name);
    const acct: Account = {
      name: disk.name,
      displayName: disk.name,
      isNative: false,
      createdAt: unixToIso(disk.createdAtUnix),
    };
    this.accounts = [...this.accounts, acct].sort((a, b) => a.name.localeCompare(b.name));
    const next = [...(appSettingsStore.preferences.accounts ?? []), acct];
    await appSettingsStore.update({ accounts: next });
    await this.refreshStatus(name);
    return acct;
  }
```

- [ ] **Step 3: Type-check**

```bash
pnpm exec tsc --noEmit 2>&1 | head -30
```

Expected: errors remain in `LauncherView.svelte`, `LauncherList.svelte`, `App.svelte`, and `launcher.svelte.ts` — all touched in Task 11.

- [ ] **Step 4: Commit**

```bash
git add src/lib/stores/appsettings.svelte.ts src/lib/stores/accounts.svelte.ts
git commit -m "stores(appsettings,accounts): match AppConfig v2 schema"
```

---

### Task 11: Replace launcher/projects stores with new gui_projects-backed store

**Files:**
- Modify: `src/lib/stores/projects.svelte.ts` (rewrite)
- Delete: `src/lib/stores/launcher.svelte.ts`
- Modify: `src/lib/ipc/client.ts`

- [ ] **Step 1: Add new IPC client methods**

In `src/lib/ipc/client.ts`:

- **Keep** the existing `listProjects`/`registerProject`/`unregisterProject` methods (settings still uses them).
- Rename them in the IpcClient to avoid name clash with the new methods: `listClaudeProjects`/`registerClaudeProject`/`unregisterClaudeProject` (or keep the names if not yet defined on the class). Search for `list_projects` callers in `src/` and update any.
- Add the new project commands:

```ts
  // --- gui projects (6) ---

  async listProjects(): Promise<ProjectEntry[]> {
    return call("gui_list_projects");
  }

  async addProject(path: string): Promise<ProjectEntry> {
    return call("add_project", { req: { path } });
  }

  async bindProject(path: string, account: string): Promise<void> {
    return call("bind_project", { req: { path, account } });
  }

  async unbindProject(path: string): Promise<void> {
    return call("unbind_project", { req: { path } });
  }

  async removeProject(path: string): Promise<void> {
    return call("remove_project", { req: { path } });
  }

  async updateProjectLaunch(path: string, launch: LaunchConfig): Promise<void> {
    return call("update_project_launch", { req: { path, launch } });
  }
```

Add the imports at the top:

```ts
import type {
  // ...existing...
  LaunchConfig,
  ProjectEntry,
} from "$lib/api/types";
```

- [ ] **Step 2: Rewrite projects store (preserving deprecated aliases for Stage 2/3 callers)**

Replace the contents of `src/lib/stores/projects.svelte.ts`:

```ts
import { ipcClient } from "$lib/ipc/client";
import type { LaunchConfig, ProjectEntry } from "$lib/api/types";

class ProjectsStore {
  /** Full list from backend; one entry per path in knownProjects. */
  entries = $state<ProjectEntry[]>([]);
  /** Path of the currently focused project (Stage 2 wires UI to this). */
  selectedPath = $state<string | null>(null);

  selected = $derived(
    this.entries.find((e) => e.path === this.selectedPath) ?? null,
  );

  // ── Deprecated aliases — kept so config.svelte.ts and Stage-2 components compile.
  //    Will be cleaned up in Stage 3 once consumers are rewritten.
  get projects(): ProjectEntry[] { return this.entries; }
  get activeProjectId(): string | null { return this.selectedPath; }
  get activeProject(): ProjectEntry | null { return this.selected; }

  async loadProjects(): Promise<void> {
    try {
      this.entries = await ipcClient.listProjects();
    } catch {
      this.entries = [];
    }
  }

  async add(path: string): Promise<void> {
    await ipcClient.addProject(path);
    await this.loadProjects();
  }

  async bind(path: string, account: string): Promise<void> {
    await ipcClient.bindProject(path, account);
    await this.loadProjects();
  }

  async unbind(path: string): Promise<void> {
    await ipcClient.unbindProject(path);
    await this.loadProjects();
  }

  async remove(path: string): Promise<void> {
    await ipcClient.removeProject(path);
    await this.loadProjects();
  }

  async updateLaunch(path: string, launch: LaunchConfig): Promise<void> {
    await ipcClient.updateProjectLaunch(path, launch);
    await this.loadProjects();
  }
}

export const projectsStore = new ProjectsStore();
```

Note: `ProjectEntry` has `path` field, not `id`. Any caller using `.id === something` needs updating — search for these in Step 5 below.

- [ ] **Step 3: Delete launcher store**

```bash
git rm src/lib/stores/launcher.svelte.ts
```

- [ ] **Step 4: Stub `LauncherView.svelte` and `LauncherList.svelte` minimally**

These components will be rewritten in Stage 3, but they currently reference the deleted `launcherStore`. Replace each file's content with a stub that compiles:

`src/lib/components/launcher/LauncherView.svelte`:

```svelte
<script lang="ts">
  // Stage 2 will rebuild this as the Project mode entry.
</script>

<div class="p-4 text-sm" style="color: var(--text-muted)">
  Launcher is being migrated to Projects mode (Stage 3).
</div>
```

`src/lib/components/launcher/LauncherList.svelte`:

```svelte
<script lang="ts">
  // Stage 2 will replace this with the Projects sidebar list.
</script>

<div class="px-4 py-2 text-xs" style="color: var(--text-muted)">
  Migrating to Projects mode…
</div>
```

- [ ] **Step 5: Fix any remaining type-check errors**

Search and patch any remaining stale references:

```bash
pnpm exec tsc --noEmit 2>&1 | head -40
```

Common fixes:
- `App.svelte` may set `--subpanel-width` from `appSettingsStore.preferences.subpanelWidth` — delete that line.
- `App.svelte` may call `appSettingsStore.update({ subpanelWidth: ... })` — delete that callback.
- Anywhere that reads `account.createdAt` and assumes ISO 8601: leave as is; the field is opaque now and that's fine.

For `config.svelte.ts` references to `projectsStore.activeProjectId` / `projectsStore.activeProject` — replace with `projectsStore.selectedPath` and `projectsStore.selected?.path`. (UI may behave oddly until Stage 2; acceptable per spec.)

- [ ] **Step 6: Confirm compile**

```bash
pnpm exec tsc --noEmit
```

Expected: 0 errors.

- [ ] **Step 7: Commit**

```bash
git add src/lib/stores/projects.svelte.ts src/lib/ipc/client.ts src/lib/components/launcher/ src/lib/stores/config.svelte.ts src/App.svelte
git commit -m "stores(projects): rewrite over gui_projects IPC; stub LauncherView until Stage 3"
```

---

### Task 12: End-to-end migration smoke test

**Files:** (manual)

- [ ] **Step 1: Snapshot the current live config**

```bash
cp ~/.dot-claude-gui/config.json ~/.dot-claude-gui/config.json.preflight.$(date +%s)
ls -la ~/.dot-claude-gui/
```

This protects your real data in case something goes wrong.

- [ ] **Step 2: Build and run**

```bash
pnpm tauri dev
```

Watch the terminal for `config migration: MigrationReport { migrated: true, bak_path: Some(...), default_injected: ... }`.

- [ ] **Step 3: Inspect the migrated file**

```bash
cat ~/.dot-claude-gui/config.json | python3 -m json.tool
ls -la ~/.dot-claude-gui/*.bak.*
```

Verify:
- Top of new config has `"schemaVersion": 2`
- `accounts` array contains `default` with `isNative: true` (if `~/.claude/` exists)
- `accounts` array contains the GUI accounts with `displayName` and `isNative: false`
- `projects` is an object keyed by absolute path, each with `account` and `launch`
- `knownProjects` is an array of absolute paths
- A `config.json.bak.<unix>` file exists alongside

- [ ] **Step 4: Restart the app**

```bash
# Ctrl-C, then again:
pnpm tauri dev
```

Confirm the log shows `migrated: false` on second start (idempotent).

- [ ] **Step 5: Spot-check Rust tests**

```bash
cargo test -p dot-claude-gui app_config::tests
cargo test -p dot-claude-gui commands::gui_projects::tests
```

Expected: both green, 19 + 6 = 25 tests total.

- [ ] **Step 6: Commit anything noticed**

If you needed to patch anything during smoke test, commit those fixes:

```bash
git add -p
git commit -m "fix(stage1): <whatever you found>"
```

---

## Acceptance criteria (per spec Stage 1)

- [x] Existing `~/.dot-claude-gui/` migrates cleanly (Task 12 confirms on real data).
- [x] `.bak` file exists post-migration (Task 5 test + Task 12 manual).
- [x] New schema validates (`schemaVersion: 2`, types match) (Task 1+12).
- [x] Default account appears when `~/.claude/` exists (Tasks 4+5+12).
- [x] New IPC commands wired and tested (Tasks 7+8).
- [x] Old fields removed; no double-write (Tasks 1+9 use only new schema).
- [x] UI compiles (Task 11 ensures `tsc --noEmit` clean); runtime behavior of Launcher/Projects views is stubbed and will be wired in Stage 2/3.

---

## Notes for the implementer

- **TDD strictness**: backend tasks (1–8) follow strict TDD. Frontend tasks (9–11) skip new tests because the project has no frontend test suite (per `CLAUDE.md`); rely on `tsc --noEmit` and the Task 12 smoke test.
- **Don't try to fix runtime errors in components.** That's Stage 2's job. Compilation must succeed; functionality is allowed to be broken.
- **Commit cadence**: each task ends in a commit. Don't squash. The branch is `main` (pre-release product, no PR required), so commits go straight in.
- **If a Rust test using `set_var("HOME", ...)` is flaky**, add `serial_test` dev-dep and `#[serial]` per Task 7's note. Don't refactor away from `HOME`-driven paths — that would leak production logic into test-only branches.
