# Phase 8 Stage 4 — Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land the final cleanup pass of Phase 8 — delete dead UUID-keyed IPC commands and their TS callers, drop legacy account-scope-switching code, fix the pre-existing svelte-check baseline errors, finish the gear panel into the 4-section layout the spec requires, and add a one-shot migration toast so v1→v2 startups are explicit.

**Architecture:** Pure cleanup stage. No new architectural decisions. Work is ordered _frontend first → backend second_ so each IPC removal is preceded by removal of every caller. Each task is one focused commit; tests/build/typecheck run between commits.

**Tech Stack:** Tauri 2.0, Svelte 5 (runes), Rust workspace (`claude-types`, `claude-config`), TypeScript (strict), `svelte-check`, Tailwind 4, `pnpm`, `cargo`.

---

## Baseline (Stage 3 end — commit `ea10c42`)

- `cargo test --workspace` → 162 passed, 1 ignored
- `pnpm exec tsc --noEmit` → 0 errors
- `pnpm exec svelte-check --threshold error` → **5 errors** (pre-existing, see Phase 3)
- `pnpm build` → clean
- Live `~/.dot-claude-gui/config.json` is still v1 schema (no `schemaVersion`, has `subpanelWidth` + `launcherProjectEnv`) — first launch under Stage 4 build will trigger `migrate_at_startup` and produce a `.bak.<unix>` file (verified via `app_config.rs` tests).

## File Structure

### Files to delete (frontend)

- `src/lib/components/launcher/LauncherView.svelte`
- `src/lib/components/launcher/LauncherList.svelte`
- `src/lib/components/launcher/` (directory becomes empty → remove)
- `src/lib/components/effective/EffectiveConfigView.svelte`
- `src/lib/components/effective/` (directory becomes empty → remove)
- `src/lib/components/plugins/ProjectActivation.svelte`
- `src/lib/components/shared/ScopeSelector.svelte`
- `src/lib/components/project-mode/ProjectModePlaceholder.svelte`

### Files to delete (backend)

- `src-tauri/src/commands/projects.rs` (entire module — UUID-keyed `list_projects` / `register_project` / `unregister_project`)

### Files to modify (frontend)

- `src/lib/components/plugins/PluginsModule.svelte` — drop `per-project` branch and `ProjectActivation` import
- `src/lib/components/plugins/PluginsSubNav.svelte` (if it advertises a "per-project" tab — verify and remove)
- `src/lib/components/memory/MemoryList.svelte` — remove stale `ScopeSelector` comments and the `isProjectScope` branch (it can never become true after Stage 4)
- `src/lib/components/claudemd/ClaudeMdList.svelte` — same: remove the project-scope branch
- `src/lib/stores/config.svelte.ts` — drop `projectSettings`, `activeScope`, `loadProjectConfig`, the project branch of `save()`/`revert()`/`reset()`
- `src/lib/stores/projects.svelte.ts` — drop deprecated aliases (`projects`/`activeProjectId`/`activeProject`/`registerProject`/`unregisterProject`)
- `src/lib/ipc/client.ts` — remove `getProjectConfig` / `updateProjectConfig` / `getEffectiveConfig` wrappers; trim unused imports
- `src/lib/components/appsettings/AppSettingsView.svelte` — reorganize into 4 sections (Appearance / Language / Terminal / About) with an About panel showing app name + version + repo URL
- `src/lib/components/shell/AppSettingsModal.svelte` — title/copy adjustments only if `i18n` keys move
- `src/App.svelte` — subscribe to a new `app-migration-report` Tauri event on mount and show a toast
- `src/lib/i18n.ts` — add new keys for About section + migration toast; ensure parity across `zh-CN` / `en-US` / `ja-JP`

### Files to modify (backend)

- `src-tauri/src/lib.rs` — drop 6 `generate_handler!` entries (`get_project_config`, `update_project_config`, `get_effective_config`, `list_projects`, `register_project`, `unregister_project`); cache the `MigrationReport` into `AppState`; emit a Tauri event `app-migration-report` on `setup`
- `src-tauri/src/commands/config.rs` — delete `get_project_config`, `update_project_config`, `get_effective_config` and any structs they alone use
- `src-tauri/src/commands/mod.rs` — drop `pub mod projects;`
- `src-tauri/src/state.rs` — remove `ProjectInfo`, `AppStateInner::projects`, `projects_file`, `load_projects`, `save_projects`, `with_projects_file`; update tests
- `src-tauri/src/commands/health.rs` (and any other consumer using `state.projects()`) — verify and clean
- `src-tauri/src/app_config.rs` — no logic change; verify `MigrationReport` derives `Clone + Serialize` so it can be emitted
- `crates/claude-config/src/merge.rs` — fix stale comment at line 38 about `enabledPlugins` merge semantics

### Files to add

- _None._ All new behavior (About section, migration toast, IPC payload type) lands inside existing files.

---

## Phase 1 — Frontend cleanup (must precede backend deletions)

> Order rationale: the Rust IPC commands stay registered until every TS caller is gone, so the build never enters a state where a registered command has no Rust handler.

### Task 1: Delete the orphaned Launcher and EffectiveConfig views

**Files:**
- Delete: `src/lib/components/launcher/LauncherView.svelte`
- Delete: `src/lib/components/launcher/LauncherList.svelte`
- Delete: `src/lib/components/launcher/` (empty directory)
- Delete: `src/lib/components/effective/EffectiveConfigView.svelte`
- Delete: `src/lib/components/effective/` (empty directory)

- [ ] **Step 1: Verify zero mounts**

Run: `grep -rn -E "LauncherView|LauncherList|EffectiveConfigView" src/`
Expected: only matches inside the four files about to be deleted (or zero matches outside them).

If any external mount turns up, STOP and report — Stage 3 left a route live.

- [ ] **Step 2: Delete the files and (empty) directories**

```bash
rm src/lib/components/launcher/LauncherView.svelte \
   src/lib/components/launcher/LauncherList.svelte \
   src/lib/components/effective/EffectiveConfigView.svelte
rmdir src/lib/components/launcher src/lib/components/effective
```

- [ ] **Step 3: Verify build is still clean**

Run: `pnpm exec tsc --noEmit`
Expected: 0 errors.

Run: `pnpm exec svelte-check --threshold error 2>&1 | tail -5`
Expected: still 5 baseline errors, no new ones (file count drops by ~3).

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "chore(stage4): delete unmounted Launcher and EffectiveConfig views"
```

### Task 2: Remove the per-project tab from PluginsModule and delete ProjectActivation

**Files:**
- Modify: `src/lib/components/plugins/PluginsModule.svelte`
- Modify (if it advertises a per-project section): `src/lib/components/plugins/PluginsSubNav.svelte` (or wherever the section keys are listed)
- Delete: `src/lib/components/plugins/ProjectActivation.svelte`

- [ ] **Step 1: Locate the per-project section advertiser**

Run: `grep -rn -E '"per-project"|per-project' src/lib/components/plugins/`
Expected: at least `PluginsModule.svelte` (the `{:else if activeSection === "per-project"}` branch); possibly a sub-nav definition.

Note every file/line that references the literal string `"per-project"` outside `ProjectActivation.svelte` itself.

- [ ] **Step 2: Edit `PluginsModule.svelte` — drop the import and the branch**

Open `src/lib/components/plugins/PluginsModule.svelte`. Make these two edits:

Old (header):
```svelte
<script lang="ts">
  import InstalledPlugins from "./InstalledPlugins.svelte";
  import MarketplaceBrowser from "./MarketplaceBrowser.svelte";
  import MarketplaceManager from "./MarketplaceManager.svelte";
  import ProjectActivation from "./ProjectActivation.svelte";

  let { activeSection = "installed" }: { activeSection: string } = $props();
</script>
```

New:
```svelte
<script lang="ts">
  import InstalledPlugins from "./InstalledPlugins.svelte";
  import MarketplaceBrowser from "./MarketplaceBrowser.svelte";
  import MarketplaceManager from "./MarketplaceManager.svelte";

  let { activeSection = "installed" }: { activeSection: string } = $props();
</script>
```

Old (template):
```svelte
  {:else if activeSection === "manage-marketplaces"}
    <MarketplaceManager />
  {:else if activeSection === "per-project"}
    <ProjectActivation />
  {:else}
```

New:
```svelte
  {:else if activeSection === "manage-marketplaces"}
    <MarketplaceManager />
  {:else}
```

- [ ] **Step 3: If a sub-nav advertises `per-project`, remove that entry**

If Step 1 turned up a `per-project` entry in a sub-nav definition (e.g. `PluginsFacet.svelte`'s section list), remove just that single entry — keep `installed` / `marketplace` / `manage-marketplaces`. Also remove any matching `t("plugins.subnav.perProject")` i18n call.

- [ ] **Step 4: Delete `ProjectActivation.svelte`**

```bash
rm src/lib/components/plugins/ProjectActivation.svelte
```

- [ ] **Step 5: Verify no leftover references**

Run: `grep -rn -E "ProjectActivation|\"per-project\"" src/`
Expected: 0 matches.

- [ ] **Step 6: Typecheck**

Run: `pnpm exec tsc --noEmit`
Expected: 0 errors.

- [ ] **Step 7: Visual smoke check**

Run: `pnpm tauri dev` (background OK), open the app, switch to Account mode → Plugins facet → confirm the sub-nav lists exactly `Installed`, `Marketplace`, `Manage marketplaces` (no per-project tab), and that `Installed` still loads its list without console errors. Then kill the dev server.

(If your machine has trouble with the Tauri dev cycle in CI, this step may be done as a manual checkpoint by the reviewer instead.)

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "chore(stage4): drop per-project plugins tab; delete ProjectActivation"
```

### Task 3: Delete `ScopeSelector` and prune project-scope branches from list components

**Files:**
- Delete: `src/lib/components/shared/ScopeSelector.svelte`
- Modify: `src/lib/components/memory/MemoryList.svelte` (remove the `isProjectScope` derived + the project-scope `{#if}` branch + stale comments mentioning `ScopeSelector`)
- Modify: `src/lib/components/claudemd/ClaudeMdList.svelte` (same: remove `configStore.activeScope === "project"` branch)

- [ ] **Step 1: Verify only stale references exist**

Run: `grep -rn -E "ScopeSelector" src/`
Expected: only comment hits in `MemoryList.svelte` (lines 7/11) and the file itself.

- [ ] **Step 2: Open `MemoryList.svelte`, locate the `isProjectScope` block**

Read `src/lib/components/memory/MemoryList.svelte`. Identify:
1. The `configStore.activeScope === "project"` derived statement (line ~9).
2. Any `{#if isProjectScope}` branches in the template.
3. The two stale comments referencing `ScopeSelector`.

- [ ] **Step 3: Edit `MemoryList.svelte`**

Replace the `isProjectScope` derived + its template branches with a straight-line "user scope" rendering (Account mode has no scope toggle any more — it is always the bound account's memory). If both branches are functionally identical you can just delete the `{#if}`/`{/if}` wrapper; if the project branch had unique markup, remove it. Also delete the two stale comments.

After the edit, re-read the file and confirm there is no remaining reference to `isProjectScope` or `ScopeSelector`.

- [ ] **Step 4: Edit `ClaudeMdList.svelte` analogously**

Open `src/lib/components/claudemd/ClaudeMdList.svelte`. Remove the `configStore.activeScope === "project"` derived/conditional. Keep the user-scope behavior.

- [ ] **Step 5: Delete `ScopeSelector.svelte`**

```bash
rm src/lib/components/shared/ScopeSelector.svelte
```

- [ ] **Step 6: Final grep**

Run: `grep -rn -E "ScopeSelector|isProjectScope" src/`
Expected: 0 matches.

- [ ] **Step 7: Typecheck + svelte-check**

Run: `pnpm exec tsc --noEmit && pnpm exec svelte-check --threshold error 2>&1 | tail -3`
Expected: 0 tsc errors; svelte-check still at 5 baseline errors (we fix those in Phase 3).

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "chore(stage4): drop ScopeSelector and project-scope branches from list views"
```

### Task 4: Delete the unmounted `ProjectModePlaceholder`

**Files:**
- Delete: `src/lib/components/project-mode/ProjectModePlaceholder.svelte`

- [ ] **Step 1: Verify zero mounts**

Run: `grep -rn "ProjectModePlaceholder" src/`
Expected: only the file itself.

- [ ] **Step 2: Delete**

```bash
rm src/lib/components/project-mode/ProjectModePlaceholder.svelte
```

- [ ] **Step 3: Typecheck**

Run: `pnpm exec tsc --noEmit`
Expected: 0 errors.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "chore(stage4): delete unmounted ProjectModePlaceholder"
```

### Task 5: Simplify `configStore` — drop project-scope branch

**Files:**
- Modify: `src/lib/stores/config.svelte.ts`

After Tasks 2–3 the only writers of `configStore.activeScope = "project"` are gone, so the project branch is dead. Collapse it.

- [ ] **Step 1: Verify no caller still flips `activeScope` to project**

Run: `grep -rn 'activeScope\s*=\s*"project"' src/`
Expected: 0 matches.

Also: `grep -rn 'configStore\.loadProjectConfig' src/`
Expected: 0 matches.

- [ ] **Step 2: Rewrite `src/lib/stores/config.svelte.ts`**

Replace the entire file with the simplified version:

```ts
import { ipcClient } from "$lib/ipc/client.js";
import type { Settings } from "$lib/api/types";
import { toastStore } from "./toast.svelte";

class ConfigStore {
  userSettings = $state<Settings>({});
  loading = $state(false);
  saving = $state(false);
  error = $state<string>("");
  isDirty = $state(false);

  /** Kept for compatibility with sub-editors that read this. Always user. */
  readonly activeScope = "user" as const;

  /** The settings being edited. */
  get activeSettings(): Settings {
    return this.userSettings;
  }

  async loadUserConfig() {
    this.loading = true;
    this.isDirty = false;
    this.error = "";
    try {
      const res = await ipcClient.getUserConfig();
      this.userSettings = res.settings;
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to load config";
    } finally {
      this.loading = false;
    }
  }

  markDirty() {
    this.isDirty = true;
  }

  async save(partialSettings: Partial<Settings>) {
    this.saving = true;
    this.error = "";
    try {
      const res = await ipcClient.updateUserConfig(partialSettings);
      this.userSettings = res.settings;
      this.isDirty = false;
      toastStore.success("Settings saved");
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to save";
      toastStore.error(this.error);
      throw e;
    } finally {
      this.saving = false;
    }
  }

  setUserConfig(settings: Settings): void {
    this.userSettings = settings;
    this.isDirty = false;
  }

  async revert(): Promise<void> {
    await this.loadUserConfig();
  }

  reset(): void {
    this.userSettings = {} as Settings;
    this.loading = false;
    this.saving = false;
    this.error = "";
    this.isDirty = false;
  }
}

export const configStore = new ConfigStore();
```

Decisions:
- `activeScope` is kept as a readonly `"user"` constant so `MemoryList.svelte`/`ClaudeMdList.svelte` callers reading the field still typecheck (and so Stage 5 can remove it once the comparators are gone). It is no longer mutable.
- `projectSettings` / `loadProjectConfig` removed.
- The `projectsStore` import is gone (it was only used for `activeProjectId`).

- [ ] **Step 3: Typecheck**

Run: `pnpm exec tsc --noEmit`
Expected: 0 errors. If a downstream component references `configStore.projectSettings` directly, replace that with `configStore.userSettings` or remove the consumer per the same logic as Task 3.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "chore(stage4): collapse configStore to user scope only"
```

### Task 6: Simplify `projectsStore` — drop deprecated aliases

**Files:**
- Modify: `src/lib/stores/projects.svelte.ts`

- [ ] **Step 1: Verify no callers**

Run: `grep -rn -E "projectsStore\.(activeProjectId|activeProject|projects|registerProject|unregisterProject)\b" src/`
Expected: 0 matches outside `projects.svelte.ts` itself.

If any caller turns up, STOP — Task 3 missed it. Patch the caller to use the new API (`selectedPath`/`selected`/`add`/`remove`) before continuing.

- [ ] **Step 2: Edit `src/lib/stores/projects.svelte.ts`**

Remove these blocks:

```ts
  // ── Deprecated aliases — kept so Stage-2 components compile.
  //    Will be cleaned up in Stage 3 once consumers are rewritten.
  get projects(): ProjectEntry[] { return this.entries; }
  get activeProjectId(): string | null { return this.selectedPath; }
  get activeProject(): ProjectEntry | null { return this.selected; }
```

and:

```ts
  // ── Deprecated compat methods (Stage-2 components call these names).
  async registerProject(path: string): Promise<void> {
    await this.add(path);
  }

  async unregisterProject(path: string): Promise<void> {
    await this.remove(path);
  }
```

- [ ] **Step 3: Typecheck**

Run: `pnpm exec tsc --noEmit`
Expected: 0 errors.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "chore(stage4): drop projectsStore deprecated aliases"
```

### Task 7: Delete the old IPC wrappers from `client.ts`

**Files:**
- Modify: `src/lib/ipc/client.ts`

- [ ] **Step 1: Verify no callers**

Run: `grep -rn -E "ipcClient\.(getProjectConfig|updateProjectConfig|getEffectiveConfig)\b" src/`
Expected: 0 matches.

- [ ] **Step 2: Edit `src/lib/ipc/client.ts`** — remove these three methods (lines 92–107):

```ts
  async getProjectConfig(projectId: string): Promise<ConfigResponse> {
    // Rust: get_project_config(project_id: String)
    return call("get_project_config", { projectId });
  }

  async updateProjectConfig(projectId: string, settings: Partial<Settings>): Promise<ConfigResponse> {
    // Rust: update_project_config(project_id: String, req: UpdateConfigRequest)
    return call("update_project_config", { projectId, req: { settings } });
  }

  async getEffectiveConfig(projectId: string): Promise<EffectiveConfig> {
    // Rust: get_effective_config(project_id: String) -> EffectiveConfigResponse
    // EffectiveConfigResponse { settings, field_sources } serializes to { settings, fieldSources }
    // which matches the TS EffectiveConfig type { settings, fieldSources }.
    return call("get_effective_config", { projectId });
  }
```

Also update the comment header for that section to read `// --- config (2) ---` (only `getUserConfig` + `updateUserConfig` remain).

If the `EffectiveConfig` type import in the `import { ... }` block at the top of the file is now unused, remove just that one type from the import list. Do NOT remove `ConfigResponse` (still used by `getUserConfig`).

- [ ] **Step 3: Typecheck**

Run: `pnpm exec tsc --noEmit`
Expected: 0 errors.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "chore(stage4): drop legacy project-config IPC wrappers"
```

---

## Phase 2 — Backend cleanup

Every TS caller is now gone. Safe to delete the Rust IPCs and the underlying state.

### Task 8: Delete the UUID-keyed `commands::config::*_project_*` handlers

**Files:**
- Modify: `src-tauri/src/commands/config.rs` (remove `get_project_config`, `update_project_config`, `get_effective_config` and any structs they alone use)
- Modify: `src-tauri/src/lib.rs` (remove the three corresponding entries from `generate_handler!`)

- [ ] **Step 1: Identify the deletions in `config.rs`**

Read `src-tauri/src/commands/config.rs`. Note the three functions and the line ranges. Note any helper struct (e.g. `EffectiveConfigResponse`, `UpdateConfigRequest` if only used by `update_project_config` — check whether `update_user_config` also uses it; if yes, keep the struct).

- [ ] **Step 2: Delete the three functions in `config.rs`**

Remove the three `#[tauri::command]` blocks. Leave `get_user_config` and `update_user_config` (and their helpers) alone.

If a helper struct becomes orphaned (no remaining callers in the crate), delete it too — verify with `grep -rn "<StructName>" src-tauri/`.

- [ ] **Step 3: Delete the entries from `src-tauri/src/lib.rs::generate_handler!`**

Open `src-tauri/src/lib.rs`. In the `tauri::generate_handler![...]` block (around line 53), remove these three lines:

```rust
            commands::config::get_project_config,
            commands::config::update_project_config,
            commands::config::get_effective_config,
```

Leave `commands::config::get_user_config` and `commands::config::update_user_config` in place.

- [ ] **Step 4: Build the workspace**

Run: `cargo build -p dot-claude-gui 2>&1 | tail -20`
Expected: clean build. If you see "function never used" warnings for helper structs, delete the struct.

- [ ] **Step 5: Run the test suite**

Run: `cargo test --workspace 2>&1 | tail -10`
Expected: 162 passed, 1 ignored (or 162 → fewer if `commands::config` had tests for the deleted functions; the count may drop. The pass count must equal the new total with zero failures).

If config.rs tests referenced the deleted functions, delete those tests in the same task.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "chore(stage4): remove UUID-keyed project-config IPC handlers"
```

### Task 9: Delete the `commands::projects` module and the project registry in `AppState`

**Files:**
- Delete: `src-tauri/src/commands/projects.rs`
- Modify: `src-tauri/src/commands/mod.rs` (remove `pub mod projects;`)
- Modify: `src-tauri/src/lib.rs` (drop `commands::projects::list_projects` / `register_project` / `unregister_project` from `generate_handler!`)
- Modify: `src-tauri/src/state.rs` (remove `ProjectInfo`, `AppStateInner::projects`, `projects_file`, `load_projects`, `save_projects`, `with_projects_file`)
- Modify: `src-tauri/src/lib.rs` (drop the `with_projects_file(...)` call from `AppState` construction in `setup`)
- Possibly modify other call sites — see Step 4

- [ ] **Step 1: Find every reference to `ProjectInfo`, `projects_file`, `load_projects`, `save_projects`, `with_projects_file`, and `state.projects()`**

Run: `grep -rn -E "ProjectInfo|projects_file|load_projects|save_projects|with_projects_file|\.projects\(\)" src-tauri/ crates/`
Note every hit outside `state.rs`, `commands/projects.rs`, and tests of these.

- [ ] **Step 2: Delete the module file**

```bash
rm src-tauri/src/commands/projects.rs
```

- [ ] **Step 3: Edit `src-tauri/src/commands/mod.rs`**

Remove the line:
```rust
pub mod projects;
```

- [ ] **Step 4: Update `state.rs`**

Open `src-tauri/src/state.rs`. Make the following changes:

1. Remove the `ProjectInfo` struct definition.
2. Remove the `projects: RwLock<Vec<ProjectInfo>>` field from `AppStateInner`.
3. Remove the `projects_file: PathBuf` field (if present).
4. Remove the `load_projects` / `save_projects` methods.
5. Remove the `with_projects_file(path)` constructor variant; keep `new()` and any other variants.
6. Update any `Default` / `new` impl that initialized these fields.
7. If there is a `projects()` getter on `AppState`, delete it.
8. Update unit tests in the same file that exercise the project registry — delete those tests.

If a test in `state.rs` is solely testing project-registry behavior (e.g. `test_register_project_appends`), delete the entire test function.

- [ ] **Step 5: Update `src-tauri/src/lib.rs`**

In `tauri::generate_handler!`, remove:

```rust
            commands::projects::list_projects,
            commands::projects::register_project,
            commands::projects::unregister_project,
```

In the `setup` closure, find the `AppState::with_projects_file(...)` call (or wherever the projects file path is wired). Replace it with `AppState::new()` (or whatever the bare constructor is now). Drop any path computation that fed the deleted `with_projects_file`.

- [ ] **Step 6: Update any remaining callers**

For each hit from Step 1 that is OUTSIDE `state.rs`/`projects.rs`/their tests, delete the call. If a command function reads `state.projects()` for a non-project purpose, refactor it to use `state.gui_projects()` (the new path-keyed registry from `commands::gui_projects::*` Stage 1) instead.

If a non-deleted command file relied on `ProjectInfo` as a return type, change it to the appropriate `ProjectEntry` from `claude-types` (the Stage 1 type).

- [ ] **Step 7: Build**

Run: `cargo build -p dot-claude-gui 2>&1 | tail -25`
Expected: clean build. Address any remaining "use of undeclared module" / "unresolved import" errors by removing the offending line.

- [ ] **Step 8: Run the test suite**

Run: `cargo test --workspace 2>&1 | tail -10`
Expected: All passing. Pass count will be lower than 162 because we deleted project-registry tests. The new floor (record it in the commit message) should still be > 150 — if it drops below, you deleted a test that shouldn't have gone.

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "chore(stage4): delete UUID project registry — commands::projects + state.projects"
```

---

## Phase 3 — Fix the pre-existing svelte-check baseline (5 errors → 0)

### Task 10: Fix `RuntimeEditor.svelte` runtime-mode type narrowing (1 error)

**Files:**
- Modify: `src/lib/components/settings/RuntimeEditor.svelte:157`

The error is:
```
"Type 'string | undefined' is not assignable to type '"auto" | "in-process" | "tmux" | undefined'."
```

This typically happens at a `<select onchange>` setter that pulls `(e.target as HTMLSelectElement).value` and assigns it to a strongly-typed field.

- [ ] **Step 1: Read the surrounding code**

Read `src/lib/components/settings/RuntimeEditor.svelte` lines 140–170. Identify the failing assignment.

- [ ] **Step 2: Add an `as` cast at the assignment**

Replace the offending line with an `as "auto" | "in-process" | "tmux"` cast, e.g.:

```svelte
onchange={(e) => update({ shellRunner: (e.target as HTMLSelectElement).value as "auto" | "in-process" | "tmux" })}
```

If the type comes from a shared TS alias (e.g. `ShellRunnerMode` in `$lib/api/types.ts`), import and use it for the cast instead of inlining the union. Verify the alias exists with `grep -n "ShellRunnerMode\|shellRunner" src/lib/api/types.ts`.

- [ ] **Step 3: Verify**

Run: `pnpm exec svelte-check --threshold error 2>&1 | grep RuntimeEditor`
Expected: only the 3 remaining errors on line 352 — the line 157 error is gone.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/settings/RuntimeEditor.svelte
git commit -m "fix(stage4): RuntimeEditor runtime-mode select narrowing"
```

### Task 11: Fix `RuntimeEditor.svelte` line 352 — undefined `owner`/`repo`/`number` (3 errors)

**Files:**
- Modify: `src/lib/components/settings/RuntimeEditor.svelte:352`

The errors are:
```
"Cannot find name 'owner'."
"Cannot find name 'repo'."
"'number' only refers to a type, but is being used as a value here."
```

Likely a template expression referencing variables that were renamed or never declared, plus a `Number(x)` vs `number(x)` typo.

- [ ] **Step 1: Read the surrounding code**

Read `src/lib/components/settings/RuntimeEditor.svelte` lines 330–360.

- [ ] **Step 2: Diagnose what `owner`/`repo`/`number` were meant to be**

Likely the surrounding context has variables under different names (e.g. `repoOwner`, `repoName`, `prNumber`) or the values are nested in an object. Use the declarations earlier in the file as ground truth.

- [ ] **Step 3: Fix the expression**

If it's a typo (`number(x)` → `Number(x)`), fix the casing. If `owner`/`repo` should be field accesses (e.g. `pr.owner`, `pr.repo`), add the missing object prefix. If the variables genuinely don't exist (dead code), delete the line or the surrounding block.

- [ ] **Step 4: Verify**

Run: `pnpm exec svelte-check --threshold error 2>&1 | grep RuntimeEditor`
Expected: 0 matches.

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/settings/RuntimeEditor.svelte
git commit -m "fix(stage4): RuntimeEditor — resolve owner/repo references and Number() typo"
```

### Task 12: Fix `SchemaKeyPicker.svelte` MessageKey assertion (1 error)

**Files:**
- Modify: `src/lib/components/settings/sub/SchemaKeyPicker.svelte:78`

The error:
```
"Argument of type 'string' is not assignable to parameter of type 'MessageKey'."
```

A plain `string` is being passed to `t(key)` (which expects a literal `MessageKey` union).

- [ ] **Step 1: Read the line**

Read `src/lib/components/settings/sub/SchemaKeyPicker.svelte` lines 60–95.

- [ ] **Step 2: Decide the right fix**

If the string is dynamic (built from a schema), passing it to `t()` is fundamentally unsafe — wrap the call in a helper that returns the raw string if the key isn't a known `MessageKey`, e.g.:

```ts
import { hasTranslation } from "$lib/i18n";
// ...
const label = hasTranslation(key) ? t(key) : key;
```

If `hasTranslation` doesn't exist, add a `function hasTranslation(k: string): k is MessageKey { ... }` predicate to `src/lib/i18n.ts` that checks against the exported message-key set.

Alternatively, if the key is actually known to be a valid `MessageKey` (the schema list is hand-curated), add a cast: `t(key as MessageKey)` — but only do this if you confirm the underlying values are all real keys (grep them).

- [ ] **Step 3: Verify**

Run: `pnpm exec svelte-check --threshold error 2>&1 | tail -3`
Expected: `0 ERRORS`.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "fix(stage4): SchemaKeyPicker — guard t() against dynamic keys"
```

---

## Phase 4 — Documentation

### Task 13: Fix the stale comment in `merge.rs`

**Files:**
- Modify: `crates/claude-config/src/merge.rs:38`

The comment says `enabledPlugins` is concatenated (Vec append). After the Stage 3 fix in commit `bb7b121`, the type is `HashMap<String, bool>` and the merge is per-key overlay (later layer wins per key, removed-key kept). Update the comment to match the implementation.

- [ ] **Step 1: Read the merge logic and the surrounding comment**

Read `crates/claude-config/src/merge.rs` around line 38 (read lines 25–80 to see the actual merge code that handles `enabledPlugins`).

- [ ] **Step 2: Replace the comment**

Update the comment block to describe the actual semantics — something like:

```rust
// `enabledPlugins` is a HashMap<String, bool> overlay: later layers' keys
// overwrite earlier layers' keys per-key. Keys absent from the later layer
// are preserved from the earlier layer (no removal). Tri-state semantics
// (true/false/absent) are intentional — see ProjectEffective merge.
```

Match the existing comment style (single `//` lines, no doc-comment markers unless adjacent code is documented this way).

- [ ] **Step 3: Run the merge tests**

Run: `cargo test -p claude-config 2>&1 | tail -5`
Expected: all passing.

- [ ] **Step 4: Commit**

```bash
git add crates/claude-config/src/merge.rs
git commit -m "docs(stage4): correct merge.rs comment — enabledPlugins is HashMap overlay"
```

---

## Phase 5 — Gear panel, migration toast, i18n audit

### Task 14: Reorganize `AppSettingsView` into 4 sections (Appearance / Language / Terminal / About)

**Files:**
- Modify: `src/lib/components/appsettings/AppSettingsView.svelte`
- Modify: `src/lib/i18n.ts` (add About-section keys to all 3 ACTIVE locales)
- Possibly modify: `package.json` (read app version)

Spec line 230: _"Complete the right-corner gear panel (Appearance / Language / Terminal / About)"_. Today the file has 2 sections: `appearance` (which includes theme + font + language) and `launcher` (terminal). The Stage 4 layout is 4 separate sections.

- [ ] **Step 1: Read the current file**

Confirm the current shape matches what was captured at planning time (Appearance + Launcher only).

- [ ] **Step 2: Add new i18n keys to `src/lib/i18n.ts`**

For each of the 3 ACTIVE locales (`zh-CN`, `en-US`, `ja-JP`), add (at minimum) these keys to the per-locale message map and to the `MessageKey` union:

| key | zh-CN | en-US | ja-JP |
|---|---|---|---|
| `appsettings.language` | 语言 | Language | 言語 |
| `appsettings.terminal` | 终端 | Terminal | ターミナル |
| `appsettings.about` | 关于 | About | このアプリについて |
| `appsettings.appName` | dot-claude-gui | dot-claude-gui | dot-claude-gui |
| `appsettings.version` | 版本 {version} | Version {version} | バージョン {version} |
| `appsettings.repo` | 仓库 | Repository | リポジトリ |

(Reuse existing `appsettings.appearance` / `appsettings.theme` / `appsettings.languageLabel` / `appsettings.preferredTerminal` keys where appropriate; only add what's missing.)

If the file currently has `appsettings.launcher` and no other consumer, you can either rename to `appsettings.terminal` or just deprecate the old key. Pick rename for cleanliness — fewer dead keys to maintain.

- [ ] **Step 3: Read app version**

The Tauri config (`src-tauri/tauri.conf.json`) and `package.json` both have a version string. Pick `package.json` since it's directly importable; alternative is exposing a backend `app_version` command. Use the imported approach:

```ts
import pkg from "../../../../package.json" with { type: "json" };
const APP_VERSION = pkg.version as string;
```

(Adjust the relative path to wherever the file ends up. Vite handles the JSON import natively.)

If TypeScript's `resolveJsonModule` is off, enable it in `tsconfig.json` first — it's the cheapest fix.

- [ ] **Step 4: Rewrite `AppSettingsView.svelte`**

Replace the file with a 4-section layout. Sketch:

```svelte
<script lang="ts">
  import { appSettingsStore } from "$lib/stores/appsettings.svelte.js";
  import { t, ACTIVE_LOCALES, localeDisplayName, type Locale } from "$lib/i18n";
  import pkg from "../../../../package.json" with { type: "json" };

  const APP_VERSION: string = (pkg as { version: string }).version;
  const REPO_URL = "https://github.com/darknight/dot-claude-gui"; // confirm in package.json or hardcode the canonical URL
</script>

<div class="p-6 space-y-8">

  <!-- 1. Appearance -->
  <section class="space-y-4">
    <h2 class="text-lg font-medium" style="color: var(--text-primary)">{t("appsettings.appearance")}</h2>
    <!-- theme select -->
    <!-- font size range -->
  </section>

  <!-- 2. Language -->
  <section class="space-y-4">
    <h2 class="text-lg font-medium" style="color: var(--text-primary)">{t("appsettings.language")}</h2>
    <!-- language select (move from Appearance) -->
  </section>

  <!-- 3. Terminal -->
  <section class="space-y-4">
    <h2 class="text-lg font-medium" style="color: var(--text-primary)">{t("appsettings.terminal")}</h2>
    <!-- preferred terminal select -->
  </section>

  <!-- 4. About -->
  <section class="space-y-4">
    <h2 class="text-lg font-medium" style="color: var(--text-primary)">{t("appsettings.about")}</h2>
    <div class="space-y-1 text-sm" style="color: var(--text-muted)">
      <div>{t("appsettings.appName")}</div>
      <div>{t("appsettings.version", { version: APP_VERSION })}</div>
      <div>
        <span>{t("appsettings.repo")}: </span>
        <a href={REPO_URL} target="_blank" rel="noreferrer" style="color: var(--accent)">{REPO_URL}</a>
      </div>
    </div>
  </section>

</div>
```

Move the existing widgets into the matching section (theme + font into Appearance; language select into Language; preferred terminal into Terminal). Don't duplicate widgets.

- [ ] **Step 5: Verify the repo URL**

Read `package.json` `repository.url` field; if it exists, use it. If not, use the canonical GitHub URL.

- [ ] **Step 6: Typecheck + svelte-check**

Run: `pnpm exec tsc --noEmit && pnpm exec svelte-check --threshold error 2>&1 | tail -3`
Expected: `0 ERRORS`.

- [ ] **Step 7: Visual smoke check**

Run: `pnpm tauri dev` (background OK), open the gear panel, confirm 4 distinct sections render and that language switching still re-renders the entire panel. Then kill the dev server.

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "feat(stage4): gear panel — 4 sections (Appearance/Language/Terminal/About)"
```

### Task 15: Add a v1→v2 migration toast

**Files:**
- Modify: `src-tauri/src/lib.rs` (emit `app-migration-report` event after `migrate_at_startup` succeeds)
- Modify: `src-tauri/src/app_config.rs` (ensure `MigrationReport` derives `Serialize + Clone`)
- Modify: `src/lib/ipc/events.ts` (add `onAppMigrationReport` listener helper + payload type)
- Modify: `src/App.svelte` (subscribe on mount; show toast)
- Modify: `src/lib/i18n.ts` (new keys for the toast)

- [ ] **Step 1: Confirm `MigrationReport` shape and derives**

Read `src-tauri/src/app_config.rs` lines 280–360. Note the `MigrationReport` struct: it has `migrated: bool` and (per the brief) a backup-path field. Add `Serialize, Clone` to the derive list if they're not already there:

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationReport {
    pub migrated: bool,
    pub backup_path: Option<PathBuf>,
    // ... existing fields
}
```

Note: per CLAUDE.md gotcha #9, the `#[serde(rename_all = "camelCase")]` is required so the frontend sees `backupPath` not `backup_path`. Apply it.

- [ ] **Step 2: Emit the event in `lib.rs::setup`**

Read `src-tauri/src/lib.rs` lines 100–140 to find the migration call. Wrap it so the report is emitted via `app.emit("app-migration-report", report)` after the migration succeeds (even when `migrated == false` — the frontend filters). For the failure branch, emit an event with `migrated: false` (or just skip emitting; frontend should be idempotent).

Example:
```rust
match app_config::migrate_at_startup(&cfg_path, native_exists) {
    Ok(report) => {
        tracing::info!("config migration: {report:?}");
        let _ = app.emit("app-migration-report", &report);
    }
    Err(e) => tracing::error!("config migration failed: {e}"),
}
```

Make sure `tauri::Manager` is in scope (so `app.emit` resolves). It's likely already imported.

- [ ] **Step 3: Add a unit test for `MigrationReport` serialization**

In `src-tauri/src/app_config.rs` (or wherever the existing migration tests live), add a small test that asserts the JSON shape is camelCase:

```rust
#[test]
fn migration_report_serializes_camel_case() {
    let r = MigrationReport { migrated: true, backup_path: Some(PathBuf::from("/tmp/foo.bak")), /* other fields */ };
    let json = serde_json::to_string(&r).unwrap();
    assert!(json.contains("\"migrated\":true"));
    assert!(json.contains("\"backupPath\":\"/tmp/foo.bak\""));
}
```

This locks the contract so future refactors of `MigrationReport` can't silently break the toast.

- [ ] **Step 4: Add the frontend listener**

Open `src/lib/ipc/events.ts`. Add (or extend):

```ts
export interface AppMigrationReport {
  migrated: boolean;
  backupPath?: string;
  // (other fields if MigrationReport has them — list them explicitly)
}

export async function onAppMigrationReport(handler: (report: AppMigrationReport) => void): Promise<() => void> {
  const unlisten = await listen<AppMigrationReport>("app-migration-report", (e) => handler(e.payload));
  return unlisten;
}
```

Match the style of the existing `onConfigChanged` / `onCommandOutput` helpers (cleanup function pattern, per CLAUDE.md gotcha #2).

- [ ] **Step 5: Subscribe in `App.svelte`**

In `App.svelte`, inside `onMount`, attach the listener and on `migrated === true` show a toast via `toastStore.success(...)`. Per CLAUDE.md gotcha #2, return the unlistener from `onMount` to ensure cleanup runs at component teardown.

Pseudo:
```ts
import { onAppMigrationReport } from "$lib/ipc/events";

onMount(() => {
  let unlistenMigration: (() => void) | null = null;
  onAppMigrationReport((report) => {
    if (report.migrated) {
      toastStore.success(t("migration.toastSuccess", { backup: report.backupPath ?? "" }));
    }
  }).then((u) => { unlistenMigration = u; });

  return () => {
    unlistenMigration?.();
  };
});
```

- [ ] **Step 6: Add the i18n keys**

In `src/lib/i18n.ts`, add `migration.toastSuccess` for the three ACTIVE locales:

| key | zh-CN | en-US | ja-JP |
|---|---|---|---|
| `migration.toastSuccess` | 已迁移到 v2 配置，旧文件备份至 {backup} | Migrated to v2 config. Old file backed up to {backup} | v2 設定に移行しました。旧ファイルのバックアップ: {backup} |

Add the new key to the `MessageKey` union.

- [ ] **Step 7: Test the migration path**

Backup the live config and force a v1 config:

```bash
cp ~/.dot-claude-gui/config.json ~/.dot-claude-gui/config.json.real-backup
# (config is already v1 per baseline; if not, hand-edit to remove schemaVersion)
```

Run the app: `pnpm tauri dev`. Verify the toast appears once with the expected text. Inspect `~/.dot-claude-gui/` for a `config.json.bak.<unix>` file. Quit the app, restart — toast should NOT appear the second time (because `migrated: false`).

Restore the original config if needed: `cp ~/.dot-claude-gui/config.json.real-backup ~/.dot-claude-gui/config.json && rm ~/.dot-claude-gui/config.json.real-backup`.

- [ ] **Step 8: Cargo test**

Run: `cargo test --workspace 2>&1 | tail -5`
Expected: previous count + 1 (from the new camelCase serialization test).

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "feat(stage4): emit and surface v1→v2 migration toast"
```

### Task 16: i18n parity audit — `zh-CN` / `en-US` / `ja-JP`

**Files:**
- Modify: `src/lib/i18n.ts` (add any missing keys per locale)

The spec requires no English fallbacks in zh-CN UI (line 235). Stage 4 likely added keys in zh-CN/en-US first; ja-JP may have lagged.

- [ ] **Step 1: Find missing keys per locale**

Use a small audit script. Add `scripts/i18n-audit.mjs` (temporary — delete in Step 4):

```js
import { ZH_CN, EN_US, JA_JP } from "../src/lib/i18n.ts"; // adjust path/imports for your file layout

const all = new Set([
  ...Object.keys(ZH_CN),
  ...Object.keys(EN_US),
  ...Object.keys(JA_JP),
]);

for (const [name, map] of [["zh-CN", ZH_CN], ["en-US", EN_US], ["ja-JP", JA_JP]]) {
  const missing = [...all].filter((k) => !(k in map));
  if (missing.length) console.log(`${name} missing ${missing.length}:`, missing);
  else console.log(`${name} complete`);
}
```

If `i18n.ts` doesn't export the locale maps directly (it likely uses a different structure), inline the comparison — read the file once via `Read`, extract the three maps by structure, diff their key sets, and print the missing keys per locale. (You can do this directly with Node + a tiny inline AST walk, or just visually diff three sorted key lists.)

- [ ] **Step 2: Fill in the missing keys**

For every reported missing key, add a translation in the right locale. For the small number of keys that genuinely should not differ per locale (e.g. `appsettings.appName`), use the same value.

If a key is missing from EN_US, that's a bug — add an English translation first, then translate.

- [ ] **Step 3: Re-audit**

Re-run the audit. Expected output: all three locales report "complete".

- [ ] **Step 4: Verify the running app**

Run: `pnpm tauri dev`, switch language to ja-JP via the gear panel, click through Account mode and Project mode, scan visually for any string still in English or Chinese. Repeat for en-US to check zh-CN didn't bleed in.

Remove the audit script if you added one:
```bash
rm scripts/i18n-audit.mjs
```

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "i18n(stage4): zh-CN / en-US / ja-JP parity"
```

---

## Phase 6 — Final verification (E2E per spec § Verification)

### Task 17: Run the seven-step end-to-end check

**Files:** _(no source changes — this is the acceptance gate)_

The spec defines seven E2E steps at design.md:254-260. Run them and document the result in the final commit message.

- [ ] **Step 1: Baseline checks (automated)**

```bash
cargo test --workspace 2>&1 | tail -3
pnpm exec tsc --noEmit
pnpm exec svelte-check --threshold error 2>&1 | tail -3
pnpm build 2>&1 | tail -5
```

Expected:
- Rust: all passing (count adjusted for deletions)
- tsc: 0 errors
- svelte-check: **0 errors** (was 5 at Stage 3 baseline)
- Vite build: clean

- [ ] **Step 2: Migration toast (spec step 1)**

Confirm the toast still appears once on the v1 config; re-launch shows no toast. (Already verified in Task 15 Step 7; re-verify after all Phase 5 commits.)

- [ ] **Step 3: Cross-mode plugin override (spec steps 2–4)**

In the app:
1. Create a test account (e.g. `e2e-acct`) via the account sidebar.
2. Account mode → `e2e-acct` → Plugins → enable a real plugin (or create a fake marketplace entry; if no marketplaces are configured, this step degrades to "verify the toggle UI renders and persists" — note this in the commit message).
3. Add a test project pointing at any local directory; bind it to `e2e-acct`.
4. Project mode → that project → Plugins ↓ → set one plugin to Disable.
5. Project > Effective → verify the plugin shows as disabled with source = `project`.

- [ ] **Step 4: Launch (spec step 5)**

Project > Launch → confirm `[Launch Claude]` opens a terminal with `CLAUDE_CONFIG_DIR=~/.dot-claude-gui/accounts/e2e-acct`, the project cwd, and the env/args from the Launch tab. (Verify with `echo $CLAUDE_CONFIG_DIR && pwd` in the spawned terminal.)

- [ ] **Step 5: Language switch (spec step 6)**

Gear → switch language to en-US → confirm all UI strings re-render in English with no Chinese leftovers. Switch back to zh-CN to leave the app in the original state.

- [ ] **Step 6: Unbound project degradation (spec step 7)**

Add another test project without binding it. Confirm in Project mode that only Binding facet is interactive; the others are greyed out and the unbound banner explains why.

- [ ] **Step 7: Old routes are dead**

```bash
grep -rn -E "EffectiveConfigView|LauncherView|ProjectActivation|ScopeSelector|ProjectModePlaceholder" src/
grep -rn -E "commands::projects|get_project_config|update_project_config|get_effective_config" src-tauri/
```

Expected: 0 matches.

- [ ] **Step 8: Final commit**

If Tasks 1–16 already covered the work, this final commit is just a docs touch (Stage 4 completion note in the project CLAUDE.md if there are new gotchas) or simply a recap commit. If the project CLAUDE.md gained no new Stage 4 gotcha, you can skip the commit.

```bash
# Optional — only if there's a Stage 4 gotcha worth recording in CLAUDE.md
git add CLAUDE.md
git commit -m "docs(stage4): record Stage 4 gotcha — <one-liner>"
```

---

## Self-review checklist

After implementing every task above, the reviewer should be able to assert:

1. `cargo test --workspace`: 0 failures.
2. `pnpm exec tsc --noEmit`: 0 errors.
3. `pnpm exec svelte-check --threshold error`: 0 errors.
4. `pnpm build`: clean.
5. `grep -rn "ProjectInfo\|with_projects_file\|getProjectConfig\|updateProjectConfig\|getEffectiveConfig\|ProjectActivation\|ScopeSelector\|EffectiveConfigView\|LauncherView\|LauncherList\|ProjectModePlaceholder" .` — only matches inside `.git/`, `node_modules/`, `target/`, or `docs/superpowers/` (planning artifacts). Zero matches in `src/`, `src-tauri/`, `crates/`.
6. Gear panel has exactly four headed sections: Appearance / Language / Terminal / About.
7. On a synthetic v1 config, app shows the migration toast once.
8. Switching language via the gear panel re-renders the full UI in the chosen locale with no fallbacks.
9. The seven-step E2E flow passes.

## Out of scope (deferred to Stage 5 or later)

These were identified during planning but deliberately excluded from Stage 4:

- **ProjectSettingsFacet sectioned UI** — current raw JSON editor stays. Refactor of `SettingsEditor` into a controlled component shared between Account and Project would be a non-trivial diff; not a cleanup task.
- **Stale-binding "Update path" action** — banner today only offers Remove. Adding an Update flow needs a new IPC + file picker; small but a feature, not cleanup.
- **Shared plugin pool, `dotclaude-launch` CLI, all-accounts bulk edit, `.claude/dotclaude.json` binding files** — per spec § Out of scope.
