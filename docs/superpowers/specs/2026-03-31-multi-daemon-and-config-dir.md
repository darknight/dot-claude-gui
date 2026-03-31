# Multi-Daemon Connections & Application Config Directory

**Date:** 2026-03-31
**Status:** Draft
**Author:** Eric Yao + Claude

---

## 1. Problem Statement

The current MVP has three limitations:

1. **No persistent app config directory.** App preferences live in `localStorage`, daemon tokens in `~/Library/Application Support/com.dotclaude.gui/daemon-token`. Software updates or browser cache clears lose all state.
2. **Single daemon connection.** The GUI hardcodes `http://127.0.0.1:7890`. Users running Claude Code in multiple environments (local macOS, Docker containers, remote servers) cannot manage them from one GUI.
3. **No daemon lifecycle management.** Users must manually start `claude-daemon` before launching the GUI.

### Target Scenario

User has:
- Local macOS environment
- Two Docker containers running Claude Code

They start the GUI once. It auto-launches a local daemon (sidecar). They add two remote daemon connections (Docker instances). They switch between environments in the header, managing each one's config, plugins, skills, and MCP servers.

---

## 2. Application Config Directory

### 2.1 Location

```
~/.dot-claude-gui/
```

A hidden directory in the user's home, following the convention of `~/.docker`, `~/.npm`, etc.

### 2.2 Directory Structure

```
~/.dot-claude-gui/
├── config.json          # App preferences (theme, language, font size)
├── connections.json     # Daemon connection registry (urls, tokens, active connection)
└── logs/                # Reserved for future daemon log storage
```

### 2.3 config.json

Migrated from `localStorage`. Stores GUI-only preferences that are independent of any daemon.

```json
{
  "theme": "dark",
  "language": "zh-CN",
  "fontSize": 14
}
```

### 2.4 connections.json

Central registry of all daemon connections and their auth tokens.

```json
{
  "activeConnectionId": "local",
  "connections": [
    {
      "id": "local",
      "name": "Local",
      "type": "local",
      "url": "http://127.0.0.1:52431",
      "token": "base64-auto-generated-token",
      "managed": true
    },
    {
      "id": "docker-dev",
      "name": "Docker Dev",
      "type": "remote",
      "url": "http://192.168.1.100:7890",
      "token": "user-provided-token",
      "managed": false
    },
    {
      "id": "docker-prod",
      "name": "Docker Prod",
      "type": "remote",
      "url": "http://10.0.0.5:7890",
      "token": "user-provided-token",
      "managed": false
    }
  ]
}
```

**Field definitions:**

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique identifier. `"local"` is reserved for the sidecar connection. |
| `name` | string | User-facing display name. |
| `type` | `"local" \| "remote"` | Local = sidecar-managed. Remote = user-managed. |
| `url` | string | Full daemon URL including port. |
| `token` | string | Bearer auth token. |
| `managed` | boolean | If true, lifecycle managed by Tauri (start/stop with app). |

**Invariants:**
- Exactly one connection with `id: "local"` must always exist; it cannot be deleted.
- `activeConnectionId` must reference a valid connection id.
- On first launch (no `connections.json` exists), create the file with only the local connection.

### 2.5 Migration from Current State

- `localStorage` preferences → `~/.dot-claude-gui/config.json` (one-time migration on first launch)
- `~/Library/Application Support/com.dotclaude.gui/daemon-token` → no longer used; token stored in `connections.json`
- `appsettings.svelte.ts` default `daemonUrl`/`daemonToken` fields → removed, replaced by connections store

---

## 3. Sidecar Daemon Lifecycle

### 3.1 Startup Flow

```
Tauri app starts
  → Read or create ~/.dot-claude-gui/connections.json
  → Generate random 256-bit auth token
  → Find available port (bind to port 0 and read assigned port, or try ports sequentially)
  → Spawn claude-daemon sidecar:
      claude-daemon --port {port} --token {token} --bind 127.0.0.1
  → Wait for health check (GET /api/v1/health) with retry (max 5s)
  → Update local connection entry in connections.json: url, token
  → Frontend reads connections.json → connects to local daemon
```

### 3.2 Shutdown Flow

```
Tauri app exits (window close or quit)
  → Kill sidecar child process (SIGTERM, then SIGKILL after timeout)
  → No cleanup needed in connections.json (url/token are ephemeral per session)
```

### 3.3 Tauri Sidecar Configuration

**tauri.conf.json** — declare the sidecar binary:

```json
{
  "bundle": {
    "externalBin": ["binaries/claude-daemon"]
  }
}
```

The `claude-daemon` binary is built separately and placed in the `src-tauri/binaries/` directory with platform-specific naming (`claude-daemon-aarch64-apple-darwin`, etc.).

**src-tauri/src/lib.rs** — spawn and manage the sidecar:

```rust
use tauri_plugin_shell::ShellExt;

// In setup hook:
let sidecar = app.shell().sidecar("claude-daemon")
    .unwrap()
    .args(["--port", &port.to_string(), "--token", &token, "--bind", "127.0.0.1"])
    .spawn()
    .expect("failed to spawn sidecar");
```

### 3.4 Daemon CLI Changes

Add `--bind` parameter to `claude-daemon`:

```rust
#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value_t = 7890)]
    port: u16,

    #[arg(long, default_value = "127.0.0.1")]
    bind: String,

    #[arg(long)]
    claude_home: Option<PathBuf>,

    #[arg(long)]
    token: Option<String>,
}
```

- Remove the token-writing logic from daemon (`main.rs:60-67`). Token persistence is now the caller's responsibility.
- Bind address: `127.0.0.1` for local (security), `0.0.0.0` for Docker/remote.

---

## 4. Multi-Connection Management — GUI

### 4.1 Header Navigation Hierarchy

The header establishes a two-level context: **Environment → Project**.

```
┌─────────────────────────────────────────────────────────────────┐
│  [dot-claude-gui]        [🟢 Local ▾]  →  [🏠 User Scope ▾]   │
└─────────────────────────────────────────────────────────────────┘
```

**Environment selector (left dropdown):**
```
┌──────────────────────┐
│ 🟢 Local             │  ← auto-managed, always present
│ 🔴 Docker Dev        │
│ 🟡 Docker Prod       │
├──────────────────────┤
│ ⚙️ 管理连接...        │  → navigates to App Settings > Connections
└──────────────────────┘
```

Status indicators:
- 🟢 Green = connected (WebSocket active)
- 🟡 Yellow = connecting / reconnecting
- 🔴 Red = disconnected / unreachable

**Project selector (right dropdown):**
```
┌──────────────────────────┐
│ 🏠 User Scope            │  ← default after environment switch
│ ~/workspace/react-app    │
│ ~/workspace/api-server   │
├──────────────────────────┤
│ + 添加项目...             │  → calls POST /api/v1/projects with path input
└──────────────────────────┘
```

- "User Scope" is a special entry representing `~/.claude/` user-level config (no project selected).
- Project list is loaded from the connected daemon's `GET /api/v1/projects`.
- "添加项目..." opens an inline path input (for remote daemons, user types the remote path).

### 4.2 Connection Switch Data Flow

```
User selects new environment
  → connectionStore.disconnect()
      → Close WebSocket
      → Null out DaemonClient
      → Reset all stores to initial state:
          configStore, projectsStore, pluginsStore, skillsStore,
          memoryStore, mcpStore
  → Update connections.json: activeConnectionId = newId
  → connectionStore.connect(newUrl, newToken)
      → Health check → WebSocket connect
  → projectsStore.load()  // fetch project list from new daemon
  → Auto-select "User Scope" (no project)
  → configStore.loadUserConfig()  // load user-scope config
```

```
User selects project (or User Scope)
  → projectsStore.setActiveProject(projectId | null)
  → configStore.loadProjectConfig(projectId)  // or user config if null
  → pluginsStore.load(), skillsStore.load(), etc.
  → All detail panels re-render with new scope
```

### 4.3 App Settings — Connections Panel

A new sub-panel under App Settings module.

**Connection List:**

```
┌─ 连接管理 ──────────────────────────────────────┐
│                                                  │
│  🟢 Local (自动管理)              [不可删除]       │
│     http://127.0.0.1:52431                       │
│                                                  │
│  🔴 Docker Dev                   [编辑] [删除]    │
│     http://192.168.1.100:7890                    │
│                                                  │
│  🔴 Docker Prod                  [编辑] [删除]    │
│     http://10.0.0.5:7890                         │
│                                                  │
│  [+ 添加连接]                                     │
└──────────────────────────────────────────────────┘
```

**Add/Edit Connection Form:**

```
┌─ 新建连接 ──────────────────────────────────────┐
│                                                  │
│  名称:    [Docker Dev                    ]       │
│  URL:     [http://192.168.1.100:7890     ]       │
│  Token:   [••••••••••••••••••••••••••    ] 👁     │
│                                                  │
│  [测试连接]  →  ✅ 连接成功 (v0.1.0)              │
│                                                  │
│                          [取消]  [保存]           │
└──────────────────────────────────────────────────┘
```

- "测试连接" calls `GET /api/v1/health` with the provided URL and token.
- Success shows daemon version; failure shows error message.
- Local connection is read-only (shows current URL/token but cannot edit).

### 4.4 Frontend Changes Summary

| File | Change |
|------|--------|
| **New:** `src/lib/stores/connections.svelte.ts` | Connection registry store: CRUD connections, read/write `~/.dot-claude-gui/connections.json`, switch active connection |
| `src/lib/stores/appsettings.svelte.ts` | Remove `daemonUrl`/`daemonToken` fields. Migrate preferences to `~/.dot-claude-gui/config.json`. Read/write via Tauri IPC (filesystem access). |
| `src/lib/stores/connection.svelte.ts` | Add `resetAllStores()` that clears config, projects, plugins, skills, memory, mcp stores. |
| `src/App.svelte` | Add environment selector and project selector in header. Wire up connection switching. |
| **New:** `src/lib/components/shared/EnvironmentSelector.svelte` | Environment dropdown with status indicators. |
| **Modify:** `src/lib/components/shared/ScopeSelector.svelte` | Refactor: project selector now includes "User Scope" as first option and "添加项目..." at bottom. Moved to header. |
| `src/lib/components/appsettings/AppSettingsView.svelte` | Add Connections sub-panel with list, add/edit/delete forms, test connection button. |
| `src/lib/api/types.ts` | Add `ConnectionEntry`, `ConnectionsConfig`, `AppConfig` types. |
| `src-tauri/src/lib.rs` | Sidecar spawn logic, config dir initialization, IPC commands for reading/writing `~/.dot-claude-gui/` files. |

### 4.5 Tauri IPC Commands (New)

The frontend cannot directly access `~/.dot-claude-gui/` from the WebView. New Tauri commands bridge this:

```rust
#[tauri::command]
fn read_app_config() -> Result<String, String>;       // reads config.json

#[tauri::command]
fn write_app_config(json: String) -> Result<(), String>;

#[tauri::command]
fn read_connections() -> Result<String, String>;       // reads connections.json

#[tauri::command]
fn write_connections(json: String) -> Result<(), String>;

#[tauri::command]
fn get_config_dir() -> Result<String, String>;         // returns ~/.dot-claude-gui path
```

Existing `get_daemon_url()` and `get_daemon_token()` commands are removed; the frontend reads connection info from `connections.json` via `read_connections()`.

---

## 5. Daemon-Side Changes

### 5.1 CLI Parameter Changes

```diff
  #[derive(Parser, Debug)]
  struct Args {
      #[arg(long, default_value_t = 7890)]
      port: u16,

+     #[arg(long, default_value = "127.0.0.1")]
+     bind: String,

      #[arg(long)]
      claude_home: Option<PathBuf>,

      #[arg(long)]
      token: Option<String>,
  }
```

### 5.2 Remove Token File Writing

Delete the token-writing logic in `main.rs:60-67` (writing to `com.dotclaude.gui/daemon-token`). The token is now passed via `--token` flag and managed by the caller (Tauri or user).

### 5.3 Bind Address

```diff
- let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
+ let addr: SocketAddr = format!("{}:{}", args.bind, args.port).parse()?;
```

For Docker deployments, users run:
```bash
claude-daemon --port 7890 --bind 0.0.0.0 --token <secret>
```

---

## 6. Error Handling

### 6.1 Sidecar Failures

| Scenario | Handling |
|----------|----------|
| Daemon binary not found | Show error dialog: "claude-daemon binary not found. Please reinstall the application." |
| Port already in use | Retry with next port (up to 10 attempts) |
| Health check timeout (5s) | Show error: "Local daemon failed to start. Check logs." Allow using remote connections. |
| Daemon crashes during session | ConnectionStore detects WS disconnect → shows reconnecting status → attempts restart sidecar → if restart fails, show error banner |

### 6.2 Remote Connection Failures

| Scenario | Handling |
|----------|----------|
| Network unreachable | Status indicator shows 🔴, error message in tooltip |
| Auth token invalid | 401 response → show "Token invalid" error, prompt to update |
| Connection lost mid-session | Auto-reconnect with backoff (existing logic). Disable write ops during disconnect. |

### 6.3 Config Directory Failures

| Scenario | Handling |
|----------|----------|
| Cannot create `~/.dot-claude-gui/` | Fatal error dialog on startup |
| `connections.json` corrupted | Log warning, recreate with defaults (local connection only) |
| Permission denied on write | Show error toast, keep in-memory state |

---

## 7. Testing Strategy

### 7.1 Rust (Daemon) Tests

```
Bind address:
  - --bind 127.0.0.1 only accepts local connections
  - --bind 0.0.0.0 accepts any connection
  - Invalid bind address returns error

Token handling:
  - --token flag sets auth token correctly
  - No --token generates random token
  - No token file is written to disk
```

### 7.2 Tauri Shell Tests

```
Sidecar lifecycle:
  - Sidecar starts and health check passes
  - Sidecar port is written to connections.json
  - App exit kills sidecar process
  - Port conflict triggers retry with different port

Config directory:
  - ~/.dot-claude-gui/ created on first launch
  - config.json and connections.json created with defaults
  - IPC commands read/write correctly
  - Corrupted JSON handled gracefully
```

### 7.3 Frontend Tests

```
Connections store:
  - Load connections from file
  - Add remote connection persists to file
  - Delete remote connection persists to file
  - Cannot delete local connection
  - Switch connection updates activeConnectionId
  - Switch connection resets all stores

Environment selector:
  - Shows all connections with correct status indicators
  - Clicking connection triggers switch
  - "管理连接" navigates to App Settings

Project selector:
  - Shows "User Scope" as first option
  - Lists projects from connected daemon
  - "添加项目" calls register project API
  - Switching environment resets to User Scope
```

---

## 8. Scope Exclusions

Not in this iteration:
- TLS support for remote connections (future: `--tls-cert`, `--tls-key`)
- SSH tunnel auto-setup for Docker connections
- Connection health monitoring (periodic ping)
- Connection groups / folders
- Import/export connections
- Sync connections across machines
