# dot-claude-gui Design Spec

> Your `.claude/` all-in-one manager GUI

**Date:** 2026-03-30
**Status:** Draft
**Author:** Eric Yao + Claude

---

## 1. Problem Statement

Claude Code's configuration is scattered across multiple JSON files, markdown files, and directories under `~/.claude/` and per-project `.claude/` directories. There is no unified management tool. Users must hand-edit JSON files, which is error-prone and tedious — especially given that Claude Code releases almost daily with frequent schema changes.

Existing community tools (Claudia, Claude Code Tool Manager, VS Code extensions) each cover only a subset of functionality. No tool provides:
- Full settings.json visual editing with per-section granularity
- Per-project plugin/skill activation profiles
- Marketplace browsing with one-click plugin installation
- Real-time bidirectional sync with the filesystem
- Remote daemon support for sandbox/cloud environments
- Effective config visualization across the four-layer hierarchy

### Core Design Principle

**Per-project customization first.** If a configuration can be scoped to a project, the GUI must support project-level override.

---

## 2. Architecture

### 2.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────┐
│                  Tauri Desktop App                   │
│  ┌───────────┬──────────────┬────────────────────┐  │
│  │  Sidebar  │  Sub-items   │   Detail/Editor    │  │
│  │   15%     │    25%       │      60%           │  │
│  └───────────┴──────────────┴────────────────────┘  │
│              Frontend (Svelte 5 + TypeScript)        │
│─────────────────────────────────────────────────────│
│              Tauri Rust Shell (IPC bridge)           │
└───────────────────┬─────────────────────────────────┘
                    │ WebSocket + REST
                    ▼
┌─────────────────────────────────────────────────────┐
│              claude-daemon (Rust binary)             │
│  ┌─────────────┐  ┌──────────┐  ┌───────────────┐  │
│  │ File Watcher │  │ Config   │  │  API Server   │  │
│  │ (notify)     │  │ Engine   │  │ (axum)        │  │
│  └─────────────┘  └──────────┘  └───────────────┘  │
│  ┌─────────────┐  ┌──────────┐  ┌───────────────┐  │
│  │ Schema      │  │ Merge    │  │  Validator    │  │
│  │ Registry    │  │ Engine   │  │               │  │
│  └─────────────┘  └──────────┘  └───────────────┘  │
└───────────────────┬─────────────────────────────────┘
                    │ fs read/write
                    ▼
            ~/.claude/ directory (or remote sandbox)
```

### 2.2 Deployment Modes

| Mode | Daemon Location | GUI Location | Communication |
|------|----------------|--------------|---------------|
| **Local** | Tauri sidecar (auto-managed) | macOS desktop | `localhost` WebSocket |
| **Remote** | Host machine / cloud sandbox | macOS desktop | Network WebSocket over TLS |

### 2.3 Component Responsibilities

**Tauri Shell (src-tauri/)**
- Manages daemon sidecar lifecycle in local mode
- Bridges frontend IPC to daemon API
- Handles native windowing, menus, system tray
- Stores app-specific config (theme, language, connections) via `tauri-plugin-store`

**claude-daemon (crates/claude-daemon/)**
- Watches `~/.claude/` and registered project `.claude/` directories for changes
- Serves REST API for configuration CRUD
- Pushes real-time change events via WebSocket
- Executes `claude plugin` and `claude mcp` CLI commands on behalf of GUI
- Validates all config mutations before writing to disk
- Performs atomic file writes (write temp → rename) to prevent corruption

**claude-config (crates/claude-config/)**
- Parses and serializes settings.json, installed_plugins.json, SKILL.md, MEMORY.md, etc.
- Implements the four-layer merge engine (managed > user > project > local)
- Schema-aware validation with clear error messages
- Preserves unknown fields on roundtrip (forward-compatible with new Claude Code versions)

**claude-types (crates/claude-types/)**
- Shared type definitions used by both daemon and Tauri shell
- API request/response types
- WebSocket event types

**Frontend (src/)**
- Svelte 5 with TypeScript
- Three-panel layout with resizable panes
- Real-time state sync via WebSocket subscription
- Form-based editors for each config section with JSON preview

---

## 3. Technology Stack

| Layer | Choice | Rationale |
|-------|--------|-----------|
| **GUI Framework** | Tauri 2.0 | Lightest cross-platform option (~30-40 MB RAM, 2.5-10 MB bundle), built-in sidecar support |
| **Frontend** | Svelte 5 + TypeScript | Small bundle, runes reactivity fits real-time data flow, Tauri's recommended frontend |
| **Styling** | Tailwind CSS 4 | Utility-first, consistent design, dark mode support |
| **Backend/Daemon** | Rust (axum + tokio) | Type-safe, high-performance async HTTP/WS server |
| **File Watching** | `notify` crate (with debouncer) | Cross-platform gold standard (FSEvents/inotify/ReadDirectoryChangesW) |
| **Config Parsing** | `serde` + `serde_json` | Preserve unknown fields via `#[serde(flatten)]` with `HashMap<String, Value>` |
| **Build** | Cargo workspace | Shared types and config engine across daemon and Tauri shell |
| **Frontend Build** | Vite | Fast HMR, Tauri integration |
| **macOS Priority** | Yes | Primary target platform; Linux/Windows support deferred |

### Cargo Workspace Structure

```
dot-claude-gui/
├── Cargo.toml                    # workspace root
├── crates/
│   ├── claude-config/            # config parsing, merge, validation, file watching
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── schema.rs         # settings.json type definitions
│   │   │   ├── merge.rs          # four-layer config merge engine
│   │   │   ├── validate.rs       # syntax + semantic validation
│   │   │   ├── watch.rs          # notify file watcher wrapper
│   │   │   ├── plugins.rs        # installed_plugins.json, blocklist, marketplaces
│   │   │   ├── skills.rs         # SKILL.md parsing
│   │   │   ├── memory.rs         # MEMORY.md and memory file parsing
│   │   │   └── mcp.rs            # MCP server config parsing
│   │   └── Cargo.toml
│   ├── claude-daemon/            # standalone daemon binary
│   │   ├── src/
│   │   │   ├── main.rs           # CLI args, daemon startup
│   │   │   ├── server.rs         # axum router setup
│   │   │   ├── api/
│   │   │   │   ├── config.rs     # settings CRUD endpoints
│   │   │   │   ├── plugins.rs    # plugin management endpoints
│   │   │   │   ├── skills.rs     # skill CRUD endpoints
│   │   │   │   ├── memory.rs     # memory CRUD endpoints
│   │   │   │   ├── mcp.rs        # MCP server endpoints
│   │   │   │   ├── projects.rs   # registered project management
│   │   │   │   └── ws.rs         # WebSocket handler
│   │   │   ├── state.rs          # in-memory config state (Arc<RwLock<...>>)
│   │   │   ├── watcher.rs        # file change → state update → WS broadcast
│   │   │   ├── executor.rs       # runs claude CLI commands (plugin install, mcp add)
│   │   │   └── auth.rs           # API key authentication middleware
│   │   └── Cargo.toml
│   └── claude-types/             # shared types
│       ├── src/
│       │   ├── lib.rs
│       │   ├── api.rs            # REST request/response types
│       │   ├── events.rs         # WebSocket event types
│       │   └── config.rs         # re-exports from claude-config for frontend codegen
│       └── Cargo.toml
├── src-tauri/                    # Tauri app shell
│   ├── src/
│   │   ├── main.rs               # Tauri setup, sidecar lifecycle
│   │   ├── commands.rs           # Tauri IPC commands (proxy to daemon API)
│   │   └── tray.rs               # system tray menu
│   ├── tauri.conf.json
│   ├── Cargo.toml
│   └── icons/
├── src/                          # Svelte frontend
│   ├── App.svelte
│   ├── app.css                   # Tailwind imports
│   ├── lib/
│   │   ├── api/
│   │   │   ├── client.ts         # REST API client
│   │   │   ├── ws.ts             # WebSocket client with auto-reconnect
│   │   │   └── types.ts          # TypeScript types (generated from claude-types)
│   │   ├── stores/
│   │   │   ├── config.svelte.ts  # reactive config state
│   │   │   ├── plugins.svelte.ts
│   │   │   ├── connection.svelte.ts  # daemon connection status
│   │   │   └── projects.svelte.ts
│   │   ├── components/
│   │   │   ├── layout/
│   │   │   │   ├── Sidebar.svelte
│   │   │   │   ├── SubPanel.svelte
│   │   │   │   ├── DetailPanel.svelte
│   │   │   │   └── ThreePanel.svelte
│   │   │   ├── settings/
│   │   │   │   ├── PermissionsEditor.svelte
│   │   │   │   ├── HooksEditor.svelte
│   │   │   │   ├── SandboxEditor.svelte
│   │   │   │   ├── EnvVarEditor.svelte
│   │   │   │   ├── GeneralEditor.svelte
│   │   │   │   ├── StatusLineEditor.svelte
│   │   │   │   └── JsonPreview.svelte
│   │   │   ├── plugins/
│   │   │   │   ├── PluginList.svelte
│   │   │   │   ├── PluginCard.svelte
│   │   │   │   ├── MarketplaceBrowser.svelte
│   │   │   │   ├── MarketplaceManager.svelte
│   │   │   │   └── ProjectActivation.svelte
│   │   │   ├── skills/
│   │   │   │   ├── SkillList.svelte
│   │   │   │   ├── SkillEditor.svelte
│   │   │   │   └── SkillPreview.svelte
│   │   │   ├── memory/
│   │   │   │   ├── MemoryList.svelte
│   │   │   │   └── MemoryEditor.svelte
│   │   │   ├── mcp/
│   │   │   │   ├── McpServerList.svelte
│   │   │   │   ├── McpServerEditor.svelte
│   │   │   │   └── McpServerAdd.svelte
│   │   │   ├── effective/
│   │   │   │   └── EffectiveConfigView.svelte
│   │   │   └── shared/
│   │   │       ├── ProjectSelector.svelte
│   │   │       ├── ConnectionStatus.svelte
│   │   │       ├── JsonEditor.svelte
│   │   │       ├── MarkdownEditor.svelte
│   │   │       └── ConfirmDialog.svelte
│   │   └── utils/
│   │       ├── validation.ts
│   │       └── formatting.ts
│   └── routes/                   # if using SvelteKit-style routing
├── package.json
├── vite.config.ts
├── svelte.config.js
├── tailwind.config.ts
├── tsconfig.json
└── docs/
    └── superpowers/
        └── specs/
            └── 2026-03-30-dot-claude-gui-design.md
```

---

## 4. Communication Protocol

### 4.1 REST API (Configuration CRUD)

All endpoints are prefixed with `/api/v1`.

**Config endpoints:**
```
GET    /config/user                    # user-level settings.json
GET    /config/project/:project_id     # project-level settings
GET    /config/local/:project_id       # project local settings
GET    /config/effective/:project_id   # merged effective config with source annotations
PUT    /config/user                    # update user settings (partial merge)
PUT    /config/project/:project_id     # update project settings
PUT    /config/local/:project_id       # update project local settings
PUT    /config/user/:section           # update a specific section (e.g., "hooks", "permissions")
PUT    /config/project/:project_id/:section
```

**Plugin endpoints:**
```
GET    /plugins                        # all installed plugins with status
GET    /plugins/marketplace/:market_id # list plugins in a marketplace
POST   /plugins/install                # { marketplace, plugin_name }
POST   /plugins/:id/uninstall
POST   /plugins/:id/toggle             # { enabled: bool }
POST   /plugins/:id/toggle/:project_id # per-project enable/disable
GET    /marketplaces                   # list registered marketplaces
POST   /marketplaces                   # add marketplace { source }
DELETE /marketplaces/:id               # remove marketplace
```

**Skill endpoints:**
```
GET    /skills                         # all skills (user + plugin-provided)
GET    /skills/:id                     # skill detail (SKILL.md content)
PUT    /skills/:id                     # update skill content
POST   /skills                         # create new skill
DELETE /skills/:id                     # delete user skill
POST   /skills/:id/toggle/:project_id  # per-project enable/disable
```

**Memory endpoints:**
```
GET    /memory/:project_id             # MEMORY.md index + all memory files
GET    /memory/:project_id/:filename   # single memory file content
PUT    /memory/:project_id/:filename   # update memory file
POST   /memory/:project_id             # create new memory file
DELETE /memory/:project_id/:filename   # delete memory file
```

**MCP server endpoints:**
```
GET    /mcp/servers                     # all servers (grouped by scope)
GET    /mcp/servers/:scope              # servers by scope (user/project/local)
POST   /mcp/servers                     # add server { name, transport, scope, ... }
PUT    /mcp/servers/:name               # update server config
DELETE /mcp/servers/:name               # remove server
POST   /mcp/servers/:name/toggle/:project_id  # per-project enable/disable
```

**Project endpoints:**
```
GET    /projects                        # list registered projects
POST   /projects                        # register { path }
DELETE /projects/:id                    # unregister
```

**System endpoints:**
```
GET    /health                          # daemon health check
GET    /version                         # daemon + claude code version
```

### 4.2 WebSocket (Real-time Events)

Connect to `ws://host:port/api/v1/ws` (or `wss://` for remote mode).

**Server → Client events:**
```json
{ "event": "config_changed", "scope": "user", "section": "hooks", "data": { ... } }
{ "event": "config_changed", "scope": "project", "project_id": "...", "section": "permissions", "data": { ... } }
{ "event": "plugin_installed", "plugin": { "id": "...", "version": "...", "status": "..." } }
{ "event": "plugin_removed", "plugin_id": "..." }
{ "event": "plugin_toggled", "plugin_id": "...", "enabled": true, "scope": "user" }
{ "event": "skill_changed", "skill_id": "...", "action": "created|updated|deleted" }
{ "event": "memory_changed", "project_id": "...", "filename": "...", "action": "created|updated|deleted" }
{ "event": "mcp_changed", "server_name": "...", "action": "added|updated|removed" }
{ "event": "validation_error", "scope": "...", "errors": [ { "path": "hooks.PreToolUse[0].matcher", "message": "..." } ] }
{ "event": "command_output", "request_id": "...", "stream": "stdout|stderr", "data": "..." }
{ "event": "command_completed", "request_id": "...", "exit_code": 0 }
```

**Client → Server messages:**
```json
{ "action": "subscribe", "topics": ["config", "plugins", "skills", "memory", "mcp"] }
{ "action": "unsubscribe", "topics": ["memory"] }
```

### 4.3 Authentication

**Local mode:**
- Daemon generates a random 256-bit token on startup
- Token written to `~/.claude/daemon-token`
- Tauri shell reads token from file, passes in `Authorization: Bearer <token>` header
- WebSocket authenticates via token in initial handshake query param

**Remote mode:**
- Same API key mechanism, but over TLS
- Daemon binds `0.0.0.0` with TLS enabled (`--tls-cert`, `--tls-key` flags)
- GUI connects via `wss://` and `https://`
- Token distributed out-of-band (SSH, env var, manual entry in GUI)

---

## 5. Feature Modules

### 5.1 Settings Editor

The settings editor decomposes `settings.json` into independent sub-editors. Each sub-editor renders a purpose-built form UI and generates a JSON fragment. The daemon's merge engine combines all fragments into a valid settings file before writing.

**Sub-editors:**

| Sub-editor | Key Fields | UI Elements |
|-----------|------------|-------------|
| **General** | `language`, `alwaysThinkingEnabled`, `autoUpdatesChannel`, `minimumVersion`, `includeCoAuthoredBy`, `skipDangerousModePermissionPrompt` | Dropdowns, toggles, text inputs |
| **Permissions** | `permissions.allow`, `permissions.deny`, `permissions.ask`, `permissions.defaultMode` | Categorized list with add/remove, mode dropdown |
| **Hooks** | `hooks.PreToolUse`, `hooks.PostToolUse`, `hooks.ConfigChange`, etc. | Dynamic form: event type selector → matcher input → hook type toggle (command/HTTP) → command/URL input → optional `if` condition editor |
| **Sandbox** | `sandbox.allowRead`, `sandbox.denyRead`, `sandbox.allowWrite`, `sandbox.excludedCommands`, `sandbox.failIfUnavailable`, `sandbox.enableWeakerNetworkIsolation` | Path list editors, toggles |
| **Environment** | `env` | Key-value table with add/remove, checkbox per var, per-project override column |
| **Status Line** | `statusLine.type`, `statusLine.command`, `statusLine.padding` | Type dropdown, command input, number input |

**Hooks editor detail:**

Hooks are the most complex sub-editor. Design:

```
┌─ Event Type: [PreToolUse ▾] ────────────────────────┐
│                                                      │
│  ┌─ Hook Rule 1 ──────────────────────────────────┐  │
│  │ Matcher:   [Bash                        ]      │  │
│  │ Type:      ○ Command  ● HTTP                   │  │
│  │ URL:       [https://hooks.example.com/pre ]    │  │
│  │ Method:    [POST ▾]                            │  │
│  │ Headers:   [+ Add Header]                      │  │
│  │ Timeout:   [30000] ms                          │  │
│  │ Condition: [optional if expression      ]      │  │
│  │                                    [Delete]    │  │
│  └────────────────────────────────────────────────┘  │
│                                                      │
│  ┌─ Hook Rule 2 ──────────────────────────────────┐  │
│  │ Matcher:   [*                           ]      │  │
│  │ Type:      ● Command  ○ HTTP                   │  │
│  │ Command:   [/usr/local/bin/audit-log    ]      │  │
│  │ Condition: [                            ]      │  │
│  │                                    [Delete]    │  │
│  └────────────────────────────────────────────────┘  │
│                                                      │
│  [+ Add Hook Rule]                                   │
│                                                      │
│  ── JSON Preview ──────────────────────────────────  │
│  {                                                   │
│    "PreToolUse": [                                   │
│      { "matcher": "Bash", "hooks": [{ ... }] },     │
│      { "matcher": "*", "hooks": [{ ... }] }         │
│    ]                                                 │
│  }                                                   │
└──────────────────────────────────────────────────────┘
```

**Supported hook events** (all event types Claude Code supports):
PreToolUse, PostToolUse, Notification, Stop, SubagentStop, CwdChanged, FileChanged, ConfigChange, StopFailure, TaskCreated, WorktreeCreate, WorktreeRemove, PostCompact, Elicitation, InstructionsLoaded

**Per-project support:**
- Top of the detail panel: `Scope: [User ▾] [Project: react-app ▾]`
- When project scope is selected, show which fields are overridden (highlighted) vs inherited (dimmed)
- Saving to project scope writes to `.claude/settings.json` or `.claude/settings.local.json` in the project directory

**Schema resilience:**
- All types use `#[serde(flatten)] pub extra: HashMap<String, serde_json::Value>` to capture unknown fields
- Unknown fields are displayed in a collapsed "Advanced / Raw" section at the bottom of each sub-editor
- Unknown fields are preserved on save

### 5.2 Plugins

**Installed plugins view:**
- List all plugins from `~/.claude/plugins/installed_plugins.json`
- Each card shows: name, marketplace, version, installed date, enabled/disabled toggle
- Action buttons: Uninstall, Update (if newer version available)
- Blocklist indicator for blocked plugins

**Marketplace browser:**
- Dropdown to select marketplace
- Lists all available plugins in the selected marketplace
- Each entry shows: name, description, version, install status
- "Install" button for uninstalled plugins
- Search/filter within marketplace

**Marketplace management:**
- List registered marketplaces from `~/.claude/plugins/known_marketplaces.json`
- Add new marketplace: input GitHub `owner/repo` → validate → add
- Remove marketplace

**Per-project activation:**
- Matrix view: rows = installed plugins, columns = registered projects
- Checkboxes to enable/disable each plugin per project
- Writes to project-level `settings.json` or `settings.local.json` `enabledPlugins` field
- Shows global default state and per-project override state

**Implementation:**
- Install/uninstall/marketplace operations execute via `claude plugin` CLI commands through the daemon's executor module
- Command output streamed to GUI via WebSocket `command_output` events
- Plugin state refreshed from filesystem after command completion

### 5.3 Skills

**Skill list:**
- Lists all skills from:
  - `~/.claude/skills/` (user-created)
  - Plugin-provided skills (read from plugin manifests)
- Each entry shows: name, description, source (user/plugin), type (rigid/flexible)

**Skill viewer/editor:**
- Markdown rendering for SKILL.md preview
- Code editor for SKILL.md editing (user-created skills only)
- Plugin-provided skills are read-only

**Create new skill:**
- Name input → creates directory under `~/.claude/skills/`
- SKILL.md template pre-filled with frontmatter

**Per-project activation:**
- Same matrix pattern as plugins
- Controls which skills are available in each project context

### 5.4 Memory

**Project memory browser:**
- Select project from dropdown
- Displays MEMORY.md index (parsed as markdown list of links)
- Lists all `.md` files in the project's `memory/` directory

**Memory editor:**
- View: Markdown rendered preview
- Edit: Markdown editor with frontmatter awareness (name, description, type fields)
- Delete: with confirmation dialog
- Create: new memory file with frontmatter template

**MEMORY.md index management:**
- When creating/deleting memory files, automatically update the MEMORY.md index

### 5.5 MCP Servers

**Server list:**
- Grouped by scope: User / Project / Local
- Each entry shows: name, transport type (stdio/SSE/HTTP), URL/command, enabled/disabled

**Add server form:**
- Name, transport type selector
- Transport-specific fields:
  - stdio: command, args, env vars
  - SSE/HTTP: URL, headers, OAuth config (client_id, client_secret, callback_port)
- Scope selector (user/project/local)

**Edit/delete:**
- Inline editing of server configuration
- Delete with confirmation

**Per-project:**
- Toggle servers on/off per project via `allowedMcpServers` / `deniedMcpServers` in project settings

### 5.6 Effective Config

**Merged config viewer:**
- Select a project → display the final merged configuration
- Each field annotated with its source: `managed` | `user` | `project` | `local`
- Overridden fields highlighted (e.g., yellow background with "overridden by project" tooltip)
- Collapsible sections matching the settings sub-editor structure

**Use case:** Debugging why a setting isn't taking effect. User can see exactly which layer is providing each value.

### 5.7 Project Launcher

**Launcher panel:**
- Select registered project
- Preview: effective config summary, active plugins, active skills, environment variables
- Environment variable checkboxes (select which vars to inject)
- "Launch Claude Code" button → spawns `claude` process with:
  - Working directory set to project path
  - Selected environment variables injected
  - Project's effective configuration applied

### 5.8 App Settings

Stored in Tauri's app data directory (`~/Library/Application Support/com.dotclaude.gui/`).

**Preferences:**
- Theme: light / dark / follow system
- Language: zh-CN, en-US (extensible)
- Font family and size
- Sidebar width (persisted)

**Connections:**
- List of daemon connections (name, host, port, auth token)
- Default: "Local" connection (auto-managed sidecar)
- Add/edit/delete remote connections
- Connection status indicator in header bar

**Registered projects:**
- Add project by selecting directory
- Remove project from management
- Per-project notes (optional)

**Future: AI Chat (V2+)**
- API key storage in system keychain (macOS Keychain)
- Model selector
- Base URL for custom endpoints

---

## 6. UI Design

### 6.1 Layout

Classic three-panel layout:

```
┌──────────────────────────────────────────────────────────────┐
│  [dot-claude-gui]   Project: [~/workspace/react-app ▾]  🟢  │  ← header bar
├────────┬───────────────┬─────────────────────────────────────┤
│        │               │                                     │
│  ⚙️ S  │  Permissions  │  ┌─ allow ───────────────────────┐  │
│  🧩 P  │  Hooks     ◀  │  │  Bash(git:*)           [x]   │  │
│  🎯 K  │  Sandbox      │  │  Bash(npm:*)           [x]   │  │
│  🧠 M  │  Environment  │  │  WebSearch             [x]   │  │
│  📡 C  │  General      │  │  [+ Add Permission]          │  │
│  📊 E  │  Status Line  │  └───────────────────────────────┘  │
│  🚀 L  │               │  ┌─ deny ────────────────────────┐  │
│        │               │  │  (empty)                      │  │
│  ──    │               │  │  [+ Add Denial]               │  │
│  ⚙️ A  │               │  └───────────────────────────────┘  │
│        │               │                                     │
│        │               │  Mode: [plan ▾]                     │
│        │               │                                     │
│        │               │  Scope: [User ▾]  [Save] [Revert]  │
│  15%   │     25%       │              60%                    │
└────────┴───────────────┴─────────────────────────────────────┘
```

**Left sidebar (15%):** Icon-based primary navigation
- S = Settings
- P = Plugins
- K = Skills
- M = Memory
- C = MCP Servers (Connections)
- E = Effective Config
- L = Launcher
- A = App Settings (bottom, separated)

**Middle panel (25%):** Secondary navigation within the selected module

**Detail panel (60%):** Content editor, forms, previews

### 6.2 Header Bar

- App name/logo (left)
- Project selector dropdown (center) — switches the active project context
- Daemon connection status indicator (right): green dot = connected, red = disconnected, yellow = reconnecting
- Connection name label (e.g., "Local" or "remote-sandbox-1")

### 6.3 Theme

- macOS-native appearance using system WebView
- Light / Dark mode following system preference (or manual override)
- Tailwind CSS with CSS variables for theming
- Subtle borders, proper spacing, native-feeling form controls

### 6.4 Responsive Behavior

- Minimum window size: 900 x 600 px
- Panels are resizable via drag handles
- Sidebar collapses to icon-only mode at narrow widths
- Middle panel can be collapsed to give detail panel more space

---

## 7. Data Flow

### 7.1 Config Edit Flow

```
User edits form in GUI
  → Frontend validates (client-side, immediate feedback)
  → PUT /api/v1/config/{scope}/{section}
  → Daemon validates (server-side, authoritative)
  → If valid: atomic write to file → 200 OK → WS event broadcast
  → If invalid: 422 with errors → GUI shows error inline
```

### 7.2 External Change Flow

```
User edits settings.json in vim/CLI
  → notify crate detects file change
  → Daemon reads + validates new content
  → Updates in-memory state
  → Broadcasts WS event: { event: "config_changed", ... }
  → GUI receives event → updates reactive stores → UI re-renders
```

### 7.3 Plugin Install Flow

```
User clicks "Install" on plugin card
  → POST /api/v1/plugins/install { marketplace, plugin_name }
  → Daemon spawns: claude plugin install <name> --marketplace <id>
  → Streams stdout/stderr via WS command_output events
  → GUI shows progress/output in real-time
  → On completion: daemon re-reads installed_plugins.json
  → Broadcasts WS plugin_installed event
  → GUI updates plugin list
```

---

## 8. Error Handling

### 8.1 Config Validation

**Two-stage validation:**
1. **Client-side (frontend):** Immediate feedback. Type checks, required fields, format validation (e.g., valid glob patterns for permissions, valid URLs for HTTP hooks).
2. **Server-side (daemon):** Authoritative. Full schema validation, semantic checks (e.g., referenced files exist, no circular hook dependencies), cross-section consistency.

**Validation errors** include:
- JSON path to the invalid field (e.g., `hooks.PreToolUse[0].hooks[0].command`)
- Human-readable error message
- Suggested fix (when deterministic)

### 8.2 Connection Handling

- GUI shows connection status in header
- Auto-reconnect WebSocket with exponential backoff (1s, 2s, 4s, 8s, max 30s)
- During disconnection: GUI shows stale data warning banner, disables write operations
- On reconnect: full state sync from daemon

### 8.3 File Write Safety

- All writes use atomic rename: write to `settings.json.tmp` → `rename()` to `settings.json`
- Before write: validate entire resulting config (not just the changed section)
- Daemon maintains a write lock per config file to prevent concurrent corruption
- If external change detected during write: abort, re-read, re-validate, notify user of conflict

---

## 9. Testing Strategy

### 9.1 Test Pyramid

```
                    ┌─────────────────┐
                    │   E2E Tests     │  Playwright
                    │  (full flows)   │  GUI ↔ Daemon ↔ FS
                    └────────┬────────┘
               ┌─────────────┴─────────────┐
         ┌─────┴──────┐             ┌──────┴───────┐
         │ Frontend   │             │ API          │
         │ Component  │             │ Integration  │
         │ Tests      │             │ Tests        │
         │ (Vitest)   │             │ (axum test)  │
         └─────┬──────┘             └──────┬───────┘
               └─────────────┬─────────────┘
                   ┌─────────┴───────────┐
                   │   Unit Tests        │
                   │ cargo test + vitest │
                   └─────────────────────┘
```

### 9.2 Rust Backend Tests

**Unit tests (`cargo test`):**
- `claude-config`: schema parsing, merge engine correctness, validator rules, unknown field preservation
- `claude-types`: serialization/deserialization roundtrip
- `claude-daemon`: auth middleware, state management

**Integration tests (`cargo test` + `tempdir`):**
- API handler tests using `axum::test` (send HTTP requests, verify responses)
- File watcher tests: write to temp dir → verify event fires within timeout
- WebSocket tests: connect → trigger file change → verify event received
- CLI executor tests: mock `claude` binary → verify correct command construction

**Contract tests (fixture-based):**
- Real `settings.json` samples (anonymized) as test fixtures
- Verify: parse → serialize → parse roundtrip produces identical output
- Verify: merge with known inputs produces expected output

**Key test scenarios:**
```
Config merge:
  - user + project merge respects precedence
  - missing project config falls back to user
  - unknown fields preserved on roundtrip
  - partial config is valid

Hooks validation:
  - valid hook accepted
  - invalid event type rejected
  - HTTP hook requires URL
  - command hook requires command path
  - if condition syntax validated

File watcher:
  - rapid changes debounced correctly
  - new file detected
  - file deletion detected
  - watcher recovers from transient FS errors

API:
  - PUT validates before write
  - invalid config returns 422 with field-level errors
  - concurrent writes serialized correctly
  - auth token required and verified
```

### 9.3 Frontend Tests

**Component tests (Vitest + @testing-library/svelte):**
- Each editor component renders with mock data
- Form interactions produce correct output
- Validation messages display on invalid input
- Project selector switches context correctly
- Connection status reflects store state

**Key test scenarios:**
```
HooksEditor:
  - renders existing hooks
  - adds new hook rule
  - switches between command and HTTP types
  - validates required fields before save

PermissionsEditor:
  - renders allow/deny/ask lists
  - adds new permission pattern
  - removes permission
  - switches default mode

PluginCard:
  - shows install button for uninstalled
  - shows toggle for installed
  - shows progress during install

EffectiveConfigView:
  - highlights overridden fields
  - shows source annotation for each field

ProjectActivation:
  - matrix checkboxes toggle correctly
  - saves per-project enabledPlugins
```

### 9.4 E2E Tests (Playwright)

**Core flows:**

1. **Settings edit roundtrip:**
   Open Settings → edit Permissions → Save → verify `settings.json` on disk matches

2. **Plugin lifecycle:**
   Browse marketplace → Install plugin → verify in installed list → disable → enable → uninstall

3. **Per-project override:**
   Select project A → disable plugin X → switch to project B → verify plugin X still enabled → verify project A's settings file correct

4. **Real-time sync:**
   Open GUI → externally modify `settings.json` → verify GUI reflects change within 2 seconds

5. **Effective config accuracy:**
   Set user-level value → set different project-level value → open Effective Config → verify merged result and source annotations

6. **Hooks editor:**
   Create new PreToolUse hook → set matcher, command, condition → save → verify JSON output

7. **Memory management:**
   Select project → view memory → create new entry → edit → delete → verify MEMORY.md index updated

### 9.5 Test Environment Isolation

All tests use temporary directories. No test touches real `~/.claude/` configuration.

```rust
struct TestFixture {
    claude_home: TempDir,       // simulates ~/.claude/
    project_dir: TempDir,       // simulates project .claude/
    daemon_port: u16,           // random available port
    daemon_handle: JoinHandle,  // spawned test daemon
    api_base: String,           // http://localhost:{port}/api/v1
}

impl TestFixture {
    async fn new() -> Self { ... }
    fn seed_settings(&self, json: &str) { ... }
    fn seed_project_settings(&self, json: &str) { ... }
    fn read_settings(&self) -> serde_json::Value { ... }
}
```

---

## 10. Acceptance Criteria

### 10.1 Module Acceptance

**Module 1: Settings Editor**
- [ ] Loads and correctly displays all known settings.json fields
- [ ] Unknown fields preserved on roundtrip (no data loss)
- [ ] Each sub-editor (Permissions, Hooks, Sandbox, Env, General, StatusLine) independently editable
- [ ] Hooks editor supports all event types, command/HTTP toggle, `if` conditions
- [ ] JSON preview updates in real-time as form changes
- [ ] Save validates before write; invalid config shows inline errors
- [ ] Per-project scope: select project → edit → saves to project settings

**Module 2: Plugins**
- [ ] Lists all installed plugins with status
- [ ] One-click enable/disable toggle
- [ ] Marketplace browser lists available plugins
- [ ] One-click install with progress streaming
- [ ] Uninstall with confirmation
- [ ] Add/remove marketplace sources
- [ ] Per-project activation matrix

**Module 3: Skills**
- [ ] Lists all skills (user + plugin-provided)
- [ ] View SKILL.md with markdown rendering
- [ ] Edit/create user skills
- [ ] Per-project enable/disable

**Module 4: Memory**
- [ ] Browse memory by project
- [ ] View/edit memory files (markdown with frontmatter)
- [ ] Create/delete memory entries
- [ ] MEMORY.md index auto-updated

**Module 5: MCP Servers**
- [ ] List servers grouped by scope
- [ ] Add server (stdio/SSE/HTTP transport forms)
- [ ] Edit/delete servers
- [ ] Per-project enable/disable

**Module 6: Effective Config**
- [ ] Displays merged config for selected project
- [ ] Each field annotated with source layer
- [ ] Overridden fields visually highlighted

**Module 7: Project Launcher**
- [ ] Select project, preview effective config
- [ ] Environment variable selection
- [ ] Launch Claude Code with correct config and env vars

**Module 8: App Settings**
- [ ] Theme switching (light/dark/system)
- [ ] Language preference
- [ ] Font customization
- [ ] Daemon connection management (local + remote)
- [ ] Project registration (add/remove directories)

### 10.2 Non-Functional Acceptance

| Metric | Target |
|--------|--------|
| **Startup time** | < 1 second (cold start, including daemon) |
| **Memory (idle)** | < 80 MB (GUI + daemon combined) |
| **Real-time sync latency** | < 2 seconds from external file change to GUI update |
| **Config write safety** | Atomic rename; no half-written files ever |
| **Schema resilience** | Unknown fields preserved; GUI doesn't crash on new Claude Code versions |
| **Connection recovery** | Disconnection shown to user; auto-reconnect with backoff; write ops disabled during disconnect |
| **macOS bundle** | .dmg < 15 MB |
| **Test coverage** | Rust: > 80% for claude-config, > 70% for claude-daemon; Frontend: key components covered |
| **Local run** | Daemon and GUI must start locally without panic; every phase must pass local run verification before considered complete |
| **No pollution** | App's own data (tokens, preferences) must NOT be written to `~/.claude/`; only Claude Code config goes there |

### 10.3 Acceptance Phases

**Phase 1: Core Infrastructure**
- [ ] Daemon starts, binds to port, serves health endpoint
- [ ] File watcher detects changes in `~/.claude/`
- [ ] GUI connects to daemon, WebSocket established
- [ ] Config read → display → save → read roundtrip is lossless
- [ ] `cargo test` and `vitest` pass

**Phase 2: Module-by-Module**
- [ ] Each module's checklist (10.1) fully checked
- [ ] E2E test covers each module's core flow

**Phase 3: Integration**
- [ ] Test on real `~/.claude/` in read-only mode (verify display accuracy)
- [ ] Full workflow: install plugin → configure hooks → set per-project profile → launch Claude Code
- [ ] External edit via vim → GUI real-time refresh verified
- [ ] All non-functional metrics (10.2) met

---

## 11. Scope Exclusions (V1)

Explicitly **not** in V1:
- Keybindings editor
- Rules management
- Config backup/restore
- AI chat integration
- Windows/Linux support (macOS only for V1)
- managed-settings.json editing (read-only display in Effective Config)
- Session/history/plans browsing
- Telemetry management

---

## 12. Open Questions (Resolved)

| Question | Resolution |
|----------|-----------|
| Frontend framework | Svelte 5 |
| Auth mechanism | API Key (local: file-based, remote: over TLS) |
| Project discovery | User manually registers projects |
| Per-project scope | Core principle: everything that CAN be per-project SHOULD be |
| Cross-platform | macOS first, others deferred |
| Daemon language | Rust (same as Tauri backend) |
