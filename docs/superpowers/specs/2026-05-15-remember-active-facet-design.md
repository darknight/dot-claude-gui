# Remember active facet per account / per project

**Status:** Draft
**Author:** Eric Yao + Claude
**Date:** 2026-05-15

## Problem

When the user switches between accounts (or between projects), the active
facet (top-level tab) does not follow what they were viewing in each
account/project. Concretely:

- Account mode's `activeFacet` lives as component-local `$state` inside
  `AccountModeView.svelte`. The component is not unmounted when
  `selectedAccount` changes, so the value carries over from the previous
  account — work was on Plugins → switching to myself still shows Plugins,
  even though that's not what myself was last on.
- Project mode's `selectedProjectFacet` is persisted to localStorage but as
  a single global key — all projects share one remembered facet.
- Sub-tab state inside Plugins / Settings facets is component-local and
  has no persistence at all.

The user wants per-account and per-project memory of the active facet
**and** the active sub-tab inside multi-sectioned facets.

## Goals

1. Switching accounts restores each account's last-active facet (and its
   sub-tab inside Settings / Plugins facets).
2. Switching projects restores each project's last-active facet (and its
   sub-tab inside Settings facet).
3. State survives app restarts (localStorage).
4. Stale entries (account deleted, project removed) are pruned without
   adding cleanup hooks to delete IPC paths.

## Non-goals

- Cross-device sync / cross-reinstall persistence (would need a disk file
  + IPC write per tab click; not worth the IO for UI state).
- Vitest unit test suite for `modeStore` (project has no frontend test
  infra; out of scope for this change).
- Memorizing scroll position, form drafts, or any sub-component state
  below the sub-tab level.

## Affected sub-tab locations

Five distinct pieces of state need persistence per account / per project:

| Mode    | Facet             | Sub-tab state            | Current default |
|---------|-------------------|--------------------------|-----------------|
| Account | (top-level)       | facet                    | `overview`      |
| Account | Settings          | `settingsSection`        | `general`       |
| Account | Plugins           | `pluginsSection`         | `installed`     |
| Project | (top-level)       | facet                    | `binding`       |
| Project | Settings          | `settingsSection`        | `runtime`       |

## Data model

New localStorage key `dot-claude-gui-mode-v2` (keeps `…-v1` untouched so
the user can roll back).

```ts
type AccountFacetKey =
  | "overview" | "settings" | "plugins" | "skills"
  | "claudemd" | "memory" | "mcp";

type ProjectFacetKey =
  | "binding" | "launch" | "plugins" | "settings"
  | "memory" | "claudemd" | "effective";

interface PerAccountUi {
  facet: AccountFacetKey;
  settingsSection?: string;
  pluginsSection?: string;
}

interface PerProjectUi {
  facet: ProjectFacetKey;
  settingsSection?: string;
}

interface PersistedModeV2 {
  version: 2;
  mode: GuiMode;
  selectedAccount: string | null;
  selectedProject: string | null;
  accounts: Record<string, PerAccountUi>;  // key = account name
  projects: Record<string, PerProjectUi>;  // key = project path
}
```

Keys:
- **Account key = `name`** (the same identifier `set_active_account` uses
  on the backend).
- **Project key = `path`** (the same identifier `bind_project` uses).

### Migration v1 → v2

On first load:
1. If `mode-v2` exists and parses → use it.
2. Else if `mode-v1` exists and parses → carry over `mode`,
   `selectedAccount`, `selectedProject`. **Drop** v1's global
   `selectedProjectFacet`. Start with `accounts: {}` and `projects: {}`.
3. Else → full defaults.

We deliberately discard v1's global facet rather than seeding all projects
with it. The user is asking for per-project memory because the global
value never matched what they wanted — propagating it would replant the
exact value they're trying to escape.

The v1 entry is left in localStorage as a soft backup but never read
again after v2 is written.

## Store API

`modeStore` (src/lib/stores/mode.svelte.ts) grows imperative getters and
setters. Getters return values, not reactive refs — callers wrap them in
`$derived` at the call site.

```ts
// Account-side
accountFacet(name: string | null): AccountFacetKey
setAccountFacet(name: string, facet: AccountFacetKey): void
accountSubsection(
  name: string | null,
  key: "settingsSection" | "pluginsSection",
): string | undefined
setAccountSubsection(
  name: string,
  key: "settingsSection" | "pluginsSection",
  val: string,
): void

// Project-side
projectFacet(path: string | null): ProjectFacetKey
setProjectFacet(path: string, facet: ProjectFacetKey): void
projectSubsection(path: string | null, key: "settingsSection"): string | undefined
setProjectSubsection(path: string, key: "settingsSection", val: string): void

// Cleanup
pruneStale(
  validAccountNames: Set<string>,
  validProjectPaths: Set<string>,
): void
```

The legacy `selectedProjectFacet` / `setSelectedProjectFacet` API is
removed (they had no per-project meaning).

### Reactivity model

The internal storage is two `$state` records (`accounts`, `projects`).
Getters read them, so consumer `$derived` re-runs when the underlying
record mutates. Setters perform an immutable replace of the relevant slot
(`this.accounts = { ...this.accounts, [name]: { ...prev, facet } }`) so
Svelte 5's shallow-equality tracking notices the change.

## Component integration

| File | Before | After |
|---|---|---|
| `account-mode/AccountModeView.svelte` | `let activeFacet = $state<Facet>("overview")` | `const activeFacet = $derived(modeStore.accountFacet(modeStore.selectedAccount))`; tab onclick calls `setAccountFacet` |
| `account-mode/PluginsFacet.svelte` | `let active = $state("installed")` | `$derived(modeStore.accountSubsection(name, "pluginsSection") ?? "installed")`; tab onclick calls `setAccountSubsection` |
| `account-mode/SettingsFacet.svelte` | `let active = $state("general")` | Same pattern as PluginsFacet |
| `project-mode/ProjectModeView.svelte` | `$derived(modeStore.selectedProjectFacet)` | `$derived(modeStore.projectFacet(modeStore.selectedProject))` |
| `project-mode/ProjectSettingsFacet.svelte` | `let activeSection = $state("runtime")` + `bind:activeSection` to `SectionedSettings` | `$derived(modeStore.projectSubsection(path, "settingsSection") ?? "runtime")`; pass `onChange` callback to `SectionedSettings` |

### SectionedSettings interface change

`SectionedSettings` is the only component that uses `bind:activeSection`,
and `ProjectSettingsFacet` is its sole caller. `$bindable` is
incompatible with `$derived` (the bind target must be writable).

Change the interface: drop `$bindable` on `activeSection`, accept it as a
one-way prop, and add `onChange: (section: string) => void`. The tab
onclick handler inside `SectionedSettings` calls `onChange(section.id)`
instead of mutating the prop directly.

Account-side `SettingsFacet` (uses `SettingsEditor` with one-way
`activeSection`) and `PluginsFacet` (uses `PluginsModule` with one-way
`activeSection`) need no interface changes — just convert their local
`active` state into a `$derived` of `modeStore.accountSubsection(...)` and
have the tab onclick call the corresponding setter.

## Cleanup policy

Lazy. On app load, after `accountsStore.loadAccounts()` and
`projectsStore.loadProjects()` resolve in `App.svelte`'s `onMount`, call:

```ts
modeStore.pruneStale(
  new Set(accountsStore.accounts.map(a => a.name)),
  new Set(projectsStore.projects.map(p => p.path)),
);
```

`pruneStale` drops any keys not in the valid sets and persists once if
anything changed. No coupling to `delete_account` / `remove_project` IPC
flows. Side effect: renaming an account or moving a project path loses
its remembered tab — acceptable; UI state is cheap to recreate.

## Error handling

| Condition | Behavior |
|---|---|
| `localStorage.getItem` throws | Use in-memory defaults; subsequent setters silently fail |
| v2 JSON parse error | Try v1 path; if also fails, defaults |
| v2 schema wrong shape | Defaults |
| Setter `setItem` throws | Swallow; in-memory state still updates so the session works |
| Unknown facet/subsection value in stored blob | Fall back to default for that key |

All of these go through `try/catch` and never throw to the caller. The
existing `loadPersisted` already takes this approach.

## Acceptance criteria

Manual verification after implementation:

1. **Cross-account facet memory** — work on Plugins/Marketplace, switch to
   myself (lands on its remembered facet or `overview`), switch back to
   work, still on Plugins/Marketplace.
2. **Cross-project facet memory** — project A on Settings/runtime, switch
   to project B, switch back to A, still on Settings/runtime.
3. **Stale prune** — delete an account via the GUI, restart the app,
   confirm the localStorage `accounts` map no longer contains that name.
4. **Fresh install** — clear localStorage, launch app, no errors;
   defaults are Overview / Binding; toggling tabs persists immediately
   (verify via DevTools localStorage panel).
5. **Migration** — manually plant a `mode-v1` blob with a non-default
   `selectedProjectFacet`, launch app, confirm new `mode-v2` is written
   without the old facet, projects all default to `binding`.

## Build / quality gates

- `cargo test --workspace` (no Rust changes, but run to confirm no
  unrelated regressions)
- `pnpm exec svelte-check` 0 errors
- `pnpm build` succeeds
- Bump version 1.0.0-alpha.4 → 1.0.0-alpha.5

## Out of scope

- Rust / IPC changes
- Backend cleanup hooks on account/project delete
- Frontend test infra
- Memorizing scroll, form drafts, transient UI below sub-tab level
