# Phase 8 Stage 5 — Backlog Cleanup + Small Features

**Date:** 2026-05-13
**Status:** Design — pending implementation plan
**Predecessor:** `2026-05-12-phase8-stage4-cleanup.md` (Stage 4 complete)
**Parent spec:** `2026-05-11-phase8-mode-based-redesign-design.md`

## Goal

Close the Stage 4 backlog. Deliver two small deferred features (A1 sectioned project settings UI, A2 stale-path Update action), fix the E2E gaps surfaced during Stage 4 acceptance (B1 i18n audit tool, B2 plugin tri-state verification prereq, B3 UnboundHint dead-code removal), and three consistency cleanups (C1 appName derivation, C2 teammateMode typing, C3 activeScope removal).

No new architecture. No items from Phase 8 spec § Out of scope (line 237-244) are reintroduced.

## Background

Stage 4 (cleanup, 21 commits) landed 2026-05-12 with a clean baseline: 155 cargo tests passing, 0 svelte-check errors, 0 tsc errors, pnpm build clean. Two categories of follow-up survived the cut:

1. **Plan-deferred backlog** (Stage 4 plan § Out of scope, line 1193-1199):
   - ProjectSettingsFacet sectioned UI — current raw JSON editor stays; refactoring `SettingsEditor` was deemed not a cleanup task.
   - Stale-binding "Update path" action — banner today only offers Remove; adding an Update flow needs a new IPC + file picker.

2. **E2E acceptance findings** (memory `project_dot_claude_gui.md` line 42-51):
   - i18n audit design blind spot — Stage 4's Cluster 10 audit checked only "key exists + non-empty," missing 10 English-as-placeholder keys.
   - Plugin tri-state E2E not run — the on-disk `myself` account has no installed plugin, so spec § Verification steps 2-4 cannot run on current data.
   - UnboundHint component is dead code — `ProjectModeView.svelte:58-59` branch is unreachable; BindingFacet's banner (added in commit `454eaec`) already covers the UX.

3. **Minor consistency cleanups** noted during Stage 4:
   - `appsettings.appName` hard-coded as `"dot-claude-gui"` across 6 locales instead of derived from `package.json#name`.
   - `RuntimeEditor.svelte:138` init-object `teammateMode` typing inconsistent with line 157 (which svelte-check flagged and Stage 4 fixed).
   - `configStore.activeScope` left as `readonly "user" as const` constant pending comparator cleanup.

Stage 5 closes all three categories in a single backlog-grooming spec.

## Cluster A — Deferred features

### A1 — SettingsEditor shared chrome + Project sectioned UI

**Status quo:**
- `src/lib/components/settings/SettingsEditor.svelte` (57 lines): section switcher that dispatches to 10 sub-editors. Each sub-editor reads `configStore.settings` directly (user scope only after Stage 4).
- `src/lib/components/project-mode/ProjectSettingsFacet.svelte` (165 lines): standalone raw JSON textarea with its own load / validate / save / revert.

**Approach:** extract shared chrome, do not unify sub-editors.

**New shared component:** `src/lib/components/shared/SectionedSettings.svelte`
- Props:
  - `sections: { id: string; label: string }[]`
  - `activeSection: string` (bound)
  - `isDirty: boolean`
  - `error: string | null`
- Snippets:
  - `header` (rendered above content)
  - `content` (called with current `activeSection`)
- Layout: left section nav (vertical list of section labels) + right content area + top dirty/error banner.
- Visual parity with current `SettingsEditor` chrome.

**Account-side refactor:** `SettingsEditor.svelte` becomes a thin caller of `SectionedSettings`. Its `content` snippet contains the same `{#if activeSection === "general"} <GeneralEditor /> {:else if ...}` switch as today. Sub-editors unchanged. Behavior preserved.

**Project-side refactor:** `ProjectSettingsFacet.svelte` becomes a caller of `SectionedSettings` with a project-specific section list:

| Section id | Sub-editor |
|---|---|
| `runtime` | New `ProjectRuntimeEditor.svelte` (project-scoped subset of runtime fields) |
| `environment` | New `ProjectEnvVarEditor.svelte` (project env vars) |
| `hooks` | New `ProjectHooksEditor.svelte` (project hooks) |
| `advanced` | New `ProjectAdvancedJsonEditor.svelte` (raw JSON fallback — current behavior) |

The project sub-editors are **separate components**, not reused from the Account `settings/` folder. Rationale: Account sub-editors directly read `configStore.settings` and bind `.update*` methods on the store; rewiring them to accept a `source` prop would force a store double-implementation that Stage 4 explicitly avoided.

**Project-side state management:**
- A `ProjectSettingsState` `$state` block (or a small class) inside `ProjectSettingsFacet.svelte` holds `{ original: Settings, current: Settings, error: string | null, saving: boolean }`.
- Initial load via existing `ipcClient.projectReadSettings(path)`.
- Save via existing `ipcClient.projectWriteSettings(path, current)`.
- Sub-editors receive `current` (or focused slice) and a patch callback. They are dumb form components.
- `isDirty` computed from `JSON.stringify(current) !== JSON.stringify(original)` (sufficient given Settings is plain JSON).

**Sections deliberately omitted from project scope:**
- `permissions`, `sandbox`, `statusLine`, `mcpServers` (a.k.a. mcp policy), `pluginsMarketplace`, `general`

Rationale: these settings are nonsensical or never read at project layer per Claude Code config semantics. Users editing project-layer JSON for these fields can still use the `advanced` section's raw editor.

**i18n keys to add** (per locale: zh-CN, en-US, ja-JP, plus the three currently-empty locales ko-KR / fr-FR / es-ES kept in parity):
- `projectMode.settings.section.runtime`
- `projectMode.settings.section.environment`
- `projectMode.settings.section.hooks`
- `projectMode.settings.section.advanced`

### A2 — Stale-binding "Update path" action

**Status quo (corrected during plan-writing — spec original had outdated assumptions):**
- `ProjectEntry` already carries a `stale: bool` field (see `src-tauri/src/commands/gui_projects.rs:46-53`), computed by `gui_list_projects` as `!std::path::Path::new(path).exists()`.
- Frontend `projectsStore.currentStale` (`src/lib/stores/projects.svelte.ts:24`) derives from it.
- `StalePathBanner.svelte` already exists with a single `Remove` button. It is rendered by `ProjectModeView.svelte:42` whenever the focused project is stale; in stale state all facet tabs are disabled (line 31: `if (isStale) return true`) and a `stalePathBlocked` empty state replaces facet content (line 57).
- Missing piece: an `Update path…` action alongside Remove, plus the backend IPC to perform the rename.

**Approach:** extend the existing `StalePathBanner` with an Update button and a Tauri folder picker; add a single new IPC.

**Backend changes (`src-tauri/src/commands/gui_projects.rs`):**

New IPC command `update_project_path`:
```rust
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectPathRequest {
    pub old_path: String,
    pub new_path: String,
}

#[tauri::command]
pub fn update_project_path(req: UpdateProjectPathRequest) -> Result<(), String>
```

Behavior:
- Validate `new_path` is a directory that exists: `std::path::Path::new(&req.new_path).is_dir()`, else return `invalid_path: <new_path>`.
- Canonicalize `new_path` (existing `canonicalize_path` helper) and `old_path`.
- Validate `new_path` (canonical form) is not already in `cfg.known_projects` — return `path_already_known: <new_path>` if so.
- Inside `mutate`:
  - Locate `old_path` (canonical) in `known_projects` — return `unknown_path: <old_path>` if not found.
  - In `known_projects`: replace the entry in-place (preserve list ordering).
  - In `projects` HashMap: if old entry exists, `remove` it and `insert` under the new canonical path with the same `ProjectBinding` value.
- Atomic write via existing `mutate` / `write_config` pattern.

Register the command in `src-tauri/src/lib.rs` (the invoke handler list at lines 60-65 currently lists the other `gui_projects::*` commands).

**Frontend changes:**

1. `src/lib/ipc/client.ts` — add wrapper:
   ```ts
   async updateProjectPath(oldPath: string, newPath: string): Promise<void> {
     await invoke("update_project_path", { req: { oldPath, newPath } });
   }
   ```
   Place adjacent to `updateProjectLaunch` (currently at line 120) for consistency.

2. `src/lib/stores/projects.svelte.ts` — add method:
   ```ts
   async updatePath(oldPath: string, newPath: string): Promise<void> {
     await ipcClient.updateProjectPath(oldPath, newPath);
     await this.loadProjects();
   }
   ```
   Plus: after a successful rename, the consumer (`StalePathBanner`) must also update `modeStore.selectedProject` to the new path so the focused project follows the rename. The store method itself stays unaware of `modeStore` (no cross-store coupling); the caller orchestrates.

3. `src/lib/components/project-mode/StalePathBanner.svelte` — add an `Update path…` button (primary, before the existing Remove):
   - Handler calls `@tauri-apps/plugin-dialog`'s `open({ directory: true, multiple: false, title: t("projectMode.staleUpdatePathDialogTitle"), defaultPath: <parent dir of old path or HOME fallback> })`.
   - On non-null string result: call `projectsStore.updatePath(path, picked)`, then `modeStore.selectedProject = picked`, then `toastStore.success(t("projectMode.stalePathUpdated"))`.
   - On null (cancel): no-op.
   - On thrown error: `toastStore.error(String(e))`.

   Keep the existing Remove button untouched.

**i18n keys to add** (under existing `projectMode.stale*` family):
- `projectMode.staleUpdatePathBtn` — button label, e.g. `Update path…` / `更新路径…` / `パスを更新…`
- `projectMode.staleUpdatePathDialogTitle` — passed to the Tauri folder picker as `title`
- `projectMode.stalePathUpdated` — toast on success, e.g. `Project path updated.`

## Cluster B — E2E gap fixes

### B1 — i18n audit script (CJK regex scanner)

**Approach:** standalone script + JSON whitelist, runnable via `pnpm run audit:i18n`.

**New file:** `scripts/audit-i18n.ts`

Algorithm:
1. Read `src/lib/locales/*.json`. The reference locale is `en-US`.
2. For each non-reference locale, walk all string values.
3. A value is **suspect** when ALL of these hold:
   - Value is a non-empty string
   - Value matches `/^[\x00-\x7f]+$/` (pure ASCII — no CJK character, no accented Latin)
   - Reference (`en-US`) value at the same key is a normal English sentence (heuristic: contains a space OR length ≥ 4 chars)
   - Neither key nor value matches any whitelist rule
4. Print a report grouped by locale: `<locale>: <key> = <value>`.
5. Exit non-zero (1) if any suspect entries found, else 0.

**Whitelist file:** `scripts/audit-i18n.whitelist.json`
```json
{
  "keys": [],
  "keySuffixes": [".cmd", ".shortcut", ".url", ".code", ".identifier"],
  "valuePatterns": [
    "^https?://",
    "^[A-Z][A-Z0-9_]*$"
  ]
}
```
Initial seed: empty `keys`. After the seed sweep, brand strings (e.g. anything mapping to product/CLI identifiers) get added explicitly.

**Locale parity:** the script also reports keys present in `en-US` but missing in another non-empty locale (Cluster 10's original concern). This is a secondary report line (`missing in <locale>: <key>`), exit code still gated on suspect-ASCII findings.

**package.json:**
- Add `"audit:i18n": "tsx scripts/audit-i18n.ts"` to `scripts`
- Add `tsx` to `devDependencies` if not present (likely already there via Vite tooling; verify before adding)

**Seed sweep (part of Stage 5):**
- Run `pnpm run audit:i18n` on a fresh Stage 5 branch
- For each flagged entry: translate to the locale's target language and commit. zh-CN and ja-JP are the primary targets; ko-KR / fr-FR / es-ES are already mostly empty strings (which the script treats as missing, not suspect, since empty strings fail the non-empty filter).
- Once the sweep is clean, `pnpm run audit:i18n` exit 0 gates the Stage 5 exit.

### B2 — Plugin tri-state E2E verification prereq

**Approach:** no code change. Spec § Verification documents manual setup.

**Spec addition (this document, § Verification below):** an explicit "E2E prerequisites" subsection listing:

```
Before running the plugin tri-state E2E:

1. Pick an account directory you can install a plugin into, e.g.
   ~/.dot-claude-gui/accounts/work

2. Register a marketplace under that account directory:
   CLAUDE_CONFIG_DIR=~/.dot-claude-gui/accounts/work \
     claude plugin marketplace add anthropics \
     https://github.com/anthropics/claude-plugins.git

3. Install a test plugin:
   CLAUDE_CONFIG_DIR=~/.dot-claude-gui/accounts/work \
     claude plugin install typescript-lsp

4. Confirm the plugin appears in the GUI under
   Account mode → work → Plugins.
```

**Rationale for no automation:**
- Adding a dev-mode "Load fixtures" menu pollutes production UI for a single verification flow.
- A setup script (`scripts/setup-e2e-fixtures.sh`) would shell out to `claude` CLI, which has version-dependent flag behavior — high maintenance cost for a one-time gate.
- The Stage 4 acceptance gap was that no one had documented the prereq, not that the prereq itself was complex.

### B3 — UnboundHint dead-code removal

**Approach:** delete the unreachable component and branch.

Files to modify:
- **Delete** `src/lib/components/project-mode/UnboundHint.svelte` (28 lines)
- **Edit** `src/lib/components/project-mode/ProjectModeView.svelte`:
  - Locate the `{:else if !isBound && activeFacet !== "binding"}` branch around line 58-59 (verify before editing)
  - Remove the branch — when a project is unbound, the top tabs disable all facets except `binding`, so this branch can never be reached
  - Remove the now-unused `UnboundHint` import

**Keep** the i18n key `projectMode.binding.unboundHint` — still consumed by `BindingFacet.svelte:69` in the unbound banner.

Net change: ~30 lines deleted, no UX change (BindingFacet banner covers the original UX).

## Cluster C — Consistency cleanups

### C1 — appsettings.appName derived from package.json

**Files:**
- `src/lib/components/appsettings/AppSettingsView.svelte:83` — replace `{t("appsettings.appName")}` with `{pkg.name}`. Add `import pkg from "../../../../package.json"` if not already imported in this file (the About section already imports `package.json` for version/repo).
- Delete `"appsettings.appName"` key from all 6 locale files: `zh-CN.json`, `en-US.json`, `ja-JP.json`, `ko-KR.json`, `fr-FR.json`, `es-ES.json`.

Behavior: gear panel > About displays `dot-claude-gui`, sourced from `package.json#name`. Renaming `package.json#name` propagates to the UI.

### C2 — RuntimeEditor.svelte:138 teammateMode typing

**File:** `src/lib/components/settings/RuntimeEditor.svelte:138`

**Current:** the init object at line 138 includes `teammateMode` without explicit typing or `satisfies` clause, while line 157 (the form-bound assignment) was already tightened in Stage 4.

**Change:** apply the same narrowing pattern at line 138 — either:
- `teammateMode: undefined as Settings["teammateMode"] | undefined,` OR
- Annotate the entire init object with `satisfies Partial<Settings>`

Pick whichever requires fewer surrounding edits; both achieve the same type strictness.

Single-line change. No behavior impact. svelte-check baseline (0 errors) preserved.

### C3 — activeScope dead field removal

**File:** `src/lib/stores/config.svelte.ts:13`

**Current:** `readonly activeScope = "user" as const;` — left as constant during Stage 4 so that any remaining comparator readers wouldn't break.

**Plan:**
1. Grep all consumers: `grep -rn "activeScope" src/`. Expected callers (per Stage 4 plan line 378): `MemoryList.svelte`, `ClaudeMdList.svelte`. Verify completeness at implementation time.
2. For each caller:
   - If the comparator (`if (activeScope === "user")` or `activeScope === "project"`) only ever takes the user-branch now: delete the branch, inline the user-side code, remove the import / field reference.
3. After no callers remain: delete `activeScope` from `configStore`.

If a comparator turns out to still be load-bearing (unlikely given Stage 4's scope collapse), keep it and update the spec — but the expectation is full removal.

## Out of scope (Stage 5)

Carried forward from Phase 8 spec § Out of scope (lines 237-244), still excluded:
- Shared plugin pool with symlinks
- `dotclaude-launch` CLI binary
- All-accounts bulk-edit settings mode
- Per-project `.claude/dotclaude.json` binding file
- Migration from existing `ccs` instances
- Backwards-compat shims / field-doubling

New Stage 5 exclusions:
- **Project layer support for the 6 omitted Settings sections** (`permissions`, `sandbox`, `statusLine`, `mcpServers`, `pluginsMarketplace`, `general`) — A1 deliberately surfaces only `runtime` / `environment` / `hooks` / `advanced`. Power users edit the rest via the `advanced` raw JSON section.
- **CI integration of `audit:i18n`** — script lands and gates Stage 5 exit manually, but no GitHub Actions workflow yet. Defer to a future stage.
- **B2 fixture automation** — no dev menu, no setup script. Manual prereq doc only.

## Verification

### E2E prerequisites (Cluster B2)

Before running the plugin tri-state E2E in the Verification flow below, complete the manual setup documented in § Cluster B / B2 above. Without this prereq, steps 2-4 of the Stage 4 § End-to-end flow have no plugin data to exercise.

### Per-cluster acceptance

**A1 — Sectioned project settings:**
1. Account mode > Settings: all 10 sub-sections render identically to Stage 4 baseline (visual regression by hand)
2. Project mode > bound project > Settings: section nav shows exactly 4 sections (`runtime`, `environment`, `hooks`, `advanced`)
3. Edit a runtime field → dirty indicator appears → click Save → re-open the project → field persists (verified by inspecting `<project>/.claude/settings.json`)
4. Advanced JSON section accepts arbitrary JSON, round-trips on save / reload
5. Switching sections within project Settings preserves unsaved edits until explicit Save or Revert

**A2 — Stale-path Update action:**
1. Bind a project to an account → no behavior change vs Stage 4
2. From shell: `mv <project> <project>-renamed` while GUI is open → file watcher fires → `StalePathBanner` (already rendered for stale paths) now shows `Update path…` + `Remove` buttons; facet tabs remain disabled with `stalePathBlocked` empty state (existing behavior preserved)
3. Click `Update path…` → native folder picker opens with default at parent dir → pick `<project>-renamed` → toast confirms update → stale banner gone → `~/.dot-claude-gui/config.json` reflects new path in `known_projects` and `projects` map → previously-bound account preserved
4. `modeStore.selectedProject` follows: user remains on the (now-renamed) project in the sidebar
5. Cancel the folder picker → no IPC call, no state change
6. Negative path: `update_project_path` rejects (a) non-existent `new_path`, (b) `new_path` already in `known_projects`, (c) `old_path` not in `known_projects`. Each surfaces as a toast error.

**B1 — i18n audit:**
1. `pnpm run audit:i18n` on Stage 5 HEAD exits 0 with empty report
2. Manually inject `"foo.bar": "Hello English"` into `src/lib/locales/zh-CN.json` and matching `"foo.bar": "Hello English"` into `en-US.json` → re-run → exits 1, output contains `zh-CN: foo.bar = Hello English` → revert change
3. Whitelist works: add `"foo.bar"` to `scripts/audit-i18n.whitelist.json#keys` → re-run with injected value → exits 0 → revert

**B2 — Plugin tri-state prereq:**
1. Following the prereq steps in this spec, the Stage 4 § End-to-end flow steps 2-4 (account enable plugin → project disable plugin → project Effective shows project source) all complete end-to-end on a freshly-prepared machine

**B3 — UnboundHint dead-code removal:**
1. `src/lib/components/project-mode/UnboundHint.svelte` does not exist
2. `grep -rn "UnboundHint" src/` returns no matches
3. Unbound project UX unchanged: BindingFacet banner still explains why facets are greyed

**C1 — appName derivation:**
1. Gear panel > About shows `dot-claude-gui`
2. Locally change `package.json#name` to `dot-claude-gui-test` → restart `pnpm tauri dev` → About label follows → revert
3. `grep -rn "appsettings.appName" src/` returns no matches

**C2 — teammateMode typing:**
1. `pnpm svelte-check` 0 errors on Stage 5 HEAD
2. Manually delete the type annotation added in C2 → `pnpm svelte-check` flags line 138 → revert (verifies the annotation is doing work)

**C3 — activeScope removal:**
1. `grep -rn "activeScope" src/` returns no matches in active code (comments-only allowed if any)
2. `MemoryList.svelte` and `ClaudeMdList.svelte` no longer reference `activeScope`
3. `configStore.activeScope` field is absent
4. cargo test + svelte-check + pnpm build all clean

### Stage 5 exit gate

All must be green:
- `cargo test --workspace` — 155 passed + 1 ignored (baseline parity)
- `pnpm svelte-check` — 0 errors
- `pnpm run audit:i18n` — exit 0
- `pnpm build` — clean
- Manual per-cluster acceptance above

### Out-of-scope verification (deferred)

Not gated by Stage 5:
- Project-layer support for the 6 omitted Settings sections (deferred indefinitely; raise only if a real user need surfaces)
- CI workflow that runs `pnpm run audit:i18n` on PRs
- Automated plugin fixture setup
