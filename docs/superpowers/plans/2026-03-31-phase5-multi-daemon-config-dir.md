# Phase 5: Multi-Daemon Connections & Config Directory Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `~/.dot-claude-gui/` config directory, Tauri sidecar daemon lifecycle management, and multi-daemon connection switching with environment → project header hierarchy.

**Architecture:** The GUI stores app preferences and daemon connection registry in `~/.dot-claude-gui/`. On startup, Tauri spawns a local `claude-daemon` sidecar with auto-generated port and token. Users can add remote daemon connections (Docker, SSH). The header provides a two-level context selector: environment (daemon) → project, with complete store reset on environment switch.

**Tech Stack:** Rust (Tauri IPC, clap CLI), Svelte 5 runes, TypeScript, `tauri-plugin-shell` sidecar, `dirs-next`

**Spec:** `docs/superpowers/specs/2026-03-31-multi-daemon-and-config-dir.md`

---

## File Structure

### New Files

| File | Responsibility |
|------|---------------|
| `src/lib/stores/connections.svelte.ts` | Connection registry store: CRUD connections, read/write `connections.json`, switch active |
| `src/lib/components/shared/EnvironmentSelector.svelte` | Header environment dropdown with status indicators |
| `src/lib/components/appsettings/ConnectionsPanel.svelte` | Connection management panel (list, add, edit, delete, test) |
| `crates/claude-daemon/tests/bind_test.rs` | Integration test for `--bind` parameter |

### Modified Files

| File | Change |
|------|--------|
| `crates/claude-daemon/src/main.rs` | Add `--bind` arg, remove token file writing |
| `src-tauri/src/lib.rs` | Sidecar spawn, config dir IPC commands, replace `get_daemon_url`/`get_daemon_token` |
| `src-tauri/Cargo.toml` | Add `rand`, `base64`, `serde_json` dependencies |
| `src-tauri/tauri.conf.json` | Add `externalBin` for sidecar, shell plugin permissions |
| `src/lib/stores/connection.svelte.ts` | Add `resetAllStores()`, integrate with connections store |
| `src/lib/stores/appsettings.svelte.ts` | Remove `daemonUrl`/`daemonToken`, use Tauri IPC for persistence |
| `src/lib/stores/config.svelte.ts` | Add `reset()` method |
| `src/lib/stores/projects.svelte.ts` | Add `reset()` method |
| `src/lib/stores/plugins.svelte.ts` | Add `reset()` method |
| `src/lib/stores/skills.svelte.ts` | Add `reset()` method |
| `src/lib/stores/memory.svelte.ts` | Add `reset()` method |
| `src/lib/stores/mcp.svelte.ts` | Add `reset()` method |
| `src/lib/api/types.ts` | Add `ConnectionEntry`, `ConnectionsFile`, `AppConfig` types |
| `src/App.svelte` | Replace header with environment + project selectors, wire connection switching |
| `src/lib/components/appsettings/AppSettingsView.svelte` | Replace inline connection UI with ConnectionsPanel |
| `src/lib/components/shared/ScopeSelector.svelte` | Refactor as project selector with "User Scope" and "添加项目..." |

---

## Task 1: Daemon CLI — Add `--bind` and Remove Token Writing

**Files:**
- Modify: `crates/claude-daemon/src/main.rs`

- [ ] **Step 1: Add `--bind` argument to Args struct**

```rust
// In the Args struct, add after the port field:
    /// Address to bind to.
    #[arg(long, default_value = "127.0.0.1")]
    bind: String,
```

- [ ] **Step 2: Remove token file writing logic**

Delete lines 60-67 in `main.rs` (the block that writes token to `com.dotclaude.gui/daemon-token`):

```rust
    // DELETE THIS BLOCK:
    // Persist the token in the app's own data directory (not in ~/.claude/).
    let app_data_dir = dirs_next::data_dir()
        .expect("cannot determine app data directory")
        .join("com.dotclaude.gui");
    std::fs::create_dir_all(&app_data_dir)?;
    let token_path = app_data_dir.join("daemon-token");
    std::fs::write(&token_path, &auth_token)?;
    info!("auth token written to {}", token_path.display());
```

- [ ] **Step 3: Use `--bind` in socket address**

Replace the hardcoded bind address:

```rust
    // Replace:
    // let addr = SocketAddr::from(([127, 0, 0, 1], args.port));

    // With:
    let addr: SocketAddr = format!("{}:{}", args.bind, args.port)
        .parse()
        .expect("invalid bind address");
    info!("claude-daemon listening on http://{addr}");
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo build -p claude-daemon`
Expected: BUILD SUCCESS

- [ ] **Step 5: Run existing tests**

Run: `cargo test --workspace`
Expected: All tests pass

- [ ] **Step 6: Commit**

```bash
git add crates/claude-daemon/src/main.rs
git commit -m "feat(daemon): add --bind parameter and remove token file writing"
```

---

## Task 2: Daemon --bind Integration Test

**Files:**
- Create: `crates/claude-daemon/tests/bind_test.rs`

- [ ] **Step 1: Write test for bind address parsing**

```rust
use std::net::SocketAddr;

#[test]
fn test_bind_address_parsing() {
    // Valid addresses
    let addr: SocketAddr = "127.0.0.1:7890".parse().unwrap();
    assert_eq!(addr.ip().to_string(), "127.0.0.1");
    assert_eq!(addr.port(), 7890);

    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    assert_eq!(addr.ip().to_string(), "0.0.0.0");
    assert_eq!(addr.port(), 8080);

    // Invalid address
    let result: Result<SocketAddr, _> = "not-an-address:7890".parse();
    assert!(result.is_err());
}
```

- [ ] **Step 2: Run the test**

Run: `cargo test -p claude-daemon --test bind_test`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add crates/claude-daemon/tests/bind_test.rs
git commit -m "test(daemon): add bind address parsing test"
```

---

## Task 3: Frontend Types — Connection and Config Types

**Files:**
- Modify: `src/lib/api/types.ts`

- [ ] **Step 1: Add connection and config types at end of file**

```typescript
// ---------------------------------------------------------------------------
// Application Config (persisted to ~/.dot-claude-gui/)
// ---------------------------------------------------------------------------

export interface ConnectionEntry {
  id: string;
  name: string;
  type: "local" | "remote";
  url: string;
  token: string;
  managed: boolean;
}

export interface ConnectionsFile {
  activeConnectionId: string;
  connections: ConnectionEntry[];
}

export interface AppConfig {
  theme: "light" | "dark" | "system";
  language: string;
  fontSize: number;
}
```

- [ ] **Step 2: Commit**

```bash
git add src/lib/api/types.ts
git commit -m "feat(types): add ConnectionEntry, ConnectionsFile, AppConfig types"
```

---

## Task 4: Tauri IPC — Config Directory and Sidecar Management

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Add dependencies to src-tauri/Cargo.toml**

Add to `[dependencies]`:

```toml
rand = "0.9"
base64 = "0.22"
tokio = { version = "1", features = ["time"] }
```

- [ ] **Step 2: Rewrite src-tauri/src/lib.rs with IPC commands and sidecar**

Replace the entire file with:

```rust
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_shell::ShellExt;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConnectionEntry {
    id: String,
    name: String,
    #[serde(rename = "type")]
    conn_type: String,
    url: String,
    token: String,
    managed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConnectionsFile {
    active_connection_id: String,
    connections: Vec<ConnectionEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppConfig {
    theme: String,
    language: String,
    font_size: u32,
}

// ---------------------------------------------------------------------------
// Config directory helpers
// ---------------------------------------------------------------------------

fn config_dir() -> Result<PathBuf, String> {
    dirs_next::home_dir()
        .map(|h| h.join(".dot-claude-gui"))
        .ok_or_else(|| "cannot determine home directory".to_string())
}

fn ensure_config_dir() -> Result<PathBuf, String> {
    let dir = config_dir()?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("failed to create config dir: {e}"))?;
    Ok(dir)
}

fn default_connections(url: &str, token: &str) -> ConnectionsFile {
    ConnectionsFile {
        active_connection_id: "local".to_string(),
        connections: vec![ConnectionEntry {
            id: "local".to_string(),
            name: "Local".to_string(),
            conn_type: "local".to_string(),
            url: url.to_string(),
            token: token.to_string(),
            managed: true,
        }],
    }
}

fn default_app_config() -> AppConfig {
    AppConfig {
        theme: "system".to_string(),
        language: "zh-CN".to_string(),
        font_size: 14,
    }
}

// ---------------------------------------------------------------------------
// Tauri IPC commands
// ---------------------------------------------------------------------------

#[tauri::command]
fn get_config_dir() -> Result<String, String> {
    config_dir().map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
fn read_connections() -> Result<String, String> {
    let dir = config_dir()?;
    let path = dir.join("connections.json");
    if path.exists() {
        std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read connections.json: {e}"))
    } else {
        // Return default with placeholder — will be updated on sidecar start
        let default = default_connections("http://127.0.0.1:7890", "");
        serde_json::to_string_pretty(&default)
            .map_err(|e| format!("failed to serialize defaults: {e}"))
    }
}

#[tauri::command]
fn write_connections(json: String) -> Result<(), String> {
    let dir = ensure_config_dir()?;
    let path = dir.join("connections.json");
    std::fs::write(&path, &json)
        .map_err(|e| format!("failed to write connections.json: {e}"))
}

#[tauri::command]
fn read_app_config() -> Result<String, String> {
    let dir = config_dir()?;
    let path = dir.join("config.json");
    if path.exists() {
        std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read config.json: {e}"))
    } else {
        let default = default_app_config();
        serde_json::to_string_pretty(&default)
            .map_err(|e| format!("failed to serialize defaults: {e}"))
    }
}

#[tauri::command]
fn write_app_config(json: String) -> Result<(), String> {
    let dir = ensure_config_dir()?;
    let path = dir.join("config.json");
    std::fs::write(&path, &json)
        .map_err(|e| format!("failed to write config.json: {e}"))
}

// ---------------------------------------------------------------------------
// Sidecar state
// ---------------------------------------------------------------------------

struct SidecarState {
    child: Option<tauri_plugin_shell::process::CommandChild>,
    port: u16,
    token: String,
}

// ---------------------------------------------------------------------------
// App entry point
// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(Mutex::new(SidecarState {
            child: None,
            port: 0,
            token: String::new(),
        }))
        .setup(|app| {
            let handle = app.handle().clone();
            // Spawn sidecar in a background thread
            tauri::async_runtime::spawn(async move {
                if let Err(e) = start_sidecar(&handle).await {
                    tracing::error!("failed to start sidecar: {e}");
                    eprintln!("failed to start sidecar: {e}");
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config_dir,
            read_connections,
            write_connections,
            read_app_config,
            write_app_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn start_sidecar(handle: &tauri::AppHandle) -> Result<(), String> {
    // Generate token
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    let token = BASE64.encode(bytes);

    // Find available port: try 7890, then 7891..7899
    let port = find_available_port(7890, 7899)?;

    // Spawn sidecar
    let (mut rx, child) = handle
        .shell()
        .sidecar("claude-daemon")
        .map_err(|e| format!("sidecar binary not found: {e}"))?
        .args([
            "--port",
            &port.to_string(),
            "--token",
            &token,
            "--bind",
            "127.0.0.1",
        ])
        .spawn()
        .map_err(|e| format!("failed to spawn sidecar: {e}"))?;

    // Store child handle for cleanup
    {
        let state = handle.state::<Mutex<SidecarState>>();
        let mut s = state.lock().unwrap();
        s.child = Some(child);
        s.port = port;
        s.token = token.clone();
    }

    // Wait for daemon to be ready (poll health endpoint)
    let url = format!("http://127.0.0.1:{port}");
    wait_for_health(&url, 10).await?;

    // Write connection info to ~/.dot-claude-gui/connections.json
    let dir = ensure_config_dir()?;
    let connections_path = dir.join("connections.json");

    let mut connections = if connections_path.exists() {
        let content = std::fs::read_to_string(&connections_path)
            .map_err(|e| format!("failed to read connections: {e}"))?;
        serde_json::from_str::<ConnectionsFile>(&content)
            .unwrap_or_else(|_| default_connections(&url, &token))
    } else {
        default_connections(&url, &token)
    };

    // Update the local connection entry
    if let Some(local) = connections.connections.iter_mut().find(|c| c.id == "local") {
        local.url = url;
        local.token = token;
    }

    let json = serde_json::to_string_pretty(&connections)
        .map_err(|e| format!("failed to serialize connections: {e}"))?;
    std::fs::write(&connections_path, &json)
        .map_err(|e| format!("failed to write connections: {e}"))?;

    // Drain sidecar output in background (prevent pipe buffer deadlock)
    tauri::async_runtime::spawn(async move {
        use tauri_plugin_shell::process::CommandEvent;
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line) => {
                    let line = String::from_utf8_lossy(&line);
                    tracing::info!("[daemon] {}", line);
                }
                CommandEvent::Stderr(line) => {
                    let line = String::from_utf8_lossy(&line);
                    tracing::warn!("[daemon] {}", line);
                }
                CommandEvent::Terminated(payload) => {
                    tracing::info!("[daemon] terminated: {:?}", payload);
                    break;
                }
                _ => {}
            }
        }
    });

    Ok(())
}

fn find_available_port(start: u16, end: u16) -> Result<u16, String> {
    for port in start..=end {
        if std::net::TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return Ok(port);
        }
    }
    Err(format!("no available port in range {start}-{end}"))
}

async fn wait_for_health(base_url: &str, max_retries: u32) -> Result<(), String> {
    let url = format!("{base_url}/api/v1/health");
    for i in 0..max_retries {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        match reqwest::get(&url).await {
            Ok(resp) if resp.status().is_success() => return Ok(()),
            _ => {
                if i == max_retries - 1 {
                    return Err("daemon health check timed out after 5s".to_string());
                }
            }
        }
    }
    Err("daemon health check failed".to_string())
}
```

- [ ] **Step 3: Wait — `reqwest` is heavy. Use a simpler health check.**

Actually, replace the `wait_for_health` function with a TCP connect check to avoid adding `reqwest` as a dependency:

```rust
async fn wait_for_health(port: u16, max_retries: u32) -> Result<(), String> {
    for i in 0..max_retries {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        if tokio::net::TcpStream::connect(("127.0.0.1", port)).await.is_ok() {
            return Ok(());
        }
        if i == max_retries - 1 {
            return Err("daemon health check timed out after 5s".to_string());
        }
    }
    Err("daemon health check failed".to_string())
}
```

And update the call site:

```rust
    wait_for_health(port, 10).await?;
```

- [ ] **Step 4: Add `tracing` dependency to src-tauri/Cargo.toml**

```toml
tracing = "0.1"
```

- [ ] **Step 5: Update tauri.conf.json for sidecar**

Add `externalBin` to the bundle section and shell plugin permissions:

```json
{
  "bundle": {
    "active": true,
    "targets": "all",
    "externalBin": ["binaries/claude-daemon"],
    "icon": [...]
  },
  "app": {
    "security": { "csp": null }
  },
  "plugins": {
    "shell": {
      "sidecar": true
    }
  }
}
```

- [ ] **Step 6: Create sidecar binary placeholder directory**

Run: `mkdir -p src-tauri/binaries`

For development, create a symlink to the built daemon binary:

Run (macOS Apple Silicon):
```bash
cd src-tauri/binaries
ln -sf ../../target/debug/claude-daemon claude-daemon-aarch64-apple-darwin
```

- [ ] **Step 7: Verify it compiles**

Run: `cargo build -p dot-claude-gui`
Expected: BUILD SUCCESS (may have warnings about unused SidecarState, that's OK)

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/Cargo.toml src-tauri/tauri.conf.json
git commit -m "feat(tauri): add config dir IPC commands and sidecar lifecycle management"
```

---

## Task 5: Connections Store

**Files:**
- Create: `src/lib/stores/connections.svelte.ts`

- [ ] **Step 1: Create the connections store**

```typescript
import { invoke } from "@tauri-apps/api/core";
import type { ConnectionEntry, ConnectionsFile } from "$lib/api/types.js";

class ConnectionsStore {
  connections = $state<ConnectionEntry[]>([]);
  activeConnectionId = $state<string>("local");
  loading = $state<boolean>(false);
  error = $state<string>("");

  get activeConnection(): ConnectionEntry | undefined {
    return this.connections.find((c) => c.id === this.activeConnectionId);
  }

  get localConnection(): ConnectionEntry | undefined {
    return this.connections.find((c) => c.id === "local");
  }

  async load(): Promise<void> {
    this.loading = true;
    this.error = "";
    try {
      const json = await invoke<string>("read_connections");
      const data: ConnectionsFile = JSON.parse(json);
      this.connections = data.connections;
      this.activeConnectionId = data.activeConnectionId;
    } catch (err) {
      this.error = err instanceof Error ? err.message : String(err);
    } finally {
      this.loading = false;
    }
  }

  async save(): Promise<void> {
    const data: ConnectionsFile = {
      activeConnectionId: this.activeConnectionId,
      connections: this.connections,
    };
    try {
      await invoke("write_connections", {
        json: JSON.stringify(data, null, 2),
      });
    } catch (err) {
      this.error = err instanceof Error ? err.message : String(err);
    }
  }

  async addConnection(entry: Omit<ConnectionEntry, "id">): Promise<void> {
    const id = crypto.randomUUID();
    const newEntry: ConnectionEntry = { ...entry, id };
    this.connections = [...this.connections, newEntry];
    await this.save();
  }

  async updateConnection(
    id: string,
    updates: Partial<ConnectionEntry>
  ): Promise<void> {
    this.connections = this.connections.map((c) =>
      c.id === id ? { ...c, ...updates } : c
    );
    await this.save();
  }

  async deleteConnection(id: string): Promise<void> {
    if (id === "local") return; // Cannot delete local
    this.connections = this.connections.filter((c) => c.id !== id);
    if (this.activeConnectionId === id) {
      this.activeConnectionId = "local";
    }
    await this.save();
  }

  async setActive(id: string): Promise<void> {
    this.activeConnectionId = id;
    await this.save();
  }
}

export const connectionsStore = new ConnectionsStore();
```

- [ ] **Step 2: Commit**

```bash
git add src/lib/stores/connections.svelte.ts
git commit -m "feat(stores): add connections store for multi-daemon registry"
```

---

## Task 6: Add `reset()` to All Data Stores

**Files:**
- Modify: `src/lib/stores/config.svelte.ts`
- Modify: `src/lib/stores/projects.svelte.ts`
- Modify: `src/lib/stores/plugins.svelte.ts`
- Modify: `src/lib/stores/skills.svelte.ts`
- Modify: `src/lib/stores/memory.svelte.ts`
- Modify: `src/lib/stores/mcp.svelte.ts`

- [ ] **Step 1: Add reset() to configStore**

Add at end of `ConfigStore` class (before the closing `}`):

```typescript
  reset(): void {
    this.userSettings = {} as Settings;
    this.projectSettings = {} as Settings;
    this.loading = false;
    this.saving = false;
    this.error = "";
    this.activeScope = "user";
    this.isDirty = false;
  }
```

- [ ] **Step 2: Add reset() to projectsStore**

Add at end of `ProjectsStore` class:

```typescript
  reset(): void {
    this.projects = [];
    this.activeProjectId = null;
    this.loading = false;
  }
```

- [ ] **Step 3: Add reset() to pluginsStore**

Add at end of `PluginsStore` class:

```typescript
  reset(): void {
    this.plugins = [];
    this.marketplaces = [];
    this.availablePlugins = [];
    this.loading = false;
    this.error = "";
  }
```

- [ ] **Step 4: Add reset() to skillsStore**

Add at end of `SkillsStore` class:

```typescript
  reset(): void {
    this.skills = [];
    this.selectedSkillId = null;
    this.loading = false;
    this.error = "";
  }
```

- [ ] **Step 5: Add reset() to memoryStore**

Add at end of `MemoryStore` class:

```typescript
  reset(): void {
    this.projects = [];
    this.activeProjectId = null;
    this.files = [];
    this.activeFile = null;
    this.loading = false;
    this.saving = false;
    this.error = "";
  }
```

- [ ] **Step 6: Add reset() to mcpStore**

Add at end of `McpStore` class:

```typescript
  reset(): void {
    this.servers = [];
    this.loading = false;
    this.error = "";
  }
```

- [ ] **Step 7: Verify frontend compiles**

Run: `pnpm build`
Expected: BUILD SUCCESS

- [ ] **Step 8: Commit**

```bash
git add src/lib/stores/config.svelte.ts src/lib/stores/projects.svelte.ts src/lib/stores/plugins.svelte.ts src/lib/stores/skills.svelte.ts src/lib/stores/memory.svelte.ts src/lib/stores/mcp.svelte.ts
git commit -m "feat(stores): add reset() method to all data stores for connection switching"
```

---

## Task 7: Update Connection Store — Reset and Switch Integration

**Files:**
- Modify: `src/lib/stores/connection.svelte.ts`

- [ ] **Step 1: Add resetAllStores and switchConnection methods**

Replace the entire file:

```typescript
import { DaemonClient } from "$lib/api/client.js";
import { DaemonWsClient } from "$lib/api/ws.js";
import { configStore } from "./config.svelte.js";
import { projectsStore } from "./projects.svelte.js";
import { pluginsStore } from "./plugins.svelte.js";
import { skillsStore } from "./skills.svelte.js";
import { memoryStore } from "./memory.svelte.js";
import { mcpStore } from "./mcp.svelte.js";
import { connectionsStore } from "./connections.svelte.js";
import type { ConnectionEntry } from "$lib/api/types.js";

type ConnectionStatus = "disconnected" | "connecting" | "connected";

class ConnectionStore {
  status = $state<ConnectionStatus>("disconnected");
  daemonVersion = $state<string>("");
  error = $state<string>("");

  client: DaemonClient | null = null;
  wsClient: DaemonWsClient | null = null;

  async connect(baseUrl: string, token: string): Promise<void> {
    this.status = "connecting";
    this.error = "";

    try {
      const client = new DaemonClient(baseUrl, token);
      const wsClient = new DaemonWsClient(baseUrl, token);

      // Verify connectivity via health check
      const health = await client.health();

      // Set up handler for the "connected" WS event before connecting
      wsClient.onEvent((event) => {
        if (event.type === "connected") {
          this.daemonVersion = event.daemonVersion;
        }
      });

      wsClient.connect();

      this.client = client;
      this.wsClient = wsClient;
      this.daemonVersion = health.version;
      this.status = "connected";
    } catch (err) {
      this.status = "disconnected";
      this.error = err instanceof Error ? err.message : String(err);
    }
  }

  disconnect(): void {
    this.wsClient?.disconnect();
    this.wsClient = null;
    this.client = null;
    this.status = "disconnected";
    this.daemonVersion = "";
  }

  resetAllStores(): void {
    configStore.reset();
    projectsStore.reset();
    pluginsStore.reset();
    skillsStore.reset();
    memoryStore.reset();
    mcpStore.reset();
  }

  async switchConnection(entry: ConnectionEntry): Promise<void> {
    this.disconnect();
    this.resetAllStores();
    await connectionsStore.setActive(entry.id);
    await this.connect(entry.url, entry.token);
  }
}

export const connectionStore = new ConnectionStore();
```

- [ ] **Step 2: Verify frontend compiles**

Run: `pnpm build`
Expected: BUILD SUCCESS

- [ ] **Step 3: Commit**

```bash
git add src/lib/stores/connection.svelte.ts
git commit -m "feat(connection): add resetAllStores and switchConnection for multi-daemon support"
```

---

## Task 8: Migrate App Settings Store to Config Directory

**Files:**
- Modify: `src/lib/stores/appsettings.svelte.ts`

- [ ] **Step 1: Rewrite to use Tauri IPC instead of localStorage**

Replace the entire file:

```typescript
import { invoke } from "@tauri-apps/api/core";
import type { AppConfig } from "$lib/api/types.js";

class AppSettingsStore {
  preferences = $state<AppConfig>({
    theme: "system",
    language: "zh-CN",
    fontSize: 14,
  });

  async load(): Promise<void> {
    try {
      const json = await invoke<string>("read_app_config");
      const saved: AppConfig = JSON.parse(json);
      this.preferences = { ...this.preferences, ...saved };
    } catch {
      // Use defaults on error
    }

    // One-time migration from localStorage
    try {
      const legacy = localStorage.getItem("dot-claude-gui-preferences");
      if (legacy) {
        const parsed = JSON.parse(legacy);
        // Only migrate theme/language/fontSize, not daemon fields
        if (parsed.theme) this.preferences.theme = parsed.theme;
        if (parsed.language) this.preferences.language = parsed.language;
        if (parsed.fontSize) this.preferences.fontSize = parsed.fontSize;
        await this.save();
        localStorage.removeItem("dot-claude-gui-preferences");
      }
    } catch {
      // Ignore migration errors
    }
  }

  async save(): Promise<void> {
    try {
      await invoke("write_app_config", {
        json: JSON.stringify(this.preferences, null, 2),
      });
    } catch {
      // Silently fail — preferences are not critical
    }
  }

  async update(partial: Partial<AppConfig>): Promise<void> {
    this.preferences = { ...this.preferences, ...partial };
    await this.save();
  }
}

export const appSettingsStore = new AppSettingsStore();
```

- [ ] **Step 2: Verify frontend compiles**

Run: `pnpm build`
Expected: BUILD SUCCESS

- [ ] **Step 3: Commit**

```bash
git add src/lib/stores/appsettings.svelte.ts
git commit -m "feat(appsettings): migrate from localStorage to ~/.dot-claude-gui/config.json"
```

---

## Task 9: Environment Selector Component

**Files:**
- Create: `src/lib/components/shared/EnvironmentSelector.svelte`

- [ ] **Step 1: Create the environment selector dropdown**

```svelte
<script lang="ts">
  import { connectionsStore } from "$lib/stores/connections.svelte.js";
  import { connectionStore } from "$lib/stores/connection.svelte.js";
  import type { ConnectionEntry } from "$lib/api/types.js";

  let open = $state(false);

  function statusColor(entry: ConnectionEntry): string {
    if (entry.id === connectionsStore.activeConnectionId) {
      if (connectionStore.status === "connected") return "bg-green-400";
      if (connectionStore.status === "connecting") return "bg-yellow-400 animate-pulse";
      return "bg-red-400";
    }
    return "bg-gray-500";
  }

  async function selectConnection(entry: ConnectionEntry) {
    open = false;
    if (entry.id === connectionsStore.activeConnectionId) return;
    await connectionStore.switchConnection(entry);
  }

  function handleManage() {
    open = false;
    // Dispatch event to navigate to App Settings > Connections
    window.dispatchEvent(new CustomEvent("navigate", { detail: { nav: "A", sub: "connections" } }));
  }
</script>

<div class="relative">
  <button
    class="flex items-center gap-2 px-3 py-1.5 rounded-md hover:bg-gray-800 text-sm"
    onclick={() => (open = !open)}
  >
    <span class="w-2 h-2 rounded-full {statusColor(connectionsStore.activeConnection!)}"></span>
    <span class="text-gray-200">{connectionsStore.activeConnection?.name ?? "Local"}</span>
    <svg class="w-3 h-3 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
    </svg>
  </button>

  {#if open}
    <!-- Backdrop -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="fixed inset-0 z-40" onclick={() => (open = false)} onkeydown={() => {}}></div>

    <div class="absolute top-full left-0 mt-1 w-56 bg-gray-800 border border-gray-700 rounded-lg shadow-xl z-50 py-1">
      {#each connectionsStore.connections as entry}
        <button
          class="w-full flex items-center gap-2 px-3 py-2 text-sm text-left hover:bg-gray-700
            {entry.id === connectionsStore.activeConnectionId ? 'text-blue-400' : 'text-gray-300'}"
          onclick={() => selectConnection(entry)}
        >
          <span class="w-2 h-2 rounded-full {statusColor(entry)}"></span>
          <span class="flex-1">{entry.name}</span>
          {#if entry.managed}
            <span class="text-xs text-gray-500">自动</span>
          {/if}
        </button>
      {/each}

      <div class="border-t border-gray-700 my-1"></div>

      <button
        class="w-full flex items-center gap-2 px-3 py-2 text-sm text-left text-gray-400 hover:bg-gray-700"
        onclick={handleManage}
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
        </svg>
        <span>管理连接...</span>
      </button>
    </div>
  {/if}
</div>
```

- [ ] **Step 2: Commit**

```bash
git add src/lib/components/shared/EnvironmentSelector.svelte
git commit -m "feat(ui): add EnvironmentSelector dropdown component"
```

---

## Task 10: Refactor Project Selector (ScopeSelector → ProjectSelector)

**Files:**
- Modify: `src/lib/components/shared/ScopeSelector.svelte`

- [ ] **Step 1: Rewrite ScopeSelector as a project selector with "User Scope" and "添加项目"**

Replace the entire file:

```svelte
<script lang="ts">
  import { projectsStore } from "$lib/stores/projects.svelte.js";
  import { configStore } from "$lib/stores/config.svelte.js";

  let open = $state(false);
  let showAddProject = $state(false);
  let newProjectPath = $state("");
  let addError = $state("");

  function selectUserScope() {
    open = false;
    projectsStore.selectProject(null);
    configStore.activeScope = "user";
    configStore.loadUserConfig();
  }

  function selectProject(id: string) {
    open = false;
    projectsStore.selectProject(id);
    configStore.activeScope = "project";
    configStore.loadProjectConfig(id);
  }

  async function addProject() {
    if (!newProjectPath.trim()) return;
    addError = "";
    try {
      await projectsStore.registerProject(newProjectPath.trim());
      newProjectPath = "";
      showAddProject = false;
    } catch (err) {
      addError = err instanceof Error ? err.message : String(err);
    }
  }

  const displayName = $derived(
    projectsStore.activeProjectId
      ? projectsStore.activeProject?.path.replace(/^.*\//, "") ?? "Project"
      : "User Scope"
  );

  const displayIcon = $derived(projectsStore.activeProjectId ? "📁" : "🏠");
</script>

<div class="relative">
  <button
    class="flex items-center gap-2 px-3 py-1.5 rounded-md hover:bg-gray-800 text-sm"
    onclick={() => (open = !open)}
  >
    <span>{displayIcon}</span>
    <span class="text-gray-200">{displayName}</span>
    <svg class="w-3 h-3 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
    </svg>
  </button>

  {#if open}
    <!-- Backdrop -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="fixed inset-0 z-40" onclick={() => (open = false)} onkeydown={() => {}}></div>

    <div class="absolute top-full left-0 mt-1 w-72 bg-gray-800 border border-gray-700 rounded-lg shadow-xl z-50 py-1">
      <!-- User Scope -->
      <button
        class="w-full flex items-center gap-2 px-3 py-2 text-sm text-left hover:bg-gray-700
          {!projectsStore.activeProjectId ? 'text-blue-400' : 'text-gray-300'}"
        onclick={selectUserScope}
      >
        <span>🏠</span>
        <span>User Scope</span>
      </button>

      {#if projectsStore.projects.length > 0}
        <div class="border-t border-gray-700 my-1"></div>
      {/if}

      <!-- Project list -->
      {#each projectsStore.projects as project}
        <button
          class="w-full flex items-center gap-2 px-3 py-2 text-sm text-left hover:bg-gray-700
            {projectsStore.activeProjectId === project.id ? 'text-blue-400' : 'text-gray-300'}"
          onclick={() => selectProject(project.id)}
        >
          <span>📁</span>
          <span class="flex-1 truncate">{project.path}</span>
        </button>
      {/each}

      <div class="border-t border-gray-700 my-1"></div>

      <!-- Add project -->
      {#if showAddProject}
        <div class="px-3 py-2">
          <div class="flex gap-2">
            <input
              type="text"
              bind:value={newProjectPath}
              placeholder="/path/to/project"
              class="flex-1 bg-gray-900 border border-gray-600 rounded px-2 py-1 text-sm text-gray-200"
              onkeydown={(e) => e.key === "Enter" && addProject()}
            />
            <button
              class="px-2 py-1 bg-blue-600 text-white text-sm rounded hover:bg-blue-500"
              onclick={addProject}
            >
              添加
            </button>
          </div>
          {#if addError}
            <p class="text-xs text-red-400 mt-1">{addError}</p>
          {/if}
        </div>
      {:else}
        <button
          class="w-full flex items-center gap-2 px-3 py-2 text-sm text-left text-gray-400 hover:bg-gray-700"
          onclick={() => (showAddProject = true)}
        >
          <span>+</span>
          <span>添加项目...</span>
        </button>
      {/if}
    </div>
  {/if}
</div>
```

- [ ] **Step 2: Verify frontend compiles**

Run: `pnpm build`
Expected: BUILD SUCCESS

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/shared/ScopeSelector.svelte
git commit -m "feat(ui): refactor ScopeSelector into ProjectSelector with User Scope and add-project"
```

---

## Task 11: Connections Management Panel in App Settings

**Files:**
- Create: `src/lib/components/appsettings/ConnectionsPanel.svelte`
- Modify: `src/lib/components/appsettings/AppSettingsView.svelte`

- [ ] **Step 1: Create ConnectionsPanel component**

```svelte
<script lang="ts">
  import { connectionsStore } from "$lib/stores/connections.svelte.js";
  import { DaemonClient } from "$lib/api/client.js";
  import type { ConnectionEntry } from "$lib/api/types.js";

  let editing = $state<ConnectionEntry | null>(null);
  let isNew = $state(false);

  // Form fields
  let formName = $state("");
  let formUrl = $state("");
  let formToken = $state("");
  let showToken = $state(false);

  // Test connection
  let testStatus = $state<"idle" | "testing" | "ok" | "error">("idle");
  let testMessage = $state("");

  function startAdd() {
    isNew = true;
    formName = "";
    formUrl = "http://";
    formToken = "";
    testStatus = "idle";
    editing = {} as ConnectionEntry;
  }

  function startEdit(entry: ConnectionEntry) {
    isNew = false;
    formName = entry.name;
    formUrl = entry.url;
    formToken = entry.token;
    testStatus = "idle";
    editing = entry;
  }

  function cancelEdit() {
    editing = null;
    isNew = false;
  }

  async function saveEdit() {
    if (isNew) {
      await connectionsStore.addConnection({
        name: formName,
        type: "remote",
        url: formUrl.replace(/\/$/, ""),
        token: formToken,
        managed: false,
      });
    } else if (editing) {
      await connectionsStore.updateConnection(editing.id, {
        name: formName,
        url: formUrl.replace(/\/$/, ""),
        token: formToken,
      });
    }
    editing = null;
    isNew = false;
  }

  async function deleteConnection(id: string) {
    await connectionsStore.deleteConnection(id);
  }

  async function testConnection() {
    testStatus = "testing";
    testMessage = "";
    try {
      const client = new DaemonClient(formUrl.replace(/\/$/, ""), formToken);
      const health = await client.health();
      testStatus = "ok";
      testMessage = `连接成功 (v${health.version})`;
    } catch (err) {
      testStatus = "error";
      testMessage = err instanceof Error ? err.message : String(err);
    }
  }
</script>

<div class="space-y-4">
  <h2 class="text-lg font-medium text-gray-100">连接管理</h2>

  <!-- Connection list -->
  <div class="space-y-2">
    {#each connectionsStore.connections as entry}
      <div class="flex items-center justify-between p-3 bg-gray-800 rounded-lg">
        <div>
          <div class="flex items-center gap-2">
            <span class="text-sm font-medium text-gray-200">{entry.name}</span>
            {#if entry.managed}
              <span class="text-xs px-1.5 py-0.5 bg-gray-700 text-gray-400 rounded">自动管理</span>
            {/if}
          </div>
          <div class="text-xs text-gray-500 mt-0.5">{entry.url}</div>
        </div>
        <div class="flex gap-2">
          {#if !entry.managed}
            <button
              class="px-2 py-1 text-xs text-gray-400 hover:text-gray-200"
              onclick={() => startEdit(entry)}
            >
              编辑
            </button>
            <button
              class="px-2 py-1 text-xs text-red-400 hover:text-red-300"
              onclick={() => deleteConnection(entry.id)}
            >
              删除
            </button>
          {/if}
        </div>
      </div>
    {/each}
  </div>

  <!-- Add button -->
  {#if !editing}
    <button
      class="px-3 py-2 text-sm bg-blue-600 text-white rounded-md hover:bg-blue-500"
      onclick={startAdd}
    >
      + 添加连接
    </button>
  {/if}

  <!-- Edit / Add form -->
  {#if editing}
    <div class="p-4 bg-gray-800 rounded-lg border border-gray-700 space-y-3">
      <h3 class="text-sm font-medium text-gray-200">
        {isNew ? "新建连接" : "编辑连接"}
      </h3>

      <div>
        <label class="block text-xs text-gray-400 mb-1">名称</label>
        <input
          type="text"
          bind:value={formName}
          placeholder="Docker Dev"
          class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm text-gray-200"
        />
      </div>

      <div>
        <label class="block text-xs text-gray-400 mb-1">URL</label>
        <input
          type="text"
          bind:value={formUrl}
          placeholder="http://192.168.1.100:7890"
          class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm text-gray-200"
        />
      </div>

      <div>
        <label class="block text-xs text-gray-400 mb-1">Token</label>
        <div class="flex gap-2">
          <input
            type={showToken ? "text" : "password"}
            bind:value={formToken}
            class="flex-1 bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm text-gray-200"
          />
          <button
            class="px-2 text-sm text-gray-400 hover:text-gray-200"
            onclick={() => (showToken = !showToken)}
          >
            {showToken ? "隐藏" : "显示"}
          </button>
        </div>
      </div>

      <!-- Test connection -->
      <div class="flex items-center gap-3">
        <button
          class="px-3 py-1.5 text-sm bg-gray-700 text-gray-200 rounded hover:bg-gray-600"
          onclick={testConnection}
          disabled={testStatus === "testing"}
        >
          {testStatus === "testing" ? "测试中..." : "测试连接"}
        </button>
        {#if testStatus === "ok"}
          <span class="text-sm text-green-400">✅ {testMessage}</span>
        {:else if testStatus === "error"}
          <span class="text-sm text-red-400">❌ {testMessage}</span>
        {/if}
      </div>

      <!-- Actions -->
      <div class="flex justify-end gap-2 pt-2 border-t border-gray-700">
        <button
          class="px-3 py-1.5 text-sm text-gray-400 hover:text-gray-200"
          onclick={cancelEdit}
        >
          取消
        </button>
        <button
          class="px-3 py-1.5 text-sm bg-blue-600 text-white rounded hover:bg-blue-500"
          onclick={saveEdit}
          disabled={!formName.trim() || !formUrl.trim()}
        >
          保存
        </button>
      </div>
    </div>
  {/if}
</div>
```

- [ ] **Step 2: Update AppSettingsView to use ConnectionsPanel**

Read the current `AppSettingsView.svelte` and replace the Connection section (the `<section>` containing "Daemon URL" and "Token" inputs) with a reference to `ConnectionsPanel`. Also replace the inline project management with a simplified version since projects are now managed via the header.

Replace the entire file content:

```svelte
<script lang="ts">
  import { appSettingsStore } from "$lib/stores/appsettings.svelte.js";
  import ConnectionsPanel from "./ConnectionsPanel.svelte";

  let { activeSub = "appearance" }: { activeSub?: string } = $props();
</script>

<div class="p-6 space-y-8">
  {#if activeSub === "connections"}
    <ConnectionsPanel />
  {:else}
    <!-- Appearance settings -->
    <section class="space-y-4">
      <h2 class="text-lg font-medium text-gray-100">外观</h2>

      <div>
        <label class="block text-sm text-gray-400 mb-1">主题</label>
        <select
          class="bg-gray-800 border border-gray-600 rounded px-3 py-1.5 text-sm text-gray-200"
          value={appSettingsStore.preferences.theme}
          onchange={(e) => appSettingsStore.update({ theme: (e.target as HTMLSelectElement).value as "light" | "dark" | "system" })}
        >
          <option value="system">跟随系统</option>
          <option value="dark">深色</option>
          <option value="light">浅色</option>
        </select>
      </div>

      <div>
        <label class="block text-sm text-gray-400 mb-1">字体大小: {appSettingsStore.preferences.fontSize}px</label>
        <input
          type="range"
          min="12"
          max="20"
          value={appSettingsStore.preferences.fontSize}
          class="w-48"
          oninput={(e) => appSettingsStore.update({ fontSize: parseInt((e.target as HTMLInputElement).value) })}
        />
      </div>
    </section>
  {/if}
</div>
```

- [ ] **Step 3: Verify frontend compiles**

Run: `pnpm build`
Expected: BUILD SUCCESS

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/appsettings/ConnectionsPanel.svelte src/lib/components/appsettings/AppSettingsView.svelte
git commit -m "feat(ui): add ConnectionsPanel and simplify AppSettingsView"
```

---

## Task 12: Update App.svelte — Header Hierarchy and Connection Flow

**Files:**
- Modify: `src/App.svelte`

- [ ] **Step 1: Read the current App.svelte to understand exact structure**

Read the file to identify the header section, the onMount initialization, and the navigation event handling. The key changes are:

1. Import new stores and components
2. Replace the header with environment selector → project selector
3. Update onMount to load connections first, then connect
4. Handle the "navigate" custom event from EnvironmentSelector
5. Add "connections" sub-navigation in App Settings

- [ ] **Step 2: Update imports**

Add to the import section at the top of the `<script>` block:

```typescript
import { connectionsStore } from "$lib/stores/connections.svelte.js";
import EnvironmentSelector from "$lib/components/shared/EnvironmentSelector.svelte";
```

Remove (if present):
```typescript
// Remove any direct import of appSettingsStore for daemon URL/token usage
```

- [ ] **Step 3: Update onMount initialization**

Replace the current connection initialization logic (which reads daemon URL/token from appSettingsStore) with:

```typescript
  onMount(async () => {
    // Load app preferences
    await appSettingsStore.load();

    // Load connection registry from ~/.dot-claude-gui/connections.json
    await connectionsStore.load();

    // Connect to the active daemon
    const active = connectionsStore.activeConnection;
    if (active) {
      await connectionStore.connect(active.url, active.token);
    }
  });
```

- [ ] **Step 4: Add effect to load data after connection**

Add an `$effect` that triggers when connection status changes to "connected":

```typescript
  $effect(() => {
    if (connectionStore.status === "connected" && connectionStore.client) {
      // Load all data from connected daemon
      configStore.loadUserConfig();
      projectsStore.loadProjects();
      pluginsStore.loadPlugins();
      pluginsStore.loadMarketplaces();
      skillsStore.loadSkills();
      memoryStore.loadProjects();
      mcpStore.loadServers();

      // Subscribe to config changes
      connectionStore.wsClient?.onEvent((event) => {
        if (event.type === "configChanged") {
          configStore.loadUserConfig();
          if (projectsStore.activeProjectId) {
            configStore.loadProjectConfig(projectsStore.activeProjectId);
          }
        }
      });
    }
  });
```

- [ ] **Step 5: Handle navigate custom event**

Add event listener for the EnvironmentSelector's "manage connections" navigation:

```typescript
  onMount(() => {
    const handleNavigate = (e: CustomEvent<{ nav: string; sub?: string }>) => {
      activeNav = e.detail.nav;
      if (e.detail.sub) {
        activeSettingsSub = e.detail.sub;
      }
    };
    window.addEventListener("navigate", handleNavigate as EventListener);
    return () => window.removeEventListener("navigate", handleNavigate as EventListener);
  });
```

- [ ] **Step 6: Update header HTML**

Replace the current header content with the two-level selector:

```svelte
  <!-- Header -->
  <header class="h-12 bg-gray-900 border-b border-gray-800 flex items-center justify-between px-4">
    <span class="text-sm font-semibold text-gray-300">dot-claude-gui</span>

    <div class="flex items-center gap-2">
      <EnvironmentSelector />
      <span class="text-gray-600">→</span>
      <ScopeSelector />
    </div>

    <ConnectionStatus />
  </header>
```

- [ ] **Step 7: Add "connections" sub-navigation for App Settings**

In the sidebar sub-panel for App Settings, add a "连接" entry:

```svelte
  {:else if activeNav === "A"}
    <button
      class="w-full text-left px-3 py-2 text-sm rounded {activeSettingsSub === 'appearance' ? 'bg-gray-700 text-white' : 'text-gray-400 hover:bg-gray-800'}"
      onclick={() => (activeSettingsSub = "appearance")}
    >
      外观
    </button>
    <button
      class="w-full text-left px-3 py-2 text-sm rounded {activeSettingsSub === 'connections' ? 'bg-gray-700 text-white' : 'text-gray-400 hover:bg-gray-800'}"
      onclick={() => (activeSettingsSub = "connections")}
    >
      连接
    </button>
```

Update the detail panel for App Settings to pass `activeSub`:

```svelte
  {:else if activeNav === "A"}
    <AppSettingsView activeSub={activeSettingsSub} />
```

Add state variable:

```typescript
  let activeSettingsSub = $state("appearance");
```

- [ ] **Step 8: Verify frontend compiles**

Run: `pnpm build`
Expected: BUILD SUCCESS

- [ ] **Step 9: Commit**

```bash
git add src/App.svelte
git commit -m "feat(app): add environment/project header selectors and connection switching"
```

---

## Task 13: Build Verification and Integration Test

**Files:** None (verification only)

- [ ] **Step 1: Build the daemon**

Run: `cargo build -p claude-daemon`
Expected: BUILD SUCCESS

- [ ] **Step 2: Run all Rust tests**

Run: `cargo test --workspace`
Expected: All tests pass

- [ ] **Step 3: Build the frontend**

Run: `pnpm build`
Expected: BUILD SUCCESS

- [ ] **Step 4: Build the full Tauri app**

First, ensure the sidecar binary is in place:

```bash
cargo build -p claude-daemon
mkdir -p src-tauri/binaries
cp target/debug/claude-daemon src-tauri/binaries/claude-daemon-aarch64-apple-darwin
```

Then build:

Run: `pnpm tauri build`
Expected: BUILD SUCCESS, `.app` bundle created

- [ ] **Step 5: Launch and verify**

Launch the built app. Verify:
1. No panic on startup
2. Sidecar daemon starts (check with `lsof -i :7890` or nearby port)
3. GUI connects to daemon (green status indicator)
4. `~/.dot-claude-gui/connections.json` created with local connection entry
5. Local `~/.claude/` data loads and displays correctly in Settings view
6. Environment selector shows "Local" with green dot
7. Project selector shows "User Scope" by default
8. App Settings > Connections shows the local connection

- [ ] **Step 6: Test remote connection flow**

In App Settings > Connections:
1. Click "添加连接"
2. Fill in name, URL, token for a test endpoint
3. Click "测试连接" — verify success/failure message
4. Save and verify it appears in the connection list
5. Verify it appears in the environment selector dropdown

- [ ] **Step 7: Commit any final fixes**

```bash
git add -A
git commit -m "fix: integration fixes for multi-daemon and config directory"
```

---

## Task 14: Final Merge Commit

- [ ] **Step 1: Verify clean state**

Run: `cargo test --workspace && pnpm build`
Expected: All pass

- [ ] **Step 2: Tag the milestone**

```bash
git tag -a phase5-multi-daemon -m "Phase 5: Multi-daemon connections and config directory"
```
