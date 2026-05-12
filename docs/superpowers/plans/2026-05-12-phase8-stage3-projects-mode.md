# Phase 8 Stage 3 — Projects Mode Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the Project-mode placeholder shipped in Stage 2 into a fully functional second perspective. Selecting a project in the Project sidebar opens 7 facets (Binding / Launch / Plugins↓ / Settings / Memory / CLAUDE.md / Effective) that operate on `<project>/.claude/` and on the **bound account's** directory, independent of which account the GUI is currently looking at in Account mode. Absorbs the legacy Launcher and Effective Config modules.

**Architecture:** A new family of *path-keyed* IPCs (`project_settings_*`, `project_claudemd_*`, `project_memory_*`, `project_plugins_*`, `project_effective_*`) on the Rust side. Each one looks up `~/.dot-claude-gui/config.json`'s `projects[path].account` to resolve which account directory to read from — **without** mutating `state.active_account_dir`. Frontend mirrors the Stage-2 facet pattern: a `ProjectModeView` shell renders a tab strip + slot, and each facet is a thin wrapper that calls the new IPCs. Unbound and stale-path projects are degraded at the view layer (only Binding clickable for unbound; all facets disabled with banner for stale).

**Tech Stack:** Rust 1.x (Tauri 2.0, tokio, serde), Svelte 5 runes, TypeScript strict, pnpm.

**Spec:** `docs/superpowers/specs/2026-05-11-phase8-mode-based-redesign-design.md` (Stage 3 = lines 216-225; Project facets = lines 63-71; edge cases = lines 72-77).

**Prior stages:**
- Stage 1 (`docs/superpowers/plans/2026-05-11-phase8-stage1-data-migration.md`): v2 schema, migration, default-account injection, `gui_projects` IPC.
- Stage 2 (`docs/superpowers/plans/2026-05-12-phase8-stage2-shell-and-accounts-mode.md`): mode-tab shell, AccountSidebar/ProjectSidebar, AccountModeView + 7 facets, `set_active_account` / `account_overview`.

---

## Architectural decisions (read before tasks)

1. **Path-keyed IPCs resolve the account from `config.json`, not from `state.active_account_dir`.** Project-mode IPCs accept `project_path: String`, read the live `AppConfig`, look up `projects[path].account`, and compute the account directory via `app_config::account_dir(home, name)`. They never touch `state.set_active_account()`. Rationale: spec §IA `Project Mode` and §Edge cases require account-agnostic project ops; Stage-2 final-review watch-out #2 forbids merging the two state machines.

2. **Old UUID-keyed IPCs (`commands::config::get_project_config`, `update_project_config`, `get_effective_config`, plus `commands::projects::*`) are NOT changed.** They stay path-disconnected. Stage 4 deletes them along with the unused `state.inner.projects: Vec<ProjectInfo>` registry. Rationale: rewriting them now bleeds risk into a non-load-bearing path; the GUI no longer calls them post-Stage-2.

3. **The Project sidebar's "selected project" is a path string, persisted via `modeStore.selectedProject`.** No UUID exists for Project-mode entities. Path-stale entries surface via `ProjectEntry.stale` from `gui_list_projects()`.

4. **Unbound vs stale are two distinct degraded states.** Unbound = `binding.account` is `None` (project in `knownProjects` but not `projects` map) → all facets except Binding are disabled and show empty-state in the tab strip. Stale = path doesn't exist on disk → **all** facets disabled, a banner at the top of `ProjectModeView` offers `Update path` / `Remove`. The `Update path` action is out of scope for Stage 3 (becomes a Stage 4 backlog item if user wants it); banner only shows `Remove` for now.

5. **Plugins↓ tri-state semantics:** the project-layer `settings.json` field is `enabledPlugins: Option<HashMap<String, bool>>`. Per-plugin states:
   - **Inherit** = key absent from project settings
   - **Enable** = key present, value `true`
   - **Disable** = key present, value `false`

   The list of *available* plugins comes from the bound account's `<account-dir>/plugins/` directory, not from `state.current_dir()`.

6. **Launcher reuse:** `commands::launcher::launch_claude` already takes `project_path`, `env`, `args`, and a preferred terminal. Stage-3 frontend calls it directly from `LaunchFacet`. The Stage-1 stubs `LauncherView.svelte` / `LauncherList.svelte` are deleted in Stage 4; in Stage 3 they remain orphans (no sidebar route points at them since Stage 2). `parseClaudeHelp.ts` + the `get_claude_args` IPC are reused for the args autocomplete inside `LaunchFacet`.

7. **CLAUDE_CONFIG_DIR injection rule (spec §Default-account semantics):** when launching:
   - `binding.account == "default"` → **do not** inject `CLAUDE_CONFIG_DIR` (claude uses native `~/.claude/`)
   - `binding.account == <other>` → inject `CLAUDE_CONFIG_DIR=~/.dot-claude-gui/accounts/<name>`

   Existing `launch_claude` accepts an `env` map from the caller. The frontend computes the dir-injection on top of binding launch env, **after** merging account-default override.

---

## File map

**New (backend):**
- `src-tauri/src/commands/project_facets.rs` — all path-keyed IPCs for Project mode: `project_read_settings`, `project_write_settings`, `project_read_claudemd`, `project_write_claudemd`, `project_list_memory`, `project_read_memory_file`, `project_write_memory_file`, `project_delete_memory_file`, `project_list_plugins`, `project_read_effective`. Each command resolves account dir from `AppConfig.projects[path].account`.

**Modify (backend):**
- `src-tauri/src/commands/mod.rs` — `pub mod project_facets;`
- `src-tauri/src/lib.rs` — register the 10 new commands in `tauri::generate_handler![...]`
- `src-tauri/src/state.rs` — add helper `pub async fn resolve_project_account_dir(&self, project_path: &str) -> Result<PathBuf, String>` on `AppState` (reads `app_config_path`, looks up binding, returns account dir; errors with stable code on unbound or unknown account)
- `src-tauri/src/commands/launcher.rs` — extend `launch_claude` to accept `account: Option<String>`; when `Some(name)` and `name != "default"`, inject `CLAUDE_CONFIG_DIR` before spawning. Add tests for both branches.

**New (frontend):**
- `src/lib/components/project-mode/ProjectModeView.svelte` — facet tab strip + active-facet slot + banners
- `src/lib/components/project-mode/BindingFacet.svelte` — account picker, path readonly, Open Terminal / Unbind / Remove
- `src/lib/components/project-mode/LaunchFacet.svelte` — env table + args list + Launch button (reuses `parseClaudeHelp` autocomplete)
- `src/lib/components/project-mode/PluginsOverrideFacet.svelte` — tri-state list (Inherit / Enable / Disable)
- `src/lib/components/project-mode/ProjectSettingsFacet.svelte` — JSON editor with validate-on-blur
- `src/lib/components/project-mode/ProjectMemoryFacet.svelte` — file list + viewer/editor scoped to bound-account's `<account>/projects/<encoded>/memory/`
- `src/lib/components/project-mode/ProjectClaudeMdFacet.svelte` — markdown textarea for `<project>/.claude/CLAUDE.md`
- `src/lib/components/project-mode/EffectiveFacet.svelte` — read-only merged view, panels grouped by source layer
- `src/lib/components/project-mode/StalePathBanner.svelte` — banner with `Remove` action
- `src/lib/components/project-mode/UnboundHint.svelte` — empty-state hint shown in tab body when a non-Binding tab is selected for an unbound project

**Modify (frontend):**
- `src/App.svelte` — replace `<ProjectModePlaceholder />` mount with `<ProjectModeView />`
- `src/lib/ipc/client.ts` — add 10 new wrappers + extend `launchClaude` signature to pass `account`
- `src/lib/api/types.ts` — add `ProjectFacetSettings`, `ProjectEffectiveResponse`, `ProjectPlugin`, `ProjectMemoryEntry` (mirroring Rust return types)
- `src/lib/stores/mode.svelte.ts` — add `selectedProjectFacet: ProjectFacetKey` with persistence (`"binding" | "launch" | "plugins" | "settings" | "memory" | "claudemd" | "effective"`); default `"binding"`
- `src/lib/stores/projects.svelte.ts` — add derived `currentBinding`, `currentLaunch`, `currentStale`, `currentBound` helpers
- `src/lib/i18n/locales/locales/zh-CN.json`, `en-US.json`, `ja-JP.json` — keys under `projectMode.*` for facet titles and banner copy

**Leave alone (Stage 4 territory):**
- `src/lib/components/launcher/LauncherView.svelte`, `LauncherList.svelte` (orphans, Stage 4 deletes)
- `src/lib/components/effective/EffectiveConfigView.svelte` (replaced by `EffectiveFacet`; old one deleted in Stage 4 along with sidebar route)
- `src-tauri/src/commands/projects.rs`, `commands::config::{get_project_config, update_project_config, get_effective_config}` — old UUID-keyed APIs (Stage 4 deletes)
- `src-tauri/src/state.rs::projects` Vec registry (Stage 4 removes)

---

## Backend tasks (1-7)

### Task 1: `resolve_project_account_dir` helper on `AppState`

**Files:**
- Modify: `src-tauri/src/state.rs`

- [ ] **Step 1: Write failing test**

Append to the existing `#[cfg(test)] mod tests` block in `src-tauri/src/state.rs`:

```rust
    #[tokio::test]
    async fn resolve_project_account_dir_returns_account_dir_for_bound() {
        use tempfile::tempdir;
        use crate::app_config::{AppConfig, AccountEntry, ProjectBinding, ProjectLaunch};

        let home = tempdir().unwrap();
        let cfg_path = home.path().join(".dot-claude-gui").join("config.json");
        std::fs::create_dir_all(cfg_path.parent().unwrap()).unwrap();

        let mut cfg = AppConfig::default();
        cfg.accounts.push(AccountEntry {
            name: "work".into(),
            display_name: "Work".into(),
            is_native: false,
            created_at: "2026-05-12T00:00:00Z".into(),
        });
        cfg.known_projects.push("/p1".into());
        cfg.projects.insert(
            "/p1".into(),
            ProjectBinding { account: "work".into(), launch: ProjectLaunch::default() },
        );
        std::fs::write(&cfg_path, serde_json::to_string(&cfg).unwrap()).unwrap();

        let state = AppState::new_for_test(home.path().to_path_buf());
        let resolved = state.resolve_project_account_dir("/p1").await.unwrap();
        assert_eq!(
            resolved,
            home.path().join(".dot-claude-gui").join("accounts").join("work")
        );
    }

    #[tokio::test]
    async fn resolve_project_account_dir_returns_native_for_default_binding() {
        use tempfile::tempdir;
        use crate::app_config::{AppConfig, ProjectBinding, ProjectLaunch};

        let home = tempdir().unwrap();
        let cfg_path = home.path().join(".dot-claude-gui").join("config.json");
        std::fs::create_dir_all(cfg_path.parent().unwrap()).unwrap();
        let mut cfg = AppConfig::default();
        cfg.known_projects.push("/p2".into());
        cfg.projects.insert(
            "/p2".into(),
            ProjectBinding { account: "default".into(), launch: ProjectLaunch::default() },
        );
        std::fs::write(&cfg_path, serde_json::to_string(&cfg).unwrap()).unwrap();

        let state = AppState::new_for_test(home.path().to_path_buf());
        let resolved = state.resolve_project_account_dir("/p2").await.unwrap();
        assert_eq!(resolved, home.path().join(".claude"));
    }

    #[tokio::test]
    async fn resolve_project_account_dir_errors_for_unbound() {
        use tempfile::tempdir;
        use crate::app_config::AppConfig;

        let home = tempdir().unwrap();
        let cfg_path = home.path().join(".dot-claude-gui").join("config.json");
        std::fs::create_dir_all(cfg_path.parent().unwrap()).unwrap();
        let mut cfg = AppConfig::default();
        cfg.known_projects.push("/p3".into());
        std::fs::write(&cfg_path, serde_json::to_string(&cfg).unwrap()).unwrap();

        let state = AppState::new_for_test(home.path().to_path_buf());
        let err = state.resolve_project_account_dir("/p3").await.unwrap_err();
        assert!(err.contains("unbound") || err.contains("Unbound"));
    }
```

`AppState::new_for_test` helper may need to exist — check `state.rs` and add a minimal `pub fn new_for_test(home: PathBuf) -> Self` gated behind `#[cfg(test)]` if absent.

- [ ] **Step 2: Run tests, expect failure**

```bash
cargo test -p dot-claude-gui state::tests::resolve_project_account_dir
```

Expected: 3 tests fail with "no method named `resolve_project_account_dir`".

- [ ] **Step 3: Implement helper**

Add to the `impl AppState` block in `src-tauri/src/state.rs`:

```rust
    pub async fn resolve_project_account_dir(&self, project_path: &str) -> Result<PathBuf, String> {
        let cfg = self.inner.app_config.read().await.clone();
        let binding = cfg
            .projects
            .get(project_path)
            .ok_or_else(|| format!("Unbound project: {project_path}"))?;
        let account_name = &binding.account;
        if cfg.accounts.iter().all(|a| &a.name != account_name) {
            return Err(format!("Unknown account: {account_name}"));
        }
        Ok(crate::app_config::account_dir(&self.inner.home, account_name))
    }
```

Adjust field path (`inner.app_config` / `inner.home`) to match what Stage 1/2 actually shipped — if `app_config` is stored under a different name, use that.

- [ ] **Step 4: Run tests, expect pass**

```bash
cargo test -p dot-claude-gui state::tests::resolve_project_account_dir
```

Expected: 3 passing.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/state.rs
git commit -m "feat(stage3): resolve_project_account_dir helper on AppState"
```

---

### Task 2: `project_facets` module + settings IPCs

**Files:**
- Create: `src-tauri/src/commands/project_facets.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write failing test**

Create `src-tauri/src/commands/project_facets.rs` with the test module first:

```rust
use crate::state::AppState;
use claude_types::Settings;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectSettingsResponse {
    pub path: String,
    pub exists: bool,
    pub settings: Settings,
}

#[derive(Debug, Deserialize)]
pub struct WriteProjectSettingsRequest {
    pub project_path: String,
    pub settings: Settings,
}

#[tauri::command]
pub async fn project_read_settings(
    _state: State<'_, AppState>,
    _project_path: String,
) -> Result<ProjectSettingsResponse, String> {
    unimplemented!()
}

#[tauri::command]
pub async fn project_write_settings(
    _state: State<'_, AppState>,
    _request: WriteProjectSettingsRequest,
) -> Result<(), String> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn project_settings_path_resolves_under_dot_claude() {
        let proj = tempdir().unwrap();
        let p = project_settings_path(proj.path().to_str().unwrap());
        assert_eq!(p, proj.path().join(".claude").join("settings.json"));
    }

    #[tokio::test]
    async fn read_missing_settings_returns_default_with_exists_false() {
        let proj = tempdir().unwrap();
        let resp = read_settings_for_path(proj.path().to_str().unwrap()).unwrap();
        assert!(!resp.exists);
        assert_eq!(resp.path, proj.path().join(".claude").join("settings.json").to_string_lossy());
    }

    #[tokio::test]
    async fn write_then_read_round_trips() {
        let proj = tempdir().unwrap();
        let mut s = Settings::default();
        s.env = Some([("FOO".to_string(), "bar".to_string())].into_iter().collect());
        write_settings_for_path(proj.path().to_str().unwrap(), &s).unwrap();
        let resp = read_settings_for_path(proj.path().to_str().unwrap()).unwrap();
        assert!(resp.exists);
        assert_eq!(resp.settings.env.as_ref().unwrap().get("FOO").unwrap(), "bar");
    }
}
```

Note the tests reference `project_settings_path`, `read_settings_for_path`, `write_settings_for_path` — these are internal helpers without the Tauri `State` dependency, so they can be unit-tested. The `#[tauri::command]` wrappers in production will call them via `state.resolve_project_account_dir` is NOT needed here (Settings live at the project path, not under an account).

- [ ] **Step 2: Run tests, expect failure**

Add `pub mod project_facets;` to `src-tauri/src/commands/mod.rs`, then:

```bash
cargo test -p dot-claude-gui commands::project_facets::tests
```

Expected: 3 tests fail (helpers don't exist).

- [ ] **Step 3: Implement helpers + commands**

Replace the `unimplemented!()` bodies with:

```rust
pub(crate) fn project_settings_path(project_path: &str) -> PathBuf {
    PathBuf::from(project_path).join(".claude").join("settings.json")
}

pub(crate) fn read_settings_for_path(project_path: &str) -> Result<ProjectSettingsResponse, String> {
    let path = project_settings_path(project_path);
    if !path.exists() {
        return Ok(ProjectSettingsResponse {
            path: path.to_string_lossy().into_owned(),
            exists: false,
            settings: Settings::default(),
        });
    }
    let raw = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let settings: Settings = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    Ok(ProjectSettingsResponse {
        path: path.to_string_lossy().into_owned(),
        exists: true,
        settings,
    })
}

pub(crate) fn write_settings_for_path(project_path: &str, settings: &Settings) -> Result<(), String> {
    let path = project_settings_path(project_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("json.tmp");
    let body = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(&tmp, body).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn project_read_settings(
    _state: State<'_, AppState>,
    project_path: String,
) -> Result<ProjectSettingsResponse, String> {
    read_settings_for_path(&project_path)
}

#[tauri::command]
pub async fn project_write_settings(
    _state: State<'_, AppState>,
    request: WriteProjectSettingsRequest,
) -> Result<(), String> {
    write_settings_for_path(&request.project_path, &request.settings)
}
```

Register the two commands in `src-tauri/src/lib.rs` inside `tauri::generate_handler![...]`:

```rust
            commands::project_facets::project_read_settings,
            commands::project_facets::project_write_settings,
```

- [ ] **Step 4: Run tests, expect pass**

```bash
cargo test -p dot-claude-gui commands::project_facets
cargo build -p dot-claude-gui
```

Expected: all green.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/project_facets.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat(stage3): project_read_settings/project_write_settings IPCs"
```

---

### Task 3: Project CLAUDE.md IPCs

**Files:**
- Modify: `src-tauri/src/commands/project_facets.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write failing test**

Append to the `tests` module in `project_facets.rs`:

```rust
    #[test]
    fn project_claudemd_path_resolves_under_dot_claude() {
        let proj = tempdir().unwrap();
        let p = project_claudemd_path(proj.path().to_str().unwrap());
        assert_eq!(p, proj.path().join(".claude").join("CLAUDE.md"));
    }

    #[test]
    fn read_missing_claudemd_returns_empty_with_exists_false() {
        let proj = tempdir().unwrap();
        let resp = read_claudemd_for_path(proj.path().to_str().unwrap()).unwrap();
        assert!(!resp.exists);
        assert_eq!(resp.content, "");
    }

    #[test]
    fn write_then_read_claudemd_round_trips() {
        let proj = tempdir().unwrap();
        write_claudemd_for_path(proj.path().to_str().unwrap(), "# Hello\n").unwrap();
        let resp = read_claudemd_for_path(proj.path().to_str().unwrap()).unwrap();
        assert!(resp.exists);
        assert_eq!(resp.content, "# Hello\n");
    }
```

Add the response type near the top:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectClaudeMdResponse {
    pub path: String,
    pub exists: bool,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct WriteProjectClaudeMdRequest {
    pub project_path: String,
    pub content: String,
}
```

- [ ] **Step 2: Run tests, expect failure**

```bash
cargo test -p dot-claude-gui commands::project_facets::tests::project_claudemd
cargo test -p dot-claude-gui commands::project_facets::tests::read_missing_claudemd
cargo test -p dot-claude-gui commands::project_facets::tests::write_then_read_claudemd
```

Expected: 3 fail (helpers don't exist).

- [ ] **Step 3: Implement**

Add to `project_facets.rs`:

```rust
pub(crate) fn project_claudemd_path(project_path: &str) -> PathBuf {
    PathBuf::from(project_path).join(".claude").join("CLAUDE.md")
}

pub(crate) fn read_claudemd_for_path(project_path: &str) -> Result<ProjectClaudeMdResponse, String> {
    let path = project_claudemd_path(project_path);
    if !path.exists() {
        return Ok(ProjectClaudeMdResponse {
            path: path.to_string_lossy().into_owned(),
            exists: false,
            content: String::new(),
        });
    }
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    Ok(ProjectClaudeMdResponse {
        path: path.to_string_lossy().into_owned(),
        exists: true,
        content,
    })
}

pub(crate) fn write_claudemd_for_path(project_path: &str, content: &str) -> Result<(), String> {
    let path = project_claudemd_path(project_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("md.tmp");
    std::fs::write(&tmp, content).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn project_read_claudemd(
    _state: State<'_, AppState>,
    project_path: String,
) -> Result<ProjectClaudeMdResponse, String> {
    read_claudemd_for_path(&project_path)
}

#[tauri::command]
pub async fn project_write_claudemd(
    _state: State<'_, AppState>,
    request: WriteProjectClaudeMdRequest,
) -> Result<(), String> {
    write_claudemd_for_path(&request.project_path, &request.content)
}
```

Register in `lib.rs`:

```rust
            commands::project_facets::project_read_claudemd,
            commands::project_facets::project_write_claudemd,
```

- [ ] **Step 4: Run tests, expect pass**

```bash
cargo test -p dot-claude-gui commands::project_facets
cargo build -p dot-claude-gui
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/project_facets.rs src-tauri/src/lib.rs
git commit -m "feat(stage3): project_read_claudemd/project_write_claudemd IPCs"
```

---

### Task 4: Project Memory IPCs (account-scoped via binding)

**Files:**
- Modify: `src-tauri/src/commands/project_facets.rs`
- Modify: `src-tauri/src/lib.rs`

These IPCs read/write files under `<account-dir>/projects/<encoded-path>/memory/`. The account dir comes from the binding, not from `state.active_account_dir`.

- [ ] **Step 1: Write failing test**

Append to `project_facets.rs` tests. Use the existing `decode_project_path` / `encode_project_path` style from `commands::memory.rs` — re-export the encoder there if needed, or copy the canonical form (`/` → `-`).

```rust
    #[tokio::test]
    async fn project_memory_dir_uses_binding_account() {
        use tempfile::tempdir;
        use crate::app_config::{AppConfig, AccountEntry, ProjectBinding, ProjectLaunch};
        use crate::state::AppState;

        let home = tempdir().unwrap();
        std::fs::create_dir_all(home.path().join(".dot-claude-gui")).unwrap();
        let mut cfg = AppConfig::default();
        cfg.accounts.push(AccountEntry {
            name: "work".into(),
            display_name: "Work".into(),
            is_native: false,
            created_at: "2026-05-12T00:00:00Z".into(),
        });
        cfg.known_projects.push("/Users/eric/code/foo".into());
        cfg.projects.insert(
            "/Users/eric/code/foo".into(),
            ProjectBinding { account: "work".into(), launch: ProjectLaunch::default() },
        );
        std::fs::write(
            home.path().join(".dot-claude-gui").join("config.json"),
            serde_json::to_string(&cfg).unwrap(),
        ).unwrap();

        let state = AppState::new_for_test(home.path().to_path_buf());
        let dir = project_memory_dir(&state, "/Users/eric/code/foo").await.unwrap();
        assert_eq!(
            dir,
            home.path()
                .join(".dot-claude-gui")
                .join("accounts")
                .join("work")
                .join("projects")
                .join("-Users-eric-code-foo")
                .join("memory")
        );
    }
```

- [ ] **Step 2: Run tests, expect failure**

```bash
cargo test -p dot-claude-gui commands::project_facets::tests::project_memory_dir
```

Expected: fail (`project_memory_dir` undefined).

- [ ] **Step 3: Implement memory IPCs**

Add to `project_facets.rs`:

```rust
use claude_types::memory::MemoryFileEntry;

pub(crate) async fn project_memory_dir(
    state: &AppState,
    project_path: &str,
) -> Result<PathBuf, String> {
    let account_dir = state.resolve_project_account_dir(project_path).await?;
    let encoded = project_path.replace('/', "-");
    Ok(account_dir.join("projects").join(encoded).join("memory"))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectMemoryListResponse {
    pub path: String,
    pub files: Vec<MemoryFileEntry>,
}

#[tauri::command]
pub async fn project_list_memory(
    state: State<'_, AppState>,
    project_path: String,
) -> Result<ProjectMemoryListResponse, String> {
    let dir = project_memory_dir(&state, &project_path).await?;
    let files = if dir.exists() {
        crate::commands::memory::list_files_in_dir(&dir).map_err(|e| e.to_string())?
    } else {
        Vec::new()
    };
    Ok(ProjectMemoryListResponse {
        path: dir.to_string_lossy().into_owned(),
        files,
    })
}

#[derive(Debug, Deserialize)]
pub struct ProjectMemoryFileRequest {
    pub project_path: String,
    pub file_name: String,
}

#[derive(Debug, Deserialize)]
pub struct WriteProjectMemoryRequest {
    pub project_path: String,
    pub file_name: String,
    pub content: String,
}

#[tauri::command]
pub async fn project_read_memory_file(
    state: State<'_, AppState>,
    request: ProjectMemoryFileRequest,
) -> Result<String, String> {
    let dir = project_memory_dir(&state, &request.project_path).await?;
    let path = dir.join(&request.file_name);
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn project_write_memory_file(
    state: State<'_, AppState>,
    request: WriteProjectMemoryRequest,
) -> Result<(), String> {
    let dir = project_memory_dir(&state, &request.project_path).await?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(&request.file_name);
    let tmp = path.with_extension("md.tmp");
    std::fs::write(&tmp, &request.content).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn project_delete_memory_file(
    state: State<'_, AppState>,
    request: ProjectMemoryFileRequest,
) -> Result<(), String> {
    let dir = project_memory_dir(&state, &request.project_path).await?;
    let path = dir.join(&request.file_name);
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}
```

If `crate::commands::memory::list_files_in_dir` does not exist as a public helper, lift the inner directory-iteration block out of `commands::memory::list_memory_files` into a `pub(crate) fn list_files_in_dir(dir: &Path) -> std::io::Result<Vec<MemoryFileEntry>>` in `commands/memory.rs` first, then reuse from here.

Register in `lib.rs`:

```rust
            commands::project_facets::project_list_memory,
            commands::project_facets::project_read_memory_file,
            commands::project_facets::project_write_memory_file,
            commands::project_facets::project_delete_memory_file,
```

- [ ] **Step 4: Run tests, expect pass**

```bash
cargo test -p dot-claude-gui commands::project_facets
cargo build -p dot-claude-gui
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/project_facets.rs src-tauri/src/commands/memory.rs src-tauri/src/lib.rs
git commit -m "feat(stage3): project_*_memory IPCs scoped to binding account"
```

---

### Task 5: Project Plugins IPC (list available from bound account)

**Files:**
- Modify: `src-tauri/src/commands/project_facets.rs`
- Modify: `src-tauri/src/lib.rs`

The frontend Plugins↓ facet needs **two** data sources: the list of plugins installed under the bound account, plus the project layer's `enabledPlugins` map (which it reads via `project_read_settings` already). This task only adds the first.

- [ ] **Step 1: Write failing test**

Append to `project_facets.rs` tests:

```rust
    #[tokio::test]
    async fn project_list_plugins_reads_from_bound_account() {
        use tempfile::tempdir;
        use crate::app_config::{AppConfig, AccountEntry, ProjectBinding, ProjectLaunch};
        use crate::state::AppState;

        let home = tempdir().unwrap();
        std::fs::create_dir_all(home.path().join(".dot-claude-gui")).unwrap();
        let account_dir = home.path().join(".dot-claude-gui").join("accounts").join("work");
        std::fs::create_dir_all(account_dir.join("plugins").join("my-plugin")).unwrap();
        std::fs::write(
            account_dir.join("plugins").join("my-plugin").join(".plugin-meta.json"),
            r#"{"name":"my-plugin","version":"1.0.0","description":"x"}"#,
        ).unwrap();

        let mut cfg = AppConfig::default();
        cfg.accounts.push(AccountEntry {
            name: "work".into(), display_name: "W".into(),
            is_native: false, created_at: "2026-05-12".into(),
        });
        cfg.known_projects.push("/p".into());
        cfg.projects.insert(
            "/p".into(),
            ProjectBinding { account: "work".into(), launch: ProjectLaunch::default() },
        );
        std::fs::write(
            home.path().join(".dot-claude-gui").join("config.json"),
            serde_json::to_string(&cfg).unwrap(),
        ).unwrap();

        let state = AppState::new_for_test(home.path().to_path_buf());
        let plugins = list_plugins_for_project(&state, "/p").await.unwrap();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].name, "my-plugin");
    }
```

- [ ] **Step 2: Run tests, expect failure**

```bash
cargo test -p dot-claude-gui commands::project_facets::tests::project_list_plugins
```

Expected: fail (`list_plugins_for_project` undefined).

- [ ] **Step 3: Implement**

Add to `project_facets.rs`:

```rust
use claude_types::plugins::PluginInfo;

pub(crate) async fn list_plugins_for_project(
    state: &AppState,
    project_path: &str,
) -> Result<Vec<PluginInfo>, String> {
    let account_dir = state.resolve_project_account_dir(project_path).await?;
    let plugins_dir = account_dir.join("plugins");
    crate::commands::plugins::list_plugins_in_dir(&plugins_dir).await
}

#[tauri::command]
pub async fn project_list_plugins(
    state: State<'_, AppState>,
    project_path: String,
) -> Result<Vec<PluginInfo>, String> {
    list_plugins_for_project(&state, &project_path).await
}
```

Lift the directory-scanning helper out of `commands::plugins::list_plugins` into `pub(crate) async fn list_plugins_in_dir(dir: &Path) -> Result<Vec<PluginInfo>, String>` first (similar pattern to Task 4). Then `commands::plugins::list_plugins` becomes:

```rust
pub async fn list_plugins(state: State<'_, AppState>) -> Result<Vec<PluginInfo>, String> {
    let dir = state.current_dir().await.join("plugins");
    list_plugins_in_dir(&dir).await
}
```

Register in `lib.rs`:

```rust
            commands::project_facets::project_list_plugins,
```

- [ ] **Step 4: Run tests, expect pass**

```bash
cargo test -p dot-claude-gui
cargo build -p dot-claude-gui
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/project_facets.rs src-tauri/src/commands/plugins.rs src-tauri/src/lib.rs
git commit -m "feat(stage3): project_list_plugins reads from bound account"
```

---

### Task 6: Project Effective Config IPC

**Files:**
- Modify: `src-tauri/src/commands/project_facets.rs`
- Modify: `src-tauri/src/lib.rs`

Merges User (from bound account's `settings.json`) → Project (`<path>/.claude/settings.json`) → Local (`<path>/.claude/settings.local.json`) with per-field source annotation.

- [ ] **Step 1: Write failing test**

Append to `project_facets.rs` tests:

```rust
    #[tokio::test]
    async fn project_read_effective_layers_overrides_correctly() {
        use tempfile::tempdir;
        use crate::app_config::{AppConfig, AccountEntry, ProjectBinding, ProjectLaunch};
        use crate::state::AppState;
        use std::collections::HashMap;

        let home = tempdir().unwrap();
        let proj = tempdir().unwrap();
        let proj_str = proj.path().to_str().unwrap();

        std::fs::create_dir_all(home.path().join(".dot-claude-gui")).unwrap();
        let account_dir = home.path().join(".dot-claude-gui").join("accounts").join("work");
        std::fs::create_dir_all(&account_dir).unwrap();
        // User layer: enabledPlugins{a=true}
        let mut user_settings = Settings::default();
        let mut user_plugins = HashMap::new();
        user_plugins.insert("a".to_string(), true);
        user_settings.enabled_plugins = Some(user_plugins);
        std::fs::write(
            account_dir.join("settings.json"),
            serde_json::to_string(&user_settings).unwrap(),
        ).unwrap();

        // Project layer: enabledPlugins{a=false, b=true}
        std::fs::create_dir_all(proj.path().join(".claude")).unwrap();
        let mut proj_settings = Settings::default();
        let mut proj_plugins = HashMap::new();
        proj_plugins.insert("a".to_string(), false);
        proj_plugins.insert("b".to_string(), true);
        proj_settings.enabled_plugins = Some(proj_plugins);
        std::fs::write(
            proj.path().join(".claude").join("settings.json"),
            serde_json::to_string(&proj_settings).unwrap(),
        ).unwrap();

        let mut cfg = AppConfig::default();
        cfg.accounts.push(AccountEntry {
            name: "work".into(), display_name: "W".into(),
            is_native: false, created_at: "2026-05-12".into(),
        });
        cfg.known_projects.push(proj_str.into());
        cfg.projects.insert(
            proj_str.into(),
            ProjectBinding { account: "work".into(), launch: ProjectLaunch::default() },
        );
        std::fs::write(
            home.path().join(".dot-claude-gui").join("config.json"),
            serde_json::to_string(&cfg).unwrap(),
        ).unwrap();

        let state = AppState::new_for_test(home.path().to_path_buf());
        let eff = read_effective_for_project(&state, proj_str).await.unwrap();
        let plugins = eff.settings.enabled_plugins.as_ref().unwrap();
        assert_eq!(plugins.get("a"), Some(&false), "project overrides user");
        assert_eq!(plugins.get("b"), Some(&true), "project-only key kept");
        // Source annotation:
        assert_eq!(eff.field_sources.get("enabledPlugins.a").map(String::as_str), Some("project"));
        assert_eq!(eff.field_sources.get("enabledPlugins.b").map(String::as_str), Some("project"));
    }
```

- [ ] **Step 2: Run tests, expect failure**

```bash
cargo test -p dot-claude-gui commands::project_facets::tests::project_read_effective
```

Expected: fail.

- [ ] **Step 3: Implement**

Add to `project_facets.rs`:

```rust
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectEffectiveResponse {
    pub project_path: String,
    pub account: String,
    pub settings: Settings,
    pub field_sources: HashMap<String, String>,
}

pub(crate) async fn read_effective_for_project(
    state: &AppState,
    project_path: &str,
) -> Result<ProjectEffectiveResponse, String> {
    let account_dir = state.resolve_project_account_dir(project_path).await?;
    let account = {
        let cfg = state.inner.app_config.read().await;
        cfg.projects.get(project_path)
            .map(|b| b.account.clone())
            .ok_or_else(|| format!("Unbound project: {project_path}"))?
    };

    let user_path = account_dir.join("settings.json");
    let project_p = project_settings_path(project_path);
    let local_p = PathBuf::from(project_path).join(".claude").join("settings.local.json");

    let user_layer: Settings = read_or_default(&user_path)?;
    let project_layer: Settings = read_or_default(&project_p)?;
    let local_layer: Settings = read_or_default(&local_p)?;

    let (settings, field_sources) =
        claude_config::merge::merge_with_sources(&user_layer, &project_layer, &local_layer);

    Ok(ProjectEffectiveResponse {
        project_path: project_path.to_string(),
        account,
        settings,
        field_sources,
    })
}

fn read_or_default(path: &Path) -> Result<Settings, String> {
    if !path.exists() { return Ok(Settings::default()); }
    let raw = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn project_read_effective(
    state: State<'_, AppState>,
    project_path: String,
) -> Result<ProjectEffectiveResponse, String> {
    read_effective_for_project(&state, &project_path).await
}
```

The function `claude_config::merge::merge_with_sources(user, project, local)` may already exist (the old `commands::config::get_effective_config` uses it). If the existing function has a different signature (e.g., takes a `managed` layer too), pass `Settings::default()` for managed. If no such function exists, port the merge logic from `commands::config::get_effective_config` into the `claude-config` crate first as a new task here.

Register in `lib.rs`:

```rust
            commands::project_facets::project_read_effective,
```

- [ ] **Step 4: Run tests, expect pass**

```bash
cargo test -p dot-claude-gui
cargo build -p dot-claude-gui
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/project_facets.rs src-tauri/src/lib.rs crates/claude-config/
git commit -m "feat(stage3): project_read_effective merges layers per binding"
```

---

### Task 7: Extend `launch_claude` to inject `CLAUDE_CONFIG_DIR` per account

**Files:**
- Modify: `src-tauri/src/commands/launcher.rs`

- [ ] **Step 1: Write failing test**

The existing test module in `launcher.rs` (or add `#[cfg(test)] mod tests` if absent) should grow:

```rust
    #[test]
    fn build_env_for_default_account_omits_claude_config_dir() {
        let home = std::path::PathBuf::from("/home/u");
        let env = build_launch_env(&home, Some("default"), &Default::default());
        assert!(!env.contains_key("CLAUDE_CONFIG_DIR"));
    }

    #[test]
    fn build_env_for_named_account_injects_claude_config_dir() {
        let home = std::path::PathBuf::from("/home/u");
        let env = build_launch_env(&home, Some("work"), &Default::default());
        assert_eq!(
            env.get("CLAUDE_CONFIG_DIR").map(String::as_str),
            Some("/home/u/.dot-claude-gui/accounts/work")
        );
    }

    #[test]
    fn user_env_overrides_account_dir_injection() {
        let home = std::path::PathBuf::from("/home/u");
        let mut user = std::collections::HashMap::new();
        user.insert("CLAUDE_CONFIG_DIR".to_string(), "/custom".to_string());
        let env = build_launch_env(&home, Some("work"), &user);
        assert_eq!(env.get("CLAUDE_CONFIG_DIR").map(String::as_str), Some("/custom"));
    }

    #[test]
    fn nil_account_omits_injection() {
        let home = std::path::PathBuf::from("/home/u");
        let env = build_launch_env(&home, None, &Default::default());
        assert!(!env.contains_key("CLAUDE_CONFIG_DIR"));
    }
```

- [ ] **Step 2: Run tests, expect failure**

```bash
cargo test -p dot-claude-gui commands::launcher
```

Expected: 4 fail (`build_launch_env` undefined).

- [ ] **Step 3: Implement `build_launch_env` + wire into `launch_claude`**

Add to `launcher.rs`:

```rust
pub(crate) fn build_launch_env(
    home: &std::path::Path,
    account: Option<&str>,
    user_env: &std::collections::HashMap<String, String>,
) -> std::collections::HashMap<String, String> {
    let mut env = user_env.clone();
    if let Some(name) = account {
        if name != "default" && !env.contains_key("CLAUDE_CONFIG_DIR") {
            let dir = crate::app_config::account_dir(home, name);
            env.insert("CLAUDE_CONFIG_DIR".to_string(), dir.to_string_lossy().into_owned());
        }
    }
    env
}
```

Extend `LaunchRequest` with `pub account: Option<String>`, and inside `launch_claude` replace the raw env pass-through with:

```rust
let home = dirs::home_dir().ok_or_else(|| "no home".to_string())?;
let env = build_launch_env(&home, req.account.as_deref(), &req.env);
```

Then use `env` for the spawn. (Keep all other behavior — terminal preference, AppleScript, etc. — unchanged.)

- [ ] **Step 4: Run tests, expect pass**

```bash
cargo test -p dot-claude-gui commands::launcher
cargo build -p dot-claude-gui
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/launcher.rs
git commit -m "feat(stage3): launch_claude injects CLAUDE_CONFIG_DIR for named accounts"
```

---

## Frontend tasks (8-20)

### Task 8: IPC client wrappers for the new path-keyed commands

**Files:**
- Modify: `src/lib/ipc/client.ts`
- Modify: `src/lib/api/types.ts`

- [ ] **Step 1: Add types to `src/lib/api/types.ts`**

```ts
export interface ProjectSettingsResponse {
  path: string;
  exists: boolean;
  settings: Settings;
}

export interface WriteProjectSettingsRequest {
  projectPath: string;
  settings: Settings;
}

export interface ProjectClaudeMdResponse {
  path: string;
  exists: boolean;
  content: string;
}

export interface ProjectMemoryListResponse {
  path: string;
  files: MemoryFileEntry[];
}

export interface ProjectEffectiveResponse {
  projectPath: string;
  account: string;
  settings: Settings;
  fieldSources: Record<string, string>;
}
```

Re-use existing `Settings`, `MemoryFileEntry`, `PluginInfo` types — don't redefine.

- [ ] **Step 2: Add 10 methods to `IpcClient` in `src/lib/ipc/client.ts`**

```ts
  projectReadSettings(projectPath: string): Promise<ProjectSettingsResponse> {
    return invoke("project_read_settings", { projectPath });
  }

  projectWriteSettings(projectPath: string, settings: Settings): Promise<void> {
    return invoke("project_write_settings", { request: { projectPath, settings } });
  }

  projectReadClaudeMd(projectPath: string): Promise<ProjectClaudeMdResponse> {
    return invoke("project_read_claudemd", { projectPath });
  }

  projectWriteClaudeMd(projectPath: string, content: string): Promise<void> {
    return invoke("project_write_claudemd", { request: { projectPath, content } });
  }

  projectListMemory(projectPath: string): Promise<ProjectMemoryListResponse> {
    return invoke("project_list_memory", { projectPath });
  }

  projectReadMemoryFile(projectPath: string, fileName: string): Promise<string> {
    return invoke("project_read_memory_file", { request: { projectPath, fileName } });
  }

  projectWriteMemoryFile(projectPath: string, fileName: string, content: string): Promise<void> {
    return invoke("project_write_memory_file", { request: { projectPath, fileName, content } });
  }

  projectDeleteMemoryFile(projectPath: string, fileName: string): Promise<void> {
    return invoke("project_delete_memory_file", { request: { projectPath, fileName } });
  }

  projectListPlugins(projectPath: string): Promise<PluginInfo[]> {
    return invoke("project_list_plugins", { projectPath });
  }

  projectReadEffective(projectPath: string): Promise<ProjectEffectiveResponse> {
    return invoke("project_read_effective", { projectPath });
  }
```

Extend the existing `launchClaude` to accept `account?: string`:

```ts
  launchClaude(opts: {
    projectPath: string;
    env?: Record<string, string>;
    args?: string[];
    preferredTerminal?: "terminal" | "iterm2";
    account?: string;
  }): Promise<void> {
    return invoke("launch_claude", {
      req: {
        projectPath: opts.projectPath,
        env: opts.env ?? {},
        args: opts.args ?? [],
        preferredTerminal: opts.preferredTerminal,
        account: opts.account,
      },
    });
  }
```

- [ ] **Step 3: Verify type-check passes**

```bash
pnpm tsc --noEmit
```

Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add src/lib/ipc/client.ts src/lib/api/types.ts
git commit -m "feat(stage3): IPC client wrappers for project_* commands"
```

---

### Task 9: ModeStore + ProjectsStore augmentation

**Files:**
- Modify: `src/lib/stores/mode.svelte.ts`
- Modify: `src/lib/stores/projects.svelte.ts`

- [ ] **Step 1: Extend mode store with `selectedProjectFacet`**

In `src/lib/stores/mode.svelte.ts`:

```ts
export type ProjectFacetKey =
  | "binding" | "launch" | "plugins" | "settings" | "memory" | "claudemd" | "effective";

const FACET_STORAGE_KEY = "dotclaude.modeStore.selectedProjectFacet";

function loadFacet(): ProjectFacetKey {
  const raw = localStorage.getItem(FACET_STORAGE_KEY);
  if (raw === "binding" || raw === "launch" || raw === "plugins" || raw === "settings"
      || raw === "memory" || raw === "claudemd" || raw === "effective") return raw;
  return "binding";
}

let _selectedProjectFacet = $state<ProjectFacetKey>(loadFacet());

export const modeStore = {
  // ...existing fields
  get selectedProjectFacet() { return _selectedProjectFacet; },
  setSelectedProjectFacet(facet: ProjectFacetKey) {
    _selectedProjectFacet = facet;
    localStorage.setItem(FACET_STORAGE_KEY, facet);
  },
};
```

- [ ] **Step 2: Extend projects store with binding helpers**

In `src/lib/stores/projects.svelte.ts`, add derived getters next to existing `selected`:

```ts
  get currentBinding(): ProjectEntry | null {
    return _entries.find((e) => e.path === _selectedPath) ?? null;
  },
  get currentBound(): boolean {
    return this.currentBinding?.account != null && this.currentBinding.account !== "";
  },
  get currentStale(): boolean {
    return this.currentBinding?.stale === true;
  },
  get currentAccount(): string | null {
    return this.currentBinding?.account ?? null;
  },
  get currentLaunch(): ProjectLaunch | null {
    return this.currentBinding?.launch ?? null;
  },
```

If `ProjectLaunch` type isn't yet imported in this store, add to `api/types.ts` if missing and import.

- [ ] **Step 3: Verify type-check**

```bash
pnpm tsc --noEmit
```

- [ ] **Step 4: Commit**

```bash
git add src/lib/stores/mode.svelte.ts src/lib/stores/projects.svelte.ts src/lib/api/types.ts
git commit -m "feat(stage3): mode store remembers selected project facet; projects store binding helpers"
```

---

### Task 10: `ProjectModeView` shell + tab strip + banners

**Files:**
- Create: `src/lib/components/project-mode/ProjectModeView.svelte`
- Create: `src/lib/components/project-mode/StalePathBanner.svelte`
- Create: `src/lib/components/project-mode/UnboundHint.svelte`
- Modify: `src/App.svelte`

- [ ] **Step 1: Create `StalePathBanner.svelte`**

```svelte
<script lang="ts">
  import { t } from "../../i18n";
  import { projectsStore } from "../../stores/projects.svelte";
  let { path }: { path: string } = $props();
  async function onRemove() {
    if (!confirm(t("projectMode.staleConfirmRemove"))) return;
    await projectsStore.remove(path);
  }
</script>

<div class="banner stale" role="alert">
  <span>{t("projectMode.staleBanner", { path })}</span>
  <button onclick={onRemove}>{t("projectMode.staleRemoveBtn")}</button>
</div>

<style>
  .banner.stale {
    background: var(--bg-warn, #fde2e2);
    color: var(--text-warn, #8a1f1f);
    padding: 8px 12px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
  }
</style>
```

- [ ] **Step 2: Create `UnboundHint.svelte`**

```svelte
<script lang="ts">
  import { t } from "../../i18n";
  import { modeStore } from "../../stores/mode.svelte";
</script>

<div class="empty">
  <p>{t("projectMode.unboundHint")}</p>
  <button onclick={() => modeStore.setSelectedProjectFacet("binding")}>
    {t("projectMode.goToBinding")}
  </button>
</div>

<style>
  .empty {
    padding: 32px;
    text-align: center;
    color: var(--text-muted);
  }
</style>
```

- [ ] **Step 3: Create `ProjectModeView.svelte`**

```svelte
<script lang="ts">
  import { t } from "../../i18n";
  import { projectsStore } from "../../stores/projects.svelte";
  import { modeStore, type ProjectFacetKey } from "../../stores/mode.svelte";
  import BindingFacet from "./BindingFacet.svelte";
  import LaunchFacet from "./LaunchFacet.svelte";
  import PluginsOverrideFacet from "./PluginsOverrideFacet.svelte";
  import ProjectSettingsFacet from "./ProjectSettingsFacet.svelte";
  import ProjectMemoryFacet from "./ProjectMemoryFacet.svelte";
  import ProjectClaudeMdFacet from "./ProjectClaudeMdFacet.svelte";
  import EffectiveFacet from "./EffectiveFacet.svelte";
  import StalePathBanner from "./StalePathBanner.svelte";
  import UnboundHint from "./UnboundHint.svelte";

  const FACETS: { key: ProjectFacetKey; labelKey: string }[] = [
    { key: "binding",   labelKey: "projectMode.facet.binding" },
    { key: "launch",    labelKey: "projectMode.facet.launch" },
    { key: "plugins",   labelKey: "projectMode.facet.plugins" },
    { key: "settings",  labelKey: "projectMode.facet.settings" },
    { key: "memory",    labelKey: "projectMode.facet.memory" },
    { key: "claudemd",  labelKey: "projectMode.facet.claudemd" },
    { key: "effective", labelKey: "projectMode.facet.effective" },
  ];

  const selected = $derived(projectsStore.currentBinding);
  const isStale = $derived(projectsStore.currentStale);
  const isBound = $derived(projectsStore.currentBound);
  const activeFacet = $derived(modeStore.selectedProjectFacet);

  function tabDisabled(key: ProjectFacetKey): boolean {
    if (isStale) return key !== "binding" ? true : true; // all disabled when stale
    if (!isBound && key !== "binding") return true;
    return false;
  }
</script>

{#if !selected}
  <div class="empty">{t("projectMode.selectProject")}</div>
{:else}
  <div class="project-mode">
    {#if isStale}
      <StalePathBanner path={selected.path} />
    {/if}
    <nav class="tabs" role="tablist">
      {#each FACETS as f (f.key)}
        <button
          role="tab"
          aria-selected={activeFacet === f.key}
          disabled={tabDisabled(f.key)}
          class:active={activeFacet === f.key}
          onclick={() => modeStore.setSelectedProjectFacet(f.key)}
        >{t(f.labelKey)}</button>
      {/each}
    </nav>
    <div class="facet">
      {#if isStale}
        <div class="empty">{t("projectMode.stalePathBlocked")}</div>
      {:else if !isBound && activeFacet !== "binding"}
        <UnboundHint />
      {:else if activeFacet === "binding"}
        <BindingFacet path={selected.path} />
      {:else if activeFacet === "launch"}
        <LaunchFacet path={selected.path} />
      {:else if activeFacet === "plugins"}
        <PluginsOverrideFacet path={selected.path} />
      {:else if activeFacet === "settings"}
        <ProjectSettingsFacet path={selected.path} />
      {:else if activeFacet === "memory"}
        <ProjectMemoryFacet path={selected.path} />
      {:else if activeFacet === "claudemd"}
        <ProjectClaudeMdFacet path={selected.path} />
      {:else if activeFacet === "effective"}
        <EffectiveFacet path={selected.path} />
      {/if}
    </div>
  </div>
{/if}

<style>
  .project-mode { display: flex; flex-direction: column; height: 100%; }
  .tabs { display: flex; gap: 4px; padding: 8px; border-bottom: 1px solid var(--border); overflow-x: auto; }
  .tabs button { padding: 4px 10px; background: transparent; border: 1px solid transparent; border-radius: 4px; }
  .tabs button.active { background: var(--bg-tab-active); border-color: var(--border); }
  .tabs button[disabled] { opacity: 0.4; cursor: not-allowed; }
  .facet { flex: 1; overflow: auto; }
  .empty { padding: 32px; text-align: center; color: var(--text-muted); }
</style>
```

Note: per CLAUDE.md Svelte-5 gotcha #4, prefer direct comparisons in `{#if}` chains.

- [ ] **Step 4: Wire into `App.svelte`**

Replace the import of `ProjectModePlaceholder` with `ProjectModeView`, and the mount point:

```svelte
{:else if modeStore.mode === "project"}
  <ProjectModeView />
```

Delete the `import ProjectModePlaceholder from ...` and the placeholder mount.

- [ ] **Step 5: Add the i18n keys to all 3 locales**

`src/lib/i18n/locales/locales/zh-CN.json` (add under root):

```json
"projectMode": {
  "selectProject": "请在左侧选择一个项目",
  "stalePathBlocked": "项目路径不存在,先在 Binding 选择 Remove",
  "staleBanner": "路径 {path} 不存在",
  "staleConfirmRemove": "确认从已知项目移除?",
  "staleRemoveBtn": "移除",
  "unboundHint": "项目尚未绑定账号",
  "goToBinding": "前往 Binding 绑定账号",
  "facet": {
    "binding":   "Binding",
    "launch":    "Launch",
    "plugins":   "Plugins ↓",
    "settings":  "Settings",
    "memory":    "Memory",
    "claudemd":  "CLAUDE.md",
    "effective": "Effective"
  }
}
```

`en-US.json`:

```json
"projectMode": {
  "selectProject": "Select a project in the sidebar",
  "stalePathBlocked": "Project path missing — use Binding > Remove",
  "staleBanner": "Path not found: {path}",
  "staleConfirmRemove": "Remove from known projects?",
  "staleRemoveBtn": "Remove",
  "unboundHint": "Project is not bound to an account",
  "goToBinding": "Bind an account",
  "facet": {
    "binding":   "Binding",
    "launch":    "Launch",
    "plugins":   "Plugins ↓",
    "settings":  "Settings",
    "memory":    "Memory",
    "claudemd":  "CLAUDE.md",
    "effective": "Effective"
  }
}
```

`ja-JP.json` (translation can be in English fallback if unfamiliar — runtime falls back to en-US):

```json
"projectMode": {
  "selectProject": "サイドバーからプロジェクトを選択してください",
  "stalePathBlocked": "プロジェクトパスが見つかりません",
  "staleBanner": "パスが見つかりません: {path}",
  "staleConfirmRemove": "既知プロジェクトから削除しますか?",
  "staleRemoveBtn": "削除",
  "unboundHint": "プロジェクトはアカウントにバインドされていません",
  "goToBinding": "Binding でアカウントを選択",
  "facet": {
    "binding":   "Binding",
    "launch":    "Launch",
    "plugins":   "Plugins ↓",
    "settings":  "Settings",
    "memory":    "Memory",
    "claudemd":  "CLAUDE.md",
    "effective": "Effective"
  }
}
```

- [ ] **Step 6: Verify**

```bash
pnpm tsc --noEmit && pnpm build
```

Expected: builds OK. UI compiles but every facet child component is missing — pnpm will fail on the imports. **That's expected**; the next 7 tasks create them. To unblock the build now, **stub each facet** as a one-line component:

```svelte
<!-- BindingFacet.svelte (and same for each other facet) -->
<script lang="ts">
  let { path }: { path: string } = $props();
</script>
<div class="stub">TODO {path}</div>
```

Create 7 such stubs under `src/lib/components/project-mode/`.

- [ ] **Step 7: Commit**

```bash
git add src/lib/components/project-mode/ src/App.svelte src/lib/i18n/locales/
git commit -m "feat(stage3): ProjectModeView shell + tab strip + stubs"
```

---

### Task 11: `BindingFacet` — account picker + actions

**Files:**
- Modify: `src/lib/components/project-mode/BindingFacet.svelte`

- [ ] **Step 1: Replace stub with full implementation**

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "../../i18n";
  import { projectsStore } from "../../stores/projects.svelte";
  import { accountsStore } from "../../stores/accounts.svelte";
  import { ipcClient } from "../../ipc/client";
  import { toastStore } from "../../stores/toast.svelte";

  let { path }: { path: string } = $props();

  const binding = $derived(projectsStore.entries.find((e) => e.path === path));
  let selectedAccount = $state<string>("");

  $effect(() => {
    selectedAccount = binding?.account ?? "";
  });

  onMount(async () => {
    if (accountsStore.entries.length === 0) await accountsStore.reload();
  });

  async function onBind() {
    if (!selectedAccount) return;
    await projectsStore.bind(path, selectedAccount);
    toastStore.show(t("projectMode.binding.bound", { account: selectedAccount }));
  }

  async function onUnbind() {
    if (!confirm(t("projectMode.binding.confirmUnbind"))) return;
    await projectsStore.unbind(path);
  }

  async function onRemove() {
    if (!confirm(t("projectMode.binding.confirmRemove"))) return;
    await projectsStore.remove(path);
  }

  async function onOpenTerminal() {
    try {
      await ipcClient.launchClaude({
        projectPath: path,
        account: binding?.account ?? "default",
      });
    } catch (e) {
      toastStore.show(String(e), "error");
    }
  }
</script>

<section class="binding-facet">
  <h2>{t("projectMode.binding.title")}</h2>

  <dl>
    <dt>{t("projectMode.binding.pathLabel")}</dt>
    <dd><code>{path}</code></dd>

    <dt>{t("projectMode.binding.accountLabel")}</dt>
    <dd>
      <select bind:value={selectedAccount}>
        <option value="">{t("projectMode.binding.selectAccount")}</option>
        {#each accountsStore.entries as a (a.name)}
          <option value={a.name}>{a.displayName} ({a.name})</option>
        {/each}
      </select>
      <button onclick={onBind} disabled={!selectedAccount || selectedAccount === binding?.account}>
        {t("projectMode.binding.bindBtn")}
      </button>
    </dd>
  </dl>

  <div class="actions">
    <button onclick={onOpenTerminal} disabled={!binding?.account}>
      {t("projectMode.binding.openTerminal")}
    </button>
    <button onclick={onUnbind} disabled={!binding?.account}>
      {t("projectMode.binding.unbindBtn")}
    </button>
    <button onclick={onRemove} class="danger">
      {t("projectMode.binding.removeBtn")}
    </button>
  </div>
</section>

<style>
  .binding-facet { padding: 16px; }
  dl { display: grid; grid-template-columns: max-content 1fr; gap: 8px 16px; }
  .actions { margin-top: 24px; display: flex; gap: 8px; }
  .danger { color: var(--danger); }
</style>
```

- [ ] **Step 2: Add i18n keys**

Under `projectMode` in each locale add a nested `binding` block (zh-CN example):

```json
"binding": {
  "title": "账号绑定",
  "pathLabel": "项目路径",
  "accountLabel": "绑定账号",
  "selectAccount": "选择账号...",
  "bindBtn": "绑定",
  "openTerminal": "在终端打开",
  "unbindBtn": "取消绑定",
  "removeBtn": "从列表移除",
  "bound": "已绑定到 {account}",
  "confirmUnbind": "确认取消绑定?",
  "confirmRemove": "确认从已知项目移除?"
}
```

Mirror in en-US and ja-JP.

- [ ] **Step 3: Verify**

```bash
pnpm tsc --noEmit
```

Manual: `pnpm tauri dev`, switch to Project mode, select a project, verify Bind/Unbind/Open Terminal/Remove all work end-to-end.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/project-mode/BindingFacet.svelte src/lib/i18n/locales/
git commit -m "feat(stage3): BindingFacet — account picker + bind/unbind/remove/open-terminal"
```

---

### Task 12: `LaunchFacet` — env editor + args + Launch button

**Files:**
- Modify: `src/lib/components/project-mode/LaunchFacet.svelte`

- [ ] **Step 1: Implement**

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "../../i18n";
  import { projectsStore } from "../../stores/projects.svelte";
  import { ipcClient } from "../../ipc/client";
  import { toastStore } from "../../stores/toast.svelte";
  import { parseClaudeHelp, type ClaudeArgSpec } from "../../utils/parseClaudeHelp";

  let { path }: { path: string } = $props();

  const binding = $derived(projectsStore.entries.find((e) => e.path === path));
  const account = $derived(binding?.account ?? "default");

  let envEntries = $state<Array<{ k: string; v: string }>>([]);
  let argEntries = $state<string[]>([]);
  let argSuggestions = $state<ClaudeArgSpec[]>([]);
  let dirty = $state(false);
  let saving = $state(false);

  $effect(() => {
    const l = binding?.launch;
    envEntries = Object.entries(l?.env ?? {}).map(([k, v]) => ({ k, v }));
    argEntries = [...(l?.args ?? [])];
    dirty = false;
  });

  onMount(async () => {
    try {
      const raw = await ipcClient.getClaudeArgs();
      argSuggestions = parseClaudeHelp(raw);
    } catch {
      argSuggestions = [];
    }
  });

  function addEnv() { envEntries = [...envEntries, { k: "", v: "" }]; dirty = true; }
  function removeEnv(i: number) { envEntries = envEntries.filter((_, j) => j !== i); dirty = true; }
  function addArg(name?: string) {
    argEntries = [...argEntries, name ?? ""];
    dirty = true;
  }
  function removeArg(i: number) { argEntries = argEntries.filter((_, j) => j !== i); dirty = true; }

  async function save() {
    saving = true;
    try {
      const envObj: Record<string, string> = {};
      for (const { k, v } of envEntries) if (k) envObj[k] = v;
      await projectsStore.updateLaunch(path, { env: envObj, args: argEntries.filter(Boolean) });
      dirty = false;
      toastStore.show(t("projectMode.launch.saved"));
    } finally {
      saving = false;
    }
  }

  async function launch() {
    if (dirty) await save();
    const envObj: Record<string, string> = {};
    for (const { k, v } of envEntries) if (k) envObj[k] = v;
    await ipcClient.launchClaude({
      projectPath: path,
      env: envObj,
      args: argEntries.filter(Boolean),
      account,
    });
  }
</script>

<section class="launch-facet">
  <h2>{t("projectMode.launch.title")}</h2>
  <p class="hint">{t("projectMode.launch.account", { account })}</p>

  <h3>{t("projectMode.launch.envTitle")}</h3>
  <table>
    {#each envEntries as e, i (i)}
      <tr>
        <td><input bind:value={e.k} placeholder="KEY" oninput={() => (dirty = true)} /></td>
        <td><input bind:value={e.v} placeholder="value" oninput={() => (dirty = true)} /></td>
        <td><button onclick={() => removeEnv(i)}>×</button></td>
      </tr>
    {/each}
  </table>
  <button onclick={addEnv}>{t("projectMode.launch.addEnv")}</button>

  <h3>{t("projectMode.launch.argsTitle")}</h3>
  <ul>
    {#each argEntries as a, i (i)}
      <li>
        <input bind:value={a} list="claude-args" oninput={() => (dirty = true)} />
        <button onclick={() => removeArg(i)}>×</button>
      </li>
    {/each}
  </ul>
  <datalist id="claude-args">
    {#each argSuggestions as s (s.flag)}
      <option value={s.flag}>{s.description}</option>
    {/each}
  </datalist>
  <button onclick={() => addArg()}>{t("projectMode.launch.addArg")}</button>

  <div class="actions">
    <button onclick={save} disabled={!dirty || saving}>
      {dirty ? t("projectMode.launch.save") : t("projectMode.launch.saved")}
    </button>
    <button onclick={launch} class="primary">{t("projectMode.launch.launchBtn")}</button>
  </div>
</section>

<style>
  .launch-facet { padding: 16px; }
  .hint { color: var(--text-muted); font-size: 0.9em; }
  table { width: 100%; margin: 8px 0; }
  table td input { width: 100%; padding: 4px; }
  ul { list-style: none; padding: 0; }
  ul li { display: flex; gap: 8px; margin: 4px 0; }
  ul li input { flex: 1; }
  .actions { margin-top: 16px; display: flex; gap: 8px; }
  .actions .primary { background: var(--accent); color: white; }
</style>
```

- [ ] **Step 2: Add i18n keys**

zh-CN under `projectMode`:

```json
"launch": {
  "title": "启动配置",
  "account": "将使用账号: {account}",
  "envTitle": "环境变量",
  "argsTitle": "启动参数",
  "addEnv": "+ 添加环境变量",
  "addArg": "+ 添加参数",
  "save": "保存",
  "saved": "已保存",
  "launchBtn": "启动 Claude Code"
}
```

Mirror to en-US, ja-JP.

- [ ] **Step 3: Verify**

```bash
pnpm tsc --noEmit
```

Manual: open a bound project's Launch facet, add env `FOO=bar`, click Launch, verify terminal opens with project cwd and the env applied (visible via `echo $FOO` inside the spawned shell).

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/project-mode/LaunchFacet.svelte src/lib/i18n/locales/
git commit -m "feat(stage3): LaunchFacet — env/args editor + Launch button"
```

---

### Task 13: `PluginsOverrideFacet` — tri-state override

**Files:**
- Modify: `src/lib/components/project-mode/PluginsOverrideFacet.svelte`

- [ ] **Step 1: Implement**

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "../../i18n";
  import { ipcClient } from "../../ipc/client";
  import { toastStore } from "../../stores/toast.svelte";
  import type { PluginInfo, Settings } from "../../api/types";

  let { path }: { path: string } = $props();

  type Tri = "inherit" | "enable" | "disable";

  let plugins = $state<PluginInfo[]>([]);
  let projectSettings = $state<Settings | null>(null);
  let loading = $state(true);
  let saving = $state(false);

  async function load() {
    loading = true;
    try {
      const [list, settingsResp] = await Promise.all([
        ipcClient.projectListPlugins(path),
        ipcClient.projectReadSettings(path),
      ]);
      plugins = list;
      projectSettings = settingsResp.settings;
    } finally {
      loading = false;
    }
  }

  $effect(() => { void path; load(); });

  function stateOf(name: string): Tri {
    const map = projectSettings?.enabledPlugins;
    if (!map || !(name in map)) return "inherit";
    return map[name] ? "enable" : "disable";
  }

  async function setState(name: string, next: Tri) {
    if (!projectSettings) return;
    saving = true;
    try {
      const cur: Record<string, boolean> = { ...(projectSettings.enabledPlugins ?? {}) };
      if (next === "inherit") delete cur[name];
      else cur[name] = next === "enable";

      const updated: Settings = {
        ...projectSettings,
        enabledPlugins: Object.keys(cur).length ? cur : undefined,
      };
      await ipcClient.projectWriteSettings(path, updated);
      projectSettings = updated;
      toastStore.show(t("projectMode.plugins.saved"));
    } catch (e) {
      toastStore.show(String(e), "error");
    } finally {
      saving = false;
    }
  }
</script>

<section class="plugins-facet">
  <h2>{t("projectMode.plugins.title")}</h2>
  <p class="hint">{t("projectMode.plugins.hint")}</p>

  {#if loading}
    <div class="empty">{t("common.loading")}</div>
  {:else if plugins.length === 0}
    <div class="empty">{t("projectMode.plugins.noPluginsAccount")}</div>
  {:else}
    <table>
      <thead>
        <tr>
          <th>{t("projectMode.plugins.name")}</th>
          <th>{t("projectMode.plugins.version")}</th>
          <th>{t("projectMode.plugins.override")}</th>
        </tr>
      </thead>
      <tbody>
        {#each plugins as p (p.name)}
          {@const st = stateOf(p.name)}
          <tr>
            <td>{p.name}</td>
            <td>{p.version}</td>
            <td>
              <div class="tri" role="radiogroup">
                <button
                  class:active={st === "disable"}
                  disabled={saving}
                  onclick={() => setState(p.name, "disable")}
                >{t("projectMode.plugins.disable")}</button>
                <button
                  class:active={st === "inherit"}
                  disabled={saving}
                  onclick={() => setState(p.name, "inherit")}
                >{t("projectMode.plugins.inherit")}</button>
                <button
                  class:active={st === "enable"}
                  disabled={saving}
                  onclick={() => setState(p.name, "enable")}
                >{t("projectMode.plugins.enable")}</button>
              </div>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</section>

<style>
  .plugins-facet { padding: 16px; }
  .hint { color: var(--text-muted); font-size: 0.9em; }
  table { width: 100%; border-collapse: collapse; margin-top: 12px; }
  th, td { padding: 6px 8px; border-bottom: 1px solid var(--border); text-align: left; }
  .tri { display: inline-flex; gap: 0; border: 1px solid var(--border); border-radius: 4px; overflow: hidden; }
  .tri button { background: transparent; border: none; padding: 4px 10px; border-right: 1px solid var(--border); }
  .tri button:last-child { border-right: none; }
  .tri button.active { background: var(--accent); color: white; }
  .empty { padding: 32px; text-align: center; color: var(--text-muted); }
</style>
```

- [ ] **Step 2: i18n keys**

```json
"plugins": {
  "title": "Plugins 覆写",
  "hint": "未选择 = 继承账号设置;Disable / Enable 写入项目 settings.json",
  "name": "插件",
  "version": "版本",
  "override": "覆写状态",
  "disable": "Disable",
  "inherit": "Inherit",
  "enable": "Enable",
  "noPluginsAccount": "绑定账号下没有已安装的插件",
  "saved": "已保存"
}
```

Mirror to en-US, ja-JP.

- [ ] **Step 3: Verify**

```bash
pnpm tsc --noEmit
```

Manual:
1. Install a plugin in Account > Plugins (work account).
2. Switch to Project mode, select a project bound to work.
3. Plugins ↓ facet shows the plugin.
4. Click Disable → check that `<project>/.claude/settings.json` now contains `"enabledPlugins": { "<name>": false }`.
5. Click Inherit → key removed; if `enabledPlugins` becomes empty, the field is absent.
6. Switch to Effective facet (next task) → confirm the Disable shows up.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/project-mode/PluginsOverrideFacet.svelte src/lib/i18n/locales/
git commit -m "feat(stage3): PluginsOverrideFacet — tri-state plugin override"
```

---

### Task 14: `ProjectSettingsFacet` — JSON editor

**Files:**
- Modify: `src/lib/components/project-mode/ProjectSettingsFacet.svelte`

Rationale: spec says project-layer settings allow all keys (no whitelist). Reusing the sectioned Account `SettingsEditor` would require a non-trivial refactor; a JSON editor is honest, low-risk, and shippable in Stage 3. Sectioned UI parity is a Stage-4 backlog item.

- [ ] **Step 1: Implement**

```svelte
<script lang="ts">
  import { t } from "../../i18n";
  import { ipcClient } from "../../ipc/client";
  import { toastStore } from "../../stores/toast.svelte";

  let { path }: { path: string } = $props();

  let raw = $state("");
  let original = $state("");
  let dirty = $state(false);
  let error = $state<string | null>(null);
  let loading = $state(true);
  let saving = $state(false);

  async function load() {
    loading = true;
    try {
      const resp = await ipcClient.projectReadSettings(path);
      raw = resp.exists ? JSON.stringify(resp.settings, null, 2) : "{}";
      original = raw;
      dirty = false;
      error = null;
    } finally {
      loading = false;
    }
  }

  $effect(() => { void path; load(); });

  function onInput() { dirty = raw !== original; }

  function validate(): boolean {
    try {
      JSON.parse(raw);
      error = null;
      return true;
    } catch (e) {
      error = (e as Error).message;
      return false;
    }
  }

  async function save() {
    if (!validate()) return;
    saving = true;
    try {
      const obj = JSON.parse(raw);
      await ipcClient.projectWriteSettings(path, obj);
      original = raw;
      dirty = false;
      toastStore.show(t("projectMode.settings.saved"));
    } catch (e) {
      toastStore.show(String(e), "error");
    } finally {
      saving = false;
    }
  }

  function revert() { raw = original; dirty = false; error = null; }
</script>

<section class="settings-facet">
  <h2>{t("projectMode.settings.title")}</h2>
  <p class="hint">{t("projectMode.settings.hint", { path: `${path}/.claude/settings.json` })}</p>

  {#if loading}
    <div class="empty">{t("common.loading")}</div>
  {:else}
    <textarea bind:value={raw} oninput={onInput} onblur={validate} spellcheck="false"></textarea>
    {#if error}
      <p class="err">{error}</p>
    {/if}
    <div class="actions">
      <button onclick={save} disabled={!dirty || saving || error !== null}>
        {t("projectMode.settings.save")}
      </button>
      <button onclick={revert} disabled={!dirty}>
        {t("projectMode.settings.revert")}
      </button>
    </div>
  {/if}
</section>

<style>
  .settings-facet { padding: 16px; height: 100%; display: flex; flex-direction: column; }
  .hint { color: var(--text-muted); font-size: 0.85em; }
  textarea {
    flex: 1; min-height: 300px; font-family: ui-monospace, Menlo, monospace;
    font-size: 13px; padding: 8px; border: 1px solid var(--border); border-radius: 4px;
    background: var(--bg-input); color: var(--text-primary);
  }
  .err { color: var(--danger); margin-top: 4px; font-family: monospace; font-size: 0.85em; }
  .actions { margin-top: 8px; display: flex; gap: 8px; }
</style>
```

- [ ] **Step 2: i18n keys**

```json
"settings": {
  "title": "项目层 settings.json",
  "hint": "覆写账号 settings 的字段:{path}",
  "save": "保存",
  "revert": "撤销修改",
  "saved": "已保存"
}
```

Mirror.

- [ ] **Step 3: Verify**

```bash
pnpm tsc --noEmit
```

Manual: add `"env": {"FOO": "bar"}`, save, reopen — content persists. Make invalid JSON → save button disabled, error message visible.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/project-mode/ProjectSettingsFacet.svelte src/lib/i18n/locales/
git commit -m "feat(stage3): ProjectSettingsFacet — JSON editor for project-layer settings"
```

---

### Task 15: `ProjectClaudeMdFacet` — markdown textarea

**Files:**
- Modify: `src/lib/components/project-mode/ProjectClaudeMdFacet.svelte`

- [ ] **Step 1: Implement**

```svelte
<script lang="ts">
  import { t } from "../../i18n";
  import { ipcClient } from "../../ipc/client";
  import { toastStore } from "../../stores/toast.svelte";

  let { path }: { path: string } = $props();

  let content = $state("");
  let original = $state("");
  let loading = $state(true);
  let saving = $state(false);
  let dirty = $derived(content !== original);

  async function load() {
    loading = true;
    try {
      const resp = await ipcClient.projectReadClaudeMd(path);
      content = resp.content;
      original = content;
    } finally {
      loading = false;
    }
  }

  $effect(() => { void path; load(); });

  async function save() {
    saving = true;
    try {
      await ipcClient.projectWriteClaudeMd(path, content);
      original = content;
      toastStore.show(t("projectMode.claudemd.saved"));
    } finally {
      saving = false;
    }
  }
</script>

<section class="claudemd-facet">
  <h2>{t("projectMode.claudemd.title")}</h2>
  <p class="hint">{t("projectMode.claudemd.hint", { path: `${path}/.claude/CLAUDE.md` })}</p>

  {#if loading}
    <div class="empty">{t("common.loading")}</div>
  {:else}
    <textarea bind:value={content} spellcheck="false"></textarea>
    <div class="actions">
      <button onclick={save} disabled={!dirty || saving}>
        {t("projectMode.claudemd.save")}
      </button>
    </div>
  {/if}
</section>

<style>
  .claudemd-facet { padding: 16px; height: 100%; display: flex; flex-direction: column; }
  .hint { color: var(--text-muted); font-size: 0.85em; }
  textarea {
    flex: 1; min-height: 300px; font-family: ui-monospace, Menlo, monospace;
    font-size: 13px; padding: 8px; border: 1px solid var(--border); border-radius: 4px;
    background: var(--bg-input); color: var(--text-primary);
  }
  .actions { margin-top: 8px; }
</style>
```

- [ ] **Step 2: i18n keys**

```json
"claudemd": {
  "title": "项目 CLAUDE.md",
  "hint": "项目专属 Claude 上下文:{path}",
  "save": "保存",
  "saved": "已保存"
}
```

Mirror.

- [ ] **Step 3: Verify**

```bash
pnpm tsc --noEmit
```

Manual: edit, save, file appears at `<path>/.claude/CLAUDE.md`.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/project-mode/ProjectClaudeMdFacet.svelte src/lib/i18n/locales/
git commit -m "feat(stage3): ProjectClaudeMdFacet — edit project CLAUDE.md"
```

---

### Task 16: `ProjectMemoryFacet` — file list + viewer/editor

**Files:**
- Modify: `src/lib/components/project-mode/ProjectMemoryFacet.svelte`

Pattern mirrors `Account>Memory` but reads from `<bound-account>/projects/<encoded-path>/memory/`. Two-pane: file list left, viewer/editor right.

- [ ] **Step 1: Implement**

```svelte
<script lang="ts">
  import { t } from "../../i18n";
  import { ipcClient } from "../../ipc/client";
  import { toastStore } from "../../stores/toast.svelte";
  import type { MemoryFileEntry } from "../../api/types";

  let { path }: { path: string } = $props();

  let files = $state<MemoryFileEntry[]>([]);
  let memoryDir = $state("");
  let selected = $state<string | null>(null);
  let content = $state("");
  let original = $state("");
  let loadingList = $state(true);
  let loadingFile = $state(false);
  let saving = $state(false);
  let dirty = $derived(selected !== null && content !== original);

  async function loadList() {
    loadingList = true;
    try {
      const resp = await ipcClient.projectListMemory(path);
      files = resp.files;
      memoryDir = resp.path;
      if (selected && !files.some((f) => f.name === selected)) selected = null;
    } finally {
      loadingList = false;
    }
  }

  $effect(() => { void path; loadList(); });

  async function openFile(name: string) {
    if (dirty && !confirm(t("projectMode.memory.discardUnsaved"))) return;
    selected = name;
    loadingFile = true;
    try {
      content = await ipcClient.projectReadMemoryFile(path, name);
      original = content;
    } finally {
      loadingFile = false;
    }
  }

  async function save() {
    if (!selected) return;
    saving = true;
    try {
      await ipcClient.projectWriteMemoryFile(path, selected, content);
      original = content;
      toastStore.show(t("projectMode.memory.saved"));
      await loadList();
    } finally {
      saving = false;
    }
  }

  async function deleteFile(name: string) {
    if (!confirm(t("projectMode.memory.confirmDelete", { name }))) return;
    await ipcClient.projectDeleteMemoryFile(path, name);
    if (selected === name) { selected = null; content = ""; original = ""; }
    await loadList();
  }

  async function newFile() {
    const name = prompt(t("projectMode.memory.newFilePrompt"));
    if (!name) return;
    await ipcClient.projectWriteMemoryFile(path, name, "");
    await loadList();
    void openFile(name);
  }
</script>

<section class="memory-facet">
  <aside class="list">
    <header>
      <h3>{t("projectMode.memory.title")}</h3>
      <button onclick={newFile}>+</button>
    </header>
    {#if loadingList}
      <div class="empty">{t("common.loading")}</div>
    {:else if files.length === 0}
      <div class="empty">{t("projectMode.memory.noFiles")}</div>
    {:else}
      <ul>
        {#each files as f (f.name)}
          <li class:active={selected === f.name}>
            <button onclick={() => openFile(f.name)}>{f.name}</button>
            <button class="del" onclick={() => deleteFile(f.name)}>×</button>
          </li>
        {/each}
      </ul>
    {/if}
    <footer class="dir">{memoryDir}</footer>
  </aside>

  <main class="viewer">
    {#if !selected}
      <div class="empty">{t("projectMode.memory.selectFile")}</div>
    {:else if loadingFile}
      <div class="empty">{t("common.loading")}</div>
    {:else}
      <textarea bind:value={content} spellcheck="false"></textarea>
      <div class="actions">
        <button onclick={save} disabled={!dirty || saving}>{t("projectMode.memory.save")}</button>
      </div>
    {/if}
  </main>
</section>

<style>
  .memory-facet { display: flex; height: 100%; }
  .list { width: 240px; border-right: 1px solid var(--border); display: flex; flex-direction: column; }
  .list header { display: flex; justify-content: space-between; align-items: center; padding: 8px 12px; }
  .list ul { list-style: none; padding: 0; margin: 0; flex: 1; overflow: auto; }
  .list li { display: flex; align-items: center; padding: 4px 12px; }
  .list li.active { background: var(--bg-list-active); }
  .list li button:first-child { flex: 1; text-align: left; background: transparent; border: none; }
  .list .del { opacity: 0; background: transparent; border: none; }
  .list li:hover .del { opacity: 1; }
  .dir { padding: 8px 12px; font-size: 0.75em; color: var(--text-muted); border-top: 1px solid var(--border); word-break: break-all; }
  .viewer { flex: 1; display: flex; flex-direction: column; padding: 16px; }
  textarea {
    flex: 1; min-height: 300px; font-family: ui-monospace, Menlo, monospace;
    font-size: 13px; padding: 8px; border: 1px solid var(--border); border-radius: 4px;
    background: var(--bg-input); color: var(--text-primary);
  }
  .actions { margin-top: 8px; }
  .empty { padding: 32px; text-align: center; color: var(--text-muted); }
</style>
```

- [ ] **Step 2: i18n keys**

```json
"memory": {
  "title": "项目记忆",
  "noFiles": "没有记忆文件",
  "selectFile": "选择一个记忆文件",
  "save": "保存",
  "saved": "已保存",
  "discardUnsaved": "放弃未保存的修改?",
  "confirmDelete": "删除 {name}?",
  "newFilePrompt": "新文件名 (含 .md)"
}
```

Mirror.

- [ ] **Step 3: Verify**

```bash
pnpm tsc --noEmit
```

Manual: create new memory file, save, observe file at `~/.dot-claude-gui/accounts/<account>/projects/<encoded-path>/memory/<name>`. Open in Account>Memory under same account, same file appears.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/project-mode/ProjectMemoryFacet.svelte src/lib/i18n/locales/
git commit -m "feat(stage3): ProjectMemoryFacet — list/edit memory under bound account"
```

---

### Task 17: `EffectiveFacet` — read-only merged view

**Files:**
- Modify: `src/lib/components/project-mode/EffectiveFacet.svelte`

- [ ] **Step 1: Implement**

```svelte
<script lang="ts">
  import { t } from "../../i18n";
  import { ipcClient } from "../../ipc/client";
  import type { ProjectEffectiveResponse } from "../../api/types";

  let { path }: { path: string } = $props();

  let resp = $state<ProjectEffectiveResponse | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  async function load() {
    loading = true; error = null;
    try {
      resp = await ipcClient.projectReadEffective(path);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  $effect(() => { void path; load(); });

  function sourceBadge(field: string): string {
    return resp?.fieldSources[field] ?? "user";
  }
</script>

<section class="effective-facet">
  <header>
    <h2>{t("projectMode.effective.title")}</h2>
    <button onclick={load}>{t("common.refresh")}</button>
  </header>

  {#if loading}
    <div class="empty">{t("common.loading")}</div>
  {:else if error}
    <div class="err">{error}</div>
  {:else if resp}
    <p class="hint">{t("projectMode.effective.account", { account: resp.account })}</p>
    <pre class="json">{JSON.stringify(resp.settings, null, 2)}</pre>
    <h3>{t("projectMode.effective.sourcesTitle")}</h3>
    <table>
      <thead><tr><th>{t("projectMode.effective.field")}</th><th>{t("projectMode.effective.source")}</th></tr></thead>
      <tbody>
        {#each Object.keys(resp.fieldSources) as f (f)}
          <tr>
            <td><code>{f}</code></td>
            <td><span class="badge {sourceBadge(f)}">{sourceBadge(f)}</span></td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</section>

<style>
  .effective-facet { padding: 16px; }
  header { display: flex; justify-content: space-between; align-items: center; }
  .hint { color: var(--text-muted); font-size: 0.9em; }
  .json {
    font-family: ui-monospace, Menlo, monospace; font-size: 12px;
    background: var(--bg-input); padding: 12px; border-radius: 4px; max-height: 400px; overflow: auto;
  }
  .err { color: var(--danger); }
  table { width: 100%; border-collapse: collapse; margin-top: 12px; }
  th, td { padding: 4px 8px; border-bottom: 1px solid var(--border); text-align: left; }
  .badge { padding: 2px 6px; border-radius: 3px; font-size: 0.85em; }
  .badge.user { background: #dbeafe; color: #1e3a8a; }
  .badge.project { background: #fef3c7; color: #92400e; }
  .badge.local { background: #fee2e2; color: #991b1b; }
</style>
```

- [ ] **Step 2: i18n keys**

```json
"effective": {
  "title": "Effective 配置 (合并视图)",
  "account": "基于账号:{account}",
  "sourcesTitle": "字段来源",
  "field": "字段",
  "source": "来源"
}
```

`common.refresh` exists already; if not, add `"refresh": "刷新"`.

- [ ] **Step 3: Verify**

```bash
pnpm tsc --noEmit
```

Manual end-to-end: in Account>Plugins enable `foo`, in Project>Plugins↓ override `foo` to Disable, switch to Effective facet — `enabledPlugins.foo === false`, source badge says `project`.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/project-mode/EffectiveFacet.svelte src/lib/i18n/locales/
git commit -m "feat(stage3): EffectiveFacet — merged view with source badges"
```

---

### Task 18: ProjectSidebar polish — show stale/unbound state

**Files:**
- Modify: `src/lib/components/shell/ProjectSidebar.svelte`

The sidebar should already render the project list with grouping by account. Verify and polish:
- Stale entries are visually muted + show a `·stale` badge
- Unbound entries appear under an "Unbound" group with a faded styling
- Clicking a stale entry still selects it (so user can hit `Remove` in BindingFacet via the banner)

- [ ] **Step 1: Read the current file**

```bash
cat src/lib/components/shell/ProjectSidebar.svelte
```

- [ ] **Step 2: Verify stale + unbound rendering**

If grouping by `entry.account ?? "__unbound__"` already exists (Stage 2 survey confirms), make sure:
- Unbound group has localized label `t("projectMode.sidebar.unboundGroup")`
- Stale entries get a class `entry.stale` styled muted
- `t("projectMode.sidebar.staleLabel")` rendered as a small badge next to the project name when stale

Otherwise, add these. Sample change (adjust based on current structure):

```svelte
<span class="badge stale" class:hidden={!entry.stale}>·{t("projectMode.sidebar.staleLabel")}</span>
```

- [ ] **Step 3: i18n keys**

```json
"sidebar": {
  "unboundGroup": "未绑定",
  "staleLabel": "路径不存在"
}
```

Mirror.

- [ ] **Step 4: Verify**

```bash
pnpm tsc --noEmit
```

Manual: `mkdir /tmp/foo`, add via Add Project, bind to work, delete dir on disk, reload — entry shows stale badge.

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/shell/ProjectSidebar.svelte src/lib/i18n/locales/
git commit -m "fix(stage3): ProjectSidebar shows stale + unbound state clearly"
```

---

### Task 19: Build + manual end-to-end verification

**Files:** none (verification only)

- [ ] **Step 1: Run all backend tests**

```bash
cargo test --workspace
```

Expected: all green (Stage 2 baseline 101 + 1 ignored, Stage 3 adds ~10).

- [ ] **Step 2: Type-check + build frontend**

```bash
pnpm tsc --noEmit && pnpm build
```

Expected: clean.

- [ ] **Step 3: Manual E2E run-through**

`pnpm tauri dev` then walk through spec §Verification end-to-end (lines 252-260):

1. Open app — Project mode tab visible, sidebar lists projects grouped by account, Unbound group present if any.
2. Click an unbound project — only Binding tab clickable, all others greyed; switching to e.g. Settings shows `UnboundHint`.
3. Bind to `work` account → other tabs become clickable.
4. Plugins ↓ → toggle a plugin to Disable → file `<project>/.claude/settings.json` contains `"enabledPlugins": { "<name>": false }`.
5. Effective tab → confirms `<name>` is disabled, source `project`.
6. Launch tab → add env `FOO=bar`, click Launch → terminal opens at `cwd=<project>` with `CLAUDE_CONFIG_DIR=~/.dot-claude-gui/accounts/work` and `FOO=bar`.
7. Switch to the `default` account-bound project → Launch → no `CLAUDE_CONFIG_DIR` injected (verify by `echo $CLAUDE_CONFIG_DIR` in spawned shell — empty).
8. Memory facet → create `note.md`, save → file appears at `~/.dot-claude-gui/accounts/work/projects/<encoded>/memory/note.md`.
9. CLAUDE.md facet → edit, save → file appears at `<project>/.claude/CLAUDE.md`.
10. Delete the project dir on disk → ProjectSidebar shows stale badge; selecting it shows banner; only Remove button enabled.
11. Click Remove → entry vanishes from sidebar.

- [ ] **Step 4: Check Tauri devtools console**

`Cmd+Option+I` — no `lifecycle_outside_component`, no `each_key_duplicate`, no unhandled rejections.

- [ ] **Step 5: Commit a checkpoint marker if anything was tweaked during E2E**

```bash
git status
# if clean, no commit needed
```

---

### Task 20: Update CLAUDE.md project notes (only if a new gotcha emerged)

**Files:**
- Modify: `CLAUDE.md` (project root) — only if new gotcha emerged

During E2E, watch for new Svelte 5 / Tauri gotchas. If something bit you that future-Claude won't expect, add it to the gotchas section. Otherwise skip this task.

- [ ] **Step 1: Decide**
- [ ] **Step 2: If yes, edit + commit**

```bash
git add CLAUDE.md
git commit -m "docs: add stage3 gotcha — <one-line>"
```

---

## Self-review checklist (run before handoff)

- [ ] All 7 Project facets exist as components and are wired into `ProjectModeView`
- [ ] Each facet calls a `project_*` IPC, never `state.current_dir`-based account IPCs
- [ ] `launch_claude` injects `CLAUDE_CONFIG_DIR` for named accounts, omits it for `default`
- [ ] Unbound project: only Binding facet works (UnboundHint elsewhere)
- [ ] Stale project: all facets disabled, banner shows Remove
- [ ] Plugins ↓ writes `enabledPlugins[name] = bool` to project settings; Inherit deletes the key
- [ ] Effective facet reads merged result with source badges
- [ ] Tri-state Plugins ↓ override visible in Effective with `project` source
- [ ] i18n keys exist for zh-CN, en-US, ja-JP under `projectMode.*`
- [ ] No new `state.set_active_account` calls — Project mode never mutates active-account state
- [ ] Old UUID-keyed IPCs (`commands::config::*_project_*`, `commands::projects::*`) untouched
- [ ] Type-check + build clean
- [ ] `cargo test --workspace` clean
- [ ] Spec §Verification 1-7 manually walked

---

## Handoff

After all 20 tasks land:

- **Open: Stage 4** — delete the old UUID-keyed project IPCs, delete `commands::projects` module, drop unused `LauncherView`/`LauncherList` stubs and old `EffectiveConfigView`, complete the right-corner gear panel (Appearance / Language / Terminal / About), prune i18n strings for removed copy.
- **Tracking issues for Stage 4 backlog**:
  - Sectioned-UI parity for `ProjectSettingsFacet` (currently raw JSON)
  - `Update path` action on stale projects (currently only Remove)
  - Drop unused `state.inner.projects: Vec<ProjectInfo>` registry
