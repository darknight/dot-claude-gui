# Phase 3: Plugins + Skills + Memory Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build Plugins module (installed list, marketplace browser, one-click install/uninstall, per-project activation), Skills module (list with validation, read-only preview, per-project toggle), and Memory module (browse, view, edit, delete per project).

**Architecture:** The daemon reads plugin/skill/memory data directly from `~/.claude/` filesystem (JSON + markdown files). Plugin install/uninstall operations execute `claude` CLI commands via a command executor that streams output over WebSocket. Frontend modules follow the same three-panel pattern as Settings.

**Tech Stack:** Rust (serde, tokio::process for CLI execution), Svelte 5, TypeScript

**Prerequisite:** Phase 2 complete (all tests passing, local run verified)

---

## File Structure (Phase 3)

### Backend — New files

```
crates/claude-types/src/
├── plugins.rs              # Plugin, Marketplace, Blocklist types
├── skills.rs               # Skill types
└── memory.rs               # Memory types

crates/claude-daemon/src/
├── executor.rs             # CLI command executor (streams output via WS)
└── api/
    ├── plugins.rs          # Plugin CRUD + marketplace endpoints
    ├── skills.rs           # Skill list + validate endpoints
    └── memory.rs           # Memory CRUD endpoints
```

### Frontend — New files

```
src/lib/
├── api/types.ts            # (modify) Add plugin/skill/memory types
├── api/client.ts           # (modify) Add new API methods
├── stores/
│   ├── plugins.svelte.ts   # Plugin state management
│   ├── skills.svelte.ts    # Skills state management
│   └── memory.svelte.ts    # Memory state management
└── components/
    ├── plugins/
    │   ├── PluginsModule.svelte        # Orchestrator
    │   ├── InstalledPlugins.svelte     # Installed list with toggle
    │   ├── MarketplaceBrowser.svelte   # Browse + install
    │   ├── MarketplaceManager.svelte   # Add/remove marketplaces
    │   └── ProjectActivation.svelte    # Per-project matrix
    ├── skills/
    │   ├── SkillsModule.svelte         # Orchestrator
    │   ├── SkillList.svelte            # List with validation status
    │   └── SkillPreview.svelte         # Read-only SKILL.md viewer
    └── memory/
        ├── MemoryModule.svelte         # Orchestrator
        ├── MemoryList.svelte           # Memory file list
        └── MemoryEditor.svelte         # View/edit/delete memory
```

---

### Task 1: Backend — Plugin + Skill + Memory Types

**Files:**
- Create: `crates/claude-types/src/plugins.rs`
- Create: `crates/claude-types/src/skills.rs`
- Create: `crates/claude-types/src/memory.rs`
- Modify: `crates/claude-types/src/lib.rs`

- [ ] **Step 1: Create plugin types**

```rust
// crates/claude-types/src/plugins.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Installed plugin entry from installed_plugins.json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledPlugin {
    pub scope: String,
    pub install_path: String,
    pub version: String,
    pub installed_at: String,
    pub last_updated: String,
    #[serde(default)]
    pub git_commit_sha: Option<String>,
}

/// installed_plugins.json root structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPluginsFile {
    pub version: u32,
    pub plugins: HashMap<String, Vec<InstalledPlugin>>,
}

/// Known marketplace entry from known_marketplaces.json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnownMarketplace {
    pub source: MarketplaceGitSource,
    #[serde(default)]
    pub install_location: Option<String>,
    #[serde(default)]
    pub last_updated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceGitSource {
    pub source: String,
    pub repo: String,
}

/// marketplace.json root structure (inside each marketplace directory)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceManifest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub owner: Option<MarketplaceOwner>,
    #[serde(default)]
    pub plugins: Vec<MarketplacePlugin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceOwner {
    pub name: String,
    #[serde(default)]
    pub email: Option<String>,
}

/// Plugin entry within a marketplace manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplacePlugin {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    /// Source can be a string path or an object with source/url/sha
    #[serde(default)]
    pub source: Option<serde_json::Value>,
}

/// Blocklist entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlocklistFile {
    pub fetched_at: String,
    pub plugins: Vec<BlockedPlugin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedPlugin {
    pub plugin: String,
    pub added_at: String,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
}

// --- API response types ---

/// Unified plugin info for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginInfo {
    /// Full ID: "name@marketplace"
    pub id: String,
    pub name: String,
    pub marketplace: String,
    pub version: String,
    pub enabled: bool,
    pub blocked: bool,
    pub installed_at: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// Marketplace info for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketplaceInfo {
    pub id: String,
    pub repo: String,
    pub plugin_count: usize,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub last_updated: Option<String>,
}

/// Available plugin in a marketplace (for browsing)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailablePlugin {
    pub name: String,
    pub marketplace: String,
    pub installed: bool,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
}

/// Command execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandRequest {
    pub request_id: String,
}
```

- [ ] **Step 2: Create skill types**

```rust
// crates/claude-types/src/skills.rs
use serde::{Deserialize, Serialize};

/// Skill info for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillInfo {
    /// Unique ID (directory name)
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    /// "user" or "plugin:<plugin-name>"
    pub source: String,
    /// Path to SKILL.md
    pub path: String,
    /// Whether the SKILL.md frontmatter is valid
    pub valid: bool,
    /// Validation error message if invalid
    #[serde(default)]
    pub validation_error: Option<String>,
}
```

- [ ] **Step 3: Create memory types**

```rust
// crates/claude-types/src/memory.rs
use serde::{Deserialize, Serialize};

/// Memory project info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryProject {
    /// Encoded project directory name
    pub id: String,
    /// Decoded human-readable project path
    pub project_path: String,
    /// Number of memory files
    pub file_count: usize,
}

/// Memory file entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryFile {
    pub filename: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub memory_type: Option<String>,
}

/// Memory file with full content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryFileDetail {
    pub filename: String,
    pub content: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub memory_type: Option<String>,
}

/// Request to create/update memory
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMemoryRequest {
    pub content: String,
}
```

- [ ] **Step 4: Update lib.rs**

Add to `crates/claude-types/src/lib.rs`:
```rust
pub mod memory;
pub mod plugins;
pub mod skills;
```

- [ ] **Step 5: Verify compilation and commit**

Run: `cargo build -p claude-types`
```bash
git add crates/claude-types/src/
git commit -m "feat: add plugin, skill, and memory type definitions"
```

---

### Task 2: Backend — CLI Command Executor

**Files:**
- Create: `crates/claude-daemon/src/executor.rs`

- [ ] **Step 1: Implement CLI executor**

The executor runs `claude` CLI commands asynchronously and streams stdout/stderr via WebSocket.

```rust
// crates/claude-daemon/src/executor.rs
use crate::state::AppState;
use claude_types::events::WsEvent;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{error, info};
use uuid::Uuid;

pub struct CommandResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Execute a claude CLI command, streaming output via WebSocket.
/// Returns the command result after completion.
pub async fn execute_claude_command(
    state: &AppState,
    args: &[&str],
    request_id: &str,
) -> anyhow::Result<CommandResult> {
    info!("executing: claude {}", args.join(" "));

    let mut child = Command::new("claude")
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let mut stdout_lines = BufReader::new(stdout).lines();
    let mut stderr_lines = BufReader::new(stderr).lines();

    let mut stdout_buf = String::new();
    let mut stderr_buf = String::new();

    let rid = request_id.to_string();

    loop {
        tokio::select! {
            line = stdout_lines.next_line() => {
                match line {
                    Ok(Some(line)) => {
                        stdout_buf.push_str(&line);
                        stdout_buf.push('\n');
                        state.broadcast(WsEvent::CommandOutput {
                            request_id: rid.clone(),
                            stream: "stdout".to_string(),
                            data: line,
                        });
                    }
                    Ok(None) => break,
                    Err(e) => {
                        error!("stdout read error: {}", e);
                        break;
                    }
                }
            }
            line = stderr_lines.next_line() => {
                match line {
                    Ok(Some(line)) => {
                        stderr_buf.push_str(&line);
                        stderr_buf.push('\n');
                        state.broadcast(WsEvent::CommandOutput {
                            request_id: rid.clone(),
                            stream: "stderr".to_string(),
                            data: line,
                        });
                    }
                    Ok(None) => {}
                    Err(e) => {
                        error!("stderr read error: {}", e);
                    }
                }
            }
        }
    }

    let status = child.wait().await?;
    let exit_code = status.code().unwrap_or(-1);

    state.broadcast(WsEvent::CommandCompleted {
        request_id: rid,
        exit_code,
    });

    Ok(CommandResult {
        exit_code,
        stdout: stdout_buf,
        stderr: stderr_buf,
    })
}

/// Generate a unique request ID for command tracking
pub fn new_request_id() -> String {
    Uuid::new_v4().to_string()
}
```

- [ ] **Step 2: Add module to lib.rs**

Add `pub mod executor;` to `crates/claude-daemon/src/lib.rs`.

- [ ] **Step 3: Verify and commit**

```bash
cargo build -p claude-daemon
git add crates/claude-daemon/src/executor.rs crates/claude-daemon/src/lib.rs
git commit -m "feat: add CLI command executor with WebSocket output streaming"
```

---

### Task 3: Backend — Plugin API Endpoints

**Files:**
- Create: `crates/claude-daemon/src/api/plugins.rs`
- Modify: `crates/claude-daemon/src/api/mod.rs`
- Modify: `crates/claude-daemon/src/server.rs`

- [ ] **Step 1: Implement plugin endpoints**

The plugin API reads from the filesystem and delegates install/uninstall to the CLI executor.

Endpoints:
- `GET /api/v1/plugins` — list installed plugins (reads installed_plugins.json, cross-references enabledPlugins from settings and blocklist)
- `POST /api/v1/plugins/:id/toggle` — enable/disable plugin (updates user settings.json enabledPlugins)
- `GET /api/v1/marketplaces` — list registered marketplaces (reads known_marketplaces.json)
- `GET /api/v1/marketplaces/:id/plugins` — list available plugins in a marketplace (reads marketplace.json)
- `POST /api/v1/plugins/install` — install plugin (executes `claude plugin install <name> --marketplace <id>`)
- `POST /api/v1/plugins/:id/uninstall` — uninstall plugin (executes `claude plugin uninstall <id>`)
- `POST /api/v1/marketplaces` — add marketplace (executes `claude plugin marketplace add <repo>`)
- `DELETE /api/v1/marketplaces/:id` — remove marketplace (executes `claude plugin marketplace remove <id>`)

Key implementation: read JSON files from `state.inner.claude_home.join("plugins/")`. Parse with the types from Task 1. For enabled/disabled status, read `enabledPlugins` from the user settings.

- [ ] **Step 2: Add routes to server.rs**

Add all plugin and marketplace routes to the protected router.

- [ ] **Step 3: Verify and commit**

```bash
cargo build -p claude-daemon
git add crates/claude-daemon/src/api/
git commit -m "feat: add plugin and marketplace API endpoints"
```

---

### Task 4: Backend — Skill API Endpoints

**Files:**
- Create: `crates/claude-daemon/src/api/skills.rs`
- Modify: `crates/claude-daemon/src/server.rs`

- [ ] **Step 1: Implement skill endpoints**

Endpoints:
- `GET /api/v1/skills` — list all skills (scan `~/.claude/skills/` directories, parse SKILL.md frontmatter, validate format)

Validation rules for SKILL.md:
- Must start with `---` (YAML frontmatter)
- Must have `name` field in frontmatter
- Must have `description` field in frontmatter
- Frontmatter must end with `---`

For each skill directory:
1. Read `SKILL.md`
2. Parse YAML frontmatter (between `---` delimiters)
3. Check `name` and `description` exist
4. Return `SkillInfo` with `valid: true/false` and `validation_error` if invalid

Also scan plugin-provided skills: for each installed plugin, check if it has a `skills/` directory with SKILL.md files.

- [ ] **Step 2: Add routes and commit**

```bash
cargo build -p claude-daemon
git add crates/claude-daemon/src/api/skills.rs crates/claude-daemon/src/server.rs
git commit -m "feat: add skill list and validation API endpoint"
```

---

### Task 5: Backend — Memory API Endpoints

**Files:**
- Create: `crates/claude-daemon/src/api/memory.rs`
- Modify: `crates/claude-daemon/src/server.rs`

- [ ] **Step 1: Implement memory endpoints**

Endpoints:
- `GET /api/v1/memory` — list all projects that have memory (scan `~/.claude/projects/*/memory/`)
- `GET /api/v1/memory/:project_id` — list memory files for a project
- `GET /api/v1/memory/:project_id/:filename` — read a memory file (full content)
- `PUT /api/v1/memory/:project_id/:filename` — update a memory file
- `DELETE /api/v1/memory/:project_id/:filename` — delete a memory file

Project ID is the encoded directory name (e.g. `-Users-eric-yao-workspace-darknight`).

For listing memory files, parse the YAML frontmatter of each `.md` file to extract `name`, `description`, `type` fields.

For MEMORY.md index: when creating/deleting files, also update the MEMORY.md index (add/remove the line referencing the file).

- [ ] **Step 2: Add routes and commit**

```bash
cargo build -p claude-daemon
git add crates/claude-daemon/src/api/memory.rs crates/claude-daemon/src/server.rs
git commit -m "feat: add memory CRUD API endpoints"
```

---

### Task 6: Backend — Integration Tests

**Files:**
- Modify: `crates/claude-daemon/tests/api_test.rs`

- [ ] **Step 1: Add plugin list test**

Test that `GET /api/v1/plugins` returns a (possibly empty) array when the test daemon has no plugins installed.

- [ ] **Step 2: Add skill list test**

Test that `GET /api/v1/skills` returns a (possibly empty) array.

- [ ] **Step 3: Add memory list test**

Test that `GET /api/v1/memory` returns an array. Create a memory file in the test fixture, then verify it appears in the list and can be read via GET.

- [ ] **Step 4: Run all tests and commit**

```bash
cargo test --workspace
git add crates/claude-daemon/tests/
git commit -m "test: add plugin, skill, and memory API integration tests"
```

---

### Task 7: Frontend — Types + API Client Extensions

**Files:**
- Modify: `src/lib/api/types.ts`
- Modify: `src/lib/api/client.ts`

- [ ] **Step 1: Add TypeScript types**

Add to types.ts:

```typescript
// --- Plugins ---
export interface PluginInfo {
  id: string;
  name: string;
  marketplace: string;
  version: string;
  enabled: boolean;
  blocked: boolean;
  installedAt: string;
  description?: string;
}

export interface MarketplaceInfo {
  id: string;
  repo: string;
  pluginCount: number;
  description?: string;
  lastUpdated?: string;
}

export interface AvailablePlugin {
  name: string;
  marketplace: string;
  installed: boolean;
  description?: string;
  version?: string;
  category?: string;
}

// --- Skills ---
export interface SkillInfo {
  id: string;
  name: string;
  description?: string;
  source: string;
  path: string;
  valid: boolean;
  validationError?: string;
}

// --- Memory ---
export interface MemoryProject {
  id: string;
  projectPath: string;
  fileCount: number;
}

export interface MemoryFile {
  filename: string;
  name?: string;
  description?: string;
  memoryType?: string;
}

export interface MemoryFileDetail {
  filename: string;
  content: string;
  name?: string;
  description?: string;
  memoryType?: string;
}
```

- [ ] **Step 2: Add API client methods**

Add to DaemonClient:

```typescript
// Plugins
async listPlugins(): Promise<PluginInfo[]>
async togglePlugin(id: string, enabled: boolean): Promise<void>
async installPlugin(name: string, marketplace: string): Promise<{ requestId: string }>
async uninstallPlugin(id: string): Promise<{ requestId: string }>
// Marketplaces
async listMarketplaces(): Promise<MarketplaceInfo[]>
async getMarketplacePlugins(marketplaceId: string): Promise<AvailablePlugin[]>
async addMarketplace(repo: string): Promise<{ requestId: string }>
async removeMarketplace(id: string): Promise<{ requestId: string }>
// Skills
async listSkills(): Promise<SkillInfo[]>
// Memory
async listMemoryProjects(): Promise<MemoryProject[]>
async listMemoryFiles(projectId: string): Promise<MemoryFile[]>
async getMemoryFile(projectId: string, filename: string): Promise<MemoryFileDetail>
async updateMemoryFile(projectId: string, filename: string, content: string): Promise<void>
async deleteMemoryFile(projectId: string, filename: string): Promise<void>
```

- [ ] **Step 3: Verify and commit**

```bash
pnpm build
git add src/lib/api/
git commit -m "feat: add plugin, skill, and memory TypeScript types and API methods"
```

---

### Task 8: Frontend — Plugins Module (Installed + Toggle)

**Files:**
- Create: `src/lib/stores/plugins.svelte.ts`
- Create: `src/lib/components/plugins/PluginsModule.svelte`
- Create: `src/lib/components/plugins/InstalledPlugins.svelte`
- Modify: `src/App.svelte`

- [ ] **Step 1: Create plugins store**

```typescript
class PluginsStore {
  plugins = $state<PluginInfo[]>([]);
  marketplaces = $state<MarketplaceInfo[]>([]);
  loading = $state(false);
  error = $state<string>("");

  async loadPlugins() { ... }
  async loadMarketplaces() { ... }
  async togglePlugin(id: string, enabled: boolean) { ... }
}
export const pluginsStore = new PluginsStore();
```

- [ ] **Step 2: Create InstalledPlugins component**

Shows each installed plugin as a card with:
- Name, marketplace, version
- Enable/disable toggle switch
- Blocked indicator (if blocked)
- Uninstall button

- [ ] **Step 3: Create PluginsModule orchestrator**

Sub-navigation: Installed, Marketplace, Manage Marketplaces, Per-Project
Routes to the correct sub-view.

- [ ] **Step 4: Wire into App.svelte**

When `currentModule === "plugins"`, show PluginsModule in the detail panel.

- [ ] **Step 5: Verify and commit**

```bash
pnpm build
git add src/lib/stores/plugins.svelte.ts src/lib/components/plugins/ src/App.svelte
git commit -m "feat: add Plugins module with installed list and enable/disable toggle"
```

---

### Task 9: Frontend — Marketplace Browser + Manager

**Files:**
- Create: `src/lib/components/plugins/MarketplaceBrowser.svelte`
- Create: `src/lib/components/plugins/MarketplaceManager.svelte`

- [ ] **Step 1: Create MarketplaceBrowser**

- Dropdown to select marketplace
- Lists available plugins in selected marketplace
- Each entry: name, description, version, category, install status
- "Install" button for uninstalled plugins (triggers install, shows progress via WS events)
- Progress display: shows command output while installing

- [ ] **Step 2: Create MarketplaceManager**

- List registered marketplaces
- Each: id, repo, plugin count, last updated
- "Add Marketplace" form: GitHub repo input + Add button
- Remove button per marketplace

- [ ] **Step 3: Verify and commit**

```bash
pnpm build
git add src/lib/components/plugins/
git commit -m "feat: add marketplace browser and marketplace manager"
```

---

### Task 10: Frontend — Per-Project Plugin Activation

**Files:**
- Create: `src/lib/components/plugins/ProjectActivation.svelte`

- [ ] **Step 1: Implement matrix view**

- Shows a table: rows = installed plugins, columns include Global and selected project
- Checkboxes for enable/disable per project
- Saves to project-level settings.json enabledPlugins field

- [ ] **Step 2: Verify and commit**

```bash
pnpm build
git add src/lib/components/plugins/
git commit -m "feat: add per-project plugin activation matrix"
```

---

### Task 11: Frontend — Skills Module

**Files:**
- Create: `src/lib/stores/skills.svelte.ts`
- Create: `src/lib/components/skills/SkillsModule.svelte`
- Create: `src/lib/components/skills/SkillList.svelte`
- Create: `src/lib/components/skills/SkillPreview.svelte`
- Modify: `src/App.svelte`

- [ ] **Step 1: Create skills store**

```typescript
class SkillsStore {
  skills = $state<SkillInfo[]>([]);
  loading = $state(false);
  async loadSkills() { ... }
}
export const skillsStore = new SkillsStore();
```

- [ ] **Step 2: Create SkillList**

Shows each skill with:
- Name, description, source (user/plugin)
- Validation status: green check if valid, red X with error message if invalid
- Click to select and preview

- [ ] **Step 3: Create SkillPreview**

Read-only markdown-rendered view of SKILL.md content. Since we don't have a markdown renderer yet, display as a `<pre>` block with proper formatting (or use basic HTML rendering of the markdown).

- [ ] **Step 4: Create SkillsModule orchestrator + wire into App.svelte**

Sub-navigation shows skill list in middle panel, preview in detail panel.

- [ ] **Step 5: Verify and commit**

```bash
pnpm build
git add src/lib/stores/skills.svelte.ts src/lib/components/skills/ src/App.svelte
git commit -m "feat: add Skills module with list, validation status, and preview"
```

---

### Task 12: Frontend — Memory Module

**Files:**
- Create: `src/lib/stores/memory.svelte.ts`
- Create: `src/lib/components/memory/MemoryModule.svelte`
- Create: `src/lib/components/memory/MemoryList.svelte`
- Create: `src/lib/components/memory/MemoryEditor.svelte`
- Modify: `src/App.svelte`

- [ ] **Step 1: Create memory store**

```typescript
class MemoryStore {
  projects = $state<MemoryProject[]>([]);
  activeProjectId = $state<string | null>(null);
  files = $state<MemoryFile[]>([]);
  activeFile = $state<MemoryFileDetail | null>(null);
  loading = $state(false);
  saving = $state(false);
  error = $state<string>("");

  async loadProjects() { ... }
  async loadFiles(projectId: string) { ... }
  async loadFile(projectId: string, filename: string) { ... }
  async saveFile(projectId: string, filename: string, content: string) { ... }
  async deleteFile(projectId: string, filename: string) { ... }
}
export const memoryStore = new MemoryStore();
```

- [ ] **Step 2: Create MemoryList**

- Project selector dropdown (from memory projects, not registered projects)
- Lists memory files for selected project: filename, name, type
- Click to select and view/edit

- [ ] **Step 3: Create MemoryEditor**

- Display memory file content in a textarea (editable)
- Shows frontmatter fields (name, description, type) parsed from content
- Save and Delete buttons
- Delete shows confirmation dialog

- [ ] **Step 4: Create MemoryModule orchestrator + wire into App.svelte**

- [ ] **Step 5: Verify and commit**

```bash
pnpm build
git add src/lib/stores/memory.svelte.ts src/lib/components/memory/ src/App.svelte
git commit -m "feat: add Memory module with browse, view, edit, and delete"
```

---

### Task 13: Full Build + Local Run Verification

- [ ] **Step 1: Run all Rust tests**

Run: `cargo test --workspace`
Expected: All tests pass

- [ ] **Step 2: Run frontend build**

Run: `pnpm build`
Expected: Succeeds

- [ ] **Step 3: Local run verification**

Start daemon: `cargo run -p claude-daemon -- --port 7890 --token dev-token`
Expected: No panic, loads settings, starts listening

Start GUI: `cargo run -p dot-claude-gui`
Expected: No panic, window opens, runs for 5+ seconds

- [ ] **Step 4: Commit any remaining changes**

---

## Phase 3 Completion Checklist

- [ ] Plugin types + API endpoints (list, toggle, install/uninstall, marketplace CRUD)
- [ ] CLI command executor with WebSocket output streaming
- [ ] Skill types + API endpoint (list with validation)
- [ ] Memory types + API endpoints (list projects, list files, read/update/delete)
- [ ] Frontend Plugins module: installed list, marketplace browser, marketplace manager, per-project activation
- [ ] Frontend Skills module: list with validation status, read-only preview
- [ ] Frontend Memory module: browse by project, view/edit/delete
- [ ] Integration tests for new endpoints
- [ ] All Rust tests passing
- [ ] Frontend building
- [ ] Local run: daemon + GUI no panic
