# Phase 8 Stage 2 — New Shell & Accounts Mode Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the 9-module 3-panel UI with a mode-based 2-panel shell (Accounts / Projects tabs + right-corner App-settings gear). Wire the Account mode to a real per-account data plane: selecting `@work` makes every facet (Settings/Plugins/Skills/CLAUDE.md/Memory/MCP) read/write from `~/.dot-claude-gui/accounts/work/`. Project mode is a stub list — Stage 3 brings it to life.

**Architecture:** Backend introduces an "active account dir" concept on `AppState` (a swap-on-IPC `PathBuf`) and refactors every existing user-layer IPC to read from it instead of the fixed `claude_home`. Frontend replaces `App.svelte` with a top-bar + mode-aware sidebar + facet-tab main panel; existing module orchestrators are reused inside facet wrappers (no rewrite of the editors themselves). Account selection in the sidebar triggers `set_active_account` → all stores reload from the new dir.

**Tech Stack:** Rust 1.x (Tauri 2.0, tokio, serde, notify), Svelte 5 runes, TypeScript strict, pnpm.

**Spec:** `docs/superpowers/specs/2026-05-11-phase8-mode-based-redesign-design.md`

**Prior stage:** Stage 1 (`docs/superpowers/plans/2026-05-11-phase8-stage1-data-migration.md`) landed the v2 data layer, migration, and `gui_projects` IPC. Stage 2 builds the user-facing shell on top of it.

---

## File map

**New (backend):**
- `src-tauri/src/commands/account_session.rs` — `set_active_account`, `account_overview` IPCs

**Modify (backend):**
- `src-tauri/src/state.rs` — add `active_account_dir: tokio::sync::RwLock<PathBuf>` to `AppStateInner`; helper `current_dir() -> PathBuf`; rename existing `load_user_settings` to read from `current_dir`
- `src-tauri/src/app_config.rs` — add `pub fn account_dir(home: &Path, name: &str) -> PathBuf` resolver
- `src-tauri/src/commands/{config,plugins,skills,claudemd,memory,mcp}.rs` — read from `state.current_dir().await` instead of `state.inner.claude_home`
- `src-tauri/src/watcher.rs` — restart-on-switch helper; watcher follows `current_dir()`
- `src-tauri/src/commands/mod.rs` — register `account_session`
- `src-tauri/src/lib.rs` — register new IPCs in `tauri::generate_handler![...]`

**New (frontend):**
- `src/lib/stores/mode.svelte.ts` — current mode + selectedAccount + selectedProject
- `src/lib/components/shell/TopBar.svelte` — mode tabs + gear button
- `src/lib/components/shell/AppSettingsModal.svelte` — modal wrapper for existing AppSettingsView
- `src/lib/components/shell/AccountSidebar.svelte` — account list + `+ Add Account`
- `src/lib/components/shell/ProjectSidebar.svelte` — project list grouped by account + `+ Add Project`
- `src/lib/components/account-mode/AccountModeView.svelte` — facet tab strip + slot
- `src/lib/components/account-mode/Overview.svelte` — new Overview facet
- `src/lib/components/account-mode/SettingsFacet.svelte` — inline sub-nav + reuse SettingsEditor
- `src/lib/components/account-mode/PluginsFacet.svelte` — inline sub-nav + reuse PluginsModule
- `src/lib/components/account-mode/SkillsFacet.svelte` — reuse SkillsModule
- `src/lib/components/account-mode/ClaudeMdFacet.svelte` — reuse ClaudeMdModule with inline list
- `src/lib/components/account-mode/MemoryFacet.svelte` — reuse MemoryModule with inline list
- `src/lib/components/account-mode/McpFacet.svelte` — inline sub-nav + reuse McpModule
- `src/lib/components/project-mode/ProjectModePlaceholder.svelte` — "Coming in Stage 3" stub

**Modify (frontend):**
- `src/App.svelte` — full layout rewrite (drop 3-panel, drop `activeNav`, mount TopBar + mode-aware sidebar + main)
- `src/lib/api/types.ts` — add `AccountOverview` interface; extend `AppConfig` with optional `lastMode`, `lastAccount`, `lastProject`
- `src/lib/stores/appsettings.svelte.ts` — schema extension defaults for the 3 new optional fields
- `src/lib/ipc/client.ts` — add `setActiveAccount(name)`, `accountOverview(name)` methods
- `src/lib/i18n/locales/locales/zh-CN.json`, `src/lib/i18n/locales/locales/en-US.json`, `src/lib/i18n/locales/locales/ja-JP.json` — new JSON keys for mode tabs / facet titles / Overview (extension locales `es-ES`/`fr-FR`/`ko-KR` use Partial maps; fine to leave empty — runtime falls back to en-US)
- `src/lib/components/plugins/PluginsModule.svelte` — accept optional `sections` prop or render inline sub-nav when wrapped by facet (see Task 14)

**Leave alone (Stage 3 territory):**
- `src/lib/components/launcher/` (LauncherView/LauncherList stubs from Stage 1 — Stage 3 rebuilds)
- `src/lib/components/effective/EffectiveConfigView.svelte` (Project facet in Stage 3)
- `src/lib/components/shared/ScopeSelector.svelte` (no longer referenced after App.svelte rewrite; can be deleted in Stage 4)
- `commands::launcher::*`, `commands::accounts::*`, `commands::gui_projects::*` IPCs (Stage 3 wires them more)

---

### Task 1: Account-dir resolver in `app_config.rs`

**Files:**
- Modify: `src-tauri/src/app_config.rs`

- [ ] **Step 1: Write failing tests**

Append to the `tests` module in `src-tauri/src/app_config.rs`:

```rust
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
```

- [ ] **Step 2: Run tests, expect failure**

```bash
cargo test -p dot-claude-gui app_config::tests::account_dir
```

Expected: 3 tests fail with "no function `account_dir`".

- [ ] **Step 3: Implement `account_dir`**

Append to `src-tauri/src/app_config.rs` ABOVE the `#[cfg(test)]` block:

```rust
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
```

- [ ] **Step 4: Run tests, expect green**

```bash
cargo test -p dot-claude-gui app_config::tests::account_dir
```

Expected: 3 tests pass; the rest of the `app_config::tests` suite still passes too.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/app_config.rs
git commit -m "feat(app-config): account_dir resolver (native vs gui account)"
```

---

### Task 2: `AppState.active_account_dir` + `current_dir()` helper

**Files:**
- Modify: `src-tauri/src/state.rs`

The existing `AppStateInner.claude_home` field is `PathBuf` (immutable post-construction). For Stage 2 we need a swappable directory that all user-layer IPCs read from. Keep `claude_home` as the **resolved home directory** (`~/`); rename mental model from "claude home" to "user home". Add `active_account_dir` as the currently-targeted account dir. The native `~/.claude/` is the default initial value.

- [ ] **Step 1: Update `AppStateInner`**

Find `pub struct AppStateInner { ... }` (around line 20) and add `active_account_dir`:

```rust
pub struct AppStateInner {
    pub claude_home: PathBuf,
    /// Currently-targeted account directory. Defaults to `claude_home`
    /// (native ~/.claude/). Mutated via `commands::account_session::set_active_account`.
    pub active_account_dir: RwLock<PathBuf>,
    pub projects_file: Option<PathBuf>,
    pub user_settings: RwLock<Settings>,
    pub project_settings: RwLock<HashMap<String, Settings>>,
    pub local_settings: RwLock<HashMap<String, Settings>>,
    pub projects: RwLock<Vec<ProjectInfo>>,
    pub started_at: std::time::Instant,
}
```

- [ ] **Step 2: Update constructors**

`with_projects_file`:

```rust
pub fn with_projects_file(claude_home: PathBuf, projects_file: Option<PathBuf>) -> Self {
    Self {
        inner: Arc::new(AppStateInner {
            active_account_dir: RwLock::new(claude_home.clone()),
            claude_home,
            projects_file,
            user_settings: RwLock::new(Settings::default()),
            project_settings: RwLock::new(HashMap::new()),
            local_settings: RwLock::new(HashMap::new()),
            projects: RwLock::new(Vec::new()),
            started_at: std::time::Instant::now(),
        }),
    }
}
```

- [ ] **Step 3: Add `current_dir()` accessor**

In the same `impl AppState`, ABOVE `load_user_settings`:

```rust
/// Snapshot of the currently-active account dir. All IPC handlers that
/// read user-layer files (settings, plugins, skills, CLAUDE.md, memory,
/// MCP) should go through this rather than `inner.claude_home`.
pub async fn current_dir(&self) -> PathBuf {
    self.inner.active_account_dir.read().await.clone()
}

/// Swap the active account dir. Caller is responsible for downstream
/// invalidation (cache reload + watcher restart) — see
/// `commands::account_session::set_active_account`.
pub async fn set_active_account_dir(&self, dir: PathBuf) {
    *self.inner.active_account_dir.write().await = dir;
}
```

- [ ] **Step 4: Update `load_user_settings` to read from active dir**

```rust
pub async fn load_user_settings(&self) -> Result<()> {
    let dir = self.current_dir().await;
    let settings_path = dir.join("settings.json");
    let settings = read_settings(&settings_path)?;
    *self.inner.user_settings.write().await = settings;
    Ok(())
}
```

- [ ] **Step 5: Update the test in this file**

The existing `app_state_new_starts_empty` test reads `state.inner.claude_home`. That's still valid. The `app_state_loads_user_settings_from_disk` test currently writes `settings.json` directly inside `dir` and expects it to be picked up. After this refactor, it still works because the default `active_account_dir == claude_home`. Run the existing tests to confirm:

```bash
cargo test -p dot-claude-gui state::tests
```

Expected: 2 tests pass.

- [ ] **Step 6: Build all**

```bash
cargo build -p dot-claude-gui
```

Expected: clean (warnings about new `active_account_dir` field unused outside state.rs are fine — Task 3 wires it up).

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/state.rs
git commit -m "feat(state): add active_account_dir + current_dir() accessor"
```

---

### Task 3: Refactor user-layer IPCs to read from `current_dir`

**Files:**
- Modify: `src-tauri/src/commands/config.rs`
- Modify: `src-tauri/src/commands/plugins.rs`
- Modify: `src-tauri/src/commands/skills.rs`
- Modify: `src-tauri/src/commands/claudemd.rs`
- Modify: `src-tauri/src/commands/memory.rs`
- Modify: `src-tauri/src/commands/mcp.rs`
- Modify: `src-tauri/src/watcher.rs`

The change is mechanical: anywhere a handler reads `state.inner.claude_home` (or builds a path off it like `state.inner.claude_home.join("settings.json")`), replace with `state.current_dir().await` (or its joined form). Project-layer logic (paths under `<project>/.claude/`) is **not** affected — only user-layer.

- [ ] **Step 1: Inventory**

```bash
grep -rn "state\.inner\.claude_home\|inner\.claude_home" src-tauri/src/commands/ src-tauri/src/watcher.rs
```

This produces an exact list of sites to update. Walk each one, decide:
- **User-layer site** (e.g., `claude_home.join("settings.json")`, `claude_home.join("plugins")`, `claude_home.join("CLAUDE.md")`, etc.) → switch to `state.current_dir().await.join(...)`.
- **Pure home-derived site** (e.g., temporarily computing the user's actual home for unrelated reasons) → leave alone if not user-layer.

Most matches will be user-layer. Make the substitution in each file.

- [ ] **Step 2: Update each file**

For each command file (config, plugins, skills, claudemd, memory, mcp):

1. Read the file
2. Replace `state.inner.claude_home.join(...)` → `state.current_dir().await.join(...)` where appropriate
3. Replace bare `state.inner.claude_home.clone()` → `state.current_dir().await` where the value is used as the user-layer root

Note: `mcp.rs` and `plugins.rs` invoke the `claude` subprocess for some operations. Those subprocess calls should pass `CLAUDE_CONFIG_DIR=<current_dir>` so the CLI also targets the active account. Update them accordingly.

- [ ] **Step 3: Update `watcher.rs`**

The watcher currently registers `claude_home` and the projects dir. Add `active_account_dir` to its watch set (or replace `claude_home`'s entry with the active dir). For Stage 2 a simple approach: watch `claude_home` AND the gui-accounts root (`~/.dot-claude-gui/accounts/`) — both are stable across account switches, so no restart needed for ordinary use. The watcher emits `config-changed`; downstream code already filters by source.

Look at the existing `start_watcher` signature and where it adds paths. Add:
```rust
// Watch the GUI accounts root so events fire for any account's settings.json.
if let Some(home) = dirs_next::home_dir() {
    let gui_accounts = home.join(".dot-claude-gui").join("accounts");
    if gui_accounts.exists() {
        // ignore errors — best-effort
        let _ = watcher.watch(&gui_accounts, RecursiveMode::Recursive);
    }
}
```

This avoids the complexity of stop-and-restart on every account switch. The cost is some extra `config-changed` events for accounts the user isn't currently viewing — the frontend filters them when stores reload.

- [ ] **Step 4: Build + run tests**

```bash
cargo build -p dot-claude-gui
cargo test -p dot-claude-gui
```

Expected: clean build; all tests pass (the existing test suite uses `state.inner.claude_home` directly for assertions and the default `active_account_dir == claude_home` makes the tests still pass).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/ src-tauri/src/watcher.rs
git commit -m "refactor(commands): read user-layer paths from active_account_dir"
```

---

### Task 4: `set_active_account` IPC

**Files:**
- Create: `src-tauri/src/commands/account_session.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs` (register handler)

- [ ] **Step 1: Create the module**

`src-tauri/src/commands/account_session.rs`:

```rust
// src-tauri/src/commands/account_session.rs
//
// IPCs that control "which account the GUI is currently inspecting".
// Distinct from CLAUDE_CONFIG_DIR injection (which is per-launch).

use std::path::PathBuf;
use tauri::State;

use crate::app_config::{account_dir, read_config, DEFAULT_ACCOUNT_NAME};
use crate::state::AppState;

fn config_path() -> Result<PathBuf, String> {
    let dir = dirs_next::home_dir()
        .ok_or("cannot determine home directory")?
        .join(".dot-claude-gui");
    Ok(dir.join("config.json"))
}

/// Switch the active account. Validates `name` against `config.json.accounts`.
/// On success, the new dir is `~/.claude/` (for `default`) or
/// `~/.dot-claude-gui/accounts/<name>/`. Reloads the user-settings cache so
/// subsequent reads serve the new account.
#[tauri::command]
pub async fn set_active_account(
    name: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Validate against config.json
    let cfg = read_config(&config_path()?)?;
    let known = name == DEFAULT_ACCOUNT_NAME
        || cfg.accounts.iter().any(|a| a.name == name);
    if !known {
        return Err(format!("unknown_account: {name}"));
    }

    let home = dirs_next::home_dir().ok_or("cannot determine home directory")?;
    let new_dir = account_dir(&home, &name);

    state.set_active_account_dir(new_dir.clone()).await;

    // Refresh the user-settings cache for the new dir. Errors are non-fatal
    // (account may not yet have a settings.json on first visit).
    if let Err(e) = state.load_user_settings().await {
        tracing::warn!("failed to reload user settings after account switch: {e}");
    }

    Ok(new_dir.to_string_lossy().to_string())
}
```

- [ ] **Step 2: Register the module**

`src-tauri/src/commands/mod.rs`:

```rust
pub mod account_session;
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

- [ ] **Step 3: Register the IPC handler**

`src-tauri/src/lib.rs` — inside `tauri::generate_handler![...]` (near `commands::accounts::*` makes sense):

```rust
            commands::account_session::set_active_account,
```

- [ ] **Step 4: Write tests**

The handler uses `tauri::State` which is awkward to construct in unit tests. Skip explicit test; the IPC will be exercised end-to-end in Task 22 (smoke test). Alternative: factor the validation+resolve logic into a pure helper and unit-test that. For Stage 2, prefer minimal new test surface.

- [ ] **Step 5: Build**

```bash
cargo build -p dot-claude-gui
```

Expected: clean.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/commands/account_session.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat(account-session): set_active_account IPC + handler registration"
```

---

### Task 5: `account_overview` IPC

**Files:**
- Modify: `src-tauri/src/commands/account_session.rs`
- Modify: `src-tauri/src/lib.rs` (register handler)

The Overview facet (Task 13) needs a single batched read: status (loggedIn / email), configDir, counts (projects under this account, plugins installed, skills available). The frontend could compose this from existing IPCs, but a dedicated command keeps the facet logic tiny and avoids 4 separate round-trips.

- [ ] **Step 1: Add the types and command**

Append to `src-tauri/src/commands/account_session.rs`:

```rust
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountOverview {
    pub name: String,
    pub display_name: String,
    pub is_native: bool,
    pub config_dir: String,
    pub logged_in: bool,
    pub email: Option<String>,
    pub project_count: u32,
    pub plugin_count: u32,
    pub skill_count: u32,
}

/// Fetch a one-shot summary for the Account > Overview facet.
/// Does NOT switch the active account — read-only.
#[tauri::command]
pub async fn account_overview(name: String) -> Result<AccountOverview, String> {
    let cfg = read_config(&config_path()?)?;
    let acct = if name == DEFAULT_ACCOUNT_NAME {
        cfg.accounts.iter().find(|a| a.name == DEFAULT_ACCOUNT_NAME).cloned()
    } else {
        cfg.accounts.iter().find(|a| a.name == name).cloned()
    }
    .ok_or_else(|| format!("unknown_account: {name}"))?;

    let home = dirs_next::home_dir().ok_or("cannot determine home directory")?;
    let dir = account_dir(&home, &name);

    // Counts: directory listings, errors → 0.
    let project_count = std::fs::read_dir(dir.join("projects"))
        .map(|it| it.filter_map(|e| e.ok()).filter(|e| e.path().is_dir()).count() as u32)
        .unwrap_or(0);

    let plugin_count = read_plugin_count(&dir);

    let skill_count = std::fs::read_dir(dir.join("skills"))
        .map(|it| it.filter_map(|e| e.ok()).filter(|e| e.path().is_dir()).count() as u32)
        .unwrap_or(0);

    // OAuth status: reuse the existing accounts::get_account_status logic.
    // For Stage 2, inline a lightweight check: ~/.claude.json or
    // <account_dir>/.claude.json exists AND contains an oauthAccount block.
    let (logged_in, email) = read_oauth_status(&dir);

    Ok(AccountOverview {
        name: acct.name,
        display_name: acct.display_name,
        is_native: acct.is_native,
        config_dir: dir.to_string_lossy().to_string(),
        logged_in,
        email,
        project_count,
        plugin_count,
        skill_count,
    })
}

fn read_plugin_count(dir: &std::path::Path) -> u32 {
    // installed.json shape: { "plugins": [...] } per existing plugins module.
    let path = dir.join("plugins").join("installed.json");
    let Ok(bytes) = std::fs::read(&path) else { return 0; };
    let Ok(json): Result<serde_json::Value, _> = serde_json::from_slice(&bytes) else { return 0; };
    json.get("plugins")
        .and_then(|v| v.as_array())
        .map(|a| a.len() as u32)
        .unwrap_or(0)
}

fn read_oauth_status(dir: &std::path::Path) -> (bool, Option<String>) {
    // .claude.json sits in the account dir for non-default accounts;
    // for default, it's at ~/.claude.json (one level up from ~/.claude/).
    let claude_json = if dir.ends_with(".claude") {
        // default account: ~/.claude.json
        dir.parent().map(|p| p.join(".claude.json"))
    } else {
        Some(dir.join(".claude.json"))
    };
    let Some(path) = claude_json else { return (false, None); };
    let Ok(bytes) = std::fs::read(&path) else { return (false, None); };
    let Ok(json): Result<serde_json::Value, _> = serde_json::from_slice(&bytes) else { return (false, None); };
    let oauth = json.get("oauthAccount");
    let logged_in = oauth.is_some();
    let email = oauth.and_then(|o| o.get("emailAddress")).and_then(|v| v.as_str()).map(String::from);
    (logged_in, email)
}
```

- [ ] **Step 2: Register the IPC handler**

In `src-tauri/src/lib.rs` `tauri::generate_handler![...]`:

```rust
            commands::account_session::set_active_account,
            commands::account_session::account_overview,
```

- [ ] **Step 3: Build**

```bash
cargo build -p dot-claude-gui
```

Expected: clean.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/account_session.rs src-tauri/src/lib.rs
git commit -m "feat(account-session): account_overview (batched status + counts)"
```

---

### Task 6: Frontend types — `AccountOverview` + `GuiMode`

**Files:**
- Modify: `src/lib/api/types.ts`

Mode persistence (lastMode / lastAccount / lastProject) lives in **localStorage**, not in `AppConfig`. Rationale: round-tripping unknown fields through Rust's `AppConfig::write_config` would drop them (the Rust struct doesn't list them). localStorage avoids that schema coupling for purely client-side UI state. The mode store in Task 7 owns the persistence.

- [ ] **Step 1: Add new types**

Append to `src/lib/api/types.ts` (after the existing Accounts block, before AppConfig):

```ts
export interface AccountOverview {
  name: string;
  displayName: string;
  isNative: boolean;
  configDir: string;
  loggedIn: boolean;
  email?: string;
  projectCount: number;
  pluginCount: number;
  skillCount: number;
}

export type GuiMode = "account" | "project";
```

Do NOT modify `AppConfig` in this task.

- [ ] **Step 2: Type-check**

```bash
pnpm exec tsc --noEmit
```

Expected: clean (no new errors).

- [ ] **Step 3: Commit**

```bash
git add src/lib/api/types.ts
git commit -m "types: AccountOverview + GuiMode"
```

---

### Task 7: Mode store

**Files:**
- Create: `src/lib/stores/mode.svelte.ts`

- [ ] **Step 1: Write the store**

Create `src/lib/stores/mode.svelte.ts`:

```ts
import type { GuiMode } from "$lib/api/types";

const STORAGE_KEY = "dot-claude-gui-mode-v1";

interface PersistedMode {
  mode: GuiMode;
  selectedAccount: string | null;
  selectedProject: string | null;
}

function loadPersisted(): PersistedMode {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      return {
        mode: parsed.mode === "project" ? "project" : "account",
        selectedAccount: typeof parsed.selectedAccount === "string" ? parsed.selectedAccount : null,
        selectedProject: typeof parsed.selectedProject === "string" ? parsed.selectedProject : null,
      };
    }
  } catch {
    // fall through to defaults
  }
  return { mode: "account", selectedAccount: null, selectedProject: null };
}

class ModeStore {
  private _persisted = loadPersisted();
  mode = $state<GuiMode>(this._persisted.mode);
  selectedAccount = $state<string | null>(this._persisted.selectedAccount);
  selectedProject = $state<string | null>(this._persisted.selectedProject);

  setMode(m: GuiMode): void {
    this.mode = m;
    this.persist();
  }

  setSelectedAccount(name: string | null): void {
    this.selectedAccount = name;
    this.persist();
  }

  setSelectedProject(path: string | null): void {
    this.selectedProject = path;
    this.persist();
  }

  private persist(): void {
    try {
      const snapshot: PersistedMode = {
        mode: this.mode,
        selectedAccount: this.selectedAccount,
        selectedProject: this.selectedProject,
      };
      localStorage.setItem(STORAGE_KEY, JSON.stringify(snapshot));
    } catch {
      // localStorage unavailable — ignore
    }
  }
}

export const modeStore = new ModeStore();
```

- [ ] **Step 2: Type-check**

```bash
pnpm exec tsc --noEmit
```

Expected: clean.

- [ ] **Step 3: Commit**

```bash
git add src/lib/stores/mode.svelte.ts
git commit -m "stores(mode): persisted Account/Project mode + selection state"
```

---

### Task 8: IPC client methods for account session

**Files:**
- Modify: `src/lib/ipc/client.ts`

- [ ] **Step 1: Add methods**

In `src/lib/ipc/client.ts`, add `AccountOverview` to the imports at the top:

```ts
import type {
  // ...existing...
  AccountOverview,
} from "$lib/api/types.js";
```

After the existing `--- accounts (4) ---` block, add:

```ts
  // --- account session (2) ---

  async setActiveAccount(name: string): Promise<string> {
    return call("set_active_account", { name });
  }

  async accountOverview(name: string): Promise<AccountOverview> {
    return call("account_overview", { name });
  }
```

- [ ] **Step 2: Type-check**

```bash
pnpm exec tsc --noEmit
```

Expected: clean.

- [ ] **Step 3: Commit**

```bash
git add src/lib/ipc/client.ts
git commit -m "ipc(client): setActiveAccount + accountOverview"
```

---

### Task 9: TopBar component (mode tabs + gear)

**Files:**
- Create: `src/lib/components/shell/TopBar.svelte`

- [ ] **Step 1: Write the component**

```svelte
<script lang="ts">
  import { modeStore } from "$lib/stores/mode.svelte";
  import { t } from "$lib/i18n";

  let { onOpenSettings } = $props<{ onOpenSettings: () => void }>();
</script>

<header
  class="flex items-center justify-between px-4 py-2"
  style="background-color: var(--bg-secondary); border-bottom: 1px solid var(--border-color)"
>
  <nav class="flex items-center gap-1" aria-label={t("shell.modeNavLabel")}>
    <button
      class="px-3 py-1.5 text-sm rounded-md transition-colors"
      style="background-color: {modeStore.mode === 'account' ? 'var(--accent-bg)' : 'transparent'}; color: {modeStore.mode === 'account' ? 'var(--accent-text)' : 'var(--text-secondary)'}"
      onclick={() => modeStore.setMode("account")}
    >
      👤 {t("shell.accountsMode")}
    </button>
    <button
      class="px-3 py-1.5 text-sm rounded-md transition-colors"
      style="background-color: {modeStore.mode === 'project' ? 'var(--accent-bg)' : 'transparent'}; color: {modeStore.mode === 'project' ? 'var(--accent-text)' : 'var(--text-secondary)'}"
      onclick={() => modeStore.setMode("project")}
    >
      📂 {t("shell.projectsMode")}
    </button>
  </nav>

  <button
    class="p-2 rounded-md transition-colors hover:bg-[var(--bg-card-hover)]"
    style="color: var(--text-secondary)"
    onclick={onOpenSettings}
    title={t("shell.appSettings")}
    aria-label={t("shell.appSettings")}
  >
    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="1.5">
      <path stroke-linecap="round" stroke-linejoin="round" d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.325.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 0 1 1.37.49l1.296 2.247a1.125 1.125 0 0 1-.26 1.431l-1.003.827c-.293.241-.438.613-.43.992a7.723 7.723 0 0 1 0 .255c-.008.378.137.75.43.991l1.004.827c.424.35.534.955.26 1.43l-1.298 2.247a1.125 1.125 0 0 1-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.47 6.47 0 0 1-.22.128c-.331.183-.581.495-.644.869l-.213 1.281c-.09.543-.56.94-1.11.94h-2.594c-.55 0-1.019-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 0 1-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 0 1-1.369-.49l-1.297-2.247a1.125 1.125 0 0 1 .26-1.431l1.004-.827c.292-.24.437-.613.43-.991a6.932 6.932 0 0 1 0-.255c.007-.38-.138-.751-.43-.992l-1.004-.827a1.125 1.125 0 0 1-.26-1.43l1.297-2.247a1.125 1.125 0 0 1 1.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.086.22-.128.332-.183.582-.495.644-.869l.214-1.28Z M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z" />
    </svg>
  </button>
</header>
```

- [ ] **Step 2: i18n keys (placeholder — full pass in Task 22)**

The project uses JSON locale files at `src/lib/i18n/locales/locales/{zh-CN,en-US,ja-JP}.json`. Add to each:

```json
// src/lib/i18n/locales/locales/zh-CN.json — add these key/value pairs
"shell.modeNavLabel": "模式切换",
"shell.accountsMode": "账号",
"shell.projectsMode": "项目",
"shell.appSettings": "应用设置",
```

```json
// src/lib/i18n/locales/locales/en-US.json — add these key/value pairs
"shell.modeNavLabel": "Mode switcher",
"shell.accountsMode": "Accounts",
"shell.projectsMode": "Projects",
"shell.appSettings": "App settings",
```

```json
// src/lib/i18n/locales/locales/ja-JP.json — add these key/value pairs
"shell.modeNavLabel": "モード切替",
"shell.accountsMode": "アカウント",
"shell.projectsMode": "プロジェクト",
"shell.appSettings": "アプリ設定",
```

The `MessageKey` type in `src/lib/i18n.ts` is derived from `keyof typeof zhCN`, so adding a key to zh-CN.json widens the type; en-US.json and ja-JP.json must mirror it (the `Record<MessageKey, string>` constraint will compile-fail otherwise).

- [ ] **Step 3: Type-check**

```bash
pnpm exec tsc --noEmit
```

Expected: clean.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/shell/TopBar.svelte src/lib/i18n/locales/locales/
git commit -m "feat(shell): TopBar with mode tabs and settings gear"
```

---

### Task 10: AppSettings as modal

**Files:**
- Create: `src/lib/components/shell/AppSettingsModal.svelte`

- [ ] **Step 1: Write the modal**

```svelte
<script lang="ts">
  import AppSettingsView from "$lib/components/appsettings/AppSettingsView.svelte";
  import { t } from "$lib/i18n";

  let { open = false, onClose } = $props<{ open: boolean; onClose: () => void }>();

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) onClose();
  }

  function handleKey(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }
</script>

<svelte:window on:keydown={handleKey} />

{#if open}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center"
    style="background-color: rgba(0, 0, 0, 0.5)"
    onclick={handleBackdropClick}
    role="dialog"
    aria-modal="true"
    aria-label={t("shell.appSettings")}
  >
    <div
      class="w-[80vw] max-w-4xl h-[80vh] rounded-lg overflow-hidden flex flex-col"
      style="background-color: var(--bg-primary); border: 1px solid var(--border-color)"
    >
      <header class="flex items-center justify-between px-4 py-3" style="border-bottom: 1px solid var(--border-color)">
        <h2 class="text-sm font-semibold" style="color: var(--text-primary)">{t("shell.appSettings")}</h2>
        <button
          class="p-1 rounded transition-colors hover:bg-[var(--bg-card-hover)]"
          style="color: var(--text-secondary)"
          onclick={onClose}
          aria-label={t("shell.close")}
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
          </svg>
        </button>
      </header>
      <div class="flex-1 overflow-auto">
        <AppSettingsView />
      </div>
    </div>
  </div>
{/if}
```

- [ ] **Step 2: i18n**

Add `"shell.close"` key in both locales (`"关闭"` / `"Close"`).

- [ ] **Step 3: Type-check + commit**

```bash
pnpm exec tsc --noEmit
git add src/lib/components/shell/AppSettingsModal.svelte src/lib/i18n/locales/
git commit -m "feat(shell): AppSettingsModal wrapping existing AppSettingsView"
```

---

### Task 11: AccountSidebar

**Files:**
- Create: `src/lib/components/shell/AccountSidebar.svelte`

The existing `src/lib/components/accounts/AccountsView.svelte` contains the account list logic + status badges + `+ Add Account` flow. For Stage 2, extract or reuse the list portion. Since AccountsView mixes list and main-panel content, the cleanest approach is to **write a new lean list component for the sidebar** and keep AccountsView intact (Stage 4 cleanup will delete it).

- [ ] **Step 1: Read AccountsView for reference**

Read `src/lib/components/accounts/AccountsView.svelte` to understand:
- How accounts are fetched (`accountsStore.accounts`)
- How statuses are read (`accountsStore.statuses[name]`)
- The Add Account validation/UX flow

- [ ] **Step 2: Write `AccountSidebar`**

```svelte
<script lang="ts">
  import { accountsStore } from "$lib/stores/accounts.svelte";
  import { modeStore } from "$lib/stores/mode.svelte";
  import { ipcClient } from "$lib/ipc/client";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { t } from "$lib/i18n";

  let addingName = $state("");
  let creating = $state(false);

  async function selectAccount(name: string) {
    if (modeStore.selectedAccount === name) return;
    modeStore.setSelectedAccount(name);
    try {
      await ipcClient.setActiveAccount(name);
      // The AccountModeView is responsible for triggering store reloads
      // via $effect on modeStore.selectedAccount.
    } catch (e) {
      toastStore.error(t("shell.switchAccountFailed"));
      console.error("setActiveAccount failed", e);
    }
  }

  async function addAccount(e: SubmitEvent) {
    e.preventDefault();
    const name = addingName.trim();
    if (!name || creating) return;
    if (accountsStore.has(name)) {
      toastStore.error(t("shell.accountAlreadyExists"));
      return;
    }
    creating = true;
    try {
      await accountsStore.createAccount(name);
      addingName = "";
      await selectAccount(name);
    } catch (e) {
      toastStore.error(t("shell.createAccountFailed"));
      console.error("createAccount failed", e);
    } finally {
      creating = false;
    }
  }
</script>

<div class="flex flex-col h-full" style="background-color: var(--bg-secondary)">
  <div class="px-4 py-3" style="border-bottom: 1px solid var(--border-color)">
    <h2 class="text-xs font-semibold uppercase tracking-wider" style="color: var(--text-muted)">
      {t("shell.accountsList")}
    </h2>
  </div>

  <ul class="flex-1 overflow-y-auto py-2">
    {#each accountsStore.accounts as account (account.name)}
      {@const isActive = modeStore.selectedAccount === account.name}
      {@const status = accountsStore.statuses[account.name]}
      <li>
        <button
          class="w-full px-4 py-2 text-left text-sm flex items-center gap-2 transition-colors {isActive ? '' : 'hover:bg-[var(--bg-card-hover)]'}"
          style="background-color: {isActive ? 'var(--accent-bg)' : 'transparent'}; color: {isActive ? 'var(--accent-text)' : 'var(--text-primary)'}"
          onclick={() => selectAccount(account.name)}
        >
          <span class="flex-1 truncate">
            {account.displayName}
            {#if account.isNative}
              <span class="text-xs" style="color: var(--text-muted)">·  {t("shell.native")}</span>
            {/if}
          </span>
          {#if status?.loggedIn}
            <span class="w-2 h-2 rounded-full" style="background-color: var(--accent-text)" title={status.email ?? ""}></span>
          {/if}
        </button>
      </li>
    {/each}
  </ul>

  <form class="px-3 py-3 flex gap-2" style="border-top: 1px solid var(--border-color)" onsubmit={addAccount}>
    <input
      class="flex-1 px-2 py-1 text-sm rounded border"
      style="background-color: var(--bg-primary); border-color: var(--border-color); color: var(--text-primary)"
      bind:value={addingName}
      placeholder={t("shell.newAccountPlaceholder")}
      disabled={creating}
    />
    <button
      type="submit"
      class="px-3 py-1 text-sm rounded transition-colors"
      style="background-color: var(--accent-bg); color: var(--accent-text)"
      disabled={creating || !addingName.trim()}
    >
      {t("shell.addAccount")}
    </button>
  </form>
</div>
```

- [ ] **Step 3: i18n keys**

Add to both locales: `shell.accountsList`, `shell.native`, `shell.newAccountPlaceholder`, `shell.addAccount`, `shell.accountAlreadyExists`, `shell.createAccountFailed`, `shell.switchAccountFailed`.

- [ ] **Step 4: Type-check + commit**

```bash
pnpm exec tsc --noEmit
git add src/lib/components/shell/AccountSidebar.svelte src/lib/i18n/locales/
git commit -m "feat(shell): AccountSidebar with selection + add-account flow"
```

---

### Task 12: ProjectSidebar (Stage-3 placeholder list)

**Files:**
- Create: `src/lib/components/shell/ProjectSidebar.svelte`

- [ ] **Step 1: Write the sidebar**

```svelte
<script lang="ts">
  import { projectsStore } from "$lib/stores/projects.svelte";
  import { modeStore } from "$lib/stores/mode.svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { t } from "$lib/i18n";

  // Group projects by bound account name (or "Unbound").
  const groups = $derived.by(() => {
    const map = new Map<string, typeof projectsStore.entries>();
    for (const entry of projectsStore.entries) {
      const key = entry.account ?? "__unbound__";
      const list = map.get(key) ?? [];
      list.push(entry);
      map.set(key, list);
    }
    return Array.from(map.entries()).sort(([a], [b]) => {
      if (a === "__unbound__") return 1;
      if (b === "__unbound__") return -1;
      return a.localeCompare(b);
    });
  });

  function basename(path: string): string {
    const parts = path.split("/").filter(Boolean);
    return parts[parts.length - 1] ?? path;
  }

  async function addProject() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected !== "string") return;
    try {
      await projectsStore.add(selected);
      modeStore.setSelectedProject(selected);
    } catch (e) {
      toastStore.error(t("shell.addProjectFailed"));
      console.error("addProject failed", e);
    }
  }
</script>

<div class="flex flex-col h-full" style="background-color: var(--bg-secondary)">
  <div class="px-4 py-3" style="border-bottom: 1px solid var(--border-color)">
    <h2 class="text-xs font-semibold uppercase tracking-wider" style="color: var(--text-muted)">
      {t("shell.projectsList")}
    </h2>
  </div>

  <ul class="flex-1 overflow-y-auto py-2">
    {#each groups as [account, projects] (account)}
      <li class="mt-2 first:mt-0">
        <h3 class="px-4 py-1 text-xs uppercase tracking-wider" style="color: var(--text-muted)">
          {account === "__unbound__" ? t("shell.unbound") : "@" + account}
        </h3>
        <ul>
          {#each projects as project (project.path)}
            {@const isActive = modeStore.selectedProject === project.path}
            <li>
              <button
                class="w-full px-4 py-2 text-left text-sm flex items-center gap-2 transition-colors {isActive ? '' : 'hover:bg-[var(--bg-card-hover)]'}"
                style="background-color: {isActive ? 'var(--accent-bg)' : 'transparent'}; color: {isActive ? 'var(--accent-text)' : project.stale ? 'var(--text-muted)' : 'var(--text-primary)'}"
                onclick={() => modeStore.setSelectedProject(project.path)}
                title={project.path}
              >
                <span class="flex-1 truncate">{basename(project.path)}</span>
                {#if project.stale}
                  <span class="text-xs" style="color: var(--text-muted)">·  {t("shell.stale")}</span>
                {/if}
              </button>
            </li>
          {/each}
        </ul>
      </li>
    {/each}
  </ul>

  <button
    class="mx-3 mb-3 px-3 py-1.5 text-sm rounded transition-colors"
    style="background-color: var(--accent-bg); color: var(--accent-text); margin-top: 0.75rem; border-top: 1px solid var(--border-color)"
    onclick={addProject}
  >
    + {t("shell.addProject")}
  </button>
</div>
```

- [ ] **Step 2: i18n keys**

`shell.projectsList`, `shell.unbound`, `shell.stale`, `shell.addProject`, `shell.addProjectFailed`.

- [ ] **Step 3: Type-check + commit**

```bash
pnpm exec tsc --noEmit
git add src/lib/components/shell/ProjectSidebar.svelte src/lib/i18n/locales/
git commit -m "feat(shell): ProjectSidebar with bound/unbound grouping"
```

---

### Task 13: ProjectModePlaceholder (main panel stub)

**Files:**
- Create: `src/lib/components/project-mode/ProjectModePlaceholder.svelte`

- [ ] **Step 1: Write the stub**

```svelte
<script lang="ts">
  import { modeStore } from "$lib/stores/mode.svelte";
  import { projectsStore } from "$lib/stores/projects.svelte";
  import { t } from "$lib/i18n";

  const selected = $derived(
    projectsStore.entries.find((p) => p.path === modeStore.selectedProject) ?? null
  );
</script>

<div class="flex-1 flex items-center justify-center p-6">
  <div class="text-center">
    <p class="text-sm" style="color: var(--text-muted)">
      {#if selected}
        {t("shell.projectModeComing", { path: selected.path })}
      {:else}
        {t("shell.projectModeSelectHint")}
      {/if}
    </p>
  </div>
</div>
```

- [ ] **Step 2: i18n**

`shell.projectModeComing` (with `{path}` interpolation), `shell.projectModeSelectHint`.

- [ ] **Step 3: Commit**

```bash
pnpm exec tsc --noEmit
git add src/lib/components/project-mode/ProjectModePlaceholder.svelte src/lib/i18n/locales/
git commit -m "feat(project-mode): Stage-3 placeholder main panel"
```

---

### Task 14: AccountModeView shell (facet tab strip + account-switch effect)

**Files:**
- Create: `src/lib/components/account-mode/AccountModeView.svelte`

The view owns:
- The 7 facet tab strip
- The active facet's render slot
- The `$effect` that, when `modeStore.selectedAccount` changes, calls `setActiveAccount` and reloads all relevant stores
- Default-selection logic (if no account is selected, pick first)

- [ ] **Step 1: Write the view**

```svelte
<script lang="ts">
  import { modeStore } from "$lib/stores/mode.svelte";
  import { accountsStore } from "$lib/stores/accounts.svelte";
  import { configStore } from "$lib/stores/config.svelte";
  import { pluginsStore } from "$lib/stores/plugins.svelte";
  import { skillsStore } from "$lib/stores/skills.svelte";
  import { memoryStore } from "$lib/stores/memory.svelte";
  import { mcpStore } from "$lib/stores/mcp.svelte";
  import { claudeMdStore } from "$lib/stores/claudemd.svelte";
  import { ipcClient } from "$lib/ipc/client";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { t, type MessageKey } from "$lib/i18n";

  import Overview from "./Overview.svelte";
  import SettingsFacet from "./SettingsFacet.svelte";
  import PluginsFacet from "./PluginsFacet.svelte";
  import SkillsFacet from "./SkillsFacet.svelte";
  import ClaudeMdFacet from "./ClaudeMdFacet.svelte";
  import MemoryFacet from "./MemoryFacet.svelte";
  import McpFacet from "./McpFacet.svelte";

  type Facet = "overview" | "settings" | "plugins" | "skills" | "claudemd" | "memory" | "mcp";

  const facets = [
    { id: "overview", labelKey: "accountMode.overview" },
    { id: "settings", labelKey: "accountMode.settings" },
    { id: "plugins", labelKey: "accountMode.plugins" },
    { id: "skills", labelKey: "accountMode.skills" },
    { id: "claudemd", labelKey: "accountMode.claudemd" },
    { id: "memory", labelKey: "accountMode.memory" },
    { id: "mcp", labelKey: "accountMode.mcp" },
  ] satisfies { id: Facet; labelKey: MessageKey }[];

  let activeFacet = $state<Facet>("overview");

  // Default-selection: if no account is selected and we have accounts, pick the first.
  $effect(() => {
    if (modeStore.selectedAccount === null && accountsStore.accounts.length > 0) {
      modeStore.setSelectedAccount(accountsStore.accounts[0].name);
    }
  });

  // When selectedAccount changes, switch the active account on the backend and
  // reload all caches that depend on the user-layer dir.
  $effect(() => {
    const name = modeStore.selectedAccount;
    if (!name) return;
    void (async () => {
      try {
        await ipcClient.setActiveAccount(name);
        await Promise.all([
          configStore.loadUserConfig(),
          pluginsStore.loadPlugins(),
          skillsStore.loadSkills(),
          memoryStore.loadProjects(),
          mcpStore.loadServers(),
          claudeMdStore.loadFiles(),
        ]);
      } catch (e) {
        toastStore.error(t("shell.switchAccountFailed"));
        console.error("account switch reload failed", e);
      }
    })();
  });
</script>

{#if modeStore.selectedAccount === null}
  <div class="flex-1 flex items-center justify-center p-6">
    <p class="text-sm" style="color: var(--text-muted)">{t("accountMode.selectAccountHint")}</p>
  </div>
{:else}
  <div class="flex flex-col flex-1 overflow-hidden">
    <!-- Facet tab strip -->
    <div
      class="flex items-center gap-0.5 px-2 pt-2"
      style="background-color: var(--bg-secondary); border-bottom: 1px solid var(--border-color)"
    >
      {#each facets as f (f.id)}
        <button
          class="px-3 py-2 text-sm rounded-t-md transition-colors {activeFacet === f.id ? '' : 'hover:bg-[var(--bg-card-hover)]'}"
          style="background-color: {activeFacet === f.id ? 'var(--bg-primary)' : 'transparent'}; color: {activeFacet === f.id ? 'var(--text-primary)' : 'var(--text-secondary)'}; border: 1px solid {activeFacet === f.id ? 'var(--border-color)' : 'transparent'}; border-bottom: none"
          onclick={() => { activeFacet = f.id; }}
        >
          {t(f.labelKey)}
        </button>
      {/each}
    </div>

    <!-- Facet body -->
    <div class="flex-1 overflow-hidden flex flex-col">
      {#if activeFacet === "overview"}
        <Overview accountName={modeStore.selectedAccount} />
      {:else if activeFacet === "settings"}
        <SettingsFacet />
      {:else if activeFacet === "plugins"}
        <PluginsFacet />
      {:else if activeFacet === "skills"}
        <SkillsFacet />
      {:else if activeFacet === "claudemd"}
        <ClaudeMdFacet />
      {:else if activeFacet === "memory"}
        <MemoryFacet />
      {:else if activeFacet === "mcp"}
        <McpFacet />
      {/if}
    </div>
  </div>
{/if}
```

- [ ] **Step 2: i18n keys (placeholder; full pass in Task 22)**

`accountMode.overview`, `accountMode.settings`, `accountMode.plugins`, `accountMode.skills`, `accountMode.claudemd`, `accountMode.memory`, `accountMode.mcp`, `accountMode.selectAccountHint`.

- [ ] **Step 3: Commit**

Defer the build/commit to the end of the facet group (Task 21) so the imports of yet-to-exist facet files don't fail tsc.

Actually — write the import stubs (each facet a one-line `<div></div>` placeholder) inline first, OR comment out the imports until each facet exists. Recommend: keep imports as-is and create each facet file in Tasks 15-20 as bare placeholders, then flesh them out per-task. After Task 14:

Create the 7 facet placeholder files (5 lines each):

```svelte
<!-- src/lib/components/account-mode/Overview.svelte -->
<script lang="ts">
  let { accountName } = $props<{ accountName: string }>();
</script>
<div class="p-4 text-sm" style="color: var(--text-muted)">Overview placeholder · {accountName}</div>
```

(Same shape for SettingsFacet, PluginsFacet, SkillsFacet, ClaudeMdFacet, MemoryFacet, McpFacet — without `accountName` prop where not needed.)

This lets Task 14 pass `tsc --noEmit` and commit.

```bash
pnpm exec tsc --noEmit
git add src/lib/components/account-mode/ src/lib/i18n/locales/
git commit -m "feat(account-mode): AccountModeView shell + facet placeholders"
```

---

### Task 15: App.svelte rewrite (2-panel shell)

**Files:**
- Modify: `src/App.svelte`

This is the biggest single edit. The existing 489-line App.svelte does 3-panel routing for 9 modules. After this task it's a thin shell.

- [ ] **Step 1: Write the new App.svelte**

Replace the entire file:

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import { configStore } from "$lib/stores/config.svelte";
  import { projectsStore } from "$lib/stores/projects.svelte";
  import { pluginsStore } from "$lib/stores/plugins.svelte";
  import { skillsStore } from "$lib/stores/skills.svelte";
  import { memoryStore } from "$lib/stores/memory.svelte";
  import { accountsStore } from "$lib/stores/accounts.svelte";
  import { mcpStore } from "$lib/stores/mcp.svelte";
  import { claudeMdStore } from "$lib/stores/claudemd.svelte";
  import { appSettingsStore } from "$lib/stores/appsettings.svelte";
  import { modeStore } from "$lib/stores/mode.svelte";
  import { onConfigChanged } from "$lib/ipc/events.js";

  import TopBar from "$lib/components/shell/TopBar.svelte";
  import AppSettingsModal from "$lib/components/shell/AppSettingsModal.svelte";
  import AccountSidebar from "$lib/components/shell/AccountSidebar.svelte";
  import ProjectSidebar from "$lib/components/shell/ProjectSidebar.svelte";
  import AccountModeView from "$lib/components/account-mode/AccountModeView.svelte";
  import ProjectModePlaceholder from "$lib/components/project-mode/ProjectModePlaceholder.svelte";
  import ResizeHandle from "$lib/components/shared/ResizeHandle.svelte";
  import Toast from "$lib/components/shared/Toast.svelte";

  // ── Theme / font / lang effects (unchanged from old App.svelte) ─────
  $effect(() => {
    const theme = appSettingsStore.preferences.theme;
    if (theme === "dark") {
      document.documentElement.classList.add("dark");
    } else if (theme === "light") {
      document.documentElement.classList.remove("dark");
    } else if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }
  });

  $effect(() => {
    const lang = appSettingsStore.preferences.language;
    if (lang) document.documentElement.lang = lang;
  });

  $effect(() => {
    document.documentElement.style.setProperty(
      "--app-font-size",
      appSettingsStore.preferences.fontSize + "px"
    );
  });

  $effect(() => {
    document.documentElement.style.setProperty(
      "--sidebar-width",
      appSettingsStore.preferences.sidebarWidth + "px"
    );
  });

  let unlistenConfigChanged: (() => void) | undefined;
  let settingsModalOpen = $state(false);

  onMount(() => {
    void (async () => {
      await appSettingsStore.load();
      await Promise.all([
        configStore.loadUserConfig(),
        projectsStore.loadProjects(),
        pluginsStore.loadPlugins(),
        skillsStore.loadSkills(),
        memoryStore.loadProjects(),
        mcpStore.loadServers(),
        claudeMdStore.loadFiles(),
        accountsStore.loadAccounts(),
      ]);
      unlistenConfigChanged = await onConfigChanged((payload) => {
        configStore.setUserConfig(payload.settings);
      });
    })();

    return () => {
      unlistenConfigChanged?.();
    };
  });
</script>

{#if appSettingsStore.loaded}
  <div
    class="flex h-screen w-screen flex-col overflow-hidden"
    style="background-color: var(--bg-primary); color: var(--text-primary)"
  >
    <TopBar onOpenSettings={() => { settingsModalOpen = true; }} />

    <div class="flex flex-1 overflow-hidden">
      <!-- Sidebar (mode-aware) -->
      <aside
        class="flex-shrink-0 overflow-hidden"
        style="width: var(--sidebar-width); background-color: var(--bg-secondary); border-right: 1px solid var(--border-color); min-width: 200px"
      >
        {#if modeStore.mode === "account"}
          <AccountSidebar />
        {:else}
          <ProjectSidebar />
        {/if}
      </aside>

      <ResizeHandle
        min={200}
        max={400}
        onResize={(w) => appSettingsStore.update({ sidebarWidth: w })}
      />

      <!-- Main -->
      <main class="flex-1 flex flex-col overflow-hidden">
        {#if modeStore.mode === "account"}
          <AccountModeView />
        {:else}
          <ProjectModePlaceholder />
        {/if}
      </main>
    </div>

    <AppSettingsModal open={settingsModalOpen} onClose={() => { settingsModalOpen = false; }} />
    <Toast />
  </div>
{:else}
  <div class="flex h-screen w-screen items-center justify-center" style="background-color: var(--bg-primary)"></div>
{/if}
```

- [ ] **Step 2: Adjust sidebar default width**

In `src/lib/stores/appsettings.svelte.ts`, change the default `sidebarWidth` from 140 to 240 (matches the spec's "fixed width ~240px, resizable"):

```ts
sidebarWidth: 240,
```

Note: existing users will retain their persisted width (140 from Stage 1). That's OK — they can resize.

The `<aside>` style sets `min-width: 200px` which means the sidebar won't collapse below that even with the existing 140 value. Reasonable enough; if the persisted 140 is jarring, the user can drag wider.

- [ ] **Step 3: Run tsc + Rust build**

```bash
pnpm exec tsc --noEmit
cargo build -p dot-claude-gui
```

Expected: tsc clean. cargo build clean.

Note: the old `nav.*` i18n keys (`nav.settings`, `nav.plugins`, etc.) are now orphans — many components no longer reference them. Don't delete yet; Task 22 i18n pass cleans them up.

- [ ] **Step 4: Commit**

```bash
git add src/App.svelte src/lib/stores/appsettings.svelte.ts
git commit -m "shell: replace 3-panel layout with mode-based 2-panel shell"
```

---

### Task 16: Overview facet (real implementation)

**Files:**
- Modify: `src/lib/components/account-mode/Overview.svelte`

- [ ] **Step 1: Implement**

```svelte
<script lang="ts">
  import { ipcClient } from "$lib/ipc/client";
  import { accountsStore } from "$lib/stores/accounts.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { t } from "$lib/i18n";
  import type { AccountOverview } from "$lib/api/types";

  let { accountName } = $props<{ accountName: string }>();

  let overview = $state<AccountOverview | null>(null);
  let loading = $state(false);
  let err = $state<string | null>(null);

  async function load() {
    loading = true;
    err = null;
    try {
      overview = await ipcClient.accountOverview(accountName);
    } catch (e) {
      err = String(e);
      overview = null;
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    // Re-fetch whenever accountName changes.
    void accountName;
    void load();
  });

  async function relogin() {
    // For Stage 2, surface this as a toast hint — relaunching the OAuth flow
    // requires the existing `launch_claude` IPC with the auth subcommand,
    // which is wired in AccountsView. Stage 3 Project > Launch absorbs this.
    toastStore.info(t("accountMode.reloginHint"));
  }

  async function openDir() {
    if (!overview) return;
    try {
      const { revealItemInDir } = await import("@tauri-apps/plugin-opener");
      await revealItemInDir(overview.configDir);
    } catch (e) {
      toastStore.error(t("accountMode.openDirFailed"));
      console.error("openDir failed", e);
    }
  }

  async function deleteAcct() {
    if (!overview || overview.isNative) return;
    if (!confirm(t("accountMode.deleteConfirm", { name: overview.name }))) return;
    try {
      await accountsStore.deleteAccount(overview.name);
      toastStore.info(t("accountMode.deleteSuccess"));
    } catch (e) {
      toastStore.error(t("accountMode.deleteFailed"));
      console.error("deleteAccount failed", e);
    }
  }
</script>

<div class="flex-1 overflow-auto p-6">
  {#if loading}
    <p class="text-sm" style="color: var(--text-muted)">{t("accountMode.loading")}</p>
  {:else if err}
    <p class="text-sm" style="color: var(--text-error)">{err}</p>
  {:else if overview}
    <div class="rounded-lg p-4 mb-4" style="background-color: var(--bg-card); border: 1px solid var(--border-color)">
      <h2 class="text-lg font-semibold mb-3" style="color: var(--text-primary)">
        {overview.displayName}
        {#if overview.isNative}
          <span class="text-xs ml-2" style="color: var(--text-muted)">{t("shell.native")}</span>
        {/if}
      </h2>
      <dl class="grid grid-cols-2 gap-y-2 text-sm">
        <dt style="color: var(--text-muted)">{t("accountMode.configDir")}</dt>
        <dd style="color: var(--text-primary)" class="font-mono text-xs break-all">{overview.configDir}</dd>
        <dt style="color: var(--text-muted)">{t("accountMode.status")}</dt>
        <dd style="color: var(--text-primary)">
          {#if overview.loggedIn}
            ✓ {overview.email ?? t("accountMode.loggedIn")}
          {:else}
            {t("accountMode.notLoggedIn")}
          {/if}
        </dd>
        <dt style="color: var(--text-muted)">{t("accountMode.projectCount")}</dt>
        <dd style="color: var(--text-primary)">{overview.projectCount}</dd>
        <dt style="color: var(--text-muted)">{t("accountMode.pluginCount")}</dt>
        <dd style="color: var(--text-primary)">{overview.pluginCount}</dd>
        <dt style="color: var(--text-muted)">{t("accountMode.skillCount")}</dt>
        <dd style="color: var(--text-primary)">{overview.skillCount}</dd>
      </dl>
    </div>

    <div class="flex gap-2">
      <button
        class="px-3 py-1.5 text-sm rounded transition-colors hover:bg-[var(--bg-card-hover)]"
        style="background-color: var(--bg-card); border: 1px solid var(--border-color); color: var(--text-primary)"
        onclick={relogin}
      >
        {t("accountMode.relogin")}
      </button>
      <button
        class="px-3 py-1.5 text-sm rounded transition-colors hover:bg-[var(--bg-card-hover)]"
        style="background-color: var(--bg-card); border: 1px solid var(--border-color); color: var(--text-primary)"
        onclick={openDir}
      >
        {t("accountMode.openDir")}
      </button>
      {#if !overview.isNative}
        <button
          class="px-3 py-1.5 text-sm rounded transition-colors"
          style="background-color: var(--bg-card); border: 1px solid var(--border-color); color: var(--text-error)"
          onclick={deleteAcct}
        >
          {t("accountMode.delete")}
        </button>
      {/if}
    </div>
  {/if}
</div>
```

- [ ] **Step 2: Add @tauri-apps/plugin-opener if not present**

```bash
grep "plugin-opener" package.json
```

If absent, add it:

```bash
pnpm add @tauri-apps/plugin-opener
```

And register the plugin in `src-tauri/Cargo.toml` + `src-tauri/src/lib.rs`. Check existing patterns (the `plugin-dialog` is already wired). For Stage 2, if plugin-opener is awkward to add, skip the `openDir` action — disable the button with a "coming soon" tooltip. The action isn't load-bearing for Stage 2 acceptance.

Simpler path: skip plugin-opener; have the Open Dir button copy the path to clipboard via `navigator.clipboard.writeText` and toast "Path copied".

- [ ] **Step 3: i18n keys**

`accountMode.loading`, `accountMode.configDir`, `accountMode.status`, `accountMode.loggedIn`, `accountMode.notLoggedIn`, `accountMode.projectCount`, `accountMode.pluginCount`, `accountMode.skillCount`, `accountMode.relogin`, `accountMode.reloginHint`, `accountMode.openDir`, `accountMode.openDirFailed`, `accountMode.delete`, `accountMode.deleteConfirm` (with `{name}`), `accountMode.deleteSuccess`, `accountMode.deleteFailed`.

- [ ] **Step 4: Type-check + commit**

```bash
pnpm exec tsc --noEmit
git add src/lib/components/account-mode/Overview.svelte src/lib/i18n/locales/
git commit -m "feat(account-mode): Overview facet (status + counts + actions)"
```

---

### Task 17: Settings facet (inline sub-nav)

**Files:**
- Modify: `src/lib/components/account-mode/SettingsFacet.svelte`

The existing `SettingsEditor` takes an `activeSection` prop. The 10 sub-sections used to live in App.svelte's sub-panel; here we render them as a horizontal strip above the editor.

- [ ] **Step 1: Implement**

```svelte
<script lang="ts">
  import SettingsEditor from "$lib/components/settings/SettingsEditor.svelte";
  import { configStore } from "$lib/stores/config.svelte";
  import { t, type MessageKey } from "$lib/i18n";

  const sections = [
    { id: "general", labelKey: "settings.general" },
    { id: "permissions", labelKey: "settings.permissions" },
    { id: "hooks", labelKey: "settings.hooks" },
    { id: "sandbox", labelKey: "settings.sandbox" },
    { id: "environment", labelKey: "settings.environment" },
    { id: "statusline", labelKey: "settings.statusLine" },
    { id: "runtime", labelKey: "settings.runtime" },
    { id: "mcpPolicy", labelKey: "settings.mcpPolicy" },
    { id: "pluginsMarketplace", labelKey: "settings.pluginsMarketplace" },
    { id: "advanced", labelKey: "settings.advanced" },
  ] satisfies { id: string; labelKey: MessageKey }[];

  let active = $state("general");
</script>

<div class="flex flex-col flex-1 overflow-hidden">
  <nav
    class="flex items-center gap-1 px-2 py-1 overflow-x-auto"
    style="background-color: var(--bg-secondary); border-bottom: 1px solid var(--border-color)"
  >
    {#each sections as section (section.id)}
      <button
        class="px-2.5 py-1 text-xs rounded transition-colors whitespace-nowrap {active === section.id ? '' : 'hover:bg-[var(--bg-card-hover)]'}"
        style="background-color: {active === section.id ? 'var(--accent-bg)' : 'transparent'}; color: {active === section.id ? 'var(--accent-text)' : 'var(--text-secondary)'}"
        onclick={() => { active = section.id; }}
      >
        {t(section.labelKey)}
      </button>
    {/each}
  </nav>

  <div class="flex-1 overflow-hidden">
    {#if configStore.loading}
      <div class="p-6">
        <p class="text-sm" style="color: var(--text-muted)">{t("nav.loadingConfig")}</p>
      </div>
    {:else}
      <SettingsEditor activeSection={active} />
    {/if}
  </div>
</div>
```

- [ ] **Step 2: tsc + commit**

```bash
pnpm exec tsc --noEmit
git add src/lib/components/account-mode/SettingsFacet.svelte
git commit -m "feat(account-mode): SettingsFacet wraps existing editor with inline sub-nav"
```

---

### Task 18: Plugins facet

**Files:**
- Modify: `src/lib/components/account-mode/PluginsFacet.svelte`

- [ ] **Step 1: Implement**

```svelte
<script lang="ts">
  import PluginsModule from "$lib/components/plugins/PluginsModule.svelte";
  import { t, type MessageKey } from "$lib/i18n";

  const sections = [
    { id: "installed", labelKey: "plugins.installed" },
    { id: "marketplace", labelKey: "plugins.marketplace" },
    { id: "manage-marketplaces", labelKey: "plugins.manageMarketplaces" },
    { id: "per-project", labelKey: "plugins.perProject" },
  ] satisfies { id: string; labelKey: MessageKey }[];

  let active = $state("installed");
</script>

<div class="flex flex-col flex-1 overflow-hidden">
  <nav
    class="flex items-center gap-1 px-2 py-1 overflow-x-auto"
    style="background-color: var(--bg-secondary); border-bottom: 1px solid var(--border-color)"
  >
    {#each sections as section (section.id)}
      <button
        class="px-2.5 py-1 text-xs rounded transition-colors whitespace-nowrap {active === section.id ? '' : 'hover:bg-[var(--bg-card-hover)]'}"
        style="background-color: {active === section.id ? 'var(--accent-bg)' : 'transparent'}; color: {active === section.id ? 'var(--accent-text)' : 'var(--text-secondary)'}"
        onclick={() => { active = section.id; }}
      >
        {t(section.labelKey)}
      </button>
    {/each}
  </nav>

  <div class="flex-1 overflow-hidden">
    <PluginsModule activeSection={active} />
  </div>
</div>
```

- [ ] **Step 2: tsc + commit**

```bash
pnpm exec tsc --noEmit
git add src/lib/components/account-mode/PluginsFacet.svelte
git commit -m "feat(account-mode): PluginsFacet wraps existing module with inline sub-nav"
```

---

### Task 19: Skills facet

**Files:**
- Modify: `src/lib/components/account-mode/SkillsFacet.svelte`

The existing `SkillsModule` does its own list + preview internally. In the old App.svelte, the SkillList was in the sub-panel (acting as the list) and SkillsModule was the detail. Together they made the Skills experience. For the facet wrapper, render BOTH side-by-side in a flex layout (since we don't have a sub-panel anymore).

Looking at SkillsModule.svelte (read it first to confirm):

- [ ] **Step 1: Read SkillsModule + SkillList**

```bash
grep -l "SkillsModule\|SkillList\|SkillPreview" src/lib/components/skills/*.svelte
cat src/lib/components/skills/SkillsModule.svelte
cat src/lib/components/skills/SkillList.svelte
```

Decide how to compose. If SkillsModule already orchestrates the list + preview internally, the facet is one line: `<SkillsModule />`. If SkillList is a separate component the old App used to put in the sub-panel, the facet needs to compose them side-by-side OR add an inline list above the preview.

- [ ] **Step 2: Implement**

If `SkillsModule` is self-contained:

```svelte
<script lang="ts">
  import SkillsModule from "$lib/components/skills/SkillsModule.svelte";
</script>
<SkillsModule />
```

If SkillList is separate:

```svelte
<script lang="ts">
  import SkillList from "$lib/components/skills/SkillList.svelte";
  import SkillsModule from "$lib/components/skills/SkillsModule.svelte";
</script>

<div class="flex flex-1 overflow-hidden">
  <aside class="w-64 flex-shrink-0 overflow-hidden" style="background-color: var(--bg-secondary); border-right: 1px solid var(--border-color)">
    <SkillList />
  </aside>
  <div class="flex-1 overflow-hidden">
    <SkillsModule />
  </div>
</div>
```

Pick the right one based on Step 1's reading.

- [ ] **Step 3: tsc + commit**

```bash
pnpm exec tsc --noEmit
git add src/lib/components/account-mode/SkillsFacet.svelte
git commit -m "feat(account-mode): SkillsFacet"
```

---

### Task 20: CLAUDE.md + Memory facets

**Files:**
- Modify: `src/lib/components/account-mode/ClaudeMdFacet.svelte`
- Modify: `src/lib/components/account-mode/MemoryFacet.svelte`

Same shape as SkillsFacet — read each module + list pair, decide composition. Both ClaudeMd and Memory had list components in the old sub-panel.

- [ ] **Step 1: Implement ClaudeMdFacet**

```svelte
<script lang="ts">
  import ClaudeMdList from "$lib/components/claudemd/ClaudeMdList.svelte";
  import ClaudeMdModule from "$lib/components/claudemd/ClaudeMdModule.svelte";
</script>

<div class="flex flex-1 overflow-hidden">
  <aside class="w-64 flex-shrink-0 overflow-hidden" style="background-color: var(--bg-secondary); border-right: 1px solid var(--border-color)">
    <ClaudeMdList />
  </aside>
  <div class="flex-1 overflow-hidden">
    <ClaudeMdModule />
  </div>
</div>
```

- [ ] **Step 2: Implement MemoryFacet** (same shape):

```svelte
<script lang="ts">
  import MemoryList from "$lib/components/memory/MemoryList.svelte";
  import MemoryModule from "$lib/components/memory/MemoryModule.svelte";
</script>

<div class="flex flex-1 overflow-hidden">
  <aside class="w-64 flex-shrink-0 overflow-hidden" style="background-color: var(--bg-secondary); border-right: 1px solid var(--border-color)">
    <MemoryList />
  </aside>
  <div class="flex-1 overflow-hidden">
    <MemoryModule />
  </div>
</div>
```

Note: `ClaudeMdList`'s current implementation reads `projectsStore.activeProjectId` (Stage 1 left the deprecated alias). That keeps working in Stage 2 since the alias is still in place. Stage 3 will rewrite ClaudeMd for project scope.

- [ ] **Step 3: tsc + commit**

```bash
pnpm exec tsc --noEmit
git add src/lib/components/account-mode/ClaudeMdFacet.svelte src/lib/components/account-mode/MemoryFacet.svelte
git commit -m "feat(account-mode): ClaudeMdFacet + MemoryFacet (compose list + module)"
```

---

### Task 21: MCP facet

**Files:**
- Modify: `src/lib/components/account-mode/McpFacet.svelte`

- [ ] **Step 1: Implement**

```svelte
<script lang="ts">
  import McpModule from "$lib/components/mcp/McpModule.svelte";
  import { t, type MessageKey } from "$lib/i18n";

  const sections = [
    { id: "servers", labelKey: "mcp.servers" },
    { id: "add", labelKey: "mcp.addServer" },
  ] satisfies { id: string; labelKey: MessageKey }[];

  let active = $state("servers");
</script>

<div class="flex flex-col flex-1 overflow-hidden">
  <nav
    class="flex items-center gap-1 px-2 py-1"
    style="background-color: var(--bg-secondary); border-bottom: 1px solid var(--border-color)"
  >
    {#each sections as section (section.id)}
      <button
        class="px-2.5 py-1 text-xs rounded transition-colors {active === section.id ? '' : 'hover:bg-[var(--bg-card-hover)]'}"
        style="background-color: {active === section.id ? 'var(--accent-bg)' : 'transparent'}; color: {active === section.id ? 'var(--accent-text)' : 'var(--text-secondary)'}"
        onclick={() => { active = section.id; }}
      >
        {t(section.labelKey)}
      </button>
    {/each}
  </nav>

  <div class="flex-1 overflow-hidden">
    <McpModule activeSection={active} />
  </div>
</div>
```

- [ ] **Step 2: tsc + commit**

```bash
pnpm exec tsc --noEmit
git add src/lib/components/account-mode/McpFacet.svelte
git commit -m "feat(account-mode): McpFacet with inline sub-nav"
```

---

### Task 22: i18n key audit

**Files:**
- Modify: `src/lib/i18n/locales/zh-CN.json`, `src/lib/i18n/locales/en-US.json`, `src/lib/i18n/locales/ja-JP.json`

After Tasks 9-21, several new keys were added piecemeal. This task is a coherence pass.

- [ ] **Step 1: Inventory new keys**

```bash
grep -rEo 't\(\s*"[^"]+"' src/lib/components/shell/ src/lib/components/account-mode/ src/lib/components/project-mode/ | grep -oE '"[^"]+"' | sort -u
```

Compare against the locale files. Ensure every key has an entry in both zh-CN AND en-US.

- [ ] **Step 2: Fill any gaps**

For each missing key, add a sensible translation in both locales. Names of new keys (full list expected from Tasks 9-21):

```
shell.modeNavLabel, shell.accountsMode, shell.projectsMode, shell.appSettings,
shell.close, shell.accountsList, shell.native, shell.newAccountPlaceholder,
shell.addAccount, shell.accountAlreadyExists, shell.createAccountFailed,
shell.switchAccountFailed, shell.projectsList, shell.unbound, shell.stale,
shell.addProject, shell.addProjectFailed, shell.projectModeComing,
shell.projectModeSelectHint,

accountMode.overview, accountMode.settings, accountMode.plugins,
accountMode.skills, accountMode.claudemd, accountMode.memory, accountMode.mcp,
accountMode.selectAccountHint, accountMode.loading, accountMode.configDir,
accountMode.status, accountMode.loggedIn, accountMode.notLoggedIn,
accountMode.projectCount, accountMode.pluginCount, accountMode.skillCount,
accountMode.relogin, accountMode.reloginHint, accountMode.openDir,
accountMode.openDirFailed, accountMode.delete, accountMode.deleteConfirm,
accountMode.deleteSuccess, accountMode.deleteFailed,
```

- [ ] **Step 3: Don't delete old keys yet**

The old `nav.settings`, `nav.plugins` etc. keys are still referenced indirectly via the old AccountsView, LauncherView stub, etc. Stage 4 cleans them up. Leave alone for now.

- [ ] **Step 4: Type-check + commit**

```bash
pnpm exec tsc --noEmit
git add src/lib/i18n*
git commit -m "i18n: fill in zh-CN/en-US keys for Stage 2 shell"
```

---

### Task 23: Manual smoke test

**Files:** (manual; no commit unless fixes needed)

- [ ] **Step 1: Snapshot the live config**

```bash
cp ~/.dot-claude-gui/config.json ~/.dot-claude-gui/config.json.preflight.$(date +%s)
```

- [ ] **Step 2: Build and run**

```bash
pnpm tauri dev
```

- [ ] **Step 3: Verify**

- The header shows mode tabs (Accounts / Projects) and a gear icon on the right.
- The sidebar shows account list. Selecting `myself` (or any non-native account) updates the right panel.
- Each facet tab (Overview / Settings / Plugins / Skills / CLAUDE.md / Memory / MCP) renders without console errors.
- Overview shows `~/.dot-claude-gui/accounts/myself/` as configDir and accurate counts (probably zero plugins/skills since the account dir likely hasn't been populated).
- Selecting `default` switches Overview to show `~/.claude/` as configDir and the user's real plugin/skill counts.
- Settings facet's editor reflects the active account's settings.json content (compare with `cat ~/.dot-claude-gui/accounts/myself/settings.json` if exists).
- Switching to Project mode: sidebar shows project list grouped (likely just `dot-claude-gui` under `@myself`); main shows placeholder.
- Clicking the gear opens AppSettingsModal; pressing Esc closes it.
- Tauri DevTools console (Cmd+Option+I) is clean of errors.

- [ ] **Step 4: Spot-check Rust tests**

```bash
cargo test -p dot-claude-gui
```

Expected: all green (Stage 2 should not have reduced the 98 baseline).

- [ ] **Step 5: Commit any patches**

If you needed to fix things during smoke test, commit them:

```bash
git add -p
git commit -m "fix(stage2): <describe>"
```

---

## Acceptance criteria (per spec Stage 2)

- [x] All account facets render real data (Tasks 14-21 wire each facet; account-switch effect in AccountModeView reloads stores)
- [x] Toggle widgets persist — the underlying modules are unchanged and were already wired (PluginsModule, McpModule, settings editors etc.)
- [x] Mode switching is instant — only client-side state (modeStore.mode), no IPC required to flip tabs
- [x] Existing operations (login, install plugin, edit CLAUDE.md) still work — same modules, just rehoused into facets
- [x] Two-panel layout (no middle sub-panel; App.svelte rewrite in Task 15)
- [x] Account list with `+ Add Account` (Task 11)
- [x] Project mode shows the list + Stage 3 placeholder (Tasks 12-13)
- [x] App settings reachable via gear button (Tasks 9-10)

---

## Notes for the implementer

- **Backend refactor (Task 3) is mechanical but spread across 6 files.** Treat it as one task with sub-steps per file rather than splitting — the substitution rule is uniform (`state.inner.claude_home` → `state.current_dir().await`) and reviewing in one diff is easier.
- **The existing `commands::config::get_project_config` already takes `project_id`** and looks up in `state.inner.projects`. That stays unchanged — project-layer files live under `<project>/.claude/`, not under any account dir. Don't accidentally re-route those.
- **`commands::plugins::install_plugin` invokes the `claude` subprocess.** Subprocess invocations should now pass `CLAUDE_CONFIG_DIR=<current_dir>` so the CLI uses the active account. The existing `commands::launcher::launch_claude` already injects this env when launching for a project; mirror that pattern in plugins.
- **i18n is incrementally built up in Tasks 9-21 and consolidated in Task 22.** If a key is missing during dev, the `t()` function should return the key string verbatim — visible but harmless. Don't block on i18n completeness; consolidate at the end.
- **Don't delete the old AccountsView, LauncherView, EffectiveConfigView yet.** They're still imported by stubs / referenced from old code paths. Stage 4 sweeps them.
- **Project mode is intentionally a placeholder.** Adding any real Project facet logic crosses into Stage 3 territory — defer.
- **Branch is `main` (pre-release, no PR required).** Frequent commits, no squashing. Stage 1 ended at commit `5aff0dc`; Stage 2 commits land on top.
- **Per `CLAUDE.md` Svelte 5 gotcha #1**: HMR will silently fail to rebuild the reactive graph after adding new `$state`/`$derived`/`$effect`. Restart `pnpm tauri dev` after each new component file. Verifying via fresh restart is mandatory after Task 15.
