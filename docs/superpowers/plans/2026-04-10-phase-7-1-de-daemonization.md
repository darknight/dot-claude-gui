# Phase 7.1: De-daemonization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Delete the `claude-daemon` sidecar crate and replace it with in-process Tauri IPC commands, preserving behavior of every existing module.

**Architecture:** Move the daemon's `AppState`, `executor`, `watcher`, and all 27 route handlers into `src-tauri/src/`. Use Tauri's managed state (`State<'_, AppState>`) instead of `Extension<AppState>`. Use `AppHandle::emit()` instead of a `broadcast::Sender<WsEvent>` channel. The frontend replaces `src/lib/api/client.ts` (HTTP) with `src/lib/ipc/client.ts` (invoke) and `src/lib/api/ws.ts` (WebSocket) with `src/lib/ipc/events.ts` (Tauri `listen`).

**Tech Stack:** Tauri 2.0, `tauri-plugin-shell` (retained for `launch` command), Svelte 5 runes, TypeScript, `@tauri-apps/api/core` + `@tauri-apps/api/event`, Rust (`tokio`, `notify`, `claude-config`, `claude-types`)

**Spec:** `docs/superpowers/specs/2026-04-10-plan-d-account-workspace-design.md` § Phase 7.1

---

## Context

The current `dot-claude-gui` architecture runs a Rust axum daemon (`crates/claude-daemon/`) as a Tauri sidecar, exposing 27 REST endpoints and a WebSocket on a picked TCP port with a Bearer token. The frontend talks to it via `DaemonClient` (REST) and `DaemonWsClient` (WS), with a multi-connection model to support local sidecar + remote daemons.

Plan D (the spec above) pivots the app to an **account-centric workspace manager**. The multi-daemon / remote-management model no longer matches user needs — the user wants a GUI to manage two local Claude Code accounts with shared plugins, not a daemon fleet. Phase 7.1 is the first step and the foundation for everything after: **delete the daemon entirely**, move its business logic into in-process Tauri commands, and rewire the frontend to use `@tauri-apps/api/core`'s `invoke()` instead of HTTP.

Per the spec: "no user-visible changes; internal rewire from daemon to in-process Tauri commands. Risk: low. Mechanical conversion with functional parity." This phase must end with all existing modules (Settings, Plugins, Skills, Memory, MCP, Effective Config, Launcher, App Settings, CLAUDE.md editor) working identically. **No data migration** — the app still reads `~/.claude/`. Account isolation arrives in Phase 7.2.

---

## Transformation Rules

Every endpoint → command conversion follows these mechanical rules. Reference this table while porting handlers.

### Rust signature mapping

| axum handler | Tauri command |
|---|---|
| `async fn handler(Extension(state): Extension<AppState>) -> Json<T>` | `#[tauri::command] async fn handler(state: State<'_, AppState>) -> Result<T, String>` |
| `Path(id): Path<String>` | `id: String` (plain parameter) |
| `Json(body): Json<Req>` | `body: Req` (plain parameter — Tauri deserializes from IPC) |
| `Query(q): Query<Q>` | `q: Q` or individual query fields as parameters |
| `Result<T, (StatusCode, Json<ErrorResponse>)>` | `Result<T, String>` |
| `Result<(StatusCode, Json<T>), ...>` | `Result<T, String>` (status code discarded) |
| `StatusCode::NO_CONTENT` return | `Ok(())` return |
| `state.inner.X.read().await` | `state.inner.X.read().await` (unchanged) |
| `state.broadcast(WsEvent::ConfigChanged(...))` | `app.emit("config-changed", payload)?` |
| `Extension(state)` closure binding | Tauri State handles this automatically |

### Error mapping

The daemon returns `(StatusCode, Json<ErrorResponse>)`. Tauri returns `Result<T, String>`. Map as follows:

- `400 INVALID_PATH` → `"invalid_path: Path does not exist: {path}"`
- `404 *_NOT_FOUND` → `"not_found: {original message}"`
- `409 *` → `"conflict: {original message}"`
- `500` → `"internal: {original message}"`

The frontend parses the prefix if it needs to differentiate, but usually just displays the message via the toast store. **Do not** construct a rich error type — a string error keeps the conversion mechanical.

### Event naming

Replace camelCase `WsEvent` variant tags with kebab-case Tauri event names:

| Daemon WsEvent | Tauri event name | Payload type (Rust) |
|---|---|---|
| `Connected { daemon_version }` | (removed — no connection concept) | — |
| `ConfigChanged { settings, source? }` | `config-changed` | `ConfigChangedPayload { settings: Settings, source: Option<String> }` |
| `ValidationError { errors }` | `validation-error` | `ValidationErrorPayload { errors: Vec<WsValidationError> }` |
| `CommandOutput { command_id, line, stream }` | `command-output` | `CommandOutputPayload { command_id: String, line: String, stream: CommandStream }` |
| `CommandCompleted { command_id, exit_code }` | `command-completed` | `CommandCompletedPayload { command_id: String, exit_code: i32 }` |

### Command naming

Daemon route paths become snake_case Rust function names and snake_case JS invoke strings (Tauri 2 uses snake_case by default; do NOT enable the `camelCase` conversion). The frontend client methods remain camelCase.

| REST path | Rust command | JS call |
|---|---|---|
| `GET /api/v1/health` | `health` | `invoke('health')` |
| `GET /api/v1/config/user` | `get_user_config` | `invoke('get_user_config')` |
| `PUT /api/v1/config/user` | `update_user_config` | `invoke('update_user_config', { settings })` |
| `GET /api/v1/config/project/{id}` | `get_project_config` | `invoke('get_project_config', { projectId })` |
| `PUT /api/v1/config/project/{id}` | `update_project_config` | `invoke('update_project_config', { projectId, settings })` |
| `GET /api/v1/config/effective/{id}` | `get_effective_config` | `invoke('get_effective_config', { projectId })` |
| `GET /api/v1/projects` | `list_projects` | `invoke('list_projects')` |
| `POST /api/v1/projects` | `register_project` | `invoke('register_project', { path })` |
| `DELETE /api/v1/projects/{id}` | `unregister_project` | `invoke('unregister_project', { id })` |
| `GET /api/v1/plugins` | `list_plugins` | `invoke('list_plugins')` |
| `POST /api/v1/plugins/{id}/toggle` | `toggle_plugin` | `invoke('toggle_plugin', { id, enabled })` |
| `POST /api/v1/plugins/install` | `install_plugin` | `invoke('install_plugin', { name, marketplace })` |
| `POST /api/v1/plugins/{id}/uninstall` | `uninstall_plugin` | `invoke('uninstall_plugin', { id })` |
| `GET /api/v1/marketplaces` | `list_marketplaces` | `invoke('list_marketplaces')` |
| `GET /api/v1/marketplaces/{id}/plugins` | `get_marketplace_plugins` | `invoke('get_marketplace_plugins', { marketplaceId })` |
| `POST /api/v1/marketplaces` | `add_marketplace` | `invoke('add_marketplace', { repo })` |
| `DELETE /api/v1/marketplaces/{id}` | `remove_marketplace` | `invoke('remove_marketplace', { id })` |
| `GET /api/v1/mcp/servers` | `list_mcp_servers` | `invoke('list_mcp_servers')` |
| `POST /api/v1/mcp/servers` | `add_mcp_server` | `invoke('add_mcp_server', { req })` |
| `DELETE /api/v1/mcp/servers/{name}` | `remove_mcp_server` | `invoke('remove_mcp_server', { name, scope })` |
| `GET /api/v1/skills` | `list_skills` | `invoke('list_skills')` |
| `GET /api/v1/skills/{id}/content` | `get_skill_content` | `invoke('get_skill_content', { id })` |
| `GET /api/v1/claudemd` | `list_claudemd_files` | `invoke('list_claudemd_files')` |
| `GET /api/v1/claudemd/{id}` | `get_claudemd_file` | `invoke('get_claudemd_file', { id })` |
| `PUT /api/v1/claudemd/{id}` | `update_claudemd_file` | `invoke('update_claudemd_file', { id, content })` |
| `DELETE /api/v1/claudemd/{id}` | `delete_claudemd_file` | `invoke('delete_claudemd_file', { id })` |
| `GET /api/v1/memory` | `list_memory_projects` | `invoke('list_memory_projects')` |
| `GET /api/v1/memory/{project_id}` | `list_memory_files` | `invoke('list_memory_files', { projectId })` |
| `GET /api/v1/memory/{project_id}/{filename}` | `get_memory_file` | `invoke('get_memory_file', { projectId, filename })` |
| `PUT /api/v1/memory/{project_id}/{filename}` | `update_memory_file` | `invoke('update_memory_file', { projectId, filename, content })` |
| `DELETE /api/v1/memory/{project_id}/{filename}` | `delete_memory_file` | `invoke('delete_memory_file', { projectId, filename })` |
| `POST /api/v1/launch` | `launch_claude` | `invoke('launch_claude', { req })` |

**Total: 1 health + 5 config + 3 projects + 8 plugins + 3 mcp + 2 skills + 4 claudemd + 5 memory + 1 launcher = 32 commands.** (Note: the `/plugins` + `/marketplaces` prefix of 8 in the daemon inventory breaks down into these individual commands.)

---

## File Structure

### New files

| File | Responsibility |
|------|----------------|
| `src-tauri/src/state.rs` | Port of daemon `AppState` / `AppStateInner` / `ProjectInfo`; drops `auth_token` and `ws_tx` fields |
| `src-tauri/src/events.rs` | Payload types for Tauri events; re-exports `WsValidationError`, `CommandStream` from `claude_types` |
| `src-tauri/src/executor.rs` | Port of daemon `executor.rs` — spawns CLI subprocesses, streams stdout/stderr lines via `app.emit("command-output", ...)`, emits `command-completed` on exit |
| `src-tauri/src/watcher.rs` | Port of daemon `watcher.rs` — uses `notify` + `AppHandle::emit` to emit `config-changed` / `validation-error` |
| `src-tauri/src/commands/mod.rs` | Command module declarations |
| `src-tauri/src/commands/health.rs` | 1 command (`health`) |
| `src-tauri/src/commands/config.rs` | 5 commands |
| `src-tauri/src/commands/projects.rs` | 3 commands |
| `src-tauri/src/commands/plugins.rs` | 8 commands |
| `src-tauri/src/commands/mcp.rs` | 3 commands |
| `src-tauri/src/commands/skills.rs` | 2 commands |
| `src-tauri/src/commands/claudemd.rs` | 4 commands |
| `src-tauri/src/commands/memory.rs` | 5 commands |
| `src-tauri/src/commands/launcher.rs` | 1 command |
| `src/lib/ipc/client.ts` | Thin wrapper around `invoke` exposing the same surface as `DaemonClient` |
| `src/lib/ipc/events.ts` | `listen` helpers for `config-changed`, `validation-error`, `command-output`, `command-completed` |

### Modified files

| File | Change |
|------|--------|
| `src-tauri/Cargo.toml` | Add `claude-types`, `claude-config`, `notify`, `anyhow`, `uuid`, `futures-util`; remove nothing (sidecar deps already minimal) |
| `src-tauri/src/lib.rs` | Register 32 commands; delete `start_sidecar` + port/token/health logic; insert `AppState` via `.manage()`; spawn watcher on setup |
| `src-tauri/tauri.conf.json` | Remove `externalBin: ["binaries/claude-daemon"]` (`bundle.externalBin` field) |
| `src-tauri/capabilities/default.json` | Remove `shell:allow-spawn` (keep `shell:allow-execute` if still needed for `launch_claude`) |
| `src/lib/stores/config.svelte.ts` | Replace `connectionStore.client.X()` with `ipcClient.X()` |
| `src/lib/stores/projects.svelte.ts` | Same pattern |
| `src/lib/stores/plugins.svelte.ts` | Same pattern |
| `src/lib/stores/skills.svelte.ts` | Same pattern |
| `src/lib/stores/memory.svelte.ts` | Same pattern |
| `src/lib/stores/mcp.svelte.ts` | Same pattern |
| `src/lib/stores/claudemd.svelte.ts` | Same pattern |
| `src/lib/api/types.ts` | No change to type definitions; stays as the single source of truth for shared types (imported by `ipc/client.ts` and stores) |
| `src/App.svelte` | Remove `connectionStore.connect()` orchestration; subscribe to `config-changed` via Tauri `listen`; remove `<EnvironmentSelector>` from header; data loads trigger directly on mount |
| `src/lib/components/appsettings/AppSettingsView.svelte` | Remove `<ConnectionsPanel>` from render tree |
| `Cargo.toml` (workspace root) | Remove `crates/claude-daemon` from `members`; remove workspace `portpicker`, `reqwest`, `axum`, `tower*`, `tokio-tungstenite` entries if they have no other consumers |
| `CLAUDE.md` | Remove "Multi-Daemon Connections" section; update "Commands" section (delete daemon build steps); delete "Rust workspace crates" line for claude-daemon; update "Three layers" architecture description |
| `package.json` | No change — `@tauri-apps/api/core` and `@tauri-apps/api/event` are already available via `@tauri-apps/api` package |

### Deleted files

| File | Reason |
|------|--------|
| `crates/claude-daemon/` (entire crate) | Replaced by in-process Tauri commands |
| `src-tauri/binaries/claude-daemon-*` | Sidecar binary no longer needed |
| `src/lib/api/client.ts` | Replaced by `src/lib/ipc/client.ts` |
| `src/lib/api/ws.ts` | Replaced by `src/lib/ipc/events.ts` |
| `src/lib/stores/connection.svelte.ts` | No connection lifecycle — IPC is always available |
| `src/lib/stores/connections.svelte.ts` | No multi-daemon model |
| `src/lib/components/shared/EnvironmentSelector.svelte` | No environments to select |
| `src/lib/components/appsettings/ConnectionsPanel.svelte` | No connections to manage |
| `~/.dot-claude-gui/connections.json` (runtime state) | Stale, but safe to leave on user disk; no code reads it after this phase |

---

## Stage A: Backend foundation

### Task 1: Scaffold backend module structure

**Files:**
- Create: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/state.rs` (stub)
- Create: `src-tauri/src/events.rs` (stub)
- Create: `src-tauri/src/executor.rs` (stub)
- Create: `src-tauri/src/watcher.rs` (stub)
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add workspace crate dependencies**

Add to `src-tauri/Cargo.toml` under `[dependencies]`:

```toml
claude-types = { path = "../crates/claude-types" }
claude-config = { path = "../crates/claude-config" }
notify = "8"
anyhow = "1"
uuid = { version = "1", features = ["v4"] }
futures-util = "0.3"
```

- [ ] **Step 2: Create stub module files**

```rust
// src-tauri/src/state.rs
// Stub — populated in Task 2
```

```rust
// src-tauri/src/events.rs
// Stub — populated in Task 12
```

```rust
// src-tauri/src/executor.rs
// Stub — populated in Task 12
```

```rust
// src-tauri/src/watcher.rs
// Stub — populated in Task 13
```

```rust
// src-tauri/src/commands/mod.rs
pub mod health;
pub mod config;
pub mod projects;
pub mod plugins;
pub mod mcp;
pub mod skills;
pub mod claudemd;
pub mod memory;
pub mod launcher;
```

Also create empty stub files for each command module so `mod.rs` compiles:

```rust
// src-tauri/src/commands/health.rs
// Stub — populated in Task 3
```

Create identical one-line stub files for: `config.rs`, `projects.rs`, `plugins.rs`, `mcp.rs`, `skills.rs`, `claudemd.rs`, `memory.rs`, `launcher.rs`.

- [ ] **Step 3: Declare modules in lib.rs**

Add to the top of `src-tauri/src/lib.rs`:

```rust
mod commands;
mod events;
mod executor;
mod state;
mod watcher;
```

- [ ] **Step 4: Verify it compiles**

```bash
cargo check -p dot-claude-gui
```

Expected: compiles with unused-module warnings (acceptable at this stage).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/
git commit -m "refactor(tauri): scaffold commands and state modules"
```

---

### Task 2: Port AppState into src-tauri

**Files:**
- Modify: `src-tauri/src/state.rs`
- Reference: `crates/claude-daemon/src/state.rs`

- [ ] **Step 1: Write the failing test**

```rust
// src-tauri/src/state.rs

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use claude_config::parse::read_settings;
use claude_types::Settings;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub id: String,
    pub path: PathBuf,
    pub name: String,
}

pub struct AppStateInner {
    pub claude_home: PathBuf,
    pub user_settings: RwLock<Settings>,
    pub project_settings: RwLock<HashMap<String, Settings>>,
    pub local_settings: RwLock<HashMap<String, Settings>>,
    pub projects: RwLock<Vec<ProjectInfo>>,
    pub started_at: std::time::Instant,
}

#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<AppStateInner>,
}

impl AppState {
    pub fn new(claude_home: PathBuf) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                claude_home,
                user_settings: RwLock::new(Settings::default()),
                project_settings: RwLock::new(HashMap::new()),
                local_settings: RwLock::new(HashMap::new()),
                projects: RwLock::new(Vec::new()),
                started_at: std::time::Instant::now(),
            }),
        }
    }

    pub async fn load_user_settings(&self) -> Result<()> {
        let settings_path = self.inner.claude_home.join("settings.json");
        let settings = read_settings(&settings_path)?;
        *self.inner.user_settings.write().await = settings;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn app_state_new_starts_empty() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());

        assert_eq!(state.inner.claude_home, dir.path());
        assert!(state.inner.user_settings.read().await.env.is_empty());
        assert!(state.inner.project_settings.read().await.is_empty());
        assert!(state.inner.projects.read().await.is_empty());
    }

    #[tokio::test]
    async fn app_state_loads_user_settings_from_disk() {
        let dir = tempdir().unwrap();
        let settings_path = dir.path().join("settings.json");
        std::fs::write(
            &settings_path,
            r#"{"env": {"FOO": "bar"}}"#,
        )
        .unwrap();

        let state = AppState::new(dir.path().to_path_buf());
        state.load_user_settings().await.unwrap();

        let loaded = state.inner.user_settings.read().await;
        assert_eq!(loaded.env.get("FOO").map(String::as_str), Some("bar"));
    }
}
```

Add `tempfile = "3"` to `src-tauri/Cargo.toml` under `[dev-dependencies]` if not already present.

- [ ] **Step 2: Run the tests**

```bash
cargo test -p dot-claude-gui state::tests
```

Expected: PASS (both tests).

If the `Settings` struct field names differ (e.g., `env` vs `environment`), adjust the test accordingly. The daemon's state.rs line 43 uses `Settings::default()`, so whatever fields are default there should also be defaults here.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/state.rs src-tauri/Cargo.toml
git commit -m "feat(tauri): port AppState from daemon crate"
```

---

### Task 3: Port health command (simplest, proves the pattern)

**Files:**
- Modify: `src-tauri/src/commands/health.rs`
- Modify: `src-tauri/src/lib.rs`
- Reference: `crates/claude-daemon/src/api/health.rs`

- [ ] **Step 1: Write the failing test**

```rust
// src-tauri/src/commands/health.rs

use claude_types::api::HealthResponse;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn health(state: State<'_, AppState>) -> Result<HealthResponse, String> {
    let uptime = state.inner.started_at.elapsed().as_secs();
    Ok(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: Some(uptime),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn health_returns_ok_with_version_and_uptime() {
        let state = AppState::new(PathBuf::from("/tmp"));
        // Tauri State is a tauri::State<'_, T> wrapper; for unit tests we invoke
        // the handler logic via a direct inner-state approach. The cleanest
        // pattern is to factor the logic into a helper or just duplicate it
        // here — we choose to call the raw logic.
        let uptime = state.inner.started_at.elapsed().as_secs();
        let resp = HealthResponse {
            status: "ok".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: Some(uptime),
        };

        assert_eq!(resp.status, "ok");
        assert!(!resp.version.is_empty());
        assert!(resp.uptime_seconds.is_some());
    }
}
```

Note: **Tauri `State<'_, T>` cannot be trivially constructed in unit tests.** The standard pattern is to either (a) extract logic into a pure helper function and test the helper, or (b) test via `tauri::test::mock_builder()`. For this migration, we take option (a): the `#[tauri::command]` wrapper is a 1-2 line shim that calls a pure async function. This keeps unit tests ergonomic. For commands that are trivial (like `health`), the test asserts the same values that the body produces, which is acceptable.

For more substantive commands in later tasks, structure as:

```rust
pub(crate) async fn do_something_logic(state: &AppState, arg: X) -> Result<Y, String> { ... }

#[tauri::command]
pub async fn do_something(state: State<'_, AppState>, arg: X) -> Result<Y, String> {
    do_something_logic(&state, arg).await
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn do_something_logic_works() {
        let state = AppState::new(tempdir.path().to_path_buf());
        let result = do_something_logic(&state, X::default()).await.unwrap();
        assert_eq!(result, expected);
    }
}
```

Adopt this pattern for every command from Task 4 onward.

- [ ] **Step 2: Register the command in lib.rs**

Edit `src-tauri/src/lib.rs` — in the `invoke_handler` macro call, add the first new command:

```rust
.invoke_handler(tauri::generate_handler![
    // existing commands...
    get_config_dir,
    read_connections,
    write_connections,
    read_app_config,
    write_app_config,
    // new: health
    commands::health::health,
])
```

- [ ] **Step 3: Insert AppState into Tauri managed state**

Still in `src-tauri/src/lib.rs`, locate the `setup` closure. At the top of the closure (before the existing sidecar spawn), add:

```rust
let claude_home = dirs_next::home_dir()
    .ok_or_else(|| "cannot determine home directory".to_string())?
    .join(".claude");
let app_state = crate::state::AppState::new(claude_home);
app.manage(app_state);
```

And before `.invoke_handler(...)`, if there isn't already a `.manage()` call, insert it via the setup closure (above is correct — `app.manage` in setup).

- [ ] **Step 4: Run tests and cargo check**

```bash
cargo test -p dot-claude-gui commands::health
cargo check -p dot-claude-gui
```

Expected: tests PASS, cargo check succeeds.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/health.rs src-tauri/src/lib.rs
git commit -m "feat(tauri): port health command to IPC"
```

---

### Task 4: Port config commands (5 commands)

**Files:**
- Modify: `src-tauri/src/commands/config.rs`
- Modify: `src-tauri/src/lib.rs`
- Reference: `crates/claude-daemon/src/api/config.rs`

- [ ] **Step 1: Write the commands with logic helpers**

```rust
// src-tauri/src/commands/config.rs

use claude_config::{merge, parse, validate, write};
use claude_types::api::{ConfigResponse, EffectiveConfig, UpdateConfigRequest};
use claude_types::Settings;
use tauri::State;

use crate::state::AppState;

// ---------- get_user_config ----------

pub(crate) async fn get_user_config_logic(state: &AppState) -> Result<ConfigResponse, String> {
    let settings = state.inner.user_settings.read().await.clone();
    Ok(ConfigResponse {
        settings,
        last_modified: None,
        version: None,
    })
}

#[tauri::command]
pub async fn get_user_config(state: State<'_, AppState>) -> Result<ConfigResponse, String> {
    get_user_config_logic(&state).await
}

// ---------- update_user_config ----------

pub(crate) async fn update_user_config_logic(
    state: &AppState,
    req: UpdateConfigRequest,
) -> Result<ConfigResponse, String> {
    let settings_path = state.inner.claude_home.join("settings.json");

    // Merge with existing on-disk settings, validate, atomically write.
    let existing = parse::read_settings(&settings_path).unwrap_or_default();
    let merged = merge::merge_settings(&existing, &req.settings);

    validate::validate_settings(&merged).map_err(|e| format!("validation: {e}"))?;

    write::write_settings_atomic(&settings_path, &merged)
        .map_err(|e| format!("write: {e}"))?;

    *state.inner.user_settings.write().await = merged.clone();

    Ok(ConfigResponse {
        settings: merged,
        last_modified: None,
        version: None,
    })
}

#[tauri::command]
pub async fn update_user_config(
    state: State<'_, AppState>,
    req: UpdateConfigRequest,
) -> Result<ConfigResponse, String> {
    update_user_config_logic(&state, req).await
}

// ---------- get_project_config ----------

pub(crate) async fn get_project_config_logic(
    state: &AppState,
    project_id: String,
) -> Result<ConfigResponse, String> {
    let projects = state.inner.projects.read().await;
    let project = projects
        .iter()
        .find(|p| p.id == project_id)
        .ok_or_else(|| format!("not_found: project '{project_id}' not found"))?;

    let settings_path = project.path.join(".claude").join("settings.json");
    let settings = parse::read_settings(&settings_path).unwrap_or_default();

    Ok(ConfigResponse {
        settings,
        last_modified: None,
        version: None,
    })
}

#[tauri::command]
pub async fn get_project_config(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<ConfigResponse, String> {
    get_project_config_logic(&state, project_id).await
}

// ---------- update_project_config ----------

pub(crate) async fn update_project_config_logic(
    state: &AppState,
    project_id: String,
    req: UpdateConfigRequest,
) -> Result<ConfigResponse, String> {
    let projects = state.inner.projects.read().await;
    let project = projects
        .iter()
        .find(|p| p.id == project_id)
        .ok_or_else(|| format!("not_found: project '{project_id}' not found"))?
        .clone();
    drop(projects);

    let settings_path = project.path.join(".claude").join("settings.json");
    let existing = parse::read_settings(&settings_path).unwrap_or_default();
    let merged = merge::merge_settings(&existing, &req.settings);

    validate::validate_settings(&merged).map_err(|e| format!("validation: {e}"))?;

    // Ensure .claude directory exists
    if let Some(parent) = settings_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir: {e}"))?;
    }

    write::write_settings_atomic(&settings_path, &merged)
        .map_err(|e| format!("write: {e}"))?;

    state
        .inner
        .project_settings
        .write()
        .await
        .insert(project_id, merged.clone());

    Ok(ConfigResponse {
        settings: merged,
        last_modified: None,
        version: None,
    })
}

#[tauri::command]
pub async fn update_project_config(
    state: State<'_, AppState>,
    project_id: String,
    req: UpdateConfigRequest,
) -> Result<ConfigResponse, String> {
    update_project_config_logic(&state, project_id, req).await
}

// ---------- get_effective_config ----------

pub(crate) async fn get_effective_config_logic(
    state: &AppState,
    project_id: String,
) -> Result<EffectiveConfig, String> {
    let user_settings = state.inner.user_settings.read().await.clone();

    let projects = state.inner.projects.read().await;
    let project = projects
        .iter()
        .find(|p| p.id == project_id)
        .ok_or_else(|| format!("not_found: project '{project_id}' not found"))?
        .clone();
    drop(projects);

    let project_settings_path = project.path.join(".claude").join("settings.json");
    let local_settings_path = project.path.join(".claude").join("settings.local.json");

    let project_settings = parse::read_settings(&project_settings_path).unwrap_or_default();
    let local_settings = parse::read_settings(&local_settings_path).unwrap_or_default();

    let (merged, sources) = merge::merge_with_sources(&[
        ("user", &user_settings),
        ("project", &project_settings),
        ("local", &local_settings),
    ]);

    Ok(EffectiveConfig {
        settings: merged,
        field_sources: sources,
    })
}

#[tauri::command]
pub async fn get_effective_config(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<EffectiveConfig, String> {
    get_effective_config_logic(&state, project_id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ProjectInfo;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[tokio::test]
    async fn get_user_config_returns_cached_settings() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());
        std::fs::write(
            dir.path().join("settings.json"),
            r#"{"env": {"KEY": "value"}}"#,
        )
        .unwrap();
        state.load_user_settings().await.unwrap();

        let resp = get_user_config_logic(&state).await.unwrap();
        assert_eq!(resp.settings.env.get("KEY").map(String::as_str), Some("value"));
    }

    #[tokio::test]
    async fn update_user_config_writes_to_disk() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());

        let mut new_settings = Settings::default();
        new_settings.env.insert("NEW".to_string(), "val".to_string());
        let req = UpdateConfigRequest { settings: new_settings };

        let resp = update_user_config_logic(&state, req).await.unwrap();

        assert_eq!(resp.settings.env.get("NEW").map(String::as_str), Some("val"));
        let on_disk = std::fs::read_to_string(dir.path().join("settings.json")).unwrap();
        assert!(on_disk.contains("NEW"));
    }

    #[tokio::test]
    async fn get_project_config_returns_not_found_for_unknown_project() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());

        let err = get_project_config_logic(&state, "unknown".to_string()).await.unwrap_err();
        assert!(err.starts_with("not_found:"));
    }

    #[tokio::test]
    async fn get_project_config_reads_project_settings() {
        let dir = tempdir().unwrap();
        let project_dir = dir.path().join("proj");
        std::fs::create_dir_all(project_dir.join(".claude")).unwrap();
        std::fs::write(
            project_dir.join(".claude").join("settings.json"),
            r#"{"env": {"PROJ": "yes"}}"#,
        )
        .unwrap();

        let state = AppState::new(dir.path().to_path_buf());
        state.inner.projects.write().await.push(ProjectInfo {
            id: "p1".to_string(),
            path: project_dir.clone(),
            name: "proj".to_string(),
        });

        let resp = get_project_config_logic(&state, "p1".to_string()).await.unwrap();
        assert_eq!(resp.settings.env.get("PROJ").map(String::as_str), Some("yes"));
    }
}
```

**Important:** The exact function names from `claude-config` (e.g. `merge::merge_settings`, `write::write_settings_atomic`, `merge::merge_with_sources`) must match what the crate actually exports. If the daemon's original `api/config.rs` uses different names, copy those. Verify by reading `crates/claude-daemon/src/api/config.rs` first and mirroring its imports and function calls exactly.

- [ ] **Step 2: Register commands in lib.rs**

Add to the `invoke_handler!` list:

```rust
commands::config::get_user_config,
commands::config::update_user_config,
commands::config::get_project_config,
commands::config::update_project_config,
commands::config::get_effective_config,
```

- [ ] **Step 3: Run tests**

```bash
cargo test -p dot-claude-gui commands::config
```

Expected: all 4 tests PASS.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/config.rs src-tauri/src/lib.rs
git commit -m "feat(tauri): port config commands to IPC"
```

---

### Task 5: Port projects commands (3 commands)

**Files:**
- Modify: `src-tauri/src/commands/projects.rs`
- Modify: `src-tauri/src/lib.rs`
- Reference: `crates/claude-daemon/src/api/projects.rs`

- [ ] **Step 1: Write the commands with logic helpers**

```rust
// src-tauri/src/commands/projects.rs

use claude_types::api::{ProjectEntry, RegisterProjectRequest};
use std::path::PathBuf;
use tauri::State;
use uuid::Uuid;

use crate::state::{AppState, ProjectInfo};

pub(crate) async fn list_projects_logic(state: &AppState) -> Result<Vec<ProjectEntry>, String> {
    let projects = state.inner.projects.read().await;
    Ok(projects
        .iter()
        .map(|p| ProjectEntry {
            id: p.id.clone(),
            name: p.name.clone(),
            path: p.path.to_string_lossy().to_string(),
            registered_at: None,
        })
        .collect())
}

#[tauri::command]
pub async fn list_projects(state: State<'_, AppState>) -> Result<Vec<ProjectEntry>, String> {
    list_projects_logic(&state).await
}

pub(crate) async fn register_project_logic(
    state: &AppState,
    req: RegisterProjectRequest,
) -> Result<ProjectEntry, String> {
    let path = PathBuf::from(&req.path);
    if !path.exists() {
        return Err(format!("invalid_path: Path does not exist: {}", req.path));
    }

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| req.path.clone());

    let id = Uuid::new_v4().to_string();

    state.inner.projects.write().await.push(ProjectInfo {
        id: id.clone(),
        path: path.clone(),
        name: name.clone(),
    });

    Ok(ProjectEntry {
        id,
        name,
        path: path.to_string_lossy().to_string(),
        registered_at: None,
    })
}

#[tauri::command]
pub async fn register_project(
    state: State<'_, AppState>,
    req: RegisterProjectRequest,
) -> Result<ProjectEntry, String> {
    register_project_logic(&state, req).await
}

pub(crate) async fn unregister_project_logic(
    state: &AppState,
    id: String,
) -> Result<(), String> {
    let mut projects = state.inner.projects.write().await;
    let before = projects.len();
    projects.retain(|p| p.id != id);
    if projects.len() == before {
        return Err(format!("not_found: project '{id}' not found"));
    }
    Ok(())
}

#[tauri::command]
pub async fn unregister_project(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    unregister_project_logic(&state, id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn list_projects_empty_by_default() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());
        let result = list_projects_logic(&state).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn register_project_adds_to_list() {
        let dir = tempdir().unwrap();
        let project_dir = dir.path().join("proj");
        std::fs::create_dir_all(&project_dir).unwrap();

        let state = AppState::new(dir.path().to_path_buf());
        let req = RegisterProjectRequest {
            path: project_dir.to_string_lossy().to_string(),
        };

        let entry = register_project_logic(&state, req).await.unwrap();
        assert_eq!(entry.name, "proj");
        assert!(!entry.id.is_empty());

        let listed = list_projects_logic(&state).await.unwrap();
        assert_eq!(listed.len(), 1);
    }

    #[tokio::test]
    async fn register_project_rejects_nonexistent_path() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());
        let req = RegisterProjectRequest {
            path: "/tmp/nonexistent-xyz-12345".to_string(),
        };
        let err = register_project_logic(&state, req).await.unwrap_err();
        assert!(err.starts_with("invalid_path:"));
    }

    #[tokio::test]
    async fn unregister_project_removes_it() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());
        state.inner.projects.write().await.push(ProjectInfo {
            id: "p1".to_string(),
            path: dir.path().to_path_buf(),
            name: "p1".to_string(),
        });

        unregister_project_logic(&state, "p1".to_string()).await.unwrap();

        let listed = list_projects_logic(&state).await.unwrap();
        assert!(listed.is_empty());
    }

    #[tokio::test]
    async fn unregister_project_returns_not_found_for_unknown_id() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());
        let err = unregister_project_logic(&state, "nope".to_string()).await.unwrap_err();
        assert!(err.starts_with("not_found:"));
    }
}
```

- [ ] **Step 2: Register commands in lib.rs**

Add:
```rust
commands::projects::list_projects,
commands::projects::register_project,
commands::projects::unregister_project,
```

- [ ] **Step 3: Run tests**

```bash
cargo test -p dot-claude-gui commands::projects
```

Expected: all 5 tests PASS.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/projects.rs src-tauri/src/lib.rs
git commit -m "feat(tauri): port projects commands to IPC"
```

---

### Task 6: Port skills commands (2 commands)

**Files:**
- Modify: `src-tauri/src/commands/skills.rs`
- Modify: `src-tauri/src/lib.rs`
- Reference: `crates/claude-daemon/src/api/skills.rs`

- [ ] **Step 1: Port the two handlers**

Open `crates/claude-daemon/src/api/skills.rs` and copy the logic. The two handlers are `list_skills` and `get_skill_content`. They scan `<claude_home>/skills/` and `<claude_home>/plugins/*/skills/` for `SKILL.md` files and parse frontmatter.

Mirror the structure:

```rust
// src-tauri/src/commands/skills.rs
// (structure identical to the daemon version — same imports, same path
// scanning, same frontmatter parsing. Only differences:
//   1. `Extension(state)` → `state: State<'_, AppState>`
//   2. Return `Result<T, String>` instead of `Result<Json<T>, (StatusCode, Json<ErrorResponse>)>`
//   3. Map errors to `"not_found:" / "internal:"` strings.)

use claude_types::api::{SkillContentResponse, SkillInfo};
use tauri::State;

use crate::state::AppState;

pub(crate) async fn list_skills_logic(state: &AppState) -> Result<Vec<SkillInfo>, String> {
    // Copy the body of daemon's list_skills handler here.
    // Replace any `state.inner.claude_home` accesses with the same.
    // Replace any error returns with `Err(format!("internal: {e}"))`.
    todo!("copy from crates/claude-daemon/src/api/skills.rs")
}

#[tauri::command]
pub async fn list_skills(state: State<'_, AppState>) -> Result<Vec<SkillInfo>, String> {
    list_skills_logic(&state).await
}

pub(crate) async fn get_skill_content_logic(
    state: &AppState,
    id: String,
) -> Result<SkillContentResponse, String> {
    // Copy the body of daemon's get_skill_content handler.
    todo!("copy from crates/claude-daemon/src/api/skills.rs")
}

#[tauri::command]
pub async fn get_skill_content(
    state: State<'_, AppState>,
    id: String,
) -> Result<SkillContentResponse, String> {
    get_skill_content_logic(&state, id).await
}
```

Replace each `todo!()` with the actual body from the daemon file. **Do not paraphrase — copy line by line** and only change the `Extension` → `state`, error mapping, and any `Json(...)` wrappers.

- [ ] **Step 2: Write tests mirroring daemon tests**

If `crates/claude-daemon/src/api/skills.rs` has `#[cfg(test)] mod tests`, copy those tests into the new file and adjust for the new signature. If the daemon file has no tests, write at minimum:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn list_skills_returns_empty_when_no_skills_dir() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());
        let result = list_skills_logic(&state).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn list_skills_finds_user_skill_with_frontmatter() {
        let dir = tempdir().unwrap();
        let skills_dir = dir.path().join("skills").join("my-skill");
        std::fs::create_dir_all(&skills_dir).unwrap();
        std::fs::write(
            skills_dir.join("SKILL.md"),
            "---\nname: my-skill\ndescription: test\n---\n\nBody",
        )
        .unwrap();

        let state = AppState::new(dir.path().to_path_buf());
        let result = list_skills_logic(&state).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "my-skill");
    }

    #[tokio::test]
    async fn get_skill_content_returns_not_found_for_unknown_id() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());
        let err = get_skill_content_logic(&state, "nope".to_string()).await.unwrap_err();
        assert!(err.starts_with("not_found:"));
    }
}
```

- [ ] **Step 3: Register commands**

```rust
commands::skills::list_skills,
commands::skills::get_skill_content,
```

- [ ] **Step 4: Run tests**

```bash
cargo test -p dot-claude-gui commands::skills
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/skills.rs src-tauri/src/lib.rs
git commit -m "feat(tauri): port skills commands to IPC"
```

---

### Task 7: Port claudemd commands (4 commands)

**Files:**
- Modify: `src-tauri/src/commands/claudemd.rs`
- Modify: `src-tauri/src/lib.rs`
- Reference: `crates/claude-daemon/src/api/claudemd.rs`

- [ ] **Step 1: Copy handlers with transformation**

Open `crates/claude-daemon/src/api/claudemd.rs` and copy the four handlers (`list_claudemd_files`, `get_claudemd_file`, `update_claudemd_file`, `delete_claudemd_file`) following the same pattern as Task 6:
- Each body becomes a `pub(crate) async fn X_logic(state: &AppState, ...)` helper
- A thin `#[tauri::command]` wrapper calls the helper
- Errors map to `"not_found:" / "invalid_path:" / "internal:"` strings
- The `id` parameter format ("global" or "project:{project_id}") is preserved verbatim

- [ ] **Step 2: Write tests**

Copy any existing daemon tests; otherwise write these minimum tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ProjectInfo;
    use tempfile::tempdir;

    #[tokio::test]
    async fn list_returns_global_entry_even_when_missing() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());
        let result = list_claudemd_files_logic(&state).await.unwrap();
        assert!(result.iter().any(|f| f.scope == "global"));
    }

    #[tokio::test]
    async fn get_global_returns_content_when_exists() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("CLAUDE.md"), "# Global").unwrap();
        let state = AppState::new(dir.path().to_path_buf());
        let detail = get_claudemd_file_logic(&state, "global".to_string()).await.unwrap();
        assert!(detail.content.contains("Global"));
    }

    #[tokio::test]
    async fn update_global_writes_file() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());
        update_claudemd_file_logic(&state, "global".to_string(), "# New".to_string())
            .await
            .unwrap();
        let on_disk = std::fs::read_to_string(dir.path().join("CLAUDE.md")).unwrap();
        assert_eq!(on_disk, "# New");
    }

    #[tokio::test]
    async fn delete_global_removes_file() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("CLAUDE.md"), "# Bye").unwrap();
        let state = AppState::new(dir.path().to_path_buf());
        delete_claudemd_file_logic(&state, "global".to_string()).await.unwrap();
        assert!(!dir.path().join("CLAUDE.md").exists());
    }
}
```

- [ ] **Step 3: Register commands**

```rust
commands::claudemd::list_claudemd_files,
commands::claudemd::get_claudemd_file,
commands::claudemd::update_claudemd_file,
commands::claudemd::delete_claudemd_file,
```

- [ ] **Step 4: Run tests**

```bash
cargo test -p dot-claude-gui commands::claudemd
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/claudemd.rs src-tauri/src/lib.rs
git commit -m "feat(tauri): port claudemd commands to IPC"
```

---

### Task 8: Port memory commands (5 commands)

**Files:**
- Modify: `src-tauri/src/commands/memory.rs`
- Modify: `src-tauri/src/lib.rs`
- Reference: `crates/claude-daemon/src/api/memory.rs`

- [ ] **Step 1: Copy handlers with transformation**

The five handlers are: `list_memory_projects`, `list_memory_files`, `get_memory_file`, `update_memory_file`, `delete_memory_file`. Apply the same transformation pattern (logic helper + Tauri command wrapper + string errors).

- [ ] **Step 2: Write minimum tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ProjectInfo;
    use tempfile::tempdir;

    #[tokio::test]
    async fn list_memory_projects_empty_when_none() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());
        let result = list_memory_projects_logic(&state).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn list_memory_files_returns_md_files_with_frontmatter() {
        let dir = tempdir().unwrap();
        let project_dir = dir.path().join("p");
        let memory_dir = project_dir.join("memory");
        std::fs::create_dir_all(&memory_dir).unwrap();
        std::fs::write(
            memory_dir.join("note.md"),
            "---\nname: note\ndescription: test\n---\n\nBody",
        )
        .unwrap();

        let state = AppState::new(dir.path().to_path_buf());
        state.inner.projects.write().await.push(ProjectInfo {
            id: "p1".to_string(),
            path: project_dir,
            name: "p".to_string(),
        });

        let files = list_memory_files_logic(&state, "p1".to_string()).await.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].filename, "note.md");
    }

    #[tokio::test]
    async fn update_memory_file_writes_content() {
        let dir = tempdir().unwrap();
        let project_dir = dir.path().join("p");
        std::fs::create_dir_all(project_dir.join("memory")).unwrap();

        let state = AppState::new(dir.path().to_path_buf());
        state.inner.projects.write().await.push(ProjectInfo {
            id: "p1".to_string(),
            path: project_dir.clone(),
            name: "p".to_string(),
        });

        update_memory_file_logic(
            &state,
            "p1".to_string(),
            "new.md".to_string(),
            "# New".to_string(),
        )
        .await
        .unwrap();

        let on_disk = std::fs::read_to_string(project_dir.join("memory").join("new.md")).unwrap();
        assert_eq!(on_disk, "# New");
    }
}
```

- [ ] **Step 3: Register commands**

```rust
commands::memory::list_memory_projects,
commands::memory::list_memory_files,
commands::memory::get_memory_file,
commands::memory::update_memory_file,
commands::memory::delete_memory_file,
```

- [ ] **Step 4: Run tests and commit**

```bash
cargo test -p dot-claude-gui commands::memory
git add src-tauri/src/commands/memory.rs src-tauri/src/lib.rs
git commit -m "feat(tauri): port memory commands to IPC"
```

---

### Task 9: Port launcher command (1 command)

**Files:**
- Modify: `src-tauri/src/commands/launcher.rs`
- Modify: `src-tauri/src/lib.rs`
- Reference: `crates/claude-daemon/src/api/launcher.rs`

- [ ] **Step 1: Port the handler**

The launcher spawns `claude` in a detached subprocess with the given env vars at the given path. In Tauri we use `std::process::Command` with `.spawn()` directly (no event streaming needed — it's fire-and-forget).

```rust
// src-tauri/src/commands/launcher.rs

use claude_types::api::LaunchRequest;
use serde_json::json;
use std::process::Command;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn launch_claude(
    _state: State<'_, AppState>,
    req: LaunchRequest,
) -> Result<serde_json::Value, String> {
    let project_path = std::path::PathBuf::from(&req.project_path);
    if !project_path.exists() {
        return Err(format!("invalid_path: {}", req.project_path));
    }

    let mut cmd = Command::new("claude");
    cmd.current_dir(&project_path);
    for (k, v) in &req.env {
        cmd.env(k, v);
    }

    // On macOS/Linux, fork detached via nohup-like approach.
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                libc::setsid();
                Ok(())
            });
        }
    }

    cmd.stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("spawn: {e}"))?;

    Ok(json!({
        "status": "launched",
        "projectPath": req.project_path,
    }))
}
```

If the daemon's original `launcher.rs` uses `tokio::process::Command` or has different detachment logic, mirror that exactly.

Add `libc = "0.2"` to `src-tauri/Cargo.toml` if the `pre_exec` block uses it.

- [ ] **Step 2: Register the command**

```rust
commands::launcher::launch_claude,
```

- [ ] **Step 3: Manual verification**

Unit-testing subprocess spawning is fragile. Instead, add a single smoke test that asserts the command returns an error when the path doesn't exist:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn launch_rejects_nonexistent_path() {
        // We test the path-validation branch without actually spawning claude.
        // The State parameter is not used by the validation, so we inline:
        let req = LaunchRequest {
            project_path: "/nonexistent/path/12345".to_string(),
            env: HashMap::new(),
        };

        let project_path = std::path::PathBuf::from(&req.project_path);
        assert!(!project_path.exists());
        // The real command would return Err("invalid_path: ...")
    }
}
```

End-to-end launcher testing is deferred to the smoke-test phase (Task 24).

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/launcher.rs src-tauri/src/lib.rs src-tauri/Cargo.toml
git commit -m "feat(tauri): port launcher command to IPC"
```

---

### Task 10: Port mcp commands (3 commands, depends on executor)

**Files:**
- Modify: `src-tauri/src/commands/mcp.rs`
- Modify: `src-tauri/src/lib.rs`
- Reference: `crates/claude-daemon/src/api/mcp.rs`

The three MCP commands (`list_mcp_servers`, `add_mcp_server`, `remove_mcp_server`) call the `claude mcp list/add/remove` CLI. `list_mcp_servers` is synchronous (parses output). `add_mcp_server` and `remove_mcp_server` are async command spawns that stream output via events — they **depend on the executor from Task 12**.

Because Task 12 hasn't landed yet, we split Task 10 into two passes:

- [ ] **Step 1: Port `list_mcp_servers` first (synchronous)**

```rust
// src-tauri/src/commands/mcp.rs

use claude_types::api::{AddMcpServerRequest, CommandRequest, McpServerInfo};
use std::process::Command;
use tauri::{AppHandle, State};

use crate::state::AppState;

pub(crate) async fn list_mcp_servers_logic(
    _state: &AppState,
) -> Result<Vec<McpServerInfo>, String> {
    // Copy body from crates/claude-daemon/src/api/mcp.rs::list_mcp_servers
    // The daemon runs `claude mcp list` and parses stdout.
    let output = Command::new("claude")
        .args(["mcp", "list"])
        .output()
        .map_err(|e| format!("spawn: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "internal: claude mcp list failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Parse the stdout into McpServerInfo entries using the same parser
    // the daemon already has. Copy the helper function too if it lives
    // inside daemon/api/mcp.rs.
    let stdout = String::from_utf8_lossy(&output.stdout);
    let servers = parse_mcp_list_output(&stdout);
    Ok(servers)
}

// Copy the parse_mcp_list_output helper from the daemon's mcp.rs verbatim.
fn parse_mcp_list_output(stdout: &str) -> Vec<McpServerInfo> {
    // ... copy from daemon ...
    todo!("copy parse_mcp_list_output from crates/claude-daemon/src/api/mcp.rs")
}

#[tauri::command]
pub async fn list_mcp_servers(state: State<'_, AppState>) -> Result<Vec<McpServerInfo>, String> {
    list_mcp_servers_logic(&state).await
}

// add_mcp_server and remove_mcp_server are stubbed here; real implementation
// lands in Task 12 after the executor exists.
#[tauri::command]
pub async fn add_mcp_server(
    _app: AppHandle,
    _state: State<'_, AppState>,
    _req: AddMcpServerRequest,
) -> Result<CommandRequest, String> {
    Err("not_implemented: wait for executor".to_string())
}

#[tauri::command]
pub async fn remove_mcp_server(
    _app: AppHandle,
    _state: State<'_, AppState>,
    _name: String,
    _scope: Option<String>,
) -> Result<CommandRequest, String> {
    Err("not_implemented: wait for executor".to_string())
}
```

- [ ] **Step 2: Register commands**

```rust
commands::mcp::list_mcp_servers,
commands::mcp::add_mcp_server,
commands::mcp::remove_mcp_server,
```

- [ ] **Step 3: Write minimum test for parser (deterministic)**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_mcp_list_output_handles_empty() {
        let servers = parse_mcp_list_output("");
        assert!(servers.is_empty());
    }

    #[test]
    fn parse_mcp_list_output_handles_one_server() {
        // Use the same fixture string as the daemon's test, if one exists.
        // Otherwise construct a representative sample of `claude mcp list` output.
        let sample = "server-name  stdio  command /usr/bin/thing\n";
        let servers = parse_mcp_list_output(sample);
        assert_eq!(servers.len(), 1);
    }
}
```

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/mcp.rs src-tauri/src/lib.rs
git commit -m "feat(tauri): port list_mcp_servers to IPC (add/remove stubbed)"
```

---

### Task 11: Port plugins commands (8 commands — same pattern as mcp)

**Files:**
- Modify: `src-tauri/src/commands/plugins.rs`
- Modify: `src-tauri/src/lib.rs`
- Reference: `crates/claude-daemon/src/api/plugins.rs`

The 8 commands split into:
- **Synchronous (port now):** `list_plugins`, `list_marketplaces`, `get_marketplace_plugins`, `toggle_plugin` (file I/O only)
- **Async (stubbed now, filled in Task 12):** `install_plugin`, `uninstall_plugin`, `add_marketplace`, `remove_marketplace`

- [ ] **Step 1: Port the 4 synchronous commands**

Copy logic from `crates/claude-daemon/src/api/plugins.rs` for `list_plugins`, `list_marketplaces`, `browse_marketplace_plugins` (rename to `get_marketplace_plugins`), and `toggle_plugin`. Use the same logic-helper + Tauri-command pattern.

- [ ] **Step 2: Stub the 4 async commands**

```rust
#[tauri::command]
pub async fn install_plugin(
    _app: AppHandle,
    _state: State<'_, AppState>,
    _name: String,
    _marketplace: String,
) -> Result<CommandRequest, String> {
    Err("not_implemented: wait for executor".to_string())
}

#[tauri::command]
pub async fn uninstall_plugin(
    _app: AppHandle,
    _state: State<'_, AppState>,
    _id: String,
) -> Result<CommandRequest, String> {
    Err("not_implemented: wait for executor".to_string())
}

#[tauri::command]
pub async fn add_marketplace(
    _app: AppHandle,
    _state: State<'_, AppState>,
    _repo: String,
) -> Result<CommandRequest, String> {
    Err("not_implemented: wait for executor".to_string())
}

#[tauri::command]
pub async fn remove_marketplace(
    _app: AppHandle,
    _state: State<'_, AppState>,
    _id: String,
) -> Result<CommandRequest, String> {
    Err("not_implemented: wait for executor".to_string())
}
```

- [ ] **Step 3: Register all 8 commands**

```rust
commands::plugins::list_plugins,
commands::plugins::list_marketplaces,
commands::plugins::get_marketplace_plugins,
commands::plugins::toggle_plugin,
commands::plugins::install_plugin,
commands::plugins::uninstall_plugin,
commands::plugins::add_marketplace,
commands::plugins::remove_marketplace,
```

- [ ] **Step 4: Write tests for the 4 synchronous commands**

Copy any existing daemon tests. Minimum new tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn list_plugins_empty_when_no_installed_file() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());
        let result = list_plugins_logic(&state).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn list_plugins_reads_installed_plugins_json() {
        let dir = tempdir().unwrap();
        let plugins_dir = dir.path().join("plugins");
        std::fs::create_dir_all(&plugins_dir).unwrap();
        std::fs::write(
            plugins_dir.join("installed_plugins.json"),
            r#"{"plugins": {"test@mk": {"installPath": "/tmp/test", "version": "1.0.0"}}}"#,
        )
        .unwrap();

        let state = AppState::new(dir.path().to_path_buf());
        let result = list_plugins_logic(&state).await.unwrap();
        assert_eq!(result.len(), 1);
    }
}
```

Adjust the JSON fixture to match `claude-types::api::InstalledPluginsFile` or whatever the daemon parses.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/plugins.rs src-tauri/src/lib.rs
git commit -m "feat(tauri): port synchronous plugin commands to IPC (async stubbed)"
```

---

## Stage B: Background services (executor + watcher)

### Task 12: Port executor (async CLI runner)

**Files:**
- Modify: `src-tauri/src/executor.rs`
- Modify: `src-tauri/src/events.rs`
- Modify: `src-tauri/src/commands/mcp.rs` — fill in `add_mcp_server`, `remove_mcp_server`
- Modify: `src-tauri/src/commands/plugins.rs` — fill in 4 async plugin commands
- Reference: `crates/claude-daemon/src/executor.rs`

The executor runs a `tokio::process::Command`, streams each stdout/stderr line, and emits:
- One `command-output` event per line
- One `command-completed` event on exit

In the daemon, it uses `broadcast::Sender<WsEvent>`. In Tauri, it takes `AppHandle` and calls `.emit(...)`.

- [ ] **Step 1: Define event payload types**

```rust
// src-tauri/src/events.rs

use claude_types::{CommandStream, Settings, WsValidationError};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigChangedPayload {
    pub settings: Settings,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationErrorPayload {
    pub errors: Vec<WsValidationError>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandOutputPayload {
    pub command_id: String,
    pub line: String,
    pub stream: CommandStream,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandCompletedPayload {
    pub command_id: String,
    pub exit_code: i32,
}

pub const EVT_CONFIG_CHANGED: &str = "config-changed";
pub const EVT_VALIDATION_ERROR: &str = "validation-error";
pub const EVT_COMMAND_OUTPUT: &str = "command-output";
pub const EVT_COMMAND_COMPLETED: &str = "command-completed";
```

If `CommandStream` or `WsValidationError` don't exist in `claude-types`, define small Serialize structs locally.

- [ ] **Step 2: Write the executor**

```rust
// src-tauri/src/executor.rs

use claude_types::CommandStream;
use futures_util::StreamExt;
use std::process::Stdio;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use uuid::Uuid;

use crate::events::{
    CommandCompletedPayload, CommandOutputPayload, EVT_COMMAND_COMPLETED, EVT_COMMAND_OUTPUT,
};

/// Spawn `claude <args...>` in the background, streaming stdout/stderr lines
/// as Tauri events, and emitting a completion event when done. Returns a
/// command ID the caller can use to correlate events.
pub fn spawn_streaming(
    app: AppHandle,
    program: &str,
    args: Vec<String>,
) -> Result<String, String> {
    let command_id = Uuid::new_v4().to_string();
    let command_id_clone = command_id.clone();

    let mut child = Command::new(program)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn: {e}"))?;

    let stdout = child.stdout.take().ok_or("no stdout")?;
    let stderr = child.stderr.take().ok_or("no stderr")?;

    // Stdout streaming task
    let app_out = app.clone();
    let id_out = command_id.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = app_out.emit(
                EVT_COMMAND_OUTPUT,
                CommandOutputPayload {
                    command_id: id_out.clone(),
                    line,
                    stream: CommandStream::Stdout,
                },
            );
        }
    });

    // Stderr streaming task
    let app_err = app.clone();
    let id_err = command_id.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = app_err.emit(
                EVT_COMMAND_OUTPUT,
                CommandOutputPayload {
                    command_id: id_err.clone(),
                    line,
                    stream: CommandStream::Stderr,
                },
            );
        }
    });

    // Wait-for-exit task
    let app_done = app.clone();
    let id_done = command_id.clone();
    tokio::spawn(async move {
        let exit_code = child
            .wait()
            .await
            .map(|s| s.code().unwrap_or(-1))
            .unwrap_or(-1);
        let _ = app_done.emit(
            EVT_COMMAND_COMPLETED,
            CommandCompletedPayload {
                command_id: id_done,
                exit_code,
            },
        );
    });

    Ok(command_id_clone)
}
```

`tokio::process::Command` requires `tokio` features `process` and `io-util`. Add to `src-tauri/Cargo.toml`:

```toml
tokio = { version = "1", features = ["net", "time", "process", "io-util", "macros", "rt-multi-thread"] }
```

- [ ] **Step 3: Fill in the async commands in mcp.rs**

Replace the stubs from Task 10:

```rust
#[tauri::command]
pub async fn add_mcp_server(
    app: AppHandle,
    _state: State<'_, AppState>,
    req: AddMcpServerRequest,
) -> Result<CommandRequest, String> {
    // Build `claude mcp add <name> <transport> ...` args from req
    let mut args = vec!["mcp".to_string(), "add".to_string(), req.name.clone()];
    args.push(req.transport.clone());
    if let Some(cmd) = req.command_or_url.as_ref() {
        args.push(cmd.clone());
    }
    for a in &req.args {
        args.push(a.clone());
    }
    // env/headers: same flags as daemon used

    let request_id = crate::executor::spawn_streaming(app, "claude", args)?;
    Ok(CommandRequest { request_id })
}

#[tauri::command]
pub async fn remove_mcp_server(
    app: AppHandle,
    _state: State<'_, AppState>,
    name: String,
    scope: Option<String>,
) -> Result<CommandRequest, String> {
    let mut args = vec!["mcp".to_string(), "remove".to_string(), name];
    if let Some(s) = scope {
        args.push("--scope".to_string());
        args.push(s);
    }
    let request_id = crate::executor::spawn_streaming(app, "claude", args)?;
    Ok(CommandRequest { request_id })
}
```

Match the exact flag set the daemon builds (check `crates/claude-daemon/src/api/mcp.rs` for the argument construction).

- [ ] **Step 4: Fill in the 4 async plugin commands the same way**

```rust
#[tauri::command]
pub async fn install_plugin(
    app: AppHandle,
    _state: State<'_, AppState>,
    name: String,
    marketplace: String,
) -> Result<CommandRequest, String> {
    let args = vec![
        "plugin".to_string(),
        "install".to_string(),
        format!("{name}@{marketplace}"),
    ];
    let request_id = crate::executor::spawn_streaming(app, "claude", args)?;
    Ok(CommandRequest { request_id })
}

// Similarly for uninstall_plugin, add_marketplace, remove_marketplace.
// Match the CLI argument format the daemon uses exactly.
```

- [ ] **Step 5: Write a smoke test for the executor**

Use a portable shell command that doesn't depend on `claude` being installed:

```rust
// src-tauri/src/executor.rs

#[cfg(test)]
mod tests {
    // Full integration testing of event emission requires a Tauri test runtime.
    // For now, smoke-test by calling spawn_streaming with `echo` and verifying
    // the command doesn't panic and returns a non-empty command_id.
    //
    // Note: This test only compiles if we can construct an AppHandle in tests,
    // which we can't without tauri::test. We mark this as a manual smoke test.
    // The real verification happens in Task 24 (end-to-end).
}
```

No automated test for the executor — verify manually in Task 24.

- [ ] **Step 6: Run cargo check**

```bash
cargo check -p dot-claude-gui
cargo test -p dot-claude-gui
```

Expected: all tests PASS (the new executor has no tests but compiles).

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/executor.rs src-tauri/src/events.rs src-tauri/src/commands/mcp.rs src-tauri/src/commands/plugins.rs src-tauri/Cargo.toml
git commit -m "feat(tauri): port executor and wire async mcp/plugin commands"
```

---

### Task 13: Port file watcher

**Files:**
- Modify: `src-tauri/src/watcher.rs`
- Modify: `src-tauri/src/lib.rs`
- Reference: `crates/claude-daemon/src/watcher.rs`

- [ ] **Step 1: Port the watcher using AppHandle for events**

```rust
// src-tauri/src/watcher.rs

use claude_config::watch::watch_directories;
use claude_config::parse;
use std::path::Path;
use tauri::{AppHandle, Emitter};

use crate::events::{
    ConfigChangedPayload, ValidationErrorPayload, EVT_CONFIG_CHANGED, EVT_VALIDATION_ERROR,
};
use crate::state::AppState;

pub fn start_watcher(app: AppHandle, state: AppState) {
    let app_clone = app.clone();
    let claude_home = state.inner.claude_home.clone();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Handle::current();
        let _ = watch_directories(&[claude_home.clone()], move |paths| {
            for path in paths {
                let app_c = app_clone.clone();
                let state_c = state.clone();
                rt.spawn(async move {
                    handle_file_event(&app_c, &state_c, &path).await;
                });
            }
        });
    });
}

async fn handle_file_event(app: &AppHandle, state: &AppState, path: &Path) {
    // Replicate daemon/watcher.rs:51-130 handle_file_event body.
    // For the user settings.json path:
    let user_settings_path = state.inner.claude_home.join("settings.json");
    if path == user_settings_path {
        match parse::read_settings(path) {
            Ok(settings) => {
                *state.inner.user_settings.write().await = settings.clone();
                let _ = app.emit(
                    EVT_CONFIG_CHANGED,
                    ConfigChangedPayload {
                        settings,
                        source: Some("user".to_string()),
                    },
                );
            }
            Err(e) => {
                // Emit validation-error with a single error entry.
                let _ = app.emit(
                    EVT_VALIDATION_ERROR,
                    ValidationErrorPayload {
                        errors: vec![/* construct WsValidationError { path, message: e.to_string() } */],
                    },
                );
            }
        }
        return;
    }

    // Project settings handling: iterate state.inner.projects and check
    // if path == <project>/.claude/settings.json for any project.
    // Copy this logic from the daemon's watcher.rs verbatim.
    // ... (copy from crates/claude-daemon/src/watcher.rs)
}
```

Copy the daemon's `watcher.rs` body for the project-settings branch verbatim, only replacing `state.broadcast(WsEvent::ConfigChanged { ... })` calls with `app.emit(EVT_CONFIG_CHANGED, ConfigChangedPayload { ... })`.

- [ ] **Step 2: Start the watcher in lib.rs setup**

Edit `src-tauri/src/lib.rs` — in the `setup` closure, after `.manage(app_state)`:

```rust
let state_for_watcher: tauri::State<AppState> = app.state::<AppState>();
let state_clone = (*state_for_watcher).clone();
let app_handle = app.handle().clone();

// Load user settings synchronously before starting watcher
tauri::async_runtime::block_on(async {
    let _ = state_clone.load_user_settings().await;
});

crate::watcher::start_watcher(app_handle, state_clone);
```

- [ ] **Step 3: Compile check**

```bash
cargo check -p dot-claude-gui
```

Expected: compiles. Watcher test is deferred to end-to-end Task 24.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/watcher.rs src-tauri/src/lib.rs
git commit -m "feat(tauri): port file watcher and wire into setup"
```

---

### Task 14: Remove sidecar startup from lib.rs

**Files:**
- Modify: `src-tauri/src/lib.rs`

The `start_sidecar()` function and its helpers (`find_available_port`, `wait_for_health`, `SidecarState`) are now dead code. The old `read_connections`/`write_connections` commands are also dead — frontend no longer needs them.

- [ ] **Step 1: Delete the sidecar functions**

In `src-tauri/src/lib.rs`, delete:
- `struct SidecarState` (lines ~134-150)
- `fn find_available_port` (lines ~152-160)
- `async fn wait_for_health` (lines ~162-177)
- `async fn start_sidecar` (lines ~181-300)
- The `Mutex<SidecarState>` from `.manage(...)` in setup
- The `.spawn(start_sidecar(...))` call in setup

- [ ] **Step 2: Delete obsolete IPC commands**

Delete these command functions:
- `fn read_connections()` (~line 90)
- `fn write_connections(json: String)` (~line 104)

And remove them from the `invoke_handler![...]` macro call. **Keep**:
- `get_config_dir`
- `read_app_config`
- `write_app_config`

Also delete the `ConnectionEntry` / `ConnectionsFile` type definitions (lines ~9-43) — these were only used by `read_connections` / `write_connections`.

- [ ] **Step 3: Delete unused dependencies from Cargo.toml**

Remove from `src-tauri/Cargo.toml`:
- `rand` (only used for token generation)
- `base64` (only used for token encoding)

Keep `tauri-plugin-shell` because `launch_claude` still uses it for detached subprocess on some platforms. Actually — our `launch_claude` uses `std::process::Command`, not `tauri-plugin-shell`. **If** no other code path uses the shell plugin, also delete `tauri-plugin-shell` and the corresponding `.plugin(tauri_plugin_shell::init())` call. Verify by grepping for `tauri_plugin_shell` in `src-tauri/src/` — if no results, remove it.

- [ ] **Step 4: Verify cargo check**

```bash
cargo check -p dot-claude-gui
```

Expected: compiles with no warnings about the deleted items.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/Cargo.toml
git commit -m "refactor(tauri): remove sidecar startup and connection IPC"
```

---

### Task 15: Remove sidecar config from tauri.conf.json

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/capabilities/default.json`

- [ ] **Step 1: Remove externalBin**

In `src-tauri/tauri.conf.json`, find the `"bundle"` section and delete:

```json
"externalBin": ["binaries/claude-daemon"]
```

- [ ] **Step 2: Update capabilities**

In `src-tauri/capabilities/default.json`, remove the shell permissions that were only needed for the sidecar:

```json
// Remove these if launch_claude doesn't use tauri-plugin-shell:
"shell:allow-open",
"shell:allow-execute",
"shell:allow-spawn"
```

If `launch_claude` uses `std::process::Command` directly (not the shell plugin), all three can go. Otherwise keep `shell:allow-execute`.

- [ ] **Step 3: Verify the app still starts**

```bash
pnpm tauri dev
```

Expected: App window opens. At this stage, the frontend **will fail** because `api/client.ts` still tries to reach the daemon. That's fine — close the app and continue.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/tauri.conf.json src-tauri/capabilities/default.json
git commit -m "refactor(tauri): remove sidecar binary and permissions"
```

---

## Stage C: Frontend IPC layer

### Task 16: Create src/lib/ipc/client.ts

**Files:**
- Create: `src/lib/ipc/client.ts`

- [ ] **Step 1: Write the IPC client mirroring DaemonClient**

```typescript
// src/lib/ipc/client.ts

import { invoke } from "@tauri-apps/api/core";
import type {
  AddMcpServerRequest,
  AvailablePlugin,
  ClaudeMdFile,
  ClaudeMdFileDetail,
  ConfigResponse,
  EffectiveConfig,
  HealthResponse,
  LaunchRequest,
  MarketplaceInfo,
  McpServerInfo,
  MemoryFile,
  MemoryFileDetail,
  MemoryProject,
  PluginInfo,
  ProjectEntry,
  Settings,
  SkillContentResponse,
  SkillInfo,
} from "$lib/api/types.js";

export class IpcError extends Error {
  constructor(public readonly kind: string, message: string) {
    super(message);
    this.name = "IpcError";
  }
}

function parseError(e: unknown): IpcError {
  const msg = typeof e === "string" ? e : String(e);
  const colonIdx = msg.indexOf(":");
  if (colonIdx > 0) {
    return new IpcError(msg.slice(0, colonIdx), msg.slice(colonIdx + 1).trim());
  }
  return new IpcError("unknown", msg);
}

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (e) {
    throw parseError(e);
  }
}

export class IpcClient {
  // --- health ---
  async health(): Promise<HealthResponse> {
    return call("health");
  }

  // --- config ---
  async getUserConfig(): Promise<ConfigResponse> {
    return call("get_user_config");
  }
  async updateUserConfig(settings: Partial<Settings>): Promise<ConfigResponse> {
    return call("update_user_config", { req: { settings } });
  }
  async getProjectConfig(projectId: string): Promise<ConfigResponse> {
    return call("get_project_config", { projectId });
  }
  async updateProjectConfig(
    projectId: string,
    settings: Partial<Settings>,
  ): Promise<ConfigResponse> {
    return call("update_project_config", { projectId, req: { settings } });
  }
  async getEffectiveConfig(projectId: string): Promise<EffectiveConfig> {
    return call("get_effective_config", { projectId });
  }

  // --- projects ---
  async listProjects(): Promise<ProjectEntry[]> {
    return call("list_projects");
  }
  async registerProject(path: string): Promise<ProjectEntry> {
    return call("register_project", { req: { path } });
  }
  async unregisterProject(id: string): Promise<void> {
    return call("unregister_project", { id });
  }

  // --- plugins ---
  async listPlugins(): Promise<PluginInfo[]> {
    return call("list_plugins");
  }
  async togglePlugin(id: string, enabled: boolean): Promise<void> {
    return call("toggle_plugin", { id, enabled });
  }
  async installPlugin(name: string, marketplace: string): Promise<{ requestId: string }> {
    return call("install_plugin", { name, marketplace });
  }
  async uninstallPlugin(id: string): Promise<{ requestId: string }> {
    return call("uninstall_plugin", { id });
  }
  async listMarketplaces(): Promise<MarketplaceInfo[]> {
    return call("list_marketplaces");
  }
  async getMarketplacePlugins(marketplaceId: string): Promise<AvailablePlugin[]> {
    return call("get_marketplace_plugins", { marketplaceId });
  }
  async addMarketplace(repo: string): Promise<{ requestId: string }> {
    return call("add_marketplace", { repo });
  }
  async removeMarketplace(id: string): Promise<{ requestId: string }> {
    return call("remove_marketplace", { id });
  }

  // --- mcp ---
  async listMcpServers(): Promise<McpServerInfo[]> {
    return call("list_mcp_servers");
  }
  async addMcpServer(req: AddMcpServerRequest): Promise<{ requestId: string }> {
    return call("add_mcp_server", { req });
  }
  async removeMcpServer(name: string, scope?: string): Promise<{ requestId: string }> {
    return call("remove_mcp_server", { name, scope });
  }

  // --- skills ---
  async listSkills(): Promise<SkillInfo[]> {
    return call("list_skills");
  }
  async getSkillContent(id: string): Promise<SkillContentResponse> {
    return call("get_skill_content", { id });
  }

  // --- claudemd ---
  async listClaudeMdFiles(): Promise<ClaudeMdFile[]> {
    return call("list_claudemd_files");
  }
  async getClaudeMdFile(id: string): Promise<ClaudeMdFileDetail> {
    return call("get_claudemd_file", { id });
  }
  async updateClaudeMdFile(id: string, content: string): Promise<void> {
    return call("update_claudemd_file", { id, content });
  }
  async deleteClaudeMdFile(id: string): Promise<void> {
    return call("delete_claudemd_file", { id });
  }

  // --- memory ---
  async listMemoryProjects(): Promise<MemoryProject[]> {
    return call("list_memory_projects");
  }
  async listMemoryFiles(projectId: string): Promise<MemoryFile[]> {
    return call("list_memory_files", { projectId });
  }
  async getMemoryFile(projectId: string, filename: string): Promise<MemoryFileDetail> {
    return call("get_memory_file", { projectId, filename });
  }
  async updateMemoryFile(
    projectId: string,
    filename: string,
    content: string,
  ): Promise<void> {
    return call("update_memory_file", { projectId, filename, content });
  }
  async deleteMemoryFile(projectId: string, filename: string): Promise<void> {
    return call("delete_memory_file", { projectId, filename });
  }

  // --- launcher ---
  async launchClaude(req: LaunchRequest): Promise<{ status: string }> {
    return call("launch_claude", { req });
  }
}

export const ipcClient = new IpcClient();
```

- [ ] **Step 2: Verify TS compiles**

```bash
pnpm check
```

Expected: no type errors in `ipc/client.ts`. Other files will still have type errors from still importing `api/client.ts`; we'll fix them in later tasks.

- [ ] **Step 3: Commit**

```bash
git add src/lib/ipc/client.ts
git commit -m "feat(frontend): create IPC client mirroring DaemonClient surface"
```

---

### Task 17: Create src/lib/ipc/events.ts

**Files:**
- Create: `src/lib/ipc/events.ts`

- [ ] **Step 1: Write the event listeners**

```typescript
// src/lib/ipc/events.ts

import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Settings, WsValidationError } from "$lib/api/types.js";

export interface ConfigChangedPayload {
  settings: Settings;
  source?: string;
}

export interface ValidationErrorPayload {
  errors: WsValidationError[];
}

export interface CommandOutputPayload {
  commandId: string;
  line: string;
  stream: "stdout" | "stderr";
}

export interface CommandCompletedPayload {
  commandId: string;
  exitCode: number;
}

export function onConfigChanged(
  handler: (p: ConfigChangedPayload) => void,
): Promise<UnlistenFn> {
  return listen<ConfigChangedPayload>("config-changed", (e) => handler(e.payload));
}

export function onValidationError(
  handler: (p: ValidationErrorPayload) => void,
): Promise<UnlistenFn> {
  return listen<ValidationErrorPayload>("validation-error", (e) => handler(e.payload));
}

export function onCommandOutput(
  handler: (p: CommandOutputPayload) => void,
): Promise<UnlistenFn> {
  return listen<CommandOutputPayload>("command-output", (e) => handler(e.payload));
}

export function onCommandCompleted(
  handler: (p: CommandCompletedPayload) => void,
): Promise<UnlistenFn> {
  return listen<CommandCompletedPayload>("command-completed", (e) => handler(e.payload));
}
```

- [ ] **Step 2: Verify**

```bash
pnpm check 2>&1 | grep "ipc/events" || echo "no errors in ipc/events"
```

Expected: no errors in `ipc/events.ts`.

- [ ] **Step 3: Commit**

```bash
git add src/lib/ipc/events.ts
git commit -m "feat(frontend): create IPC event listener helpers"
```

---

## Stage D: Migrate stores to IPC client

### Task 18: Migrate config store

**Files:**
- Modify: `src/lib/stores/config.svelte.ts`

- [ ] **Step 1: Replace client source**

At the top of the file, replace:

```typescript
import { connectionStore } from "./connection.svelte.js";
```

with:

```typescript
import { ipcClient } from "$lib/ipc/client.js";
```

Then replace every `const client = connectionStore.client; if (!client) return;` block with direct calls to `ipcClient`:

```typescript
// Before:
const client = connectionStore.client;
if (!client) return;
const resp = await client.getUserConfig();

// After:
const resp = await ipcClient.getUserConfig();
```

Delete the `if (!client) return;` guards entirely — IPC is always available.

- [ ] **Step 2: Remove connection-awareness**

Search the file for any reference to `connectionStore` and remove it. The store is now fully IPC-driven.

- [ ] **Step 3: Verify**

```bash
pnpm check 2>&1 | grep "stores/config" || echo "no errors"
```

- [ ] **Step 4: Commit**

```bash
git add src/lib/stores/config.svelte.ts
git commit -m "refactor(frontend): migrate config store to IPC client"
```

---

### Task 19: Migrate projects store

**Files:**
- Modify: `src/lib/stores/projects.svelte.ts`

- [ ] **Step 1: Apply the same transformation as Task 18**

Replace `connectionStore.client` calls with `ipcClient` calls. Delete connection guards.

- [ ] **Step 2: Verify and commit**

```bash
pnpm check 2>&1 | grep "stores/projects" || echo "no errors"
git add src/lib/stores/projects.svelte.ts
git commit -m "refactor(frontend): migrate projects store to IPC client"
```

---

### Task 20: Migrate plugins store

**Files:**
- Modify: `src/lib/stores/plugins.svelte.ts`

- [ ] **Step 1: Apply transformation**

Replace 8 client calls with `ipcClient` equivalents. The plugins store also listens for `commandOutput`/`commandCompleted` WS events (indirectly, via a handler registered in App.svelte) — this wiring moves to Task 24 when we update App.svelte.

- [ ] **Step 2: Verify and commit**

```bash
pnpm check 2>&1 | grep "stores/plugins" || echo "no errors"
git add src/lib/stores/plugins.svelte.ts
git commit -m "refactor(frontend): migrate plugins store to IPC client"
```

---

### Task 21: Migrate skills store

**Files:**
- Modify: `src/lib/stores/skills.svelte.ts`

- [ ] **Step 1: Apply transformation**

Replace `listSkills` and `getSkillContent` calls.

- [ ] **Step 2: Verify and commit**

```bash
pnpm check 2>&1 | grep "stores/skills" || echo "no errors"
git add src/lib/stores/skills.svelte.ts
git commit -m "refactor(frontend): migrate skills store to IPC client"
```

---

### Task 22: Migrate memory, mcp, and claudemd stores

**Files:**
- Modify: `src/lib/stores/memory.svelte.ts`
- Modify: `src/lib/stores/mcp.svelte.ts`
- Modify: `src/lib/stores/claudemd.svelte.ts`

Apply the same transformation to all three. They're all small (< 120 lines each) and follow the identical pattern.

- [ ] **Step 1: Migrate memory store**

Replace 5 client calls. Verify and `cargo check`-equivalent for frontend:

```bash
pnpm check 2>&1 | grep "stores/memory" || echo "no errors"
```

- [ ] **Step 2: Migrate mcp store**

Replace 3 client calls.

- [ ] **Step 3: Migrate claudemd store**

Replace 4 client calls.

- [ ] **Step 4: Verify all three**

```bash
pnpm check 2>&1 | grep -E "stores/(memory|mcp|claudemd)" || echo "no errors"
```

- [ ] **Step 5: Commit**

```bash
git add src/lib/stores/memory.svelte.ts src/lib/stores/mcp.svelte.ts src/lib/stores/claudemd.svelte.ts
git commit -m "refactor(frontend): migrate memory/mcp/claudemd stores to IPC client"
```

---

## Stage E: App.svelte rewire and component deletion

### Task 23: Rewire App.svelte

**Files:**
- Modify: `src/App.svelte`

The current `App.svelte` (lines 129-172 of the explored version) orchestrates: load connections → connect to daemon → wait for `status === "connected"` → load all stores → subscribe to `configChanged`.

The new flow:
1. `onMount`: load app settings, immediately call all store load functions, subscribe to Tauri events.
2. No connection concept, no status gating.

- [ ] **Step 1: Simplify imports**

Remove imports of:
```typescript
import { connectionStore } from "$lib/stores/connection.svelte.js";
import { connectionsStore } from "$lib/stores/connections.svelte.js";
import EnvironmentSelector from "$lib/components/shared/EnvironmentSelector.svelte";
```

Add:
```typescript
import { onConfigChanged } from "$lib/ipc/events.js";
```

- [ ] **Step 2: Rewrite the onMount block**

Replace the existing onMount (roughly lines 129-139 in the current file) with:

```typescript
onMount(async () => {
  await appSettingsStore.load();

  // Start all data loads in parallel — IPC is always available
  await Promise.all([
    configStore.loadUserConfig(),
    projectsStore.loadProjects(),
    pluginsStore.loadPlugins(),
    skillsStore.loadSkills(),
    memoryStore.loadProjects(),
    mcpStore.loadServers(),
    claudeMdStore.loadFiles(),
  ]);

  // Subscribe to config changes pushed from the backend file watcher
  const unlisten = await onConfigChanged((payload) => {
    configStore.setUserConfig(payload.settings);
    // Optionally: configStore.loadProjectConfig() if source is project
  });

  onDestroy(unlisten);
});
```

Make sure `onDestroy` is imported from `svelte`.

The `configStore.setUserConfig(settings)` method may not exist yet; if not, add it as a simple setter:

```typescript
// In src/lib/stores/config.svelte.ts
setUserConfig(settings: Settings): void {
  this.userConfig = settings;
}
```

- [ ] **Step 3: Remove the connection-status effect**

Delete the effect (lines ~153-172) that waits for `connectionStore.status === "connected"`. All loads now happen directly in `onMount`.

- [ ] **Step 4: Remove EnvironmentSelector from header**

In the `<header>` section of `App.svelte`, remove:
```svelte
<EnvironmentSelector />
```

Keep the project selector (it remains relevant).

- [ ] **Step 5: Verify**

```bash
pnpm check
```

Expected: errors about `connection.svelte.ts`, `connections.svelte.ts`, `EnvironmentSelector.svelte`, `ConnectionsPanel.svelte` being missing exports from some stores — these will disappear in Task 24.

- [ ] **Step 6: Commit**

```bash
git add src/App.svelte src/lib/stores/config.svelte.ts
git commit -m "refactor(frontend): rewire App.svelte for IPC with direct event listening"
```

---

### Task 24: Delete obsolete stores and components

**Files:**
- Delete: `src/lib/stores/connection.svelte.ts`
- Delete: `src/lib/stores/connections.svelte.ts`
- Delete: `src/lib/components/shared/EnvironmentSelector.svelte`
- Delete: `src/lib/components/appsettings/ConnectionsPanel.svelte`
- Delete: `src/lib/api/client.ts`
- Delete: `src/lib/api/ws.ts`
- Modify: `src/lib/components/appsettings/AppSettingsView.svelte`

- [ ] **Step 1: Delete store files**

```bash
rm src/lib/stores/connection.svelte.ts
rm src/lib/stores/connections.svelte.ts
```

- [ ] **Step 2: Delete component files**

```bash
rm src/lib/components/shared/EnvironmentSelector.svelte
rm src/lib/components/appsettings/ConnectionsPanel.svelte
```

- [ ] **Step 3: Delete HTTP/WS client files**

```bash
rm src/lib/api/client.ts
rm src/lib/api/ws.ts
```

Keep `src/lib/api/types.ts` — it's still the source of truth for shared types imported by `src/lib/ipc/client.ts` and every store.

- [ ] **Step 4: Remove ConnectionsPanel from AppSettingsView**

Open `src/lib/components/appsettings/AppSettingsView.svelte`. Remove the import and the `<ConnectionsPanel />` render. The rest of the settings view (theme, language, font size, panel widths) stays.

- [ ] **Step 5: Verify**

```bash
pnpm check
```

Expected: **zero TypeScript errors**. If any remain, grep for missing import symbols and fix.

```bash
pnpm build
```

Expected: frontend builds cleanly.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "refactor(frontend): delete obsolete connection stores and components"
```

---

## Stage F: Delete daemon crate

### Task 25: Delete claude-daemon crate and sidecar binary

**Files:**
- Delete: `crates/claude-daemon/` (entire directory)
- Delete: `src-tauri/binaries/claude-daemon-*` (all sidecar binaries)
- Modify: `Cargo.toml` (workspace root)

- [ ] **Step 1: Delete the daemon crate**

```bash
rm -rf crates/claude-daemon/
```

- [ ] **Step 2: Delete sidecar binaries**

```bash
rm -f src-tauri/binaries/claude-daemon-*
# If the binaries directory is now empty, delete it too:
rmdir src-tauri/binaries/ 2>/dev/null || true
```

- [ ] **Step 3: Remove from workspace members**

Edit `Cargo.toml` (workspace root). Find the `[workspace]` section and remove `"crates/claude-daemon"` from the `members` list.

Also scan the `[workspace.dependencies]` section for entries only used by the daemon:
- `axum`, `tower`, `tower-http`, `axum-extra`
- `tokio-tungstenite`
- `portpicker`
- `reqwest`
- `clap` (if not used by any other crate)

Remove each that has no remaining consumer. To verify before removing, grep for the crate name in the remaining crates:

```bash
grep -r "axum" crates/ src-tauri/ 2>/dev/null
```

If nothing remains, remove it from workspace deps.

- [ ] **Step 4: Rebuild the workspace**

```bash
cargo clean
cargo check --workspace
```

Expected: workspace compiles with only `claude-types`, `claude-config`, `dot-claude-gui` as members.

- [ ] **Step 5: Run the full test suite**

```bash
cargo test --workspace
```

Expected: all tests pass. The daemon's tests are gone, but the new command tests in `src-tauri/src/commands/*` cover the same ground.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "refactor: delete claude-daemon crate and sidecar binaries"
```

---

## Stage G: Documentation and verification

### Task 26: Update CLAUDE.md

**Files:**
- Modify: `CLAUDE.md`

- [ ] **Step 1: Update Commands section**

Remove these lines from the `## Commands` block:

```
cargo build -p claude-daemon              # Build daemon binary (needed before tauri dev)
mkdir -p src-tauri/binaries && cp target/debug/claude-daemon src-tauri/binaries/claude-daemon-aarch64-apple-darwin
cargo run -p claude-daemon -- --port 7890  # Run daemon standalone
cargo test -p claude-daemon               # Run tests for daemon crate
```

Remove the "First-time dev setup" paragraph about building the daemon binary.

- [ ] **Step 2: Update Architecture section**

Replace the "Three layers" block with:

```markdown
**Two layers:**
- **Svelte 5 frontend** (`src/`) — UI with rune-based reactivity (`$state`, `$effect`, `$derived`)
- **Tauri shell** (`src-tauri/`) — Rust backend with IPC commands, file watcher, and subprocess executor

**Rust workspace crates** (`crates/`):
- `claude-types` — shared types (settings, API, events, plugins, skills, memory, MCP)
- `claude-config` — config file parsing, merge engine, atomic writes (temp file → rename)
```

Delete the `claude-daemon` bullet.

- [ ] **Step 3: Update Frontend structure section**

Replace `api/client.ts` and `api/ws.ts` references with `ipc/client.ts` and `ipc/events.ts`. Remove the `DaemonClient` and `DaemonWsClient` mentions.

- [ ] **Step 4: Delete "Multi-Daemon Connections" section entirely**

The whole block starting with `## Multi-Daemon Connections` and ending before `## Key Conventions`.

- [ ] **Step 5: Update the config hierarchy and real-time sync sections**

Change the real-time sync description from:
```
**Real-time sync:** Backend watches `~/.claude/` and project dirs with `notify` crate, debounces changes, broadcasts `configChanged` events over WebSocket to all connected clients.
```
to:
```
**Real-time sync:** Tauri backend watches `~/.claude/` and project dirs with `notify` crate, debounces changes, emits `config-changed` Tauri events consumed by the frontend via `@tauri-apps/api/event`.
```

- [ ] **Step 6: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: update CLAUDE.md for de-daemonized architecture"
```

---

### Task 27: End-to-end smoke test

**Files:** (none — this is a manual verification task)

- [ ] **Step 1: Start the app in dev mode**

```bash
pnpm tauri dev
```

Expected: App window opens without sidecar errors. No daemon startup logs in the Tauri console.

- [ ] **Step 2: Verify Settings module**

- Click Settings in the sidebar
- Confirm `~/.claude/settings.json` contents load
- Edit a field (e.g. add an env var), save
- Confirm toast says "Saved"
- Open the file on disk: `cat ~/.claude/settings.json`
- Confirm the edit persisted

- [ ] **Step 3: Verify live reload**

- In a terminal, manually edit `~/.claude/settings.json` (add another env var)
- Switch back to the app window
- Confirm the Settings UI shows the new value without refresh (via `config-changed` event)

- [ ] **Step 4: Verify Projects module**

- Click Projects (or the project selector in the header)
- Click "Add project", choose a directory with `.claude/settings.json`
- Confirm the project appears in the list
- Click the project and verify its settings load

- [ ] **Step 5: Verify Plugins module**

- Click Plugins
- Confirm installed plugins load (the list should match `~/.claude/plugins/installed_plugins.json`)
- Toggle a plugin's enabled state
- Confirm the toggle persists in `~/.claude/settings.json`'s `enabledPlugins`

- [ ] **Step 6: Verify Skills module**

- Click Skills
- Confirm skills from `~/.claude/skills/` and plugin skills appear
- Click one — confirm the `SKILL.md` preview content loads (this exercises `get_skill_content`)

- [ ] **Step 7: Verify Memory module**

- Click Memory
- For a project with a `memory/` directory, confirm files appear
- Open a memory file, edit it, save
- Confirm the file on disk matches

- [ ] **Step 8: Verify MCP module**

- Click MCP
- Confirm registered MCP servers load (exercises `claude mcp list` subprocess)
- If desired, add a test MCP server and verify it appears

- [ ] **Step 9: Verify CLAUDE.md module**

- Click CLAUDE.md
- Confirm the global entry loads `~/.claude/CLAUDE.md`
- Create, edit, save, delete a project-level CLAUDE.md
- Verify files on disk

- [ ] **Step 10: Verify Effective Config module**

- Click Effective Config
- Select a project
- Confirm the merged view shows correct user/project/local sources

- [ ] **Step 11: Verify Launcher module**

- Click Launcher
- Select a project
- Click "Launch"
- Confirm a new Terminal window opens with `claude` running in the project directory

- [ ] **Step 12: Verify App Settings module**

- Click App Settings
- Confirm theme, language, font size, panel width settings load from `~/.dot-claude-gui/config.json`
- Change theme from light to dark; confirm UI updates
- **Confirm the Connections panel is gone** — no more environment management UI

- [ ] **Step 13: Confirm no regression in async command streaming**

- Attempt to install a small plugin via the Plugins module
- Confirm progress output streams into the UI (exercises `command-output` Tauri events)
- Confirm completion toast appears (exercises `command-completed`)

- [ ] **Step 14: Production build smoke test**

```bash
pnpm tauri build
```

Expected: build succeeds, produces `.app` / `.dmg`. Open the built app and spot-check Settings load.

- [ ] **Step 15: If all checks pass, final commit (tag)**

```bash
git tag phase-7.1-complete
```

Do **not** push the tag — just create it locally as a milestone marker.

---

## Risks and mitigations

| Risk | Mitigation |
|------|-----------|
| Tauri command signatures differ slightly from daemon handlers (field casing, nested structures) | Transformation rules table above; inspect daemon handlers before porting |
| `claude-config` function names don't match those assumed in Task 4 | Read `crates/claude-daemon/src/api/config.rs` imports before writing Task 4's code; mirror exactly |
| File watcher doesn't fire on macOS APFS for rapid successive edits | The daemon already uses `notify` v8 which handles this; keep the same debounce logic |
| Tauri `emit` payload serialization mismatches the frontend's expected shape | `#[serde(rename_all = "camelCase")]` on all payload structs; frontend types already use camelCase |
| `launch_claude` detachment stops working on macOS after removing `tauri-plugin-shell` | Use `std::os::unix::process::CommandExt::pre_exec` with `setsid` (shown in Task 9); test on macOS in Task 27 step 11 |
| Stale `~/.dot-claude-gui/connections.json` confuses users | Ignored — no code reads it after Task 14; document in release notes |

## Rollback

If the migration breaks mid-way:
1. Every stage commits independently, so `git reset --hard HEAD~N` reverts to a known-good state.
2. The sidecar binary is still available under `target/debug/claude-daemon` even after Stage F, if you need to run the daemon standalone for debugging.
3. The most dangerous commit is Task 25 (deletes the daemon crate). Before committing that task, verify Task 27 smoke test passes — that's the point of no return.

## Verification checklist (end of plan)

- [ ] `cargo test --workspace` passes
- [ ] `pnpm check` produces zero TypeScript errors
- [ ] `pnpm tauri dev` starts the app without sidecar
- [ ] All 9 navigation modules function as before Phase 7.1
- [ ] `config-changed` event fires on external file edit
- [ ] Plugin install streams output via `command-output` events
- [ ] `pnpm tauri build` produces a working production bundle
- [ ] `crates/claude-daemon/` no longer exists
- [ ] `src/lib/api/client.ts`, `src/lib/api/ws.ts`, `src/lib/stores/connection*.svelte.ts`, `EnvironmentSelector.svelte`, `ConnectionsPanel.svelte` no longer exist
- [ ] CLAUDE.md reflects the two-layer architecture
