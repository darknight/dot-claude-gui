# Phase 4: MCP Servers, Effective Config, Launcher, App Settings

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete the MVP by implementing the final four modules: MCP server management, Effective Config viewer with field-source annotations, Project Launcher with env var selection, and App Settings (theme, connections, project registration).

**Architecture:** MCP management delegates to `claude mcp` CLI commands via the existing executor. Effective Config uses the existing merge engine with source tracking. Launcher spawns `claude` processes with custom env vars. App Settings stores preferences in Tauri's app data directory via `tauri-plugin-store`.

**Tech Stack:** Rust (axum, tokio::process), Svelte 5, TypeScript, tauri-plugin-store

**Prerequisite:** Phase 3 complete (46 tests passing, local run verified)

---

## File Structure (Phase 4)

### Backend

```
crates/claude-types/src/
└── mcp.rs                          # MCP server types

crates/claude-daemon/src/api/
├── mcp.rs                          # MCP server CRUD endpoints
└── launcher.rs                     # Launch claude process endpoint
```

### Frontend

```
src/lib/
├── stores/
│   ├── mcp.svelte.ts               # MCP server state
│   └── appsettings.svelte.ts       # App preferences state
├── components/
│   ├── mcp/
│   │   ├── McpModule.svelte         # Orchestrator
│   │   ├── McpServerList.svelte     # Server list grouped by scope
│   │   └── McpServerEditor.svelte   # Add/edit server form
│   ├── effective/
│   │   └── EffectiveConfigView.svelte  # Merged config with source annotations
│   ├── launcher/
│   │   └── LauncherView.svelte      # Project launcher with env selection
│   └── appsettings/
│       └── AppSettingsView.svelte   # Theme, connections, projects
```

---

### Task 1: Backend — MCP Types

**Files:**
- Create: `crates/claude-types/src/mcp.rs`
- Modify: `crates/claude-types/src/lib.rs`

- [ ] **Step 1: Create MCP types**

```rust
// crates/claude-types/src/mcp.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP server info for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerInfo {
    pub name: String,
    pub scope: String,          // "user", "project", "local"
    pub transport: String,      // "stdio", "sse", "http"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,    // for stdio
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,          // for stdio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,        // for sse/http
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
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
    pub scope: String,  // default "local"
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

/// Request to launch claude in a project
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchRequest {
    pub project_path: String,
    #[serde(default)]
    pub env: HashMap<String, String>,
}
```

- [ ] **Step 2: Register module and commit**

```bash
cargo build -p claude-types
git add crates/claude-types/
git commit -m "feat: add MCP server and launcher type definitions"
```

---

### Task 2: Backend — MCP API Endpoints

**Files:**
- Create: `crates/claude-daemon/src/api/mcp.rs`
- Modify: `crates/claude-daemon/src/api/mod.rs`
- Modify: `crates/claude-daemon/src/server.rs`

- [ ] **Step 1: Implement MCP endpoints**

Endpoints:
- `GET /api/v1/mcp/servers` — List MCP servers by executing `claude mcp list` and parsing output
- `POST /api/v1/mcp/servers` — Add server via `claude mcp add`
- `DELETE /api/v1/mcp/servers/:name` — Remove server via `claude mcp remove`

For listing: run `claude mcp list` and parse the text output. Each server shows as:
```
  ❯ server-name: command-or-url - status
```
Parse name, command/url, and status. Since `claude mcp list` doesn't output JSON, we'll parse the text.

For add: construct `claude mcp add` command from `AddMcpServerRequest` fields:
```
claude mcp add --transport {transport} --scope {scope} [-e KEY=VAL...] [-H header...] {name} {commandOrUrl} [args...]
```

For remove: `claude mcp remove --scope {scope} {name}`

All commands use the executor for async execution with WS streaming.

- [ ] **Step 2: Add routes and commit**

```bash
cargo build -p claude-daemon
git add crates/claude-daemon/
git commit -m "feat: add MCP server list, add, remove API endpoints"
```

---

### Task 3: Backend — Launcher Endpoint

**Files:**
- Create: `crates/claude-daemon/src/api/launcher.rs`
- Modify: `crates/claude-daemon/src/api/mod.rs`
- Modify: `crates/claude-daemon/src/server.rs`

- [ ] **Step 1: Implement launcher endpoint**

`POST /api/v1/launch` — Launch claude in a project directory with custom env vars.

```rust
pub async fn launch_claude(
    Extension(state): Extension<AppState>,
    Json(req): Json<LaunchRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    // Spawn claude process detached (not waiting for completion)
    let mut cmd = tokio::process::Command::new("claude");
    cmd.current_dir(&req.project_path);
    for (k, v) in &req.env {
        cmd.env(k, v);
    }
    cmd.spawn().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
            error: format!("Failed to launch: {}", e),
            details: None,
        }))
    })?;

    Ok(Json(serde_json::json!({ "status": "launched" })))
}
```

- [ ] **Step 2: Add route and commit**

```bash
cargo build -p claude-daemon
git add crates/claude-daemon/
git commit -m "feat: add project launcher endpoint"
```

---

### Task 4: Backend — Integration Tests

**Files:**
- Modify: `crates/claude-daemon/tests/api_test.rs`

- [ ] **Step 1: Add MCP list test**

Test that `GET /api/v1/mcp/servers` returns 200 (may be empty or contain servers depending on test environment).

- [ ] **Step 2: Add launcher test**

Test that `POST /api/v1/launch` with a valid project path returns 200.

- [ ] **Step 3: Run tests and commit**

```bash
cargo test -p claude-daemon --test api_test
git add crates/claude-daemon/tests/
git commit -m "test: add MCP and launcher integration tests"
```

---

### Task 5: Frontend — MCP Types + API Client

**Files:**
- Modify: `src/lib/api/types.ts`
- Modify: `src/lib/api/client.ts`

- [ ] **Step 1: Add types**

```typescript
export interface McpServerInfo {
  name: string;
  scope: string;
  transport: string;
  command?: string;
  args?: string[];
  url?: string;
  env?: Record<string, string>;
  headers?: Record<string, string>;
}

export interface AddMcpServerRequest {
  name: string;
  transport: string;
  commandOrUrl?: string;
  args?: string[];
  scope?: string;
  env?: Record<string, string>;
  headers?: Record<string, string>;
}

export interface LaunchRequest {
  projectPath: string;
  env?: Record<string, string>;
}
```

- [ ] **Step 2: Add API methods**

```typescript
// MCP
async listMcpServers(): Promise<McpServerInfo[]>
async addMcpServer(req: AddMcpServerRequest): Promise<{ requestId: string }>
async removeMcpServer(name: string, scope?: string): Promise<{ requestId: string }>

// Launcher
async launchClaude(req: LaunchRequest): Promise<{ status: string }>
```

- [ ] **Step 3: Verify and commit**

```bash
pnpm build
git add src/lib/api/
git commit -m "feat: add MCP and launcher TypeScript types and API methods"
```

---

### Task 6: Frontend — MCP Module

**Files:**
- Create: `src/lib/stores/mcp.svelte.ts`
- Create: `src/lib/components/mcp/McpModule.svelte`
- Create: `src/lib/components/mcp/McpServerList.svelte`
- Create: `src/lib/components/mcp/McpServerEditor.svelte`
- Modify: `src/App.svelte`

- [ ] **Step 1: Create MCP store**

```typescript
class McpStore {
  servers = $state<McpServerInfo[]>([]);
  loading = $state(false);
  error = $state<string>("");

  async loadServers() { ... }
  async addServer(req: AddMcpServerRequest) { ... }
  async removeServer(name: string, scope?: string) { ... }
}
export const mcpStore = new McpStore();
```

- [ ] **Step 2: Create McpServerList**

Groups servers by scope (User / Project / Local / Plugin). Each server shows:
- Name, transport type badge (stdio/sse/http), command/URL
- Remove button

- [ ] **Step 3: Create McpServerEditor**

Form for adding new MCP server:
- Name input
- Transport selector (stdio / sse / http)
- For stdio: command input + args input
- For sse/http: URL input + headers
- Scope selector (user / project / local)
- Env vars section (key-value pairs)
- "Add Server" button

- [ ] **Step 4: Create McpModule orchestrator**

Sub-navigation: "Servers" (list), "Add Server" (form)

- [ ] **Step 5: Wire into App.svelte**

When `currentModule === "mcp"`:
- Sub-panel: "Servers" and "Add Server" nav items
- Detail: `<McpModule activeSection={mcpSection} />`
- On mount: `mcpStore.loadServers()`

- [ ] **Step 6: Verify and commit**

```bash
pnpm build
git add src/lib/stores/mcp.svelte.ts src/lib/components/mcp/ src/App.svelte
git commit -m "feat: add MCP Servers module with list, add, remove"
```

---

### Task 7: Frontend — Effective Config View

**Files:**
- Create: `src/lib/components/effective/EffectiveConfigView.svelte`
- Modify: `src/App.svelte`

- [ ] **Step 1: Implement EffectiveConfigView**

This component:
1. Requires a project to be selected (`projectsStore.activeProject`)
2. Fetches effective config via `client.getEffectiveConfig(projectId)`
3. Displays the merged settings as a tree
4. Each field annotated with source badge: User (blue), Project (green), Local (yellow), Managed (red)
5. Overridden fields (where source != User) are highlighted

Structure:
- Top: project name + "Refresh" button
- Body: collapsible sections (same as settings: General, Permissions, Hooks, etc.)
- Each field shows: key, value, source badge
- If no project selected: show message to select one first

For field sources: the `getEffectiveConfig` API returns `{ settings, fieldSources: Record<string, string> }`. Use `fieldSources` to look up the source for each key.

Since rendering a full tree from arbitrary JSON with source annotations is complex, use a simplified approach:
- Show the effective JSON in a formatted view
- Below each top-level section, show which source it came from
- Highlight fields that differ from user-level defaults

- [ ] **Step 2: Wire into App.svelte**

When `currentModule === "effective"`: render `<EffectiveConfigView />` in detail panel. No sub-navigation needed.

- [ ] **Step 3: Verify and commit**

```bash
pnpm build
git add src/lib/components/effective/ src/App.svelte
git commit -m "feat: add Effective Config viewer with field source annotations"
```

---

### Task 8: Frontend — Project Launcher

**Files:**
- Create: `src/lib/components/launcher/LauncherView.svelte`
- Modify: `src/App.svelte`

- [ ] **Step 1: Implement LauncherView**

The launcher panel:
1. Project selector (from registered projects)
2. Effective config summary (quick overview: language, model, plugin count)
3. Environment variable section:
   - Show env vars from effective config
   - Checkboxes to select which vars to include
   - Add custom env vars (key-value inputs)
4. "Launch Claude Code" button
   - Calls `client.launchClaude({ projectPath, env })`
   - Shows success/error feedback

- [ ] **Step 2: Wire into App.svelte**

When `currentModule === "launcher"`: render `<LauncherView />`.

- [ ] **Step 3: Verify and commit**

```bash
pnpm build
git add src/lib/components/launcher/ src/App.svelte
git commit -m "feat: add Project Launcher with env var selection"
```

---

### Task 9: Frontend — App Settings

**Files:**
- Create: `src/lib/stores/appsettings.svelte.ts`
- Create: `src/lib/components/appsettings/AppSettingsView.svelte`
- Modify: `src/App.svelte`

- [ ] **Step 1: Create app settings store**

App settings are stored locally (not on daemon). Use localStorage for now (tauri-plugin-store can be added later for native integration).

```typescript
interface AppPreferences {
  theme: "light" | "dark" | "system";
  language: string;
  fontSize: number;
  daemonUrl: string;
  daemonToken: string;
}

class AppSettingsStore {
  preferences = $state<AppPreferences>({
    theme: "system",
    language: "zh-CN",
    fontSize: 14,
    daemonUrl: "http://127.0.0.1:7890",
    daemonToken: "dev-token",
  });

  load() {
    const saved = localStorage.getItem("app-preferences");
    if (saved) this.preferences = { ...this.preferences, ...JSON.parse(saved) };
  }

  save() {
    localStorage.setItem("app-preferences", JSON.stringify(this.preferences));
  }

  update(partial: Partial<AppPreferences>) {
    this.preferences = { ...this.preferences, ...partial };
    this.save();
  }
}
export const appSettingsStore = new AppSettingsStore();
```

- [ ] **Step 2: Create AppSettingsView**

Three sections:

**Appearance:**
- Theme dropdown (Light / Dark / System)
- Font size slider (12-20)

**Connection:**
- Daemon URL input
- Daemon token input (password field)
- "Test Connection" button (calls health endpoint)
- Connection status display

**Projects:**
- List of registered projects (from projectsStore)
- "Add Project" button with directory path input
- Remove button per project

- [ ] **Step 3: Wire into App.svelte**

When `currentModule === "appsettings"`: render `<AppSettingsView />`.

Also: on app mount, load app settings and use `daemonUrl` + `daemonToken` from preferences for the connection instead of hardcoded values.

Apply theme by setting `document.documentElement.classList` based on theme preference.

- [ ] **Step 4: Verify and commit**

```bash
pnpm build
git add src/lib/stores/appsettings.svelte.ts src/lib/components/appsettings/ src/App.svelte
git commit -m "feat: add App Settings with theme, connection, and project management"
```

---

### Task 10: App.svelte Cleanup — Remove Unused Nav Items

**Files:**
- Modify: `src/App.svelte`

The current App.svelte has placeholder nav buttons (S, P, K, E, L, A) from Phase 1 that are not Phase 4 modules. Clean up:

- [ ] **Step 1: Remove/update unused nav items**

Keep only the implemented modules in the sidebar:
1. S — Settings
2. P — Plugins
3. K — Skills
4. M — Memory
5. C — MCP Servers
6. E — Effective Config
7. L — Launcher
8. A — App Settings (at bottom, separated)

Remove any placeholder nav items that don't map to real modules.

- [ ] **Step 2: Verify and commit**

```bash
pnpm build
git add src/App.svelte
git commit -m "chore: clean up sidebar navigation, remove placeholder items"
```

---

### Task 11: Full Build + Local Run Verification

- [ ] **Step 1: Run all Rust tests**

Run: `cargo test --workspace`
Expected: All tests pass

- [ ] **Step 2: Run frontend build**

Run: `pnpm build`
Expected: Succeeds

- [ ] **Step 3: Local daemon run**

```bash
cargo run -p claude-daemon -- --port 7890 --token dev-token
```
Expected: Starts without panic, all endpoints respond

- [ ] **Step 4: Local GUI run**

```bash
cargo run -p dot-claude-gui
```
Expected: Starts without panic, runs for 5+ seconds

- [ ] **Step 5: Verify all modules accessible**

With both running:
- Settings module: sub-editors load
- Plugins: installed list loads
- Skills: skill list loads
- Memory: project list loads
- MCP: server list loads
- Effective Config: shows merged config
- Launcher: project selector works
- App Settings: preferences save/load

- [ ] **Step 6: Final commit if needed**

---

## Phase 4 Completion Checklist

- [ ] MCP types + API endpoints (list, add, remove)
- [ ] Launcher endpoint (spawn claude process)
- [ ] Frontend MCP module (server list, add form, remove)
- [ ] Frontend Effective Config viewer (merged config with source annotations)
- [ ] Frontend Project Launcher (env var selection, launch button)
- [ ] Frontend App Settings (theme, connection config, project management)
- [ ] Sidebar cleaned up with all 8 modules
- [ ] Integration tests for new endpoints
- [ ] All Rust tests passing
- [ ] Frontend building
- [ ] Local run: daemon + GUI no panic

**After Phase 4: MVP Complete!**
