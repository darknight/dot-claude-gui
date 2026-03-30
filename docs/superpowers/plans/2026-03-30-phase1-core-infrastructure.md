# Phase 1: Core Infrastructure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the foundation: Cargo workspace, config parsing library, daemon with REST/WebSocket API, Tauri app shell with three-panel Svelte UI that connects to the daemon and displays user config.

**Architecture:** Cargo workspace with three crates (`claude-types`, `claude-config`, `claude-daemon`) plus a Tauri 2 app shell. The daemon watches `~/.claude/` for changes and serves config via axum REST + WebSocket. The Svelte 5 frontend connects to the daemon and renders a three-panel layout. Local mode uses Tauri's sidecar to auto-manage the daemon process.

**Tech Stack:** Rust (axum, tokio, notify, serde, clap), Tauri 2.0, Svelte 5 (runes), TypeScript, Vite, Tailwind CSS 4

**Plan decomposition:** This is Phase 1 of 4 total phases. Subsequent phases:
- Phase 2: Settings Editor module (all sub-editors)
- Phase 3: Plugins + Skills + Memory modules
- Phase 4: MCP + Effective Config + Launcher + App Settings

---

## File Structure (Phase 1)

```
dot-claude-gui/
├── Cargo.toml                          # workspace root
├── crates/
│   ├── claude-types/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                  # re-exports
│   │       ├── settings.rs             # Settings struct with all sections
│   │       ├── api.rs                  # REST request/response types
│   │       └── events.rs              # WebSocket event types
│   ├── claude-config/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                  # public API
│   │       ├── parse.rs               # read settings from disk
│   │       ├── merge.rs               # four-layer merge engine
│   │       ├── validate.rs            # config validation
│   │       ├── write.rs               # atomic file write
│   │       └── watch.rs              # file watcher wrapper
│   └── claude-daemon/
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs                # CLI entry, server startup
│           ├── state.rs               # AppState (Arc<RwLock<...>>)
│           ├── auth.rs                # Bearer token middleware
│           ├── server.rs              # axum router assembly
│           ├── watcher.rs             # file watcher → state → WS broadcast
│           └── api/
│               ├── mod.rs
│               ├── config.rs          # GET/PUT config endpoints
│               ├── projects.rs        # project registration endpoints
│               ├── health.rs          # health + version
│               └── ws.rs             # WebSocket handler
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json
│   ├── icons/                         # app icons
│   └── src/
│       ├── main.rs                    # Tauri setup
│       └── lib.rs                     # Tauri commands
├── src/
│   ├── App.svelte                     # root component
│   ├── app.css                        # Tailwind imports
│   ├── main.ts                        # Svelte mount
│   ├── lib/
│   │   ├── api/
│   │   │   ├── client.ts             # REST API client
│   │   │   ├── ws.ts                 # WebSocket client with reconnect
│   │   │   └── types.ts             # TypeScript types matching claude-types
│   │   ├── stores/
│   │   │   ├── connection.svelte.ts  # daemon connection state
│   │   │   ├── config.svelte.ts      # config data state
│   │   │   └── projects.svelte.ts    # registered projects state
│   │   └── components/
│   │       ├── layout/
│   │       │   ├── ThreePanel.svelte  # resizable three-panel layout
│   │       │   ├── Sidebar.svelte     # left navigation
│   │       │   ├── SubPanel.svelte    # middle panel
│   │       │   └── DetailPanel.svelte # right panel
│   │       └── shared/
│   │           ├── ConnectionStatus.svelte
│   │           ├── ProjectSelector.svelte
│   │           └── JsonPreview.svelte
├── package.json
├── vite.config.ts
├── svelte.config.js
├── tailwind.config.ts
├── tsconfig.json
└── tests/
    └── fixtures/
        ├── settings-minimal.json      # minimal valid settings
        ├── settings-full.json         # full settings with all fields
        └── settings-unknown-fields.json # settings with future unknown fields
```

---

### Task 1: Cargo Workspace + claude-types Crate

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `crates/claude-types/Cargo.toml`
- Create: `crates/claude-types/src/lib.rs`
- Create: `crates/claude-types/src/settings.rs`
- Create: `crates/claude-types/src/api.rs`
- Create: `crates/claude-types/src/events.rs`
- Create: `tests/fixtures/settings-minimal.json`
- Create: `tests/fixtures/settings-full.json`
- Create: `tests/fixtures/settings-unknown-fields.json`

- [ ] **Step 1: Create workspace root Cargo.toml**

```toml
# Cargo.toml
[workspace]
resolver = "2"
members = [
    "crates/claude-types",
    "crates/claude-config",
    "crates/claude-daemon",
    "src-tauri",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
thiserror = "2"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

- [ ] **Step 2: Create claude-types Cargo.toml**

```toml
# crates/claude-types/Cargo.toml
[package]
name = "claude-types"
version.workspace = true
edition.workspace = true

[dependencies]
serde.workspace = true
serde_json.workspace = true
uuid.workspace = true
chrono.workspace = true
```

- [ ] **Step 3: Create settings types in `crates/claude-types/src/settings.rs`**

This is the core data model. All structs use `#[serde(default)]` and a `flatten` catch-all for forward compatibility.

```rust
// crates/claude-types/src/settings.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level settings.json structure.
/// Unknown fields are captured in `extra` to preserve them on roundtrip.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(default)]
    pub env: HashMap<String, String>,

    #[serde(default)]
    pub include_co_authored_by: Option<bool>,

    #[serde(default)]
    pub permissions: Option<Permissions>,

    #[serde(default)]
    pub hooks: Option<HashMap<String, Vec<HookGroup>>>,

    #[serde(default)]
    pub denied_mcp_servers: Option<Vec<McpServerRef>>,

    #[serde(default)]
    pub status_line: Option<StatusLine>,

    #[serde(default)]
    pub enabled_plugins: Option<HashMap<String, bool>>,

    #[serde(default)]
    pub extra_known_marketplaces: Option<HashMap<String, MarketplaceSource>>,

    #[serde(default)]
    pub language: Option<String>,

    #[serde(default)]
    pub always_thinking_enabled: Option<bool>,

    #[serde(default)]
    pub auto_updates_channel: Option<String>,

    #[serde(default)]
    pub minimum_version: Option<String>,

    #[serde(default)]
    pub skip_dangerous_mode_permission_prompt: Option<bool>,

    #[serde(default)]
    pub sandbox: Option<SandboxConfig>,

    #[serde(default)]
    pub model_overrides: Option<HashMap<String, String>>,

    /// Catch-all for unknown fields (forward compatibility)
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Permissions {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
    #[serde(default)]
    pub ask: Vec<String>,
    #[serde(default)]
    pub default_mode: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HookGroup {
    pub matcher: String,
    pub hooks: Vec<HookDefinition>,
    #[serde(default, rename = "if")]
    pub condition: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HookDefinition {
    #[serde(rename = "type")]
    pub hook_type: String, // "command" or "http"
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
    #[serde(default)]
    pub timeout: Option<u64>,
    #[serde(default)]
    pub allowed_env_vars: Option<Vec<String>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct McpServerRef {
    #[serde(default)]
    pub server_url: Option<String>,
    #[serde(default)]
    pub server_name: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StatusLine {
    #[serde(rename = "type")]
    pub status_type: Option<String>,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub padding: Option<u32>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MarketplaceSource {
    pub source: MarketplaceSourceInfo,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MarketplaceSourceInfo {
    pub source: String,
    pub repo: String,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SandboxConfig {
    #[serde(default)]
    pub allow_read: Option<Vec<String>>,
    #[serde(default)]
    pub deny_read: Option<Vec<String>>,
    #[serde(default)]
    pub allow_write: Option<Vec<String>>,
    #[serde(default)]
    pub excluded_commands: Option<Vec<String>>,
    #[serde(default)]
    pub fail_if_unavailable: Option<bool>,
    #[serde(default)]
    pub enable_weaker_network_isolation: Option<bool>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Wrapper that tracks which config layer a value came from
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EffectiveValue<T> {
    pub value: T,
    pub source: ConfigSource,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConfigSource {
    Managed,
    User,
    Project,
    Local,
    Default,
}
```

- [ ] **Step 4: Create API types in `crates/claude-types/src/api.rs`**

```rust
// crates/claude-types/src/api.rs
use serde::{Deserialize, Serialize};

use crate::settings::Settings;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigResponse {
    pub scope: String,
    pub settings: Settings,
    /// Optional project path if scope is "project" or "local"
    pub project_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateConfigRequest {
    /// Partial settings to merge into the existing config
    pub settings: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectEntry {
    pub id: String,
    pub path: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterProjectRequest {
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub claude_code_version: Option<String>,
    pub uptime_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub error: String,
    pub details: Option<Vec<ValidationError>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}
```

- [ ] **Step 5: Create WebSocket event types in `crates/claude-types/src/events.rs`**

```rust
// crates/claude-types/src/events.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum WsEvent {
    ConfigChanged {
        scope: String,
        section: Option<String>,
        project_id: Option<String>,
        data: serde_json::Value,
    },
    ValidationError {
        scope: String,
        errors: Vec<super::api::ValidationError>,
    },
    CommandOutput {
        request_id: String,
        stream: String,
        data: String,
    },
    CommandCompleted {
        request_id: String,
        exit_code: i32,
    },
    Connected {
        daemon_version: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum WsClientMessage {
    Subscribe { topics: Vec<String> },
    Unsubscribe { topics: Vec<String> },
}
```

- [ ] **Step 6: Create lib.rs**

```rust
// crates/claude-types/src/lib.rs
pub mod api;
pub mod events;
pub mod settings;
```

- [ ] **Step 7: Create test fixtures**

`tests/fixtures/settings-minimal.json`:
```json
{
  "permissions": {
    "defaultMode": "plan"
  }
}
```

`tests/fixtures/settings-full.json`:
```json
{
  "env": {},
  "includeCoAuthoredBy": false,
  "permissions": {
    "allow": ["Bash(git:*)", "WebSearch"],
    "deny": [],
    "ask": [],
    "defaultMode": "plan"
  },
  "deniedMcpServers": [
    { "serverUrl": "https://mcp.example.com/*" }
  ],
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "/usr/local/bin/hook"
          }
        ]
      }
    ]
  },
  "statusLine": {
    "type": "command",
    "command": "echo hello",
    "padding": 0
  },
  "enabledPlugins": {
    "superpowers@claude-plugins-official": true,
    "playwright@claude-plugins-official": false
  },
  "extraKnownMarketplaces": {
    "test-market": {
      "source": {
        "source": "github",
        "repo": "owner/repo"
      }
    }
  },
  "language": "en-US",
  "alwaysThinkingEnabled": true,
  "autoUpdatesChannel": "stable",
  "minimumVersion": "2.1.63",
  "skipDangerousModePermissionPrompt": true,
  "sandbox": {
    "allowRead": ["/tmp"],
    "failIfUnavailable": false
  }
}
```

`tests/fixtures/settings-unknown-fields.json`:
```json
{
  "permissions": {
    "defaultMode": "plan",
    "futureField": "should be preserved"
  },
  "someNewTopLevelField": {
    "nested": true,
    "data": [1, 2, 3]
  },
  "language": "en-US"
}
```

- [ ] **Step 8: Write roundtrip tests for settings types**

Add to `crates/claude-types/src/settings.rs` at the bottom:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_settings() {
        let json = include_str!("../../../tests/fixtures/settings-minimal.json");
        let settings: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(
            settings.permissions.as_ref().unwrap().default_mode.as_deref(),
            Some("plan")
        );
    }

    #[test]
    fn parse_full_settings() {
        let json = include_str!("../../../tests/fixtures/settings-full.json");
        let settings: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.language.as_deref(), Some("en-US"));
        assert_eq!(settings.always_thinking_enabled, Some(true));
        let perms = settings.permissions.as_ref().unwrap();
        assert_eq!(perms.allow.len(), 2);
        assert_eq!(perms.allow[0], "Bash(git:*)");
        let hooks = settings.hooks.as_ref().unwrap();
        assert!(hooks.contains_key("PreToolUse"));
        assert_eq!(hooks["PreToolUse"].len(), 1);
        assert_eq!(hooks["PreToolUse"][0].matcher, "Bash");
    }

    #[test]
    fn roundtrip_preserves_unknown_fields() {
        let json = include_str!("../../../tests/fixtures/settings-unknown-fields.json");
        let settings: Settings = serde_json::from_str(json).unwrap();

        // Unknown top-level field should be in extra
        assert!(settings.extra.contains_key("someNewTopLevelField"));

        // Unknown field inside permissions should be in permissions.extra
        let perms = settings.permissions.as_ref().unwrap();
        assert!(perms.extra.contains_key("futureField"));

        // Roundtrip: serialize back and parse again
        let serialized = serde_json::to_string_pretty(&settings).unwrap();
        let reparsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        let original: serde_json::Value = serde_json::from_str(json).unwrap();
        assert_eq!(original, reparsed);
    }

    #[test]
    fn empty_json_parses_to_default() {
        let settings: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(settings, Settings::default());
    }
}
```

- [ ] **Step 9: Run tests to verify**

Run: `cd /Users/eric.yao/workspace/darknight/dot-claude-gui && cargo test -p claude-types`
Expected: All 4 tests pass

- [ ] **Step 10: Commit**

```bash
git add Cargo.toml crates/claude-types/ tests/
git commit -m "feat: add claude-types crate with settings schema and roundtrip tests"
```

---

### Task 2: claude-config Crate — Parse + Write

**Files:**
- Create: `crates/claude-config/Cargo.toml`
- Create: `crates/claude-config/src/lib.rs`
- Create: `crates/claude-config/src/parse.rs`
- Create: `crates/claude-config/src/write.rs`

- [ ] **Step 1: Create claude-config Cargo.toml**

```toml
# crates/claude-config/Cargo.toml
[package]
name = "claude-config"
version.workspace = true
edition.workspace = true

[dependencies]
claude-types = { path = "../claude-types" }
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
thiserror.workspace = true
tracing.workspace = true
notify = "8"
notify-debouncer-full = "0.4"
tokio.workspace = true

[dev-dependencies]
tempfile = "3"
tokio = { workspace = true, features = ["test-util"] }
```

- [ ] **Step 2: Write the parse module**

```rust
// crates/claude-config/src/parse.rs
use anyhow::{Context, Result};
use claude_types::settings::Settings;
use std::path::Path;

/// Read and parse a settings.json file. Returns default Settings if file doesn't exist.
pub fn read_settings(path: &Path) -> Result<Settings> {
    if !path.exists() {
        return Ok(Settings::default());
    }
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    parse_settings(&content)
}

/// Parse settings from a JSON string.
pub fn parse_settings(json: &str) -> Result<Settings> {
    let settings: Settings =
        serde_json::from_str(json).context("failed to parse settings JSON")?;
    Ok(settings)
}

/// Resolve the standard paths for the four config layers.
pub struct ConfigPaths {
    pub managed: Option<std::path::PathBuf>,
    pub user: std::path::PathBuf,
    pub project: Option<std::path::PathBuf>,
    pub local: Option<std::path::PathBuf>,
}

impl ConfigPaths {
    /// Create config paths for a given claude home and optional project directory.
    pub fn new(claude_home: &Path, project_dir: Option<&Path>) -> Self {
        let managed = claude_home.join("managed-settings.json");
        Self {
            managed: if managed.exists() { Some(managed) } else { None },
            user: claude_home.join("settings.json"),
            project: project_dir.map(|p| p.join(".claude").join("settings.json")),
            local: project_dir.map(|p| p.join(".claude").join("settings.local.json")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn read_nonexistent_returns_default() {
        let result = read_settings(Path::new("/nonexistent/settings.json")).unwrap();
        assert_eq!(result, Settings::default());
    }

    #[test]
    fn read_valid_settings_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, r#"{"language": "zh-CN"}"#).unwrap();

        let settings = read_settings(&path).unwrap();
        assert_eq!(settings.language.as_deref(), Some("zh-CN"));
    }

    #[test]
    fn read_invalid_json_returns_error() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, "not json").unwrap();

        assert!(read_settings(&path).is_err());
    }

    #[test]
    fn config_paths_without_project() {
        let dir = TempDir::new().unwrap();
        let paths = ConfigPaths::new(dir.path(), None);
        assert_eq!(paths.user, dir.path().join("settings.json"));
        assert!(paths.project.is_none());
        assert!(paths.local.is_none());
    }

    #[test]
    fn config_paths_with_project() {
        let home = TempDir::new().unwrap();
        let project = TempDir::new().unwrap();
        let paths = ConfigPaths::new(home.path(), Some(project.path()));
        assert!(paths.project.is_some());
        assert!(paths.local.is_some());
    }
}
```

- [ ] **Step 3: Write the atomic write module**

```rust
// crates/claude-config/src/write.rs
use anyhow::{Context, Result};
use claude_types::settings::Settings;
use std::path::Path;

/// Atomically write settings to a file.
/// Writes to a temp file first, then renames to prevent corruption.
pub fn write_settings(path: &Path, settings: &Settings) -> Result<()> {
    let json = serde_json::to_string_pretty(settings)
        .context("failed to serialize settings")?;
    atomic_write(path, json.as_bytes())
}

/// Atomically write bytes to a file using write-then-rename.
pub fn atomic_write(path: &Path, data: &[u8]) -> Result<()> {
    let tmp_path = path.with_extension("json.tmp");

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory {}", parent.display()))?;
    }

    std::fs::write(&tmp_path, data)
        .with_context(|| format!("failed to write temp file {}", tmp_path.display()))?;

    std::fs::rename(&tmp_path, path).with_context(|| {
        // Clean up temp file on rename failure
        let _ = std::fs::remove_file(&tmp_path);
        format!("failed to rename {} to {}", tmp_path.display(), path.display())
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn atomic_write_creates_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.json");

        atomic_write(&path, b"hello").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello");
    }

    #[test]
    fn atomic_write_no_temp_file_left() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.json");

        atomic_write(&path, b"data").unwrap();
        let tmp = path.with_extension("json.tmp");
        assert!(!tmp.exists());
    }

    #[test]
    fn write_settings_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("settings.json");

        let mut settings = Settings::default();
        settings.language = Some("zh-CN".to_string());

        write_settings(&path, &settings).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let parsed: Settings = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.language.as_deref(), Some("zh-CN"));
    }

    #[test]
    fn atomic_write_creates_parent_dirs() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("sub").join("dir").join("test.json");

        atomic_write(&path, b"nested").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "nested");
    }
}
```

- [ ] **Step 4: Create lib.rs**

```rust
// crates/claude-config/src/lib.rs
pub mod merge;
pub mod parse;
pub mod validate;
pub mod watch;
pub mod write;
```

Create empty placeholder modules so it compiles:

```rust
// crates/claude-config/src/merge.rs
// Implemented in Task 3

// crates/claude-config/src/validate.rs
// Implemented in Task 4

// crates/claude-config/src/watch.rs
// Implemented in Task 5
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p claude-config`
Expected: All 9 tests pass (5 parse + 4 write)

- [ ] **Step 6: Commit**

```bash
git add crates/claude-config/
git commit -m "feat: add claude-config crate with parse and atomic write"
```

---

### Task 3: claude-config — Merge Engine

**Files:**
- Modify: `crates/claude-config/src/merge.rs`

- [ ] **Step 1: Write failing tests for merge behavior**

```rust
// crates/claude-config/src/merge.rs
use claude_types::settings::{ConfigSource, Settings};
use serde_json::Value;
use std::collections::HashMap;

/// A labeled config layer for merging.
#[derive(Debug, Clone)]
pub struct ConfigLayer {
    pub source: ConfigSource,
    pub settings: Settings,
}

/// Result of merging: the final settings plus per-field source tracking.
#[derive(Debug, Clone)]
pub struct MergedConfig {
    pub settings: Settings,
    pub field_sources: HashMap<String, ConfigSource>,
}

/// Merge config layers in priority order (last wins).
/// Layers should be ordered: [managed, user, project, local]
/// Later layers override earlier ones for non-None fields.
pub fn merge_layers(layers: &[ConfigLayer]) -> MergedConfig {
    let mut result = Settings::default();
    let mut sources: HashMap<String, ConfigSource> = HashMap::new();

    for layer in layers {
        let source = layer.source;
        let s = &layer.settings;

        // Merge each Option field: if the layer has Some, it overrides
        macro_rules! merge_field {
            ($field:ident, $key:expr) => {
                if s.$field.is_some() {
                    result.$field = s.$field.clone();
                    sources.insert($key.to_string(), source);
                }
            };
        }

        // env is a HashMap, merge key by key
        for (k, v) in &s.env {
            result.env.insert(k.clone(), v.clone());
            sources.insert(format!("env.{}", k), source);
        }

        merge_field!(include_co_authored_by, "includeCoAuthoredBy");
        merge_field!(language, "language");
        merge_field!(always_thinking_enabled, "alwaysThinkingEnabled");
        merge_field!(auto_updates_channel, "autoUpdatesChannel");
        merge_field!(minimum_version, "minimumVersion");
        merge_field!(skip_dangerous_mode_permission_prompt, "skipDangerousModePermissionPrompt");
        merge_field!(status_line, "statusLine");
        merge_field!(sandbox, "sandbox");
        merge_field!(model_overrides, "modelOverrides");
        merge_field!(denied_mcp_servers, "deniedMcpServers");

        // Permissions: merge allow/deny/ask lists (concatenate, not replace)
        if let Some(ref perms) = s.permissions {
            let result_perms = result.permissions.get_or_insert_with(Default::default);
            if !perms.allow.is_empty() {
                result_perms.allow.extend(perms.allow.iter().cloned());
                sources.insert("permissions.allow".to_string(), source);
            }
            if !perms.deny.is_empty() {
                result_perms.deny.extend(perms.deny.iter().cloned());
                sources.insert("permissions.deny".to_string(), source);
            }
            if !perms.ask.is_empty() {
                result_perms.ask.extend(perms.ask.iter().cloned());
                sources.insert("permissions.ask".to_string(), source);
            }
            if perms.default_mode.is_some() {
                result_perms.default_mode = perms.default_mode.clone();
                sources.insert("permissions.defaultMode".to_string(), source);
            }
        }

        // Hooks: merge by event type (later layer replaces per event type)
        if let Some(ref hooks) = s.hooks {
            let result_hooks = result.hooks.get_or_insert_with(HashMap::new);
            for (event, groups) in hooks {
                result_hooks.insert(event.clone(), groups.clone());
                sources.insert(format!("hooks.{}", event), source);
            }
        }

        // enabledPlugins: merge key by key
        if let Some(ref plugins) = s.enabled_plugins {
            let result_plugins = result.enabled_plugins.get_or_insert_with(HashMap::new);
            for (k, v) in plugins {
                result_plugins.insert(k.clone(), *v);
                sources.insert(format!("enabledPlugins.{}", k), source);
            }
        }

        // extraKnownMarketplaces: merge key by key
        if let Some(ref markets) = s.extra_known_marketplaces {
            let result_markets = result.extra_known_marketplaces.get_or_insert_with(HashMap::new);
            for (k, v) in markets {
                result_markets.insert(k.clone(), v.clone());
                sources.insert(format!("extraKnownMarketplaces.{}", k), source);
            }
        }

        // Extra (unknown) fields: merge key by key
        for (k, v) in &s.extra {
            result.extra.insert(k.clone(), v.clone());
            sources.insert(k.clone(), source);
        }
    }

    MergedConfig {
        settings: result,
        field_sources: sources,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claude_types::settings::Permissions;

    fn make_layer(source: ConfigSource, settings: Settings) -> ConfigLayer {
        ConfigLayer { source, settings }
    }

    #[test]
    fn empty_layers_produce_default() {
        let result = merge_layers(&[]);
        assert_eq!(result.settings, Settings::default());
        assert!(result.field_sources.is_empty());
    }

    #[test]
    fn single_layer_passes_through() {
        let mut s = Settings::default();
        s.language = Some("zh-CN".to_string());
        let layers = vec![make_layer(ConfigSource::User, s)];
        let result = merge_layers(&layers);
        assert_eq!(result.settings.language.as_deref(), Some("zh-CN"));
        assert_eq!(result.field_sources["language"], ConfigSource::User);
    }

    #[test]
    fn later_layer_overrides_scalar_field() {
        let mut user = Settings::default();
        user.language = Some("en-US".to_string());

        let mut project = Settings::default();
        project.language = Some("zh-CN".to_string());

        let layers = vec![
            make_layer(ConfigSource::User, user),
            make_layer(ConfigSource::Project, project),
        ];
        let result = merge_layers(&layers);
        assert_eq!(result.settings.language.as_deref(), Some("zh-CN"));
        assert_eq!(result.field_sources["language"], ConfigSource::Project);
    }

    #[test]
    fn none_field_does_not_override() {
        let mut user = Settings::default();
        user.language = Some("en-US".to_string());

        let project = Settings::default(); // language is None

        let layers = vec![
            make_layer(ConfigSource::User, user),
            make_layer(ConfigSource::Project, project),
        ];
        let result = merge_layers(&layers);
        assert_eq!(result.settings.language.as_deref(), Some("en-US"));
        assert_eq!(result.field_sources["language"], ConfigSource::User);
    }

    #[test]
    fn permissions_allow_lists_concatenate() {
        let mut user = Settings::default();
        user.permissions = Some(Permissions {
            allow: vec!["Bash(git:*)".to_string()],
            ..Default::default()
        });

        let mut project = Settings::default();
        project.permissions = Some(Permissions {
            allow: vec!["WebSearch".to_string()],
            ..Default::default()
        });

        let layers = vec![
            make_layer(ConfigSource::User, user),
            make_layer(ConfigSource::Project, project),
        ];
        let result = merge_layers(&layers);
        let perms = result.settings.permissions.unwrap();
        assert_eq!(perms.allow.len(), 2);
        assert!(perms.allow.contains(&"Bash(git:*)".to_string()));
        assert!(perms.allow.contains(&"WebSearch".to_string()));
    }

    #[test]
    fn hooks_replaced_per_event_type() {
        let mut user = Settings::default();
        user.hooks = Some(HashMap::from([(
            "PreToolUse".to_string(),
            vec![],
        )]));

        let mut project = Settings::default();
        project.hooks = Some(HashMap::from([(
            "PreToolUse".to_string(),
            vec![], // different hooks
        )]));

        let layers = vec![
            make_layer(ConfigSource::User, user),
            make_layer(ConfigSource::Project, project),
        ];
        let result = merge_layers(&layers);
        assert_eq!(
            result.field_sources["hooks.PreToolUse"],
            ConfigSource::Project
        );
    }

    #[test]
    fn enabled_plugins_merge_per_key() {
        let mut user = Settings::default();
        user.enabled_plugins = Some(HashMap::from([
            ("a".to_string(), true),
            ("b".to_string(), true),
        ]));

        let mut project = Settings::default();
        project.enabled_plugins = Some(HashMap::from([
            ("b".to_string(), false), // override b
        ]));

        let layers = vec![
            make_layer(ConfigSource::User, user),
            make_layer(ConfigSource::Project, project),
        ];
        let result = merge_layers(&layers);
        let plugins = result.settings.enabled_plugins.unwrap();
        assert_eq!(plugins["a"], true);
        assert_eq!(plugins["b"], false); // overridden
        assert_eq!(result.field_sources["enabledPlugins.b"], ConfigSource::Project);
    }

    #[test]
    fn unknown_fields_merge_across_layers() {
        let mut user = Settings::default();
        user.extra.insert("customA".to_string(), serde_json::json!(1));

        let mut project = Settings::default();
        project.extra.insert("customB".to_string(), serde_json::json!(2));

        let layers = vec![
            make_layer(ConfigSource::User, user),
            make_layer(ConfigSource::Project, project),
        ];
        let result = merge_layers(&layers);
        assert_eq!(result.settings.extra["customA"], serde_json::json!(1));
        assert_eq!(result.settings.extra["customB"], serde_json::json!(2));
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p claude-config -- merge`
Expected: All 8 merge tests pass

- [ ] **Step 3: Commit**

```bash
git add crates/claude-config/src/merge.rs
git commit -m "feat: add four-layer config merge engine with source tracking"
```

---

### Task 4: claude-config — Validation

**Files:**
- Modify: `crates/claude-config/src/validate.rs`

- [ ] **Step 1: Write validation module with tests**

```rust
// crates/claude-config/src/validate.rs
use claude_types::api::ValidationError;
use claude_types::settings::Settings;

/// Validate a settings struct. Returns a list of validation errors (empty = valid).
pub fn validate_settings(settings: &Settings) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Validate permissions.defaultMode
    if let Some(ref perms) = settings.permissions {
        if let Some(ref mode) = perms.default_mode {
            let valid_modes = [
                "acceptEdits",
                "bypassPermissions",
                "default",
                "dontAsk",
                "plan",
                "auto",
            ];
            if !valid_modes.contains(&mode.as_str()) {
                errors.push(ValidationError {
                    path: "permissions.defaultMode".to_string(),
                    message: format!(
                        "invalid mode '{}', must be one of: {}",
                        mode,
                        valid_modes.join(", ")
                    ),
                });
            }
        }
    }

    // Validate hooks
    if let Some(ref hooks) = settings.hooks {
        let valid_events = [
            "PreToolUse",
            "PostToolUse",
            "Notification",
            "Stop",
            "SubagentStop",
            "CwdChanged",
            "FileChanged",
            "ConfigChange",
            "StopFailure",
            "TaskCreated",
            "WorktreeCreate",
            "WorktreeRemove",
            "PostCompact",
            "Elicitation",
            "InstructionsLoaded",
        ];
        for (event, groups) in hooks {
            if !valid_events.contains(&event.as_str()) {
                errors.push(ValidationError {
                    path: format!("hooks.{}", event),
                    message: format!(
                        "unknown hook event '{}', valid events: {}",
                        event,
                        valid_events.join(", ")
                    ),
                });
            }
            for (i, group) in groups.iter().enumerate() {
                for (j, hook) in group.hooks.iter().enumerate() {
                    let base = format!("hooks.{}[{}].hooks[{}]", event, i, j);
                    match hook.hook_type.as_str() {
                        "command" => {
                            if hook.command.as_ref().map_or(true, |c| c.is_empty()) {
                                errors.push(ValidationError {
                                    path: format!("{}.command", base),
                                    message: "command hook requires a non-empty 'command' field"
                                        .to_string(),
                                });
                            }
                        }
                        "http" => {
                            if hook.url.as_ref().map_or(true, |u| u.is_empty()) {
                                errors.push(ValidationError {
                                    path: format!("{}.url", base),
                                    message: "http hook requires a non-empty 'url' field"
                                        .to_string(),
                                });
                            }
                        }
                        other => {
                            errors.push(ValidationError {
                                path: format!("{}.type", base),
                                message: format!(
                                    "invalid hook type '{}', must be 'command' or 'http'",
                                    other
                                ),
                            });
                        }
                    }
                }
            }
        }
    }

    // Validate statusLine
    if let Some(ref sl) = settings.status_line {
        if let Some(ref t) = sl.status_type {
            if t != "command" {
                errors.push(ValidationError {
                    path: "statusLine.type".to_string(),
                    message: format!("invalid statusLine type '{}', expected 'command'", t),
                });
            }
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use claude_types::settings::{HookDefinition, HookGroup, Permissions};
    use std::collections::HashMap;

    #[test]
    fn valid_settings_no_errors() {
        let json = include_str!("../../../tests/fixtures/settings-full.json");
        let settings: Settings = serde_json::from_str(json).unwrap();
        let errors = validate_settings(&settings);
        assert!(errors.is_empty(), "errors: {:?}", errors);
    }

    #[test]
    fn invalid_default_mode() {
        let mut settings = Settings::default();
        settings.permissions = Some(Permissions {
            default_mode: Some("invalid_mode".to_string()),
            ..Default::default()
        });
        let errors = validate_settings(&settings);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].path, "permissions.defaultMode");
    }

    #[test]
    fn unknown_hook_event() {
        let mut settings = Settings::default();
        settings.hooks = Some(HashMap::from([(
            "InvalidEvent".to_string(),
            vec![],
        )]));
        let errors = validate_settings(&settings);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("unknown hook event"));
    }

    #[test]
    fn command_hook_missing_command() {
        let mut settings = Settings::default();
        settings.hooks = Some(HashMap::from([(
            "PreToolUse".to_string(),
            vec![HookGroup {
                matcher: "Bash".to_string(),
                hooks: vec![HookDefinition {
                    hook_type: "command".to_string(),
                    command: None,
                    url: None,
                    method: None,
                    headers: None,
                    timeout: None,
                    allowed_env_vars: None,
                    extra: HashMap::new(),
                }],
                condition: None,
                extra: HashMap::new(),
            }],
        )]));
        let errors = validate_settings(&settings);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].path.contains("command"));
    }

    #[test]
    fn http_hook_missing_url() {
        let mut settings = Settings::default();
        settings.hooks = Some(HashMap::from([(
            "PreToolUse".to_string(),
            vec![HookGroup {
                matcher: "*".to_string(),
                hooks: vec![HookDefinition {
                    hook_type: "http".to_string(),
                    command: None,
                    url: None,
                    method: None,
                    headers: None,
                    timeout: None,
                    allowed_env_vars: None,
                    extra: HashMap::new(),
                }],
                condition: None,
                extra: HashMap::new(),
            }],
        )]));
        let errors = validate_settings(&settings);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].path.contains("url"));
    }

    #[test]
    fn invalid_hook_type() {
        let mut settings = Settings::default();
        settings.hooks = Some(HashMap::from([(
            "PostToolUse".to_string(),
            vec![HookGroup {
                matcher: "*".to_string(),
                hooks: vec![HookDefinition {
                    hook_type: "websocket".to_string(),
                    command: None,
                    url: None,
                    method: None,
                    headers: None,
                    timeout: None,
                    allowed_env_vars: None,
                    extra: HashMap::new(),
                }],
                condition: None,
                extra: HashMap::new(),
            }],
        )]));
        let errors = validate_settings(&settings);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("invalid hook type"));
    }

    #[test]
    fn default_settings_valid() {
        let errors = validate_settings(&Settings::default());
        assert!(errors.is_empty());
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p claude-config -- validate`
Expected: All 7 validation tests pass

- [ ] **Step 3: Commit**

```bash
git add crates/claude-config/src/validate.rs
git commit -m "feat: add settings validation with hook and permission checks"
```

---

### Task 5: claude-config — File Watcher

**Files:**
- Modify: `crates/claude-config/src/watch.rs`

- [ ] **Step 1: Write file watcher wrapper**

```rust
// crates/claude-config/src/watch.rs
use anyhow::Result;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;
use tracing::{debug, error, warn};

/// A file change event emitted by the watcher.
#[derive(Debug, Clone)]
pub struct FileChangeEvent {
    pub path: PathBuf,
    pub kind: FileChangeKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileChangeKind {
    Created,
    Modified,
    Removed,
}

/// Watch a list of directories and send file change events through a channel.
/// Returns the watcher handle (must be kept alive) and a receiver for events.
pub fn watch_directories(
    paths: &[PathBuf],
) -> Result<(RecommendedWatcher, mpsc::Receiver<FileChangeEvent>)> {
    let (tx, rx) = mpsc::channel();

    let mut watcher = RecommendedWatcher::new(
        move |result: Result<Event, notify::Error>| match result {
            Ok(event) => {
                let kind = match event.kind {
                    notify::EventKind::Create(_) => Some(FileChangeKind::Created),
                    notify::EventKind::Modify(_) => Some(FileChangeKind::Modified),
                    notify::EventKind::Remove(_) => Some(FileChangeKind::Removed),
                    _ => None,
                };
                if let Some(kind) = kind {
                    for path in event.paths {
                        // Only care about .json and .md files
                        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                        if ext == "json" || ext == "md" {
                            debug!("file change: {:?} {:?}", kind, path);
                            let _ = tx.send(FileChangeEvent {
                                path: path.clone(),
                                kind: kind.clone(),
                            });
                        }
                    }
                }
            }
            Err(e) => error!("watch error: {:?}", e),
        },
        Config::default().with_poll_interval(Duration::from_secs(2)),
    )?;

    for path in paths {
        if path.exists() {
            watcher.watch(path, RecursiveMode::Recursive)?;
            debug!("watching: {}", path.display());
        } else {
            warn!("skipping non-existent path: {}", path.display());
        }
    }

    Ok((watcher, rx))
}

/// Add a new watch path to an existing watcher.
pub fn add_watch_path(watcher: &mut RecommendedWatcher, path: &Path) -> Result<()> {
    if path.exists() {
        watcher.watch(path, RecursiveMode::Recursive)?;
        debug!("added watch: {}", path.display());
    }
    Ok(())
}

/// Remove a watch path from an existing watcher.
pub fn remove_watch_path(watcher: &mut RecommendedWatcher, path: &Path) -> Result<()> {
    let _ = watcher.unwatch(path);
    debug!("removed watch: {}", path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use tempfile::TempDir;

    #[test]
    fn watch_detects_json_file_creation() {
        let dir = TempDir::new().unwrap();
        let (_watcher, rx) = watch_directories(&[dir.path().to_path_buf()]).unwrap();

        // Give watcher time to initialize
        thread::sleep(Duration::from_millis(200));

        // Create a json file
        let file_path = dir.path().join("settings.json");
        std::fs::write(&file_path, "{}").unwrap();

        // Wait for event
        let event = rx.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(event.path, file_path);
        assert!(matches!(
            event.kind,
            FileChangeKind::Created | FileChangeKind::Modified
        ));
    }

    #[test]
    fn watch_detects_file_modification() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("settings.json");
        std::fs::write(&file_path, "{}").unwrap();

        let (_watcher, rx) = watch_directories(&[dir.path().to_path_buf()]).unwrap();
        thread::sleep(Duration::from_millis(200));

        // Modify the file
        std::fs::write(&file_path, r#"{"language":"en"}"#).unwrap();

        let event = rx.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(event.kind, FileChangeKind::Modified);
    }

    #[test]
    fn watch_ignores_non_json_md_files() {
        let dir = TempDir::new().unwrap();
        let (_watcher, rx) = watch_directories(&[dir.path().to_path_buf()]).unwrap();
        thread::sleep(Duration::from_millis(200));

        // Create a .txt file — should be ignored
        std::fs::write(dir.path().join("notes.txt"), "hello").unwrap();

        let result = rx.recv_timeout(Duration::from_secs(2));
        assert!(result.is_err(), "should not receive event for .txt file");
    }

    #[test]
    fn watch_nonexistent_path_does_not_error() {
        let result = watch_directories(&[PathBuf::from("/nonexistent/path")]);
        assert!(result.is_ok());
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p claude-config -- watch`
Expected: All 4 watcher tests pass (may need up to 10 seconds due to FS event timing)

- [ ] **Step 3: Commit**

```bash
git add crates/claude-config/src/watch.rs
git commit -m "feat: add file watcher wrapper with JSON/MD filtering"
```

---

### Task 6: claude-daemon — Server Skeleton + Auth + Health

**Files:**
- Create: `crates/claude-daemon/Cargo.toml`
- Create: `crates/claude-daemon/src/main.rs`
- Create: `crates/claude-daemon/src/state.rs`
- Create: `crates/claude-daemon/src/auth.rs`
- Create: `crates/claude-daemon/src/server.rs`
- Create: `crates/claude-daemon/src/api/mod.rs`
- Create: `crates/claude-daemon/src/api/health.rs`

- [ ] **Step 1: Create claude-daemon Cargo.toml**

```toml
# crates/claude-daemon/Cargo.toml
[package]
name = "claude-daemon"
version.workspace = true
edition.workspace = true

[[bin]]
name = "claude-daemon"
path = "src/main.rs"

[dependencies]
claude-types = { path = "../claude-types" }
claude-config = { path = "../claude-config" }
serde.workspace = true
serde_json.workspace = true
tokio.workspace = true
anyhow.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
uuid.workspace = true
axum = { version = "0.8", features = ["ws"] }
axum-extra = { version = "0.10", features = ["typed-header"] }
tower-http = { version = "0.6", features = ["cors"] }
tower = "0.5"
clap = { version = "4", features = ["derive"] }
rand = "0.8"
base64 = "0.22"
notify = "8"

[dev-dependencies]
reqwest = { version = "0.12", features = ["json"] }
tokio = { workspace = true, features = ["test-util"] }
tempfile = "3"
```

- [ ] **Step 2: Create state module**

```rust
// crates/claude-daemon/src/state.rs
use claude_config::parse::ConfigPaths;
use claude_types::events::WsEvent;
use claude_types::settings::Settings;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Registered project entry.
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub id: String,
    pub path: PathBuf,
    pub name: String,
}

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<AppStateInner>,
}

pub struct AppStateInner {
    /// Path to ~/.claude/
    pub claude_home: PathBuf,
    /// User-level settings (cached)
    pub user_settings: RwLock<Settings>,
    /// Project-level settings (cached, keyed by project id)
    pub project_settings: RwLock<HashMap<String, Settings>>,
    /// Local project settings (cached, keyed by project id)
    pub local_settings: RwLock<HashMap<String, Settings>>,
    /// Registered projects
    pub projects: RwLock<Vec<ProjectInfo>>,
    /// Auth token
    pub auth_token: String,
    /// WebSocket broadcast channel
    pub ws_tx: broadcast::Sender<WsEvent>,
    /// Daemon start time
    pub started_at: std::time::Instant,
}

impl AppState {
    pub fn new(claude_home: PathBuf, auth_token: String) -> Self {
        let (ws_tx, _) = broadcast::channel(256);
        Self {
            inner: Arc::new(AppStateInner {
                claude_home,
                user_settings: RwLock::new(Settings::default()),
                project_settings: RwLock::new(HashMap::new()),
                local_settings: RwLock::new(HashMap::new()),
                projects: RwLock::new(Vec::new()),
                auth_token,
                ws_tx,
                started_at: std::time::Instant::now(),
            }),
        }
    }

    /// Load user settings from disk into cache.
    pub async fn load_user_settings(&self) -> anyhow::Result<()> {
        let path = self.inner.claude_home.join("settings.json");
        let settings = claude_config::parse::read_settings(&path)?;
        *self.inner.user_settings.write().await = settings;
        Ok(())
    }

    /// Broadcast a WebSocket event to all connected clients.
    pub fn broadcast(&self, event: WsEvent) {
        // Ignore send errors (no subscribers)
        let _ = self.inner.ws_tx.send(event);
    }
}
```

- [ ] **Step 3: Create auth middleware**

```rust
// crates/claude-daemon/src/auth.rs
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};

/// Extract Bearer token from Authorization header and verify it.
pub async fn auth_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get the expected token from app state
    let state = request
        .extensions()
        .get::<crate::state::AppState>()
        .cloned()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let token = auth_header
        .strip_prefix("Bearer ")
        .unwrap_or("");

    if token != state.inner.auth_token {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(request).await)
}
```

- [ ] **Step 4: Create health endpoint**

```rust
// crates/claude-daemon/src/api/mod.rs
pub mod config;
pub mod health;
pub mod projects;
pub mod ws;
```

Create placeholder files so it compiles:

```rust
// crates/claude-daemon/src/api/config.rs
// Implemented in Task 7

// crates/claude-daemon/src/api/projects.rs
// Implemented in Task 8

// crates/claude-daemon/src/api/ws.rs
// Implemented in Task 9
```

```rust
// crates/claude-daemon/src/api/health.rs
use axum::extract::State;
use axum::Json;
use claude_types::api::HealthResponse;
use crate::state::AppState;

pub async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    let uptime = state.inner.started_at.elapsed().as_secs();
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        claude_code_version: None,
        uptime_seconds: uptime,
    })
}
```

- [ ] **Step 5: Create server router**

```rust
// crates/claude-daemon/src/server.rs
use axum::{
    middleware,
    routing::get,
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use crate::api;
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Public routes (no auth required)
    let public = Router::new()
        .route("/api/v1/health", get(api::health::health));

    // Protected routes (auth required)
    let protected = Router::new()
        // Config routes (Task 7)
        // Project routes (Task 8)
        // WebSocket route (Task 9)
        .layer(middleware::from_fn(crate::auth::auth_middleware));

    Router::new()
        .merge(public)
        .merge(protected)
        .layer(cors)
        .with_state(state.clone())
        .layer(axum::Extension(state))
}
```

- [ ] **Step 6: Create main entry point**

```rust
// crates/claude-daemon/src/main.rs
mod api;
mod auth;
mod server;
mod state;
mod watcher;

use anyhow::Result;
use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;
use tracing::info;

#[derive(Parser)]
#[command(name = "claude-daemon", about = "Daemon for dot-claude-gui")]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value = "7890")]
    port: u16,

    /// Path to ~/.claude/ directory
    #[arg(long, env = "CLAUDE_HOME")]
    claude_home: Option<PathBuf>,

    /// Auth token (auto-generated if not provided)
    #[arg(long, env = "CLAUDE_DAEMON_TOKEN")]
    token: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "claude_daemon=info".into()),
        )
        .init();

    let args = Args::parse();

    let claude_home = args
        .claude_home
        .or_else(|| dirs_next::home_dir().map(|h| h.join(".claude")))
        .expect("could not determine claude home directory");

    // Generate or use provided auth token
    let token = args.token.unwrap_or_else(|| {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: [u8; 32] = rng.gen();
        base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, bytes)
    });

    // Write token file for GUI to read
    let token_path = claude_home.join("daemon-token");
    std::fs::write(&token_path, &token)?;
    info!("auth token written to {}", token_path.display());

    let state = state::AppState::new(claude_home.clone(), token);

    // Load initial settings
    state.load_user_settings().await?;
    info!("loaded user settings");

    let app = server::create_router(state);
    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    info!("daemon listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

Also create a placeholder watcher module:

```rust
// crates/claude-daemon/src/watcher.rs
// Implemented in Task 10
```

- [ ] **Step 7: Add `dirs-next` dependency to Cargo.toml**

Add to `crates/claude-daemon/Cargo.toml` under `[dependencies]`:
```toml
dirs-next = "2"
```

- [ ] **Step 8: Verify it compiles**

Run: `cargo build -p claude-daemon`
Expected: Builds successfully (with warnings about unused code, which is fine)

- [ ] **Step 9: Commit**

```bash
git add crates/claude-daemon/
git commit -m "feat: add claude-daemon skeleton with auth middleware and health endpoint"
```

---

### Task 7: claude-daemon — Config API Endpoints

**Files:**
- Modify: `crates/claude-daemon/src/api/config.rs`
- Modify: `crates/claude-daemon/src/server.rs`

- [ ] **Step 1: Implement config endpoints**

```rust
// crates/claude-daemon/src/api/config.rs
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use claude_config::{merge, parse, validate, write};
use claude_types::{
    api::{ConfigResponse, ErrorResponse, UpdateConfigRequest},
    settings::{ConfigSource, Settings},
};
use crate::state::AppState;

/// GET /api/v1/config/user
pub async fn get_user_config(State(state): State<AppState>) -> Json<ConfigResponse> {
    let settings = state.inner.user_settings.read().await;
    Json(ConfigResponse {
        scope: "user".to_string(),
        settings: settings.clone(),
        project_path: None,
    })
}

/// GET /api/v1/config/project/:project_id
pub async fn get_project_config(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<ConfigResponse>, (StatusCode, Json<ErrorResponse>)> {
    let projects = state.inner.projects.read().await;
    let project = projects
        .iter()
        .find(|p| p.id == project_id)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("project '{}' not found", project_id),
                    details: None,
                }),
            )
        })?;
    let path = project.path.join(".claude").join("settings.json");
    let project_path = project.path.to_string_lossy().to_string();

    let settings = parse::read_settings(&path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
                details: None,
            }),
        )
    })?;

    Ok(Json(ConfigResponse {
        scope: "project".to_string(),
        settings,
        project_path: Some(project_path),
    }))
}

/// GET /api/v1/config/effective/:project_id
pub async fn get_effective_config(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let user = state.inner.user_settings.read().await.clone();

    let projects = state.inner.projects.read().await;
    let project = projects.iter().find(|p| p.id == project_id);

    let mut layers = vec![merge::ConfigLayer {
        source: ConfigSource::User,
        settings: user,
    }];

    if let Some(project) = project {
        let proj_path = project.path.join(".claude").join("settings.json");
        if let Ok(proj_settings) = parse::read_settings(&proj_path) {
            layers.push(merge::ConfigLayer {
                source: ConfigSource::Project,
                settings: proj_settings,
            });
        }
        let local_path = project.path.join(".claude").join("settings.local.json");
        if let Ok(local_settings) = parse::read_settings(&local_path) {
            layers.push(merge::ConfigLayer {
                source: ConfigSource::Local,
                settings: local_settings,
            });
        }
    }

    let merged = merge::merge_layers(&layers);
    Ok(Json(serde_json::json!({
        "settings": merged.settings,
        "fieldSources": merged.field_sources,
    })))
}

/// PUT /api/v1/config/user
pub async fn update_user_config(
    State(state): State<AppState>,
    Json(req): Json<UpdateConfigRequest>,
) -> Result<Json<ConfigResponse>, (StatusCode, Json<ErrorResponse>)> {
    let path = state.inner.claude_home.join("settings.json");

    // Read current, merge with update
    let mut current = parse::read_settings(&path).unwrap_or_default();
    let update: Settings = serde_json::from_value(req.settings).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("invalid settings: {}", e),
                details: None,
            }),
        )
    })?;

    // Apply update by merging (update overrides current)
    let merged = merge::merge_layers(&[
        merge::ConfigLayer {
            source: ConfigSource::User,
            settings: current,
        },
        merge::ConfigLayer {
            source: ConfigSource::User,
            settings: update,
        },
    ]);
    let new_settings = merged.settings;

    // Validate
    let errors = validate::validate_settings(&new_settings);
    if !errors.is_empty() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ErrorResponse {
                error: "validation failed".to_string(),
                details: Some(errors),
            }),
        ));
    }

    // Atomic write
    write::write_settings(&path, &new_settings).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
                details: None,
            }),
        )
    })?;

    // Update cache
    *state.inner.user_settings.write().await = new_settings.clone();

    Ok(Json(ConfigResponse {
        scope: "user".to_string(),
        settings: new_settings,
        project_path: None,
    }))
}
```

- [ ] **Step 2: Add routes to server.rs**

Replace the protected router section in `crates/claude-daemon/src/server.rs`:

```rust
// crates/claude-daemon/src/server.rs
use axum::{
    middleware,
    routing::{get, put},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use crate::api;
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let public = Router::new()
        .route("/api/v1/health", get(api::health::health));

    let protected = Router::new()
        .route("/api/v1/config/user", get(api::config::get_user_config))
        .route("/api/v1/config/user", put(api::config::update_user_config))
        .route(
            "/api/v1/config/project/{project_id}",
            get(api::config::get_project_config),
        )
        .route(
            "/api/v1/config/effective/{project_id}",
            get(api::config::get_effective_config),
        )
        // Project routes (Task 8)
        // WebSocket route (Task 9)
        .layer(middleware::from_fn(crate::auth::auth_middleware));

    Router::new()
        .merge(public)
        .merge(protected)
        .layer(cors)
        .with_state(state.clone())
        .layer(axum::Extension(state))
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build -p claude-daemon`
Expected: Builds successfully

- [ ] **Step 4: Commit**

```bash
git add crates/claude-daemon/src/api/config.rs crates/claude-daemon/src/server.rs
git commit -m "feat: add config CRUD API endpoints with validation"
```

---

### Task 8: claude-daemon — Project + WebSocket Endpoints

**Files:**
- Modify: `crates/claude-daemon/src/api/projects.rs`
- Modify: `crates/claude-daemon/src/api/ws.rs`
- Modify: `crates/claude-daemon/src/server.rs`

- [ ] **Step 1: Implement project endpoints**

```rust
// crates/claude-daemon/src/api/projects.rs
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use claude_types::api::{ErrorResponse, ProjectEntry, RegisterProjectRequest};
use crate::state::{AppState, ProjectInfo};
use uuid::Uuid;

/// GET /api/v1/projects
pub async fn list_projects(State(state): State<AppState>) -> Json<Vec<ProjectEntry>> {
    let projects = state.inner.projects.read().await;
    Json(
        projects
            .iter()
            .map(|p| ProjectEntry {
                id: p.id.clone(),
                path: p.path.to_string_lossy().to_string(),
                name: p.name.clone(),
            })
            .collect(),
    )
}

/// POST /api/v1/projects
pub async fn register_project(
    State(state): State<AppState>,
    Json(req): Json<RegisterProjectRequest>,
) -> Result<(StatusCode, Json<ProjectEntry>), (StatusCode, Json<ErrorResponse>)> {
    let path = std::path::PathBuf::from(&req.path);
    if !path.exists() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("path does not exist: {}", req.path),
                details: None,
            }),
        ));
    }

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| req.path.clone());

    let id = Uuid::new_v4().to_string();

    let entry = ProjectInfo {
        id: id.clone(),
        path: path.clone(),
        name: name.clone(),
    };

    state.inner.projects.write().await.push(entry);

    Ok((
        StatusCode::CREATED,
        Json(ProjectEntry {
            id,
            path: req.path,
            name,
        }),
    ))
}

/// DELETE /api/v1/projects/:id
pub async fn unregister_project(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut projects = state.inner.projects.write().await;
    let len_before = projects.len();
    projects.retain(|p| p.id != id);
    if projects.len() == len_before {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("project '{}' not found", id),
                details: None,
            }),
        ));
    }
    Ok(StatusCode::NO_CONTENT)
}
```

- [ ] **Step 2: Implement WebSocket handler**

```rust
// crates/claude-daemon/src/api/ws.rs
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::StatusCode,
    response::Response,
};
use claude_types::events::{WsClientMessage, WsEvent};
use crate::state::AppState;
use serde::Deserialize;
use tracing::{debug, error};

#[derive(Deserialize)]
pub struct WsQuery {
    token: Option<String>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    // Authenticate via query param
    let token = query.token.unwrap_or_default();
    if token != state.inner.auth_token {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state)))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    debug!("WebSocket client connected");

    // Send connected event
    let connected = WsEvent::Connected {
        daemon_version: env!("CARGO_PKG_VERSION").to_string(),
    };
    if let Ok(msg) = serde_json::to_string(&connected) {
        let _ = socket.send(Message::Text(msg.into())).await;
    }

    // Subscribe to broadcast events
    let mut rx = state.inner.ws_tx.subscribe();

    loop {
        tokio::select! {
            // Forward broadcast events to client
            Ok(event) = rx.recv() => {
                match serde_json::to_string(&event) {
                    Ok(msg) => {
                        if socket.send(Message::Text(msg.into())).await.is_err() {
                            break; // Client disconnected
                        }
                    }
                    Err(e) => error!("failed to serialize event: {}", e),
                }
            }
            // Handle client messages
            Some(Ok(msg)) = socket.recv() => {
                match msg {
                    Message::Text(text) => {
                        if let Ok(client_msg) = serde_json::from_str::<WsClientMessage>(&text) {
                            debug!("client message: {:?}", client_msg);
                            // Topic filtering can be implemented later
                        }
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
            else => break,
        }
    }

    debug!("WebSocket client disconnected");
}
```

- [ ] **Step 3: Update server.rs with all routes**

```rust
// crates/claude-daemon/src/server.rs
use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use crate::api;
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let public = Router::new()
        .route("/api/v1/health", get(api::health::health))
        // WebSocket uses query param auth, not header auth
        .route("/api/v1/ws", get(api::ws::ws_handler));

    let protected = Router::new()
        // Config routes
        .route("/api/v1/config/user", get(api::config::get_user_config))
        .route("/api/v1/config/user", put(api::config::update_user_config))
        .route(
            "/api/v1/config/project/{project_id}",
            get(api::config::get_project_config),
        )
        .route(
            "/api/v1/config/effective/{project_id}",
            get(api::config::get_effective_config),
        )
        // Project routes
        .route("/api/v1/projects", get(api::projects::list_projects))
        .route("/api/v1/projects", post(api::projects::register_project))
        .route(
            "/api/v1/projects/{id}",
            delete(api::projects::unregister_project),
        )
        .layer(middleware::from_fn(crate::auth::auth_middleware));

    Router::new()
        .merge(public)
        .merge(protected)
        .layer(cors)
        .with_state(state.clone())
        .layer(axum::Extension(state))
}
```

- [ ] **Step 4: Verify compilation**

Run: `cargo build -p claude-daemon`
Expected: Builds successfully

- [ ] **Step 5: Commit**

```bash
git add crates/claude-daemon/src/
git commit -m "feat: add project registration and WebSocket endpoints"
```

---

### Task 9: claude-daemon — File Watcher Integration

**Files:**
- Modify: `crates/claude-daemon/src/watcher.rs`
- Modify: `crates/claude-daemon/src/main.rs`

- [ ] **Step 1: Implement daemon watcher that bridges file events to WebSocket**

```rust
// crates/claude-daemon/src/watcher.rs
use crate::state::AppState;
use claude_config::{parse, watch};
use claude_types::events::WsEvent;
use std::path::PathBuf;
use tracing::{error, info};

/// Start the file watcher in a background task.
/// Watches claude_home and all registered project directories.
/// On file changes, reloads config and broadcasts WS events.
pub fn start_watcher(state: AppState) -> anyhow::Result<()> {
    let claude_home = state.inner.claude_home.clone();
    let paths = vec![claude_home.clone()];

    let (_watcher, rx) = watch::watch_directories(&paths)?;

    // Spawn a blocking thread to receive sync events and bridge to async
    let state_clone = state.clone();
    std::thread::spawn(move || {
        loop {
            match rx.recv() {
                Ok(event) => {
                    let state = state_clone.clone();
                    let claude_home = claude_home.clone();

                    // Use a tokio runtime handle to run async code
                    tokio::runtime::Handle::current().spawn(async move {
                        handle_file_event(&state, &claude_home, &event.path).await;
                    });
                }
                Err(_) => {
                    error!("file watcher channel closed");
                    break;
                }
            }
        }
    });

    info!("file watcher started");
    Ok(())
}

async fn handle_file_event(state: &AppState, claude_home: &PathBuf, changed_path: &PathBuf) {
    let file_name = changed_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    // Check if it's the user settings file
    let user_settings_path = claude_home.join("settings.json");
    if changed_path == &user_settings_path {
        info!("user settings changed, reloading");
        match parse::read_settings(&user_settings_path) {
            Ok(new_settings) => {
                let data = serde_json::to_value(&new_settings).unwrap_or_default();
                *state.inner.user_settings.write().await = new_settings;
                state.broadcast(WsEvent::ConfigChanged {
                    scope: "user".to_string(),
                    section: None,
                    project_id: None,
                    data,
                });
            }
            Err(e) => {
                error!("failed to reload user settings: {}", e);
                state.broadcast(WsEvent::ValidationError {
                    scope: "user".to_string(),
                    errors: vec![claude_types::api::ValidationError {
                        path: "settings.json".to_string(),
                        message: e.to_string(),
                    }],
                });
            }
        }
        return;
    }

    // Check if it's a project settings file
    let projects = state.inner.projects.read().await;
    for project in projects.iter() {
        let proj_settings = project.path.join(".claude").join("settings.json");
        let local_settings = project.path.join(".claude").join("settings.local.json");

        if changed_path == &proj_settings || changed_path == &local_settings {
            let scope = if changed_path == &proj_settings {
                "project"
            } else {
                "local"
            };
            info!("{} settings changed for project {}", scope, project.name);

            match parse::read_settings(changed_path) {
                Ok(new_settings) => {
                    let data = serde_json::to_value(&new_settings).unwrap_or_default();
                    state.broadcast(WsEvent::ConfigChanged {
                        scope: scope.to_string(),
                        section: None,
                        project_id: Some(project.id.clone()),
                        data,
                    });
                }
                Err(e) => {
                    error!("failed to reload {} settings: {}", scope, e);
                }
            }
            return;
        }
    }
}
```

- [ ] **Step 2: Integrate watcher into main.rs**

Add after `state.load_user_settings().await?;` in main.rs:

```rust
    // Start file watcher
    watcher::start_watcher(state.clone())?;
```

- [ ] **Step 3: Verify compilation and basic run**

Run: `cargo build -p claude-daemon`
Expected: Builds successfully

Run a quick manual smoke test:
```bash
# Start daemon in background
cargo run -p claude-daemon -- --port 17890 --claude-home /tmp/test-claude &
DAEMON_PID=$!

# Read token
TOKEN=$(cat /tmp/test-claude/daemon-token)

# Test health endpoint
curl -s http://127.0.0.1:17890/api/v1/health | python3 -m json.tool

# Test auth (should get 401)
curl -s -w "%{http_code}" http://127.0.0.1:17890/api/v1/config/user

# Test with auth (should get 200)
curl -s -H "Authorization: Bearer $TOKEN" http://127.0.0.1:17890/api/v1/config/user | python3 -m json.tool

# Cleanup
kill $DAEMON_PID
rm -rf /tmp/test-claude
```

Expected: Health returns 200 with JSON, unauthenticated returns 401, authenticated returns user config.

- [ ] **Step 4: Commit**

```bash
git add crates/claude-daemon/src/watcher.rs crates/claude-daemon/src/main.rs
git commit -m "feat: integrate file watcher with config reload and WS broadcast"
```

---

### Task 10: Tauri App Shell + Svelte Frontend Scaffolding

**Files:**
- Create: `package.json`
- Create: `vite.config.ts`
- Create: `svelte.config.js`
- Create: `tailwind.config.ts`
- Create: `tsconfig.json`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/capabilities/default.json`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/lib.rs`
- Create: `src/main.ts`
- Create: `src/App.svelte`
- Create: `src/app.css`
- Create: `index.html`

- [ ] **Step 1: Create package.json**

```json
{
  "name": "dot-claude-gui",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "tauri": "tauri",
    "check": "svelte-check --tsconfig ./tsconfig.json"
  },
  "devDependencies": {
    "@sveltejs/vite-plugin-svelte": "^5",
    "@tailwindcss/vite": "^4",
    "@tauri-apps/cli": "^2",
    "@tsconfig/svelte": "^5",
    "svelte": "^5",
    "svelte-check": "^4",
    "tailwindcss": "^4",
    "typescript": "^5.7",
    "vite": "^6"
  },
  "dependencies": {
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-shell": "^2"
  }
}
```

- [ ] **Step 2: Create vite.config.ts**

```typescript
// vite.config.ts
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [svelte(), tailwindcss()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
});
```

- [ ] **Step 3: Create svelte.config.js**

```javascript
// svelte.config.js
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

export default {
  preprocess: vitePreprocess(),
};
```

- [ ] **Step 4: Create tailwind.config.ts**

```typescript
// tailwind.config.ts
// Tailwind CSS v4 uses CSS-based config, this file is minimal
export default {};
```

- [ ] **Step 5: Create tsconfig.json**

```json
{
  "extends": "@tsconfig/svelte/tsconfig.json",
  "compilerOptions": {
    "target": "ESNext",
    "useDefineForClassFields": true,
    "module": "ESNext",
    "resolveJsonModule": true,
    "allowJs": true,
    "checkJs": true,
    "isolatedModules": true,
    "moduleDetection": "force",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "baseUrl": ".",
    "paths": {
      "$lib/*": ["src/lib/*"]
    }
  },
  "include": ["src/**/*.ts", "src/**/*.svelte"]
}
```

- [ ] **Step 6: Create index.html**

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>dot-claude-gui</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```

- [ ] **Step 7: Create src/app.css**

```css
/* src/app.css */
@import "tailwindcss";

:root {
  --sidebar-width: 56px;
  --subpanel-width: 240px;
}

body {
  margin: 0;
  font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", "Segoe UI",
    Roboto, sans-serif;
  -webkit-font-smoothing: antialiased;
}

/* Dark mode support */
@media (prefers-color-scheme: dark) {
  :root {
    color-scheme: dark;
  }
}
```

- [ ] **Step 8: Create src/main.ts**

```typescript
// src/main.ts
import App from "./App.svelte";
import "./app.css";
import { mount } from "svelte";

const app = mount(App, { target: document.getElementById("app")! });

export default app;
```

- [ ] **Step 9: Create src/App.svelte (placeholder)**

```svelte
<!-- src/App.svelte -->
<script lang="ts">
  let currentModule = $state("settings");
</script>

<div class="flex h-screen bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100">
  <!-- Sidebar -->
  <aside class="w-14 bg-gray-100 dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 flex flex-col items-center py-4 gap-2">
    <button
      class="w-10 h-10 rounded-lg flex items-center justify-center hover:bg-gray-200 dark:hover:bg-gray-700"
      class:bg-blue-100={currentModule === "settings"}
      class:dark:bg-blue-900={currentModule === "settings"}
      onclick={() => (currentModule = "settings")}
      title="Settings"
    >
      S
    </button>
    <button
      class="w-10 h-10 rounded-lg flex items-center justify-center hover:bg-gray-200 dark:hover:bg-gray-700"
      class:bg-blue-100={currentModule === "plugins"}
      class:dark:bg-blue-900={currentModule === "plugins"}
      onclick={() => (currentModule = "plugins")}
      title="Plugins"
    >
      P
    </button>
    <button
      class="w-10 h-10 rounded-lg flex items-center justify-center hover:bg-gray-200 dark:hover:bg-gray-700"
      class:bg-blue-100={currentModule === "skills"}
      class:dark:bg-blue-900={currentModule === "skills"}
      onclick={() => (currentModule = "skills")}
      title="Skills"
    >
      K
    </button>
    <button
      class="w-10 h-10 rounded-lg flex items-center justify-center hover:bg-gray-200 dark:hover:bg-gray-700"
      class:bg-blue-100={currentModule === "memory"}
      class:dark:bg-blue-900={currentModule === "memory"}
      onclick={() => (currentModule = "memory")}
      title="Memory"
    >
      M
    </button>
  </aside>

  <!-- Sub panel -->
  <div class="w-60 border-r border-gray-200 dark:border-gray-700 p-4">
    <h2 class="text-sm font-semibold uppercase text-gray-500 mb-4">
      {currentModule}
    </h2>
    <p class="text-sm text-gray-400">Sub-items will go here</p>
  </div>

  <!-- Detail panel -->
  <main class="flex-1 p-6 overflow-auto">
    <h1 class="text-xl font-bold mb-4">dot-claude-gui</h1>
    <p class="text-gray-500">
      Select a module from the sidebar. Currently viewing: <strong>{currentModule}</strong>
    </p>
    <p class="text-sm text-gray-400 mt-4">Phase 1 scaffold — feature modules coming in Phase 2-4.</p>
  </main>
</div>
```

- [ ] **Step 10: Create Tauri config files**

`src-tauri/Cargo.toml`:
```toml
[package]
name = "dot-claude-gui"
version.workspace = true
edition.workspace = true

[lib]
name = "dot_claude_gui_lib"
crate-type = ["lib", "cdylib", "staticlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
claude-types = { path = "../crates/claude-types" }
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"
serde.workspace = true
serde_json.workspace = true
```

`src-tauri/tauri.conf.json`:
```json
{
  "$schema": "https://raw.githubusercontent.com/nickkuk/schemastore/tauri-v2/src/schemas/json/tauri.json",
  "productName": "dot-claude-gui",
  "identifier": "com.dotclaude.gui",
  "version": "0.1.0",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "pnpm dev",
    "beforeBuildCommand": "pnpm build"
  },
  "app": {
    "windows": [
      {
        "title": "dot-claude-gui",
        "width": 1200,
        "height": 800,
        "minWidth": 900,
        "minHeight": 600,
        "resizable": true,
        "decorations": true
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

`src-tauri/capabilities/default.json`:
```json
{
  "$schema": "https://raw.githubusercontent.com/nickkuk/schemastore/tauri-v2/src/schemas/json/tauri-capability.json",
  "identifier": "default",
  "description": "Default capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-open",
    "shell:allow-execute",
    "shell:allow-spawn"
  ]
}
```

Create `src-tauri/build.rs`:
```rust
fn main() {
    tauri_build::build();
}
```

- [ ] **Step 11: Create Tauri Rust source files**

`src-tauri/src/lib.rs`:
```rust
use tauri::Manager;

#[tauri::command]
fn get_daemon_url() -> String {
    // In local mode, daemon runs as sidecar on localhost
    "http://127.0.0.1:7890".to_string()
}

#[tauri::command]
fn get_daemon_token() -> Result<String, String> {
    let home = dirs_next::home_dir().ok_or("cannot find home directory")?;
    let token_path = home.join(".claude").join("daemon-token");
    std::fs::read_to_string(&token_path)
        .map_err(|e| format!("failed to read daemon token: {}", e))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_daemon_url,
            get_daemon_token,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`src-tauri/src/main.rs`:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    dot_claude_gui_lib::run();
}
```

Add to `src-tauri/Cargo.toml`:
```toml
dirs-next = "2"
```

- [ ] **Step 12: Create placeholder icons**

```bash
mkdir -p src-tauri/icons
# Create minimal placeholder icons (will be replaced with real icons later)
# Use a simple 1x1 pixel PNG for now
python3 -c "
import struct, zlib
def png(w, h):
    raw = b''
    for y in range(h):
        raw += b'\x00' + b'\x00\x00\xff\xff' * w
    return (b'\x89PNG\r\n\x1a\n' +
            struct.pack('>I', 13) + b'IHDR' + struct.pack('>IIBB', w, h, 8, 6) + b'\x00\x00\x00' + struct.pack('>I', 0) +
            struct.pack('>I', len(zlib.compress(raw))) + b'IDAT' + zlib.compress(raw) + struct.pack('>I', 0) +
            struct.pack('>I', 0) + b'IEND' + struct.pack('>I', 0))

for name, size in [('32x32.png', 32), ('128x128.png', 128), ('128x128@2x.png', 256)]:
    with open(f'src-tauri/icons/{name}', 'wb') as f:
        f.write(png(size, size))
"
```

Note: The icon generation above is simplified. For a real build, use `tauri icon` to generate proper icons.

- [ ] **Step 13: Install dependencies and verify frontend builds**

```bash
cd /Users/eric.yao/workspace/darknight/dot-claude-gui
pnpm install
pnpm build
```

Expected: Vite builds successfully, outputs to `dist/`

- [ ] **Step 14: Verify Tauri compiles**

```bash
cargo build -p dot-claude-gui
```

Expected: Tauri app compiles (may take longer on first build due to WebKit frameworks)

- [ ] **Step 15: Commit**

```bash
git add package.json vite.config.ts svelte.config.js tailwind.config.ts tsconfig.json index.html
git add src/ src-tauri/
git commit -m "feat: add Tauri 2 app shell with Svelte 5 three-panel layout"
```

---

### Task 11: Frontend API Client + WebSocket Client

**Files:**
- Create: `src/lib/api/types.ts`
- Create: `src/lib/api/client.ts`
- Create: `src/lib/api/ws.ts`

- [ ] **Step 1: Create TypeScript types matching claude-types**

```typescript
// src/lib/api/types.ts

export interface Settings {
  env?: Record<string, string>;
  includeCoAuthoredBy?: boolean;
  permissions?: Permissions;
  hooks?: Record<string, HookGroup[]>;
  deniedMcpServers?: McpServerRef[];
  statusLine?: StatusLine;
  enabledPlugins?: Record<string, boolean>;
  extraKnownMarketplaces?: Record<string, MarketplaceSource>;
  language?: string;
  alwaysThinkingEnabled?: boolean;
  autoUpdatesChannel?: string;
  minimumVersion?: string;
  skipDangerousModePermissionPrompt?: boolean;
  sandbox?: SandboxConfig;
  modelOverrides?: Record<string, string>;
  [key: string]: unknown; // catch-all for unknown fields
}

export interface Permissions {
  allow: string[];
  deny: string[];
  ask: string[];
  defaultMode?: string;
  [key: string]: unknown;
}

export interface HookGroup {
  matcher: string;
  hooks: HookDefinition[];
  if?: string;
  [key: string]: unknown;
}

export interface HookDefinition {
  type: "command" | "http";
  command?: string;
  url?: string;
  method?: string;
  headers?: Record<string, string>;
  timeout?: number;
  allowedEnvVars?: string[];
  [key: string]: unknown;
}

export interface McpServerRef {
  serverUrl?: string;
  serverName?: string;
  [key: string]: unknown;
}

export interface StatusLine {
  type?: string;
  command?: string;
  padding?: number;
  [key: string]: unknown;
}

export interface MarketplaceSource {
  source: { source: string; repo: string };
  [key: string]: unknown;
}

export interface SandboxConfig {
  allowRead?: string[];
  denyRead?: string[];
  allowWrite?: string[];
  excludedCommands?: string[];
  failIfUnavailable?: boolean;
  enableWeakerNetworkIsolation?: boolean;
  [key: string]: unknown;
}

export interface ConfigResponse {
  scope: string;
  settings: Settings;
  projectPath?: string;
}

export interface ProjectEntry {
  id: string;
  path: string;
  name: string;
}

export interface HealthResponse {
  status: string;
  version: string;
  claudeCodeVersion?: string;
  uptimeSeconds: number;
}

export interface ErrorResponse {
  error: string;
  details?: ValidationError[];
}

export interface ValidationError {
  path: string;
  message: string;
}

export interface EffectiveConfig {
  settings: Settings;
  fieldSources: Record<string, string>;
}

// WebSocket events
export type WsEvent =
  | { event: "config_changed"; scope: string; section?: string; projectId?: string; data: unknown }
  | { event: "validation_error"; scope: string; errors: ValidationError[] }
  | { event: "command_output"; requestId: string; stream: string; data: string }
  | { event: "command_completed"; requestId: string; exitCode: number }
  | { event: "connected"; daemonVersion: string };
```

- [ ] **Step 2: Create REST API client**

```typescript
// src/lib/api/client.ts
import type {
  ConfigResponse,
  EffectiveConfig,
  ErrorResponse,
  HealthResponse,
  ProjectEntry,
  Settings,
} from "./types";

export class DaemonClient {
  private baseUrl: string;
  private token: string;

  constructor(baseUrl: string, token: string) {
    this.baseUrl = baseUrl.replace(/\/$/, "");
    this.token = token;
  }

  private async fetch<T>(path: string, init?: RequestInit): Promise<T> {
    const url = `${this.baseUrl}${path}`;
    const res = await fetch(url, {
      ...init,
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${this.token}`,
        ...init?.headers,
      },
    });

    if (!res.ok) {
      const body: ErrorResponse = await res.json().catch(() => ({
        error: `HTTP ${res.status}`,
        details: undefined,
      }));
      throw new ApiError(res.status, body.error, body.details);
    }

    return res.json();
  }

  // Health (no auth required)
  async health(): Promise<HealthResponse> {
    const res = await fetch(`${this.baseUrl}/api/v1/health`);
    return res.json();
  }

  // Config
  async getUserConfig(): Promise<ConfigResponse> {
    return this.fetch("/api/v1/config/user");
  }

  async getProjectConfig(projectId: string): Promise<ConfigResponse> {
    return this.fetch(`/api/v1/config/project/${projectId}`);
  }

  async getEffectiveConfig(projectId: string): Promise<EffectiveConfig> {
    return this.fetch(`/api/v1/config/effective/${projectId}`);
  }

  async updateUserConfig(settings: Partial<Settings>): Promise<ConfigResponse> {
    return this.fetch("/api/v1/config/user", {
      method: "PUT",
      body: JSON.stringify({ settings }),
    });
  }

  // Projects
  async listProjects(): Promise<ProjectEntry[]> {
    return this.fetch("/api/v1/projects");
  }

  async registerProject(path: string): Promise<ProjectEntry> {
    return this.fetch("/api/v1/projects", {
      method: "POST",
      body: JSON.stringify({ path }),
    });
  }

  async unregisterProject(id: string): Promise<void> {
    await this.fetch(`/api/v1/projects/${id}`, { method: "DELETE" });
  }
}

export class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
    public details?: { path: string; message: string }[],
  ) {
    super(message);
    this.name = "ApiError";
  }
}
```

- [ ] **Step 3: Create WebSocket client with auto-reconnect**

```typescript
// src/lib/api/ws.ts
import type { WsEvent } from "./types";

type WsEventHandler = (event: WsEvent) => void;

export class DaemonWsClient {
  private url: string;
  private token: string;
  private ws: WebSocket | null = null;
  private handlers: Set<WsEventHandler> = new Set();
  private reconnectDelay = 1000;
  private maxReconnectDelay = 30000;
  private shouldReconnect = true;
  private _connected = false;

  constructor(baseUrl: string, token: string) {
    const wsUrl = baseUrl.replace(/^http/, "ws");
    this.url = `${wsUrl}/api/v1/ws?token=${encodeURIComponent(token)}`;
    this.token = token;
  }

  get connected(): boolean {
    return this._connected;
  }

  connect(): void {
    this.shouldReconnect = true;
    this.doConnect();
  }

  disconnect(): void {
    this.shouldReconnect = false;
    this.ws?.close();
    this.ws = null;
    this._connected = false;
  }

  onEvent(handler: WsEventHandler): () => void {
    this.handlers.add(handler);
    return () => this.handlers.delete(handler);
  }

  private doConnect(): void {
    try {
      this.ws = new WebSocket(this.url);

      this.ws.onopen = () => {
        this._connected = true;
        this.reconnectDelay = 1000;
        console.log("[ws] connected");
      };

      this.ws.onmessage = (msg) => {
        try {
          const event: WsEvent = JSON.parse(msg.data);
          for (const handler of this.handlers) {
            handler(event);
          }
        } catch (e) {
          console.error("[ws] failed to parse message:", e);
        }
      };

      this.ws.onclose = () => {
        this._connected = false;
        console.log("[ws] disconnected");
        if (this.shouldReconnect) {
          console.log(`[ws] reconnecting in ${this.reconnectDelay}ms`);
          setTimeout(() => this.doConnect(), this.reconnectDelay);
          this.reconnectDelay = Math.min(
            this.reconnectDelay * 2,
            this.maxReconnectDelay,
          );
        }
      };

      this.ws.onerror = (e) => {
        console.error("[ws] error:", e);
      };
    } catch (e) {
      console.error("[ws] connection failed:", e);
      if (this.shouldReconnect) {
        setTimeout(() => this.doConnect(), this.reconnectDelay);
      }
    }
  }
}
```

- [ ] **Step 4: Commit**

```bash
git add src/lib/
git commit -m "feat: add TypeScript API client and WebSocket client with auto-reconnect"
```

---

### Task 12: Frontend Stores + Connection Status

**Files:**
- Create: `src/lib/stores/connection.svelte.ts`
- Create: `src/lib/stores/config.svelte.ts`
- Create: `src/lib/stores/projects.svelte.ts`
- Create: `src/lib/components/shared/ConnectionStatus.svelte`
- Modify: `src/App.svelte`

- [ ] **Step 1: Create connection store**

```typescript
// src/lib/stores/connection.svelte.ts
import { DaemonClient } from "$lib/api/client";
import { DaemonWsClient } from "$lib/api/ws";
import type { WsEvent } from "$lib/api/types";

class ConnectionStore {
  status = $state<"disconnected" | "connecting" | "connected">("disconnected");
  daemonVersion = $state<string>("");
  error = $state<string>("");

  client: DaemonClient | null = null;
  wsClient: DaemonWsClient | null = null;

  async connect(baseUrl: string, token: string) {
    this.status = "connecting";
    this.error = "";

    this.client = new DaemonClient(baseUrl, token);
    this.wsClient = new DaemonWsClient(baseUrl, token);

    try {
      const health = await this.client.health();
      this.daemonVersion = health.version;

      this.wsClient.onEvent((event: WsEvent) => {
        if (event.event === "connected") {
          this.daemonVersion = event.daemonVersion;
        }
      });

      this.wsClient.connect();
      this.status = "connected";
    } catch (e) {
      this.status = "disconnected";
      this.error = e instanceof Error ? e.message : "Connection failed";
    }
  }

  disconnect() {
    this.wsClient?.disconnect();
    this.client = null;
    this.wsClient = null;
    this.status = "disconnected";
  }
}

export const connectionStore = new ConnectionStore();
```

- [ ] **Step 2: Create config store**

```typescript
// src/lib/stores/config.svelte.ts
import { connectionStore } from "./connection.svelte";
import type { Settings } from "$lib/api/types";

class ConfigStore {
  userSettings = $state<Settings>({});
  loading = $state(false);
  error = $state<string>("");

  async loadUserConfig() {
    const client = connectionStore.client;
    if (!client) return;

    this.loading = true;
    this.error = "";
    try {
      const res = await client.getUserConfig();
      this.userSettings = res.settings;
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to load config";
    } finally {
      this.loading = false;
    }
  }

  async updateUserConfig(settings: Partial<Settings>) {
    const client = connectionStore.client;
    if (!client) return;

    this.error = "";
    try {
      const res = await client.updateUserConfig(settings);
      this.userSettings = res.settings;
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to update config";
      throw e;
    }
  }
}

export const configStore = new ConfigStore();
```

- [ ] **Step 3: Create projects store**

```typescript
// src/lib/stores/projects.svelte.ts
import { connectionStore } from "./connection.svelte";
import type { ProjectEntry } from "$lib/api/types";

class ProjectsStore {
  projects = $state<ProjectEntry[]>([]);
  activeProjectId = $state<string | null>(null);
  loading = $state(false);

  get activeProject(): ProjectEntry | undefined {
    return this.projects.find((p) => p.id === this.activeProjectId);
  }

  async loadProjects() {
    const client = connectionStore.client;
    if (!client) return;

    this.loading = true;
    try {
      this.projects = await client.listProjects();
    } finally {
      this.loading = false;
    }
  }

  async registerProject(path: string) {
    const client = connectionStore.client;
    if (!client) return;

    const entry = await client.registerProject(path);
    this.projects = [...this.projects, entry];
    return entry;
  }

  async unregisterProject(id: string) {
    const client = connectionStore.client;
    if (!client) return;

    await client.unregisterProject(id);
    this.projects = this.projects.filter((p) => p.id !== id);
    if (this.activeProjectId === id) {
      this.activeProjectId = null;
    }
  }

  selectProject(id: string | null) {
    this.activeProjectId = id;
  }
}

export const projectsStore = new ProjectsStore();
```

- [ ] **Step 4: Create ConnectionStatus component**

```svelte
<!-- src/lib/components/shared/ConnectionStatus.svelte -->
<script lang="ts">
  import { connectionStore } from "$lib/stores/connection.svelte";

  const statusColors = {
    disconnected: "bg-red-500",
    connecting: "bg-yellow-500 animate-pulse",
    connected: "bg-green-500",
  };
</script>

<div class="flex items-center gap-2 text-sm">
  <span
    class="w-2.5 h-2.5 rounded-full {statusColors[connectionStore.status]}"
  ></span>
  <span class="text-gray-500 dark:text-gray-400">
    {#if connectionStore.status === "connected"}
      v{connectionStore.daemonVersion}
    {:else if connectionStore.status === "connecting"}
      Connecting...
    {:else}
      Disconnected
    {/if}
  </span>
  {#if connectionStore.error}
    <span class="text-red-500 text-xs">{connectionStore.error}</span>
  {/if}
</div>
```

- [ ] **Step 5: Update App.svelte with connection logic and header**

```svelte
<!-- src/App.svelte -->
<script lang="ts">
  import { onMount } from "svelte";
  import ConnectionStatus from "$lib/components/shared/ConnectionStatus.svelte";
  import { connectionStore } from "$lib/stores/connection.svelte";
  import { configStore } from "$lib/stores/config.svelte";
  import { projectsStore } from "$lib/stores/projects.svelte";

  let currentModule = $state("settings");

  const modules = [
    { id: "settings", label: "S", title: "Settings" },
    { id: "plugins", label: "P", title: "Plugins" },
    { id: "skills", label: "K", title: "Skills" },
    { id: "memory", label: "M", title: "Memory" },
    { id: "mcp", label: "C", title: "MCP Servers" },
    { id: "effective", label: "E", title: "Effective Config" },
    { id: "launcher", label: "L", title: "Launcher" },
  ];

  onMount(async () => {
    // In development, connect directly to daemon
    // In production Tauri app, get URL/token from Tauri commands
    const baseUrl = "http://127.0.0.1:7890";
    const token = "dev-token"; // Will be replaced with Tauri IPC call

    await connectionStore.connect(baseUrl, token);
    if (connectionStore.status === "connected") {
      await configStore.loadUserConfig();
      await projectsStore.loadProjects();

      // Subscribe to config changes via WebSocket
      connectionStore.wsClient?.onEvent((event) => {
        if (event.event === "config_changed" && event.scope === "user") {
          configStore.loadUserConfig();
        }
      });
    }
  });
</script>

<div class="flex flex-col h-screen bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100">
  <!-- Header -->
  <header class="h-12 border-b border-gray-200 dark:border-gray-700 flex items-center px-4 justify-between shrink-0">
    <span class="font-semibold text-sm">dot-claude-gui</span>
    <div class="flex items-center gap-4">
      {#if projectsStore.projects.length > 0}
        <select
          class="text-sm bg-transparent border border-gray-300 dark:border-gray-600 rounded px-2 py-1"
          onchange={(e) => projectsStore.selectProject((e.target as HTMLSelectElement).value || null)}
        >
          <option value="">No project selected</option>
          {#each projectsStore.projects as project}
            <option value={project.id}>{project.name}</option>
          {/each}
        </select>
      {/if}
      <ConnectionStatus />
    </div>
  </header>

  <div class="flex flex-1 overflow-hidden">
    <!-- Sidebar -->
    <aside class="w-14 bg-gray-50 dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 flex flex-col items-center py-3 gap-1.5 shrink-0">
      {#each modules as mod}
        <button
          class="w-10 h-10 rounded-lg flex items-center justify-center text-sm font-medium transition-colors hover:bg-gray-200 dark:hover:bg-gray-700"
          class:bg-blue-100={currentModule === mod.id}
          class:dark:bg-blue-900={currentModule === mod.id}
          class:text-blue-700={currentModule === mod.id}
          class:dark:text-blue-300={currentModule === mod.id}
          onclick={() => (currentModule = mod.id)}
          title={mod.title}
        >
          {mod.label}
        </button>
      {/each}

      <div class="flex-1"></div>

      <button
        class="w-10 h-10 rounded-lg flex items-center justify-center text-sm font-medium transition-colors hover:bg-gray-200 dark:hover:bg-gray-700"
        class:bg-blue-100={currentModule === "appsettings"}
        class:dark:bg-blue-900={currentModule === "appsettings"}
        onclick={() => (currentModule = "appsettings")}
        title="App Settings"
      >
        A
      </button>
    </aside>

    <!-- Sub panel -->
    <div class="w-60 border-r border-gray-200 dark:border-gray-700 p-4 overflow-auto shrink-0">
      <h2 class="text-xs font-semibold uppercase tracking-wider text-gray-400 mb-3">
        {modules.find((m) => m.id === currentModule)?.title ?? "App Settings"}
      </h2>

      {#if currentModule === "settings"}
        <nav class="space-y-1">
          {#each ["General", "Permissions", "Hooks", "Sandbox", "Environment", "Status Line"] as item}
            <button class="w-full text-left px-3 py-2 rounded text-sm hover:bg-gray-100 dark:hover:bg-gray-700">
              {item}
            </button>
          {/each}
        </nav>
      {:else}
        <p class="text-sm text-gray-400">Coming in Phase 2-4</p>
      {/if}
    </div>

    <!-- Detail panel -->
    <main class="flex-1 p-6 overflow-auto">
      {#if configStore.loading}
        <p class="text-gray-400">Loading...</p>
      {:else if connectionStore.status !== "connected"}
        <div class="flex items-center justify-center h-full">
          <div class="text-center">
            <p class="text-lg text-gray-400 mb-2">Not connected to daemon</p>
            <p class="text-sm text-gray-500">Start claude-daemon and refresh</p>
          </div>
        </div>
      {:else}
        <h1 class="text-lg font-semibold mb-4">
          {modules.find((m) => m.id === currentModule)?.title ?? "App Settings"}
        </h1>

        {#if currentModule === "settings"}
          <div class="bg-gray-50 dark:bg-gray-800 rounded-lg p-4 font-mono text-xs overflow-auto max-h-[600px]">
            <pre>{JSON.stringify(configStore.userSettings, null, 2)}</pre>
          </div>
        {:else}
          <p class="text-gray-500">Module "{currentModule}" will be implemented in Phase 2-4.</p>
        {/if}
      {/if}
    </main>
  </div>
</div>
```

- [ ] **Step 6: Commit**

```bash
git add src/
git commit -m "feat: add Svelte stores, API client integration, and connected three-panel UI"
```

---

### Task 13: Integration Test — Daemon + Frontend End-to-End

**Files:**
- Create: `tests/integration/daemon_api_test.rs` (conceptual — actual location in `crates/claude-daemon/tests/`)
- Create: `crates/claude-daemon/tests/api_test.rs`

- [ ] **Step 1: Write daemon API integration test**

```rust
// crates/claude-daemon/tests/api_test.rs
use reqwest::Client;
use serde_json::json;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::time::{sleep, Duration};

async fn start_test_daemon() -> (TempDir, String, u16, tokio::task::JoinHandle<()>) {
    let dir = TempDir::new().unwrap();
    let claude_home = dir.path().to_path_buf();

    // Write a minimal settings.json
    std::fs::write(
        claude_home.join("settings.json"),
        r#"{"language": "en-US", "permissions": {"defaultMode": "plan"}}"#,
    )
    .unwrap();

    let token = "test-token-12345".to_string();
    let port = portpicker::pick_unused_port().unwrap();

    let state =
        claude_daemon::state::AppState::new(claude_home.clone(), token.clone());
    state.load_user_settings().await.unwrap();

    let app = claude_daemon::server::create_router(state);
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Wait for server to start
    sleep(Duration::from_millis(100)).await;

    (dir, token, port, handle)
}

#[tokio::test]
async fn health_endpoint_works() {
    let (_dir, _token, port, _handle) = start_test_daemon().await;
    let client = Client::new();

    let res = client
        .get(format!("http://127.0.0.1:{}/api/v1/health", port))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn config_requires_auth() {
    let (_dir, _token, port, _handle) = start_test_daemon().await;
    let client = Client::new();

    let res = client
        .get(format!("http://127.0.0.1:{}/api/v1/config/user", port))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn get_user_config_with_auth() {
    let (_dir, token, port, _handle) = start_test_daemon().await;
    let client = Client::new();

    let res = client
        .get(format!("http://127.0.0.1:{}/api/v1/config/user", port))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["settings"]["language"], "en-US");
    assert_eq!(body["scope"], "user");
}

#[tokio::test]
async fn update_user_config() {
    let (dir, token, port, _handle) = start_test_daemon().await;
    let client = Client::new();
    let base = format!("http://127.0.0.1:{}", port);

    // Update language
    let res = client
        .put(format!("{}/api/v1/config/user", base))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "settings": {
                "language": "zh-CN"
            }
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["settings"]["language"], "zh-CN");

    // Verify file was written
    let content = std::fs::read_to_string(dir.path().join("settings.json")).unwrap();
    let saved: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(saved["language"], "zh-CN");
}

#[tokio::test]
async fn update_config_with_invalid_mode_returns_422() {
    let (_dir, token, port, _handle) = start_test_daemon().await;
    let client = Client::new();

    let res = client
        .put(format!("http://127.0.0.1:{}/api/v1/config/user", port))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "settings": {
                "permissions": {
                    "defaultMode": "invalid_mode"
                }
            }
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 422);
}

#[tokio::test]
async fn project_crud() {
    let (_dir, token, port, _handle) = start_test_daemon().await;
    let client = Client::new();
    let base = format!("http://127.0.0.1:{}", port);

    // List projects (empty)
    let res = client
        .get(format!("{}/api/v1/projects", base))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();
    let projects: Vec<serde_json::Value> = res.json().await.unwrap();
    assert!(projects.is_empty());

    // Register project
    let res = client
        .post(format!("{}/api/v1/projects", base))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({ "path": "/tmp" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
    let project: serde_json::Value = res.json().await.unwrap();
    let project_id = project["id"].as_str().unwrap().to_string();

    // List projects (1 entry)
    let res = client
        .get(format!("{}/api/v1/projects", base))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();
    let projects: Vec<serde_json::Value> = res.json().await.unwrap();
    assert_eq!(projects.len(), 1);

    // Unregister
    let res = client
        .delete(format!("{}/api/v1/projects/{}", base, project_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);
}
```

- [ ] **Step 2: Add portpicker dev dependency**

Add to `crates/claude-daemon/Cargo.toml` under `[dev-dependencies]`:
```toml
portpicker = "0.1"
```

Make the state and server modules public for integration tests. Add to `crates/claude-daemon/src/main.rs` or create a `lib.rs`:

Create `crates/claude-daemon/src/lib.rs`:
```rust
pub mod api;
pub mod auth;
pub mod server;
pub mod state;
pub mod watcher;
```

Update `crates/claude-daemon/src/main.rs` to use lib:
```rust
use claude_daemon::{server, state, watcher};
// ... rest of main unchanged, just use claude_daemon:: prefix
```

- [ ] **Step 3: Run integration tests**

Run: `cargo test -p claude-daemon -- --test-threads=1`
Expected: All 6 integration tests pass

- [ ] **Step 4: Commit**

```bash
git add crates/claude-daemon/
git commit -m "test: add daemon API integration tests for config CRUD and projects"
```

---

### Task 14: Full Build Verification + Final Cleanup

- [ ] **Step 1: Run all tests across workspace**

```bash
cargo test --workspace
```

Expected: All tests pass across `claude-types`, `claude-config`, `claude-daemon`

- [ ] **Step 2: Verify frontend builds**

```bash
pnpm build
```

Expected: Vite outputs to `dist/`

- [ ] **Step 3: Verify Tauri compiles**

```bash
cargo build -p dot-claude-gui
```

Expected: Tauri app binary compiles

- [ ] **Step 4: Add .gitignore**

```gitignore
# .gitignore
/target
/dist
/node_modules
*.DS_Store

# Tauri
src-tauri/target

# IDE
.idea
.vscode
*.swp
```

- [ ] **Step 5: Update README.md**

```markdown
# dot-claude-gui

> Your `.claude/` all-in-one manager GUI

A desktop application for managing Claude Code configuration, plugins, skills, memory, and MCP servers.

## Architecture

- **Frontend:** Svelte 5 + TypeScript + Tailwind CSS 4
- **GUI Shell:** Tauri 2.0
- **Backend Daemon:** Rust (axum + tokio + notify)

## Development

### Prerequisites

- Rust (latest stable)
- Node.js 20+
- pnpm

### Setup

```bash
pnpm install
```

### Run daemon (standalone)

```bash
cargo run -p claude-daemon -- --port 7890
```

### Run Tauri dev

```bash
pnpm tauri dev
```

### Run tests

```bash
cargo test --workspace
```

## Project Structure

See [Design Spec](docs/superpowers/specs/2026-03-30-dot-claude-gui-design.md) for full architecture details.
```

- [ ] **Step 6: Final commit**

```bash
git add .gitignore README.md
git commit -m "chore: add gitignore and update README with dev instructions"
```

---

## Phase 1 Completion Checklist

At the end of Phase 1, you should have:

- [ ] Cargo workspace with 4 crates compiling cleanly
- [ ] `claude-types`: Settings schema with roundtrip tests (unknown field preservation)
- [ ] `claude-config`: Parse, write (atomic), merge engine, validation, file watcher
- [ ] `claude-daemon`: Health, config CRUD, project registration, WebSocket, file watcher integration
- [ ] Tauri 2 app shell launching a window
- [ ] Svelte 5 frontend with three-panel layout
- [ ] Frontend connects to daemon, displays user config as JSON
- [ ] WebSocket connected with auto-reconnect
- [ ] Header with project selector and connection status
- [ ] All Rust tests passing (`cargo test --workspace`)
- [ ] Frontend building (`pnpm build`)

**What comes next:**
- **Phase 2:** Settings Editor module — form-based sub-editors for each settings section
- **Phase 3:** Plugins (marketplace + install + per-project), Skills, Memory modules
- **Phase 4:** MCP Servers, Effective Config viewer, Project Launcher, App Settings
