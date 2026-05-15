# Remember Active Facet Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make active facet (and active sub-tab inside Settings / Plugins facets) survive account/project switches, with per-account and per-project memory.

**Architecture:** Extend the existing `modeStore` (already on localStorage). Add a `mode-v2` schema with `accounts: Record<name, PerAccountUi>` and `projects: Record<path, PerProjectUi>` maps. Each component that currently holds local `$state` for its active tab switches to a `$derived` of a modeStore getter + an onclick that calls the corresponding setter. Lazy cleanup on app load drops entries for accounts/projects that no longer exist. No backend / IPC changes.

**Tech Stack:** Svelte 5 runes (`$state`, `$derived`, `$effect`), TypeScript strict, localStorage. Spec at `docs/superpowers/specs/2026-05-15-remember-active-facet-design.md`.

**Verification model:** Frontend has no test suite (per CLAUDE.md). Each task ends with `pnpm exec svelte-check --tsconfig ./tsconfig.json` (must be 0 errors) and a commit. Final task runs `pnpm build` and the manual acceptance checklist from the spec.

**HMR caveat (CLAUDE.md gotcha #1):** Adding/changing runes in `<script>` does NOT reliably hot-reload. After Task 1 you MUST kill and restart `pnpm tauri dev` before manually testing — don't trust HMR for store rune changes.

---

### Task 1: Extend modeStore with v2 schema, migration, and new API

**Files:**
- Modify: `src/lib/stores/mode.svelte.ts` (full rewrite)

This task is additive — the new API exists alongside the old `selectedProjectFacet` / `setSelectedProjectFacet` field+setter so nothing breaks. Task 8 removes the legacy field after all callers migrate.

- [ ] **Step 1: Replace the file contents**

Write the full new `mode.svelte.ts`:

```ts
import type { GuiMode } from "$lib/api/types";

const STORAGE_KEY_V1 = "dot-claude-gui-mode-v1";
const STORAGE_KEY_V2 = "dot-claude-gui-mode-v2";

export type ProjectFacetKey =
  | "binding"
  | "launch"
  | "plugins"
  | "settings"
  | "memory"
  | "claudemd"
  | "effective";

export type AccountFacetKey =
  | "overview"
  | "settings"
  | "plugins"
  | "skills"
  | "claudemd"
  | "memory"
  | "mcp";

const VALID_PROJECT_FACETS: readonly ProjectFacetKey[] = [
  "binding", "launch", "plugins", "settings", "memory", "claudemd", "effective",
];

const VALID_ACCOUNT_FACETS: readonly AccountFacetKey[] = [
  "overview", "settings", "plugins", "skills", "claudemd", "memory", "mcp",
];

const DEFAULT_ACCOUNT_FACET: AccountFacetKey = "overview";
const DEFAULT_PROJECT_FACET: ProjectFacetKey = "binding";

export type AccountSubsectionKey = "settingsSection" | "pluginsSection";
export type ProjectSubsectionKey = "settingsSection";

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
  accounts: Record<string, PerAccountUi>;
  projects: Record<string, PerProjectUi>;
}

function defaultPersisted(): PersistedModeV2 {
  return {
    version: 2,
    mode: "account",
    selectedAccount: null,
    selectedProject: null,
    accounts: {},
    projects: {},
  };
}

function loadPersisted(): PersistedModeV2 {
  // v2 first.
  try {
    const raw = localStorage.getItem(STORAGE_KEY_V2);
    if (raw) {
      const parsed = JSON.parse(raw);
      if (parsed && parsed.version === 2) {
        return {
          version: 2,
          mode: parsed.mode === "project" ? "project" : "account",
          selectedAccount: typeof parsed.selectedAccount === "string" ? parsed.selectedAccount : null,
          selectedProject: typeof parsed.selectedProject === "string" ? parsed.selectedProject : null,
          accounts: sanitizeAccounts(parsed.accounts),
          projects: sanitizeProjects(parsed.projects),
        };
      }
    }
  } catch {
    // fall through
  }

  // v1 migration: keep mode + selections; drop global selectedProjectFacet on purpose
  // (per-project memory is the whole point of v2).
  try {
    const raw = localStorage.getItem(STORAGE_KEY_V1);
    if (raw) {
      const parsed = JSON.parse(raw);
      return {
        version: 2,
        mode: parsed.mode === "project" ? "project" : "account",
        selectedAccount: typeof parsed.selectedAccount === "string" ? parsed.selectedAccount : null,
        selectedProject: typeof parsed.selectedProject === "string" ? parsed.selectedProject : null,
        accounts: {},
        projects: {},
      };
    }
  } catch {
    // fall through
  }

  return defaultPersisted();
}

function sanitizeAccounts(input: unknown): Record<string, PerAccountUi> {
  if (!input || typeof input !== "object") return {};
  const out: Record<string, PerAccountUi> = {};
  for (const [k, v] of Object.entries(input as Record<string, unknown>)) {
    if (!v || typeof v !== "object") continue;
    const raw = v as Record<string, unknown>;
    const facet = typeof raw.facet === "string" && VALID_ACCOUNT_FACETS.includes(raw.facet as AccountFacetKey)
      ? (raw.facet as AccountFacetKey)
      : DEFAULT_ACCOUNT_FACET;
    out[k] = {
      facet,
      settingsSection: typeof raw.settingsSection === "string" ? raw.settingsSection : undefined,
      pluginsSection: typeof raw.pluginsSection === "string" ? raw.pluginsSection : undefined,
    };
  }
  return out;
}

function sanitizeProjects(input: unknown): Record<string, PerProjectUi> {
  if (!input || typeof input !== "object") return {};
  const out: Record<string, PerProjectUi> = {};
  for (const [k, v] of Object.entries(input as Record<string, unknown>)) {
    if (!v || typeof v !== "object") continue;
    const raw = v as Record<string, unknown>;
    const facet = typeof raw.facet === "string" && VALID_PROJECT_FACETS.includes(raw.facet as ProjectFacetKey)
      ? (raw.facet as ProjectFacetKey)
      : DEFAULT_PROJECT_FACET;
    out[k] = {
      facet,
      settingsSection: typeof raw.settingsSection === "string" ? raw.settingsSection : undefined,
    };
  }
  return out;
}

class ModeStore {
  private _persisted = loadPersisted();
  mode = $state<GuiMode>(this._persisted.mode);
  selectedAccount = $state<string | null>(this._persisted.selectedAccount);
  selectedProject = $state<string | null>(this._persisted.selectedProject);
  accounts = $state<Record<string, PerAccountUi>>(this._persisted.accounts);
  projects = $state<Record<string, PerProjectUi>>(this._persisted.projects);

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

  // ── Account-side facet/subsection ─────────────────────────────────

  accountFacet(name: string | null): AccountFacetKey {
    if (!name) return DEFAULT_ACCOUNT_FACET;
    return this.accounts[name]?.facet ?? DEFAULT_ACCOUNT_FACET;
  }

  setAccountFacet(name: string, facet: AccountFacetKey): void {
    const prev = this.accounts[name] ?? { facet: DEFAULT_ACCOUNT_FACET };
    this.accounts = { ...this.accounts, [name]: { ...prev, facet } };
    this.persist();
  }

  accountSubsection(name: string | null, key: AccountSubsectionKey): string | undefined {
    if (!name) return undefined;
    return this.accounts[name]?.[key];
  }

  setAccountSubsection(name: string, key: AccountSubsectionKey, val: string): void {
    const prev = this.accounts[name] ?? { facet: DEFAULT_ACCOUNT_FACET };
    this.accounts = { ...this.accounts, [name]: { ...prev, [key]: val } };
    this.persist();
  }

  // ── Project-side facet/subsection ─────────────────────────────────

  projectFacet(path: string | null): ProjectFacetKey {
    if (!path) return DEFAULT_PROJECT_FACET;
    return this.projects[path]?.facet ?? DEFAULT_PROJECT_FACET;
  }

  setProjectFacet(path: string, facet: ProjectFacetKey): void {
    const prev = this.projects[path] ?? { facet: DEFAULT_PROJECT_FACET };
    this.projects = { ...this.projects, [path]: { ...prev, facet } };
    this.persist();
  }

  projectSubsection(path: string | null, key: ProjectSubsectionKey): string | undefined {
    if (!path) return undefined;
    return this.projects[path]?.[key];
  }

  setProjectSubsection(path: string, key: ProjectSubsectionKey, val: string): void {
    const prev = this.projects[path] ?? { facet: DEFAULT_PROJECT_FACET };
    this.projects = { ...this.projects, [path]: { ...prev, [key]: val } };
    this.persist();
  }

  // ── Cleanup ───────────────────────────────────────────────────────

  pruneStale(validAccountNames: Set<string>, validProjectPaths: Set<string>): void {
    let mutated = false;
    const nextAccounts: Record<string, PerAccountUi> = {};
    for (const [k, v] of Object.entries(this.accounts)) {
      if (validAccountNames.has(k)) nextAccounts[k] = v;
      else mutated = true;
    }
    const nextProjects: Record<string, PerProjectUi> = {};
    for (const [k, v] of Object.entries(this.projects)) {
      if (validProjectPaths.has(k)) nextProjects[k] = v;
      else mutated = true;
    }
    if (mutated) {
      this.accounts = nextAccounts;
      this.projects = nextProjects;
      this.persist();
    }
  }

  private persist(): void {
    try {
      const snapshot: PersistedModeV2 = {
        version: 2,
        mode: this.mode,
        selectedAccount: this.selectedAccount,
        selectedProject: this.selectedProject,
        accounts: this.accounts,
        projects: this.projects,
      };
      localStorage.setItem(STORAGE_KEY_V2, JSON.stringify(snapshot));
    } catch {
      // localStorage unavailable — ignore
    }
  }
}

export const modeStore = new ModeStore();
```

Notes:
- The legacy `selectedProjectFacet` / `setSelectedProjectFacet` are **gone** in this rewrite. That breaks `ProjectModeView.svelte` temporarily — Task 3 fixes that callsite. svelte-check at the end of this task WILL report errors from `ProjectModeView`. That is expected; do not commit Task 1 alone — combine with Task 3 if you prefer a green tree per commit. We sequence Task 1 → Task 3 first so the only red interval is one task wide.

- [ ] **Step 2: Run svelte-check, expect errors only from ProjectModeView**

Run:

```bash
pnpm exec svelte-check --tsconfig ./tsconfig.json 2>&1 | grep -E "ERROR|ProjectModeView"
```

Expected: errors mentioning `selectedProjectFacet` / `setSelectedProjectFacet` in `ProjectModeView.svelte`. Any other ERROR means the rewrite has a bug — fix before continuing.

- [ ] **Step 3: Do NOT commit yet** — proceed straight to Task 3 which fixes ProjectModeView. Task 1 + Task 3 commit together.

---

### Task 2: Update SectionedSettings to use onChange instead of $bindable

**Files:**
- Modify: `src/lib/components/shared/SectionedSettings.svelte:7-19,28-29`

ProjectSettingsFacet is the only caller. We change the contract here AND update that caller in the same task.

- [ ] **Step 1: Edit SectionedSettings props block**

Replace lines 7-19 (the `let { … } = $props()` block) with:

```ts
  let {
    sections,
    activeSection,
    onChange,
    isDirty,
    error,
    content,
  }: {
    sections: Section[];
    activeSection: string;
    onChange: (id: string) => void;
    isDirty: boolean;
    error: string | null;
    content: Snippet<[string]>;
  } = $props();
```

- [ ] **Step 2: Edit the tab onclick**

Replace line 29 (`onclick={() => (activeSection = section.id)}`) with:

```svelte
        onclick={() => onChange(section.id)}
```

- [ ] **Step 3: Update ProjectSettingsFacet caller**

Edit `src/lib/components/project-mode/ProjectSettingsFacet.svelte` line 85, replace:

```svelte
    <SectionedSettings {sections} bind:activeSection {isDirty} {error}>
```

with:

```svelte
    <SectionedSettings
      {sections}
      {activeSection}
      onChange={(id) => (activeSection = id)}
      {isDirty}
      {error}
    >
```

This still uses the local `$state activeSection` — Task 7 swaps it to modeStore.

- [ ] **Step 4: svelte-check**

Run: `pnpm exec svelte-check --tsconfig ./tsconfig.json 2>&1 | grep -E "ERROR.*SectionedSettings|ERROR.*ProjectSettingsFacet"`
Expected: no output (no errors in these files).

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/shared/SectionedSettings.svelte \
        src/lib/components/project-mode/ProjectSettingsFacet.svelte
git commit -m "refactor(SectionedSettings): replace \$bindable activeSection with onChange prop"
```

---

### Task 3: Wire ProjectModeView and Task 1 commit

**Files:**
- Modify: `src/lib/components/project-mode/ProjectModeView.svelte:26,53`

- [ ] **Step 1: Replace the activeFacet derivation**

In `ProjectModeView.svelte` line 26, change:

```ts
  const activeFacet = $derived(modeStore.selectedProjectFacet);
```

to:

```ts
  const activeFacet = $derived(modeStore.projectFacet(modeStore.selectedProject));
```

- [ ] **Step 2: Replace the onclick**

In `ProjectModeView.svelte` line 53, change:

```svelte
          onclick={() => modeStore.setSelectedProjectFacet(f.key)}
```

to:

```svelte
          onclick={() => {
            if (modeStore.selectedProject) {
              modeStore.setProjectFacet(modeStore.selectedProject, f.key);
            }
          }}
```

- [ ] **Step 3: svelte-check across the whole project**

Run: `pnpm exec svelte-check --tsconfig ./tsconfig.json 2>&1 | tail -5`
Expected: `0 ERRORS` on the final summary line. (Warnings about unrelated files are fine.)

- [ ] **Step 4: Commit Tasks 1 + 3 together**

```bash
git add src/lib/stores/mode.svelte.ts \
        src/lib/components/project-mode/ProjectModeView.svelte
git commit -m "feat(modeStore): v2 schema with per-account / per-project state

Adds accounts and projects maps to mode-v2 in localStorage. New
getter/setter API: accountFacet/setAccountFacet, projectFacet/
setProjectFacet, *Subsection variants, and pruneStale. v1's
global selectedProjectFacet is dropped on migration; per-project
memory is the whole point of v2.

ProjectModeView now resolves its active facet from
projectFacet(selectedProject). Other callers migrate in
follow-up commits."
```

---

### Task 4: Per-account facet in AccountModeView

**Files:**
- Modify: `src/lib/components/account-mode/AccountModeView.svelte:31,77,80`

- [ ] **Step 1: Replace the local `activeFacet` declaration**

Find the line (around line 31):

```ts
  let activeFacet = $state<Facet>("overview");
```

Replace with:

```ts
  const activeFacet = $derived<Facet>(
    modeStore.accountFacet(modeStore.selectedAccount) as Facet,
  );
```

The `as Facet` cast is safe — `accountFacet` returns one of the same 7 keys defined in `AccountFacetKey`.

- [ ] **Step 2: Replace the tab onclick**

Find the line:

```svelte
          onclick={() => { activeFacet = f.id; }}
```

Replace with:

```svelte
          onclick={() => {
            if (modeStore.selectedAccount) {
              modeStore.setAccountFacet(modeStore.selectedAccount, f.id);
            }
          }}
```

- [ ] **Step 3: svelte-check**

Run: `pnpm exec svelte-check --tsconfig ./tsconfig.json 2>&1 | tail -3`
Expected: `0 ERRORS`.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/account-mode/AccountModeView.svelte
git commit -m "feat(account-mode): per-account active facet memory"
```

---

### Task 5: Per-account sub-tab in account SettingsFacet

**Files:**
- Modify: `src/lib/components/account-mode/SettingsFacet.svelte:20,32`

- [ ] **Step 1: Add modeStore import**

Add to the imports near the top of the `<script>` block:

```ts
  import { modeStore } from "$lib/stores/mode.svelte";
```

- [ ] **Step 2: Replace the `active` declaration**

Find:

```ts
  let active = $state("general");
```

Replace with:

```ts
  const active = $derived(
    modeStore.accountSubsection(modeStore.selectedAccount, "settingsSection") ?? "general",
  );
```

- [ ] **Step 3: Replace the tab onclick**

Find:

```svelte
        onclick={() => { active = section.id; }}
```

Replace with:

```svelte
        onclick={() => {
          if (modeStore.selectedAccount) {
            modeStore.setAccountSubsection(modeStore.selectedAccount, "settingsSection", section.id);
          }
        }}
```

- [ ] **Step 4: svelte-check**

Run: `pnpm exec svelte-check --tsconfig ./tsconfig.json 2>&1 | tail -3`
Expected: `0 ERRORS`.

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/account-mode/SettingsFacet.svelte
git commit -m "feat(account-mode): per-account settings section memory"
```

---

### Task 6: Per-account sub-tab in account PluginsFacet

**Files:**
- Modify: `src/lib/components/account-mode/PluginsFacet.svelte:11,25`

- [ ] **Step 1: Add modeStore import**

Add to imports:

```ts
  import { modeStore } from "$lib/stores/mode.svelte";
```

- [ ] **Step 2: Replace the `active` declaration**

Find:

```ts
  let active = $state("installed");
```

Replace with:

```ts
  const active = $derived(
    modeStore.accountSubsection(modeStore.selectedAccount, "pluginsSection") ?? "installed",
  );
```

- [ ] **Step 3: Replace the tab onclick**

Find:

```svelte
        onclick={() => { active = section.id; }}
```

Replace with:

```svelte
        onclick={() => {
          if (modeStore.selectedAccount) {
            modeStore.setAccountSubsection(modeStore.selectedAccount, "pluginsSection", section.id);
          }
        }}
```

- [ ] **Step 4: svelte-check**

Run: `pnpm exec svelte-check --tsconfig ./tsconfig.json 2>&1 | tail -3`
Expected: `0 ERRORS`.

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/account-mode/PluginsFacet.svelte
git commit -m "feat(account-mode): per-account plugins section memory"
```

---

### Task 7: Per-project sub-tab in ProjectSettingsFacet

**Files:**
- Modify: `src/lib/components/project-mode/ProjectSettingsFacet.svelte:18,85-91`

Task 2 already converted this file to use `onChange`. Now we delegate the state to modeStore.

- [ ] **Step 1: Add modeStore import**

Add to imports at the top of the `<script>` block:

```ts
  import { modeStore } from "$lib/stores/mode.svelte";
```

- [ ] **Step 2: Replace the `activeSection` declaration**

Find:

```ts
  let activeSection = $state("runtime");
```

Replace with:

```ts
  const activeSection = $derived(
    modeStore.projectSubsection(path, "settingsSection") ?? "runtime",
  );
```

- [ ] **Step 3: Update onChange on the SectionedSettings caller**

Find (Task 2's output):

```svelte
    <SectionedSettings
      {sections}
      {activeSection}
      onChange={(id) => (activeSection = id)}
      {isDirty}
      {error}
    >
```

Replace `onChange` to write through modeStore:

```svelte
    <SectionedSettings
      {sections}
      {activeSection}
      onChange={(id) => modeStore.setProjectSubsection(path, "settingsSection", id)}
      {isDirty}
      {error}
    >
```

- [ ] **Step 4: svelte-check**

Run: `pnpm exec svelte-check --tsconfig ./tsconfig.json 2>&1 | tail -3`
Expected: `0 ERRORS`.

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/project-mode/ProjectSettingsFacet.svelte
git commit -m "feat(project-mode): per-project settings section memory"
```

---

### Task 8: Wire pruneStale in App.svelte

**Files:**
- Modify: `src/App.svelte:62-83` (the `onMount` async IIFE)

- [ ] **Step 1: Add modeStore import**

The file already imports modeStore via downstream components but not at top level. Add to imports at the top:

```ts
  import { modeStore } from "$lib/stores/mode.svelte";
```

(If it's already imported via `accountsStore`/`projectsStore` you can skip this — check first with: `grep "modeStore" src/App.svelte`.)

- [ ] **Step 2: Add the pruneStale call**

Inside the `onMount` async IIFE, after the existing `Promise.all([…])` that loads `accountsStore` and `projectsStore`, add:

```ts
      modeStore.pruneStale(
        new Set(accountsStore.accounts.map((a) => a.name)),
        new Set(projectsStore.projects.map((p) => p.path)),
      );
```

It goes right after the `await Promise.all([…])` line and before the `onConfigChanged` listener registration. The exact insertion point is between lines 73 and 74 in the current file (after `accountsStore.loadAccounts()` resolves).

- [ ] **Step 3: svelte-check**

Run: `pnpm exec svelte-check --tsconfig ./tsconfig.json 2>&1 | tail -3`
Expected: `0 ERRORS`.

- [ ] **Step 4: Commit**

```bash
git add src/App.svelte
git commit -m "feat(modeStore): prune stale account/project UI entries on app load"
```

---

### Task 9: Version bump + build + manual verification

**Files:**
- Modify: `package.json:4` (version `1.0.0-alpha.4` → `1.0.0-alpha.5`)
- Modify: `src-tauri/tauri.conf.json:4` (same bump)
- Modify: `Cargo.toml:10` (workspace `version = "1.0.0-alpha.4"` → `"1.0.0-alpha.5"`)

- [ ] **Step 1: Bump all three version files**

Edit `package.json`, `src-tauri/tauri.conf.json`, `Cargo.toml` to change `1.0.0-alpha.4` to `1.0.0-alpha.5`.

Verify:

```bash
grep -n "1.0.0-alpha" package.json src-tauri/tauri.conf.json Cargo.toml
```

Expected: all three lines show `1.0.0-alpha.5`.

- [ ] **Step 2: Run full quality gates**

```bash
pnpm exec svelte-check --tsconfig ./tsconfig.json 2>&1 | tail -3
cargo test --manifest-path src-tauri/Cargo.toml --workspace 2>&1 | tail -5
pnpm build 2>&1 | tail -5
```

Expected:
- svelte-check: `0 ERRORS`
- cargo test: `test result: ok` for each crate
- pnpm build: `✓ built in …s`

- [ ] **Step 3: Restart dev server (HMR caveat)**

Kill any running `pnpm tauri dev` and restart fresh:

```bash
pkill -f "pnpm tauri dev" 2>/dev/null
pnpm tauri dev &
```

This is required because Task 1 added new runes to `modeStore`; HMR alone will not rebuild the reactive graph (CLAUDE.md gotcha #1).

- [ ] **Step 4: Manual acceptance checklist**

Walk through the 5 cases from the spec. Each must pass:

1. **Cross-account facet memory:** Account mode. Select `work`, click Plugins tab, click Marketplace sub-tab. Select `myself` (lands on its remembered facet, likely Overview on first run). Select `work` again — must be on Plugins/Marketplace.

2. **Cross-project facet memory:** Project mode. Select project A, click Settings tab, click Environment section. Select project B (lands on its remembered facet or Binding). Select project A again — must be on Settings/Environment.

3. **Stale prune on app load:** Note an entry in `localStorage.getItem('dot-claude-gui-mode-v2')` under `accounts` for some account. Delete that account via the GUI. Close and reopen the app. Inspect localStorage in DevTools — that account's entry must be gone.

4. **Fresh install:** In DevTools console run `localStorage.clear()`, restart the app. No errors in DevTools console. Account mode lands on Overview. Project mode (after picking a project) lands on Binding. Toggling tabs writes to `dot-claude-gui-mode-v2` immediately.

5. **v1 migration:** In DevTools console:

   ```js
   localStorage.setItem('dot-claude-gui-mode-v1', JSON.stringify({
     mode: 'project',
     selectedAccount: 'work',
     selectedProject: null,
     selectedProjectFacet: 'effective'
   }));
   localStorage.removeItem('dot-claude-gui-mode-v2');
   ```

   Restart the app. Inspect `dot-claude-gui-mode-v2` — must show `version: 2`, `mode: "project"`, `selectedAccount: "work"`, `projects: {}` (effective from v1 was dropped on purpose).

If any case fails, the implementation is wrong — do NOT commit Step 5 until all 5 pass.

- [ ] **Step 5: Commit**

```bash
git add package.json src-tauri/tauri.conf.json Cargo.toml Cargo.lock
git commit -m "chore(version): bump 1.0.0-alpha.4 → 1.0.0-alpha.5

Per-account / per-project active facet memory ships in this build."
```

(Note: `Cargo.lock` updates on the next build — include it if `git status` shows it modified.)

---

## Self-Review Notes

**Spec coverage check:**
- ✅ Data model (PersistedModeV2) — Task 1
- ✅ Migration v1 → v2 — Task 1
- ✅ Store API — Task 1
- ✅ Per-account facet — Task 4
- ✅ Account settings section — Task 5
- ✅ Account plugins section — Task 6
- ✅ Per-project facet — Task 3
- ✅ Project settings section + SectionedSettings change — Tasks 2 + 7
- ✅ Cleanup (pruneStale) — Task 1 (logic) + Task 8 (wiring)
- ✅ Error handling — Task 1 (sanitize helpers, try/catch around storage)
- ✅ Acceptance criteria — Task 9 Step 4
- ✅ Version bump — Task 9

**Placeholder scan:** No TBD / TODO / "similar to" / "implement later" — every step contains the exact code or command.

**Type consistency check:**
- `AccountFacetKey`, `ProjectFacetKey`, `AccountSubsectionKey`, `ProjectSubsectionKey` defined in Task 1 and used consistently in Tasks 3–7.
- `accountFacet` / `setAccountFacet` / `accountSubsection` / `setAccountSubsection` / `projectFacet` / `setProjectFacet` / `projectSubsection` / `setProjectSubsection` / `pruneStale` — names unchanged across tasks.
- `SectionedSettings` interface change in Task 2 (`onChange` added, `$bindable` removed) is referenced consistently in Task 7.

**Commit-tree state check:**
- Only Task 1 creates a temporarily-broken tree, and Task 3 closes that gap in the same commit. Tasks 2, 4, 5, 6, 7, 8, 9 each leave a green tree.
