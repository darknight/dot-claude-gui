# Phase 8 Stage 5 — Backlog Cleanup + Small Features — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the Stage 4 backlog — sectioned project Settings UI (A1), stale-binding Update action (A2), i18n audit script (B1), plugin tri-state verification prereq doc (B2), UnboundHint dead-code removal (B3), and three consistency cleanups (C1 appName, C2 teammateMode typing, C3 activeScope removal).

**Architecture:** Twelve tasks ordered low-risk-first: cleanups → audit infrastructure → A2 small feature → A1 larger feature → seed sweep → exit gate. Frontend tasks verify via `pnpm svelte-check` + visual smoke (no frontend test suite per CLAUDE.md). Backend A2 uses TDD with cargo tests against the existing `gui_projects` module pattern. The new shared `SectionedSettings.svelte` component extracts the section-nav + dirty/error chrome currently inlined in `SettingsEditor.svelte`; project sub-editors are new dumb form components driven by a local `ProjectSettingsState`, not by `configStore` (which Stage 4 collapsed to user-only).

**Tech Stack:** Svelte 5 runes (`$state` / `$derived` / `$effect`), Tauri 2.0 IPC + `@tauri-apps/plugin-dialog` (already registered at `src-tauri/src/lib.rs:52`), Rust workspace (`claude-config`, `claude-types`), TypeScript strict, pnpm, `tsx` for the audit script.

**Spec:** `docs/superpowers/specs/2026-05-13-phase8-stage5-design.md` (commit `f8680a2`).

---

## File Structure

**New files:**
- `src/lib/components/shared/SectionedSettings.svelte` — extracted section-nav chrome
- `src/lib/components/project-mode/settings/ProjectRuntimeEditor.svelte` — small runtime form for project scope
- `src/lib/components/project-mode/settings/ProjectEnvVarEditor.svelte` — key/value env editor
- `src/lib/components/project-mode/settings/ProjectHooksEditor.svelte` — project hooks form
- `src/lib/components/project-mode/settings/ProjectAdvancedJsonEditor.svelte` — raw JSON fallback
- `scripts/audit-i18n.ts` — locale value scanner
- `scripts/audit-i18n.whitelist.json` — whitelist data

**Modified:**
- `src/lib/components/settings/SettingsEditor.svelte` — refactor to call `SectionedSettings`
- `src/lib/components/project-mode/ProjectSettingsFacet.svelte` — full rewrite to call `SectionedSettings` + 4 project sub-editors
- `src/lib/components/project-mode/StalePathBanner.svelte` — add Update path button + folder picker
- `src/lib/components/project-mode/ProjectModeView.svelte` — drop dead unbound branch
- `src/lib/components/appsettings/AppSettingsView.svelte` — read `pkg.name` instead of i18n key
- `src/lib/components/settings/RuntimeEditor.svelte` — tighten init-object typing at line 138
- `src/lib/stores/config.svelte.ts` — drop `activeScope` constant
- `src/lib/stores/projects.svelte.ts` — add `updatePath` method
- `src/lib/ipc/client.ts` — add `updateProjectPath` wrapper
- `src/lib/locales/{zh-CN,en-US,ja-JP,ko-KR,fr-FR,es-ES}.json` — add new keys; delete `appsettings.appName`
- `src-tauri/src/commands/gui_projects.rs` — `update_project_path` IPC + tests
- `src-tauri/src/lib.rs` — register new IPC
- `package.json` — add `audit:i18n` script; add `tsx` to devDependencies if missing

**Deleted:**
- `src/lib/components/project-mode/UnboundHint.svelte`

---

## Task 1: Remove `activeScope` dead constant (C3)

**Files:**
- Modify: `src/lib/stores/config.svelte.ts:13`

- [ ] **Step 1: Confirm zero readers**

Run: `grep -rn "activeScope" src/ --include="*.svelte" --include="*.ts"`
Expected output: exactly one line — the declaration in `src/lib/stores/config.svelte.ts:13`. If anything else shows up, stop and report; the spec assumes no readers.

- [ ] **Step 2: Delete the constant**

In `src/lib/stores/config.svelte.ts`, remove line 13:
```ts
  readonly activeScope = "user" as const;
```

- [ ] **Step 3: Run type check**

Run: `pnpm svelte-check`
Expected: 0 errors (Stage 4 baseline preserved).

- [ ] **Step 4: Commit**

```bash
git add src/lib/stores/config.svelte.ts
git commit -m "chore(stage5): drop dead configStore.activeScope constant (C3)"
```

---

## Task 2: Tighten `RuntimeEditor.svelte` init-object typing (C2)

**Files:**
- Modify: `src/lib/components/settings/RuntimeEditor.svelte:138`

- [ ] **Step 1: Read the current init-object**

Run: `sed -n '130,160p' src/lib/components/settings/RuntimeEditor.svelte`
Locate the init-object literal around line 138 that lacks the typing narrowing applied at line 157 in Stage 4.

- [ ] **Step 2: Apply `satisfies Partial<Settings>` to the init-object**

Append `satisfies Partial<Settings>` to the object literal so any field with implicit-`undefined` narrows to the matching `Settings` field's optional type. Example shape (preserve the actual field list at line 138):
```ts
const initial = {
  teammateMode: undefined,
  // ...other fields unchanged
} satisfies Partial<Settings>;
```

Ensure `Settings` is already imported in this file (it is — Stage 4 commit `cc72e50` added the imports).

- [ ] **Step 3: Run type check**

Run: `pnpm svelte-check`
Expected: 0 errors.

- [ ] **Step 4: Sanity-check the typing did work**

Manually edit the file to inject `teammateMode: "not-a-real-mode-value"` into the init object → `pnpm svelte-check` → expect a type error pointing at line 138 → revert.

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/settings/RuntimeEditor.svelte
git commit -m "chore(stage5): tighten RuntimeEditor init-object typing (C2)"
```

---

## Task 3: Derive `appName` from `package.json` (C1)

**Files:**
- Modify: `src/lib/components/appsettings/AppSettingsView.svelte:83`
- Modify: `src/lib/locales/zh-CN.json`, `en-US.json`, `ja-JP.json`, `ko-KR.json`, `fr-FR.json`, `es-ES.json`

- [ ] **Step 1: Add `APP_NAME` constant near existing `APP_VERSION` / `REPO_URL`**

Edit `src/lib/components/appsettings/AppSettingsView.svelte` — insert after the existing `REPO_URL` definition (current lines 6-9):
```ts
  const APP_NAME = (pkg as { name: string }).name;
```

- [ ] **Step 2: Replace `t("appsettings.appName")` with `{APP_NAME}`**

In `AppSettingsView.svelte:83` (the `<div>{t("appsettings.appName")}</div>` row in the About section), replace with:
```svelte
      <div>{APP_NAME}</div>
```

- [ ] **Step 3: Delete the `appsettings.appName` key from all six locales**

For each of `src/lib/locales/zh-CN.json`, `en-US.json`, `ja-JP.json`, `ko-KR.json`, `fr-FR.json`, `es-ES.json`: remove the `"appsettings.appName": "..."` line. Preserve trailing comma / JSON validity (the line is sandwiched between other keys, so just delete the one line).

- [ ] **Step 4: Confirm no other consumer**

Run: `grep -rn "appsettings.appName" src/`
Expected: zero matches.

- [ ] **Step 5: Type check**

Run: `pnpm svelte-check`
Expected: 0 errors.

- [ ] **Step 6: Smoke-test the About section**

Run: `pnpm tauri dev` → open gear panel → About section → confirm the name row shows `dot-claude-gui`. Close dev session.

- [ ] **Step 7: Commit**

```bash
git add src/lib/components/appsettings/AppSettingsView.svelte src/lib/locales/
git commit -m "chore(stage5): derive appName from package.json; drop i18n key (C1)"
```

---

## Task 4: Delete `UnboundHint` dead code (B3)

**Files:**
- Delete: `src/lib/components/project-mode/UnboundHint.svelte`
- Modify: `src/lib/components/project-mode/ProjectModeView.svelte`

- [ ] **Step 1: Confirm the branch is unreachable**

Read `src/lib/components/project-mode/ProjectModeView.svelte` around lines 25-65. Verify that:
- `isBound` is true ⇒ tabs enabled ⇒ user can be on any facet ⇒ `activeFacet !== "binding"` is reachable but the branch's *guard* is `!isBound && activeFacet !== "binding"`, so the branch only fires when `isBound` is false.
- When `isBound` is false: the tabs effect (`if (!isBound && key !== "binding") return true` on line 32) disables every non-binding tab, so `activeFacet` cannot be anything other than `"binding"`.
- Conclusion: `!isBound && activeFacet !== "binding"` is unreachable.

If this is not the case (e.g. tab disabling has been changed since this plan was written), stop and re-evaluate.

- [ ] **Step 2: Delete the UnboundHint import line**

In `ProjectModeView.svelte`, delete the line:
```ts
  import UnboundHint from "./UnboundHint.svelte";
```

- [ ] **Step 3: Delete the dead branch**

In `ProjectModeView.svelte:58-59` (or whatever lines match `{:else if !isBound && activeFacet !== "binding"}` followed by `<UnboundHint />`), delete that `{:else if ...}` block entirely. Keep the surrounding `{#if isStale} ... {:else if ...} ... {/if}` chain valid.

- [ ] **Step 4: Delete the component file**

Run: `rm src/lib/components/project-mode/UnboundHint.svelte`

- [ ] **Step 5: Confirm no other reference**

Run: `grep -rn "UnboundHint" src/`
Expected: zero matches.

- [ ] **Step 6: Verify `projectMode.binding.unboundHint` i18n key is still used by BindingFacet**

Run: `grep -rn "projectMode.binding.unboundHint" src/`
Expected: at least one match in `BindingFacet.svelte` (the banner added in commit `454eaec`). Do NOT delete this key — UnboundHint and BindingFacet share it.

- [ ] **Step 7: Type check**

Run: `pnpm svelte-check`
Expected: 0 errors.

- [ ] **Step 8: Commit**

```bash
git add -A src/lib/components/project-mode/
git commit -m "chore(stage5): delete UnboundHint dead component + unreachable branch (B3)"
```

---

## Task 5: Add `audit-i18n` script + whitelist (B1, infrastructure)

**Files:**
- Create: `scripts/audit-i18n.ts`
- Create: `scripts/audit-i18n.whitelist.json`
- Modify: `package.json`

- [ ] **Step 1: Confirm `tsx` availability**

Run: `pnpm list tsx --depth=0`
- If `tsx` is already a devDependency, skip Step 2's install.
- If not, the next step adds it.

- [ ] **Step 2: Add `tsx` (if missing) and the audit script**

If `tsx` is missing:
```bash
pnpm add -D tsx
```

Edit `package.json` — add to the `scripts` object (preserve formatting / position; insert after the existing `tauri` script):
```json
  "audit:i18n": "tsx scripts/audit-i18n.ts"
```

- [ ] **Step 3: Create the whitelist file**

Write `scripts/audit-i18n.whitelist.json`:
```json
{
  "keys": [],
  "keySuffixes": [".cmd", ".shortcut", ".url", ".code", ".identifier"],
  "valuePatterns": [
    "^https?://",
    "^[A-Z][A-Z0-9_]*$",
    "^v?\\d+(\\.\\d+)*$"
  ]
}
```

- [ ] **Step 4: Write the audit script**

Create `scripts/audit-i18n.ts`:
```ts
#!/usr/bin/env tsx
// Locale audit: flags non-en-US values that look like English-as-placeholder
// (pure ASCII while en-US value is a normal English sentence), and locale-parity
// gaps (keys present in en-US but missing in another locale).
//
// Exit codes:
//   0 — no suspect values found
//   1 — at least one suspect value found (gates Stage 5 exit)

import { readFileSync, readdirSync } from "node:fs";
import { join, basename } from "node:path";

type Whitelist = {
  keys: string[];
  keySuffixes: string[];
  valuePatterns: string[];
};

const LOCALE_DIR = join(process.cwd(), "src", "lib", "locales");
const REFERENCE = "en-US";
const WHITELIST_PATH = join(process.cwd(), "scripts", "audit-i18n.whitelist.json");

const whitelist = JSON.parse(readFileSync(WHITELIST_PATH, "utf8")) as Whitelist;
const valueRegexes = whitelist.valuePatterns.map((p) => new RegExp(p));

function loadLocale(file: string): Record<string, string> {
  const raw = JSON.parse(readFileSync(join(LOCALE_DIR, file), "utf8"));
  return raw as Record<string, string>;
}

function isWhitelisted(key: string, value: string): boolean {
  if (whitelist.keys.includes(key)) return true;
  if (whitelist.keySuffixes.some((s) => key.endsWith(s))) return true;
  if (valueRegexes.some((re) => re.test(value))) return true;
  return false;
}

function isNormalEnglishSentence(value: string): boolean {
  return value.length >= 4 || value.includes(" ");
}

function isPureAscii(value: string): boolean {
  return /^[\x00-\x7f]+$/.test(value);
}

function audit(): number {
  const localeFiles = readdirSync(LOCALE_DIR).filter((f) => f.endsWith(".json"));
  if (!localeFiles.includes(`${REFERENCE}.json`)) {
    console.error(`reference locale ${REFERENCE}.json not found in ${LOCALE_DIR}`);
    return 1;
  }
  const reference = loadLocale(`${REFERENCE}.json`);
  const suspect: { locale: string; key: string; value: string }[] = [];
  const missing: { locale: string; key: string }[] = [];

  for (const file of localeFiles) {
    const locale = basename(file, ".json");
    if (locale === REFERENCE) continue;
    const data = loadLocale(file);

    for (const [key, refValue] of Object.entries(reference)) {
      if (typeof refValue !== "string") continue;
      if (!(key in data)) {
        missing.push({ locale, key });
        continue;
      }
      const value = data[key];
      if (typeof value !== "string" || value === "") continue;
      if (!isNormalEnglishSentence(refValue)) continue;
      if (!isPureAscii(value)) continue;
      if (isWhitelisted(key, value)) continue;
      suspect.push({ locale, key, value });
    }
  }

  if (missing.length > 0) {
    console.log(`Locale-parity gaps (informational, not gated):`);
    for (const m of missing) console.log(`  missing in ${m.locale}: ${m.key}`);
  }
  if (suspect.length === 0) {
    console.log(`audit-i18n: 0 suspect values across ${localeFiles.length - 1} locales`);
    return 0;
  }
  console.log(`audit-i18n: ${suspect.length} suspect value(s):`);
  for (const s of suspect) console.log(`  ${s.locale}: ${s.key} = ${s.value}`);
  return 1;
}

process.exit(audit());
```

- [ ] **Step 5: Smoke-test on current locale data**

Run: `pnpm run audit:i18n`
Expected (best case): exit 0. If exit 1, the script has surfaced real English-as-placeholder values in zh-CN / ja-JP that Cluster 10 missed — those will be addressed in Task 11 (seed sweep), not here.

If the audit reports missing-key gaps in `ko-KR` / `fr-FR` / `es-ES` (which are mostly empty strings), that's expected and does not gate the script's exit.

- [ ] **Step 6: Verify whitelist works**

Inject a test entry into `src/lib/locales/zh-CN.json`:
```json
"appsettings.appName": "TEST_ENGLISH_VALUE",
```
Run: `pnpm run audit:i18n`
Expected: exits 1 with line `zh-CN: appsettings.appName = TEST_ENGLISH_VALUE` (assuming Task 3 hasn't deleted this key yet; if it has, use any other key for the test).

Revert the inject.

- [ ] **Step 7: Verify whitelist file works**

Add `"appsettings.appName"` to `scripts/audit-i18n.whitelist.json#keys`, re-inject the test value, re-run → exit 0. Revert whitelist edit and test value.

- [ ] **Step 8: Commit**

```bash
git add scripts/audit-i18n.ts scripts/audit-i18n.whitelist.json package.json
git commit -m "feat(stage5): audit-i18n script + whitelist + pnpm command (B1 infra)"
```

Note: this commit does NOT gate Stage 5 exit yet; the seed sweep in Task 11 makes the script clean before the exit gate runs.

---

## Task 6: `update_project_path` IPC backend (A2 backend, TDD)

**Files:**
- Modify: `src-tauri/src/commands/gui_projects.rs`
- Modify: `src-tauri/src/lib.rs:53-65` (invoke handler registration)

- [ ] **Step 1: Write the failing tests**

In `src-tauri/src/commands/gui_projects.rs`, inside the existing `#[cfg(test)] mod tests` block, append these tests:

```rust
    #[test]
    #[serial_test::serial]
    fn update_path_moves_known_and_bound_entry() {
        let _g = isolated();
        let cfg_path = config_path().unwrap();
        let mut cfg = AppConfig::default();
        cfg.accounts.push(Account {
            name: "work".into(), display_name: "work".into(),
            is_native: false, created_at: "x".into(),
        });
        write_config(&cfg_path, &cfg).unwrap();

        // Bind a real directory (use HOME so we have a guaranteed-existing canonicalizable path).
        let real = std::env::current_dir().unwrap().to_string_lossy().to_string();
        add_project(AddProjectRequest { path: real.clone() }).unwrap();
        bind_project(BindProjectRequest { path: real.clone(), account: "work".into() }).unwrap();

        // Prepare a different real path to move TO.
        let dir = tempfile::tempdir().unwrap();
        let new_path = dir.path().to_string_lossy().to_string();

        update_project_path(UpdateProjectPathRequest {
            old_path: real.clone(),
            new_path: new_path.clone(),
        }).unwrap();

        let list = gui_list_projects().unwrap();
        // Old path gone.
        assert!(list.iter().all(|p| p.path != real),
            "old path should be removed from known_projects");
        // New path present and still bound to "work".
        let new_canonical = std::path::Path::new(&new_path)
            .canonicalize().unwrap().to_string_lossy().to_string();
        let moved = list.iter().find(|p| p.path == new_canonical)
            .expect("new path present");
        assert_eq!(moved.account.as_deref(), Some("work"));
    }

    #[test]
    #[serial_test::serial]
    fn update_path_rejects_nonexistent_new_path() {
        let _g = isolated();
        let real = std::env::current_dir().unwrap().to_string_lossy().to_string();
        add_project(AddProjectRequest { path: real.clone() }).unwrap();
        let res = update_project_path(UpdateProjectPathRequest {
            old_path: real,
            new_path: "/definitely/does/not/exist/abc123".into(),
        });
        assert!(res.is_err(), "non-existent new_path must be rejected");
    }

    #[test]
    #[serial_test::serial]
    fn update_path_rejects_unknown_old_path() {
        let _g = isolated();
        let dir = tempfile::tempdir().unwrap();
        let new_path = dir.path().to_string_lossy().to_string();
        let res = update_project_path(UpdateProjectPathRequest {
            old_path: "/never/added/path".into(),
            new_path,
        });
        assert!(res.is_err(), "unknown old_path must be rejected");
    }

    #[test]
    #[serial_test::serial]
    fn update_path_rejects_new_path_already_known() {
        let _g = isolated();
        let dir_a = tempfile::tempdir().unwrap();
        let dir_b = tempfile::tempdir().unwrap();
        let a = dir_a.path().to_string_lossy().to_string();
        let b = dir_b.path().to_string_lossy().to_string();
        add_project(AddProjectRequest { path: a.clone() }).unwrap();
        add_project(AddProjectRequest { path: b.clone() }).unwrap();
        let res = update_project_path(UpdateProjectPathRequest {
            old_path: a,
            new_path: b,
        });
        assert!(res.is_err(), "new_path already known must be rejected");
    }
```

- [ ] **Step 2: Run tests to confirm they fail**

Run: `cargo test -p dot-claude-gui-tauri --lib update_path`
Expected: 4 tests fail with errors like `cannot find function update_project_path` or `cannot find type UpdateProjectPathRequest`.

(Crate name: check `src-tauri/Cargo.toml` for the actual `[package].name`. If it differs, substitute that name.)

- [ ] **Step 3: Implement the command**

In `src-tauri/src/commands/gui_projects.rs`, append after the existing `update_project_launch` command (around line 166):

```rust
// ── Update path (rename a known project) ────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectPathRequest {
    pub old_path: String,
    pub new_path: String,
}

#[tauri::command]
pub fn update_project_path(req: UpdateProjectPathRequest) -> Result<(), String> {
    let new_p = std::path::Path::new(&req.new_path);
    if !new_p.is_dir() {
        return Err(format!("invalid_path: {}", req.new_path));
    }
    let new_canonical = new_p
        .canonicalize()
        .map_err(|e| format!("canonicalize path: {e}"))?
        .to_string_lossy()
        .to_string();
    let old_canonical = canonicalize_path(&req.old_path);

    if old_canonical == new_canonical {
        return Ok(());
    }

    mutate(|cfg| {
        if !cfg.known_projects.contains(&old_canonical) {
            return Err(format!("unknown_path: {}", req.old_path));
        }
        if cfg.known_projects.contains(&new_canonical) {
            return Err(format!("path_already_known: {}", req.new_path));
        }
        // Replace in known_projects (preserve list ordering).
        for entry in cfg.known_projects.iter_mut() {
            if entry == &old_canonical {
                *entry = new_canonical.clone();
            }
        }
        // Move binding (if any) to the new key.
        if let Some(binding) = cfg.projects.remove(&old_canonical) {
            cfg.projects.insert(new_canonical.clone(), binding);
        }
        Ok(())
    })?;
    Ok(())
}
```

- [ ] **Step 4: Register the command in the invoke handler**

In `src-tauri/src/lib.rs:65` (after `commands::gui_projects::update_project_launch,`), add:
```rust
            commands::gui_projects::update_project_path,
```

- [ ] **Step 5: Run tests to confirm they pass**

Run: `cargo test -p dot-claude-gui-tauri --lib update_path`
Expected: 4 tests pass.

- [ ] **Step 6: Run full workspace tests for regression**

Run: `cargo test --workspace`
Expected: baseline parity (Stage 4 baseline was 155 passed + 1 ignored; expect 159 passed + 1 ignored now — 4 new tests).

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/commands/gui_projects.rs src-tauri/src/lib.rs
git commit -m "feat(stage5): update_project_path IPC + 4 cargo tests (A2 backend)"
```

---

## Task 7: A2 frontend — Update path button + IPC wiring + i18n

**Files:**
- Modify: `src/lib/ipc/client.ts`
- Modify: `src/lib/stores/projects.svelte.ts`
- Modify: `src/lib/components/project-mode/StalePathBanner.svelte`
- Modify: `src/lib/locales/zh-CN.json`, `en-US.json`, `ja-JP.json` (the three active locales; ko/fr/es-ES remain mostly empty)

- [ ] **Step 1: Add the IPC wrapper**

Open `src/lib/ipc/client.ts`. Find `updateProjectLaunch` (around line 120). Add immediately after it:
```ts
  async updateProjectPath(oldPath: string, newPath: string): Promise<void> {
    await invoke("update_project_path", { req: { oldPath, newPath } });
  }
```
(Verify that adjacent methods use the `{ req: { ... } }` envelope — the pattern matches Stage 3's camelCase serde rename convention noted in CLAUDE.md gotcha #9.)

- [ ] **Step 2: Add the store method**

In `src/lib/stores/projects.svelte.ts`, append after `updateLaunch` (around line 61):
```ts
  async updatePath(oldPath: string, newPath: string): Promise<void> {
    await ipcClient.updateProjectPath(oldPath, newPath);
    await this.loadProjects();
  }
```

- [ ] **Step 3: Add the three new i18n keys to active locales**

In `src/lib/locales/en-US.json`, add three keys grouped with the existing `projectMode.stale*` family (preserving JSON validity / trailing commas):
```json
  "projectMode.staleUpdatePathBtn": "Update path…",
  "projectMode.staleUpdatePathDialogTitle": "Select the new project directory",
  "projectMode.stalePathUpdated": "Project path updated.",
```

In `src/lib/locales/zh-CN.json`:
```json
  "projectMode.staleUpdatePathBtn": "更新路径…",
  "projectMode.staleUpdatePathDialogTitle": "选择新的项目目录",
  "projectMode.stalePathUpdated": "项目路径已更新。",
```

In `src/lib/locales/ja-JP.json`:
```json
  "projectMode.staleUpdatePathBtn": "パスを更新…",
  "projectMode.staleUpdatePathDialogTitle": "新しいプロジェクトディレクトリを選択",
  "projectMode.stalePathUpdated": "プロジェクトパスを更新しました。",
```

For `ko-KR.json`, `fr-FR.json`, `es-ES.json` (mostly-empty locales): add the keys with empty-string values so locale-parity holds:
```json
  "projectMode.staleUpdatePathBtn": "",
  "projectMode.staleUpdatePathDialogTitle": "",
  "projectMode.stalePathUpdated": "",
```

- [ ] **Step 4: Rewrite `StalePathBanner.svelte`**

Replace the contents of `src/lib/components/project-mode/StalePathBanner.svelte` with:
```svelte
<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { t } from "$lib/i18n";
  import { projectsStore } from "$lib/stores/projects.svelte";
  import { modeStore } from "$lib/stores/mode.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";

  let { path }: { path: string } = $props();

  async function onUpdate() {
    try {
      const parent = path.replace(/[\\/][^\\/]+[\\/]?$/, "") || undefined;
      const picked = await open({
        directory: true,
        multiple: false,
        title: t("projectMode.staleUpdatePathDialogTitle"),
        defaultPath: parent,
      });
      if (typeof picked !== "string") return; // cancelled
      await projectsStore.updatePath(path, picked);
      modeStore.selectedProject = picked;
      toastStore.success(t("projectMode.stalePathUpdated"));
    } catch (e) {
      toastStore.error(String(e));
    }
  }

  async function onRemove() {
    if (!confirm(t("projectMode.staleConfirmRemove"))) return;
    await projectsStore.remove(path);
  }
</script>

<div class="banner" role="alert">
  <span>{t("projectMode.staleBanner", { path })}</span>
  <div class="actions">
    <button class="update-btn" onclick={onUpdate}>{t("projectMode.staleUpdatePathBtn")}</button>
    <button class="remove-btn" onclick={onRemove}>{t("projectMode.staleRemoveBtn")}</button>
  </div>
</div>

<style>
  .banner {
    background: var(--bg-warn, #fde2e2);
    color: var(--text-warn, #8a1f1f);
    padding: 8px 12px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
  }
  .actions {
    display: flex;
    gap: 8px;
  }
  .update-btn,
  .remove-btn {
    background: transparent;
    border: 1px solid currentColor;
    padding: 2px 10px;
    border-radius: 4px;
    cursor: pointer;
    color: inherit;
  }
  .update-btn {
    font-weight: 600;
  }
</style>
```

- [ ] **Step 5: Confirm `modeStore.selectedProject` is writable from outside**

Run: `grep -n "selectedProject" src/lib/stores/mode.svelte.ts`
Expected: it is a `$state` field (writable), not a `$derived`. If it is derived and not assignable, stop and add a setter method to `modeStore` instead of direct assignment.

- [ ] **Step 6: Type check**

Run: `pnpm svelte-check`
Expected: 0 errors.

- [ ] **Step 7: Manual smoke-test**

```bash
pnpm tauri dev
```

In another shell:
```bash
mkdir -p /tmp/stage5-rename-test-old
```

In the GUI: Projects mode → Add project → pick `/tmp/stage5-rename-test-old` → Bind to any account.

Back in the shell:
```bash
mv /tmp/stage5-rename-test-old /tmp/stage5-rename-test-new
```

In the GUI: the project should turn stale → `StalePathBanner` shows both `Update path…` and `Remove` buttons → click `Update path…` → folder picker opens → pick `/tmp/stage5-rename-test-new` → toast appears → banner gone → facet tabs re-enabled → binding preserved.

Cleanup:
```bash
rm -rf /tmp/stage5-rename-test-new
```

- [ ] **Step 8: Commit**

```bash
git add src/lib/ipc/client.ts src/lib/stores/projects.svelte.ts \
        src/lib/components/project-mode/StalePathBanner.svelte src/lib/locales/
git commit -m "feat(stage5): StalePathBanner Update action + IPC wiring + i18n (A2 frontend)"
```

---

## Task 8: `SectionedSettings` shared chrome + Account refactor (A1.1)

**Files:**
- Create: `src/lib/components/shared/SectionedSettings.svelte`
- Modify: `src/lib/components/settings/SettingsEditor.svelte`

- [ ] **Step 1: Create `SectionedSettings.svelte`**

Write `src/lib/components/shared/SectionedSettings.svelte`:
```svelte
<script lang="ts">
  import type { Snippet } from "svelte";
  import { t } from "$lib/i18n";

  type Section = { id: string; label: string };

  let {
    sections,
    activeSection = $bindable(),
    isDirty,
    error,
    content,
  }: {
    sections: Section[];
    activeSection: string;
    isDirty: boolean;
    error: string | null;
    content: Snippet<[string]>;
  } = $props();
</script>

<div class="sectioned">
  <nav class="section-nav" aria-label="Sections">
    {#each sections as section (section.id)}
      <button
        type="button"
        class="section-link"
        class:active={section.id === activeSection}
        onclick={() => (activeSection = section.id)}
      >{section.label}</button>
    {/each}
  </nav>

  <div class="section-body">
    {#if isDirty}
      <div class="dirty-bar">
        <span>{t("common.unsavedChanges")}</span>
      </div>
    {/if}
    {#if error}
      <div class="error-bar">{error}</div>
    {/if}
    <div class="section-content">
      {@render content(activeSection)}
    </div>
  </div>
</div>

<style>
  .sectioned {
    display: grid;
    grid-template-columns: 200px 1fr;
    height: 100%;
    min-height: 0;
  }
  .section-nav {
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--border, transparent);
    background: var(--bg-secondary, transparent);
    padding: 8px 0;
    overflow-y: auto;
  }
  .section-link {
    text-align: left;
    padding: 6px 16px;
    background: transparent;
    border: 0;
    color: var(--text-primary);
    cursor: pointer;
    font-size: 13px;
  }
  .section-link.active {
    background: var(--accent-bg, rgba(44,108,255,0.12));
    color: var(--accent-text, inherit);
    font-weight: 600;
  }
  .section-link:hover:not(.active) {
    background: var(--bg-hover, rgba(0,0,0,0.04));
  }
  .section-body {
    display: flex;
    flex-direction: column;
    min-height: 0;
    flex: 1;
  }
  .dirty-bar {
    border-bottom: 1px solid var(--border, transparent);
    background: var(--bg-secondary, transparent);
    padding: 6px 16px;
    font-size: 12px;
    color: var(--status-warning-text, inherit);
  }
  .error-bar {
    border-bottom: 1px solid var(--status-error-text, transparent);
    background: var(--status-error-bg, rgba(196,68,68,0.08));
    color: var(--status-error-text, inherit);
    padding: 6px 16px;
    font-size: 12px;
  }
  .section-content {
    flex: 1;
    overflow: auto;
    padding: 16px;
  }
</style>
```

- [ ] **Step 2: Refactor `SettingsEditor.svelte` to use `SectionedSettings`**

Note that the *current* `SettingsEditor.svelte` does NOT own the section navigation — it only does the `{#if activeSection === "general"}` switch and a dirty/error header. The section navigation lives elsewhere (likely in `AccountSettingsFacet` or whichever parent). For Account-side, the refactor here is minimal: just wrap the existing dirty/error header + content switch in a way that's API-compatible with how `SectionedSettings` will be used at the Project side.

Replace `src/lib/components/settings/SettingsEditor.svelte` contents with:
```svelte
<script lang="ts">
  import { configStore } from "$lib/stores/config.svelte";
  import GeneralEditor from "./GeneralEditor.svelte";
  import PermissionsEditor from "./PermissionsEditor.svelte";
  import HooksEditor from "./HooksEditor.svelte";
  import SandboxEditor from "./SandboxEditor.svelte";
  import EnvVarEditor from "./EnvVarEditor.svelte";
  import StatusLineEditor from "./StatusLineEditor.svelte";
  import RuntimeEditor from "./RuntimeEditor.svelte";
  import McpPolicyEditor from "./McpPolicyEditor.svelte";
  import PluginsMarketplaceEditor from "./PluginsMarketplaceEditor.svelte";
  import AdvancedJsonEditor from "./AdvancedJsonEditor.svelte";
  import { t } from "$lib/i18n";

  let { activeSection = "general" }: { activeSection: string } = $props();
</script>

{#if configStore.isDirty}
  <div class="flex items-center justify-end border-b px-4 py-2"
       style="border-color: var(--border-color); background-color: var(--bg-secondary)">
    <span class="text-xs" style="color: var(--status-warning-text)">{t("common.unsavedChanges")}</span>
  </div>
{/if}

{#if configStore.error}
  <div class="border-b px-4 py-2"
       style="border-color: var(--status-error-text); background-color: var(--status-error-bg)">
    <p class="text-xs" style="color: var(--status-error-text)">{configStore.error}</p>
  </div>
{/if}

<div class="flex-1 overflow-auto p-6">
  {#if activeSection === "general"}<GeneralEditor />
  {:else if activeSection === "permissions"}<PermissionsEditor />
  {:else if activeSection === "hooks"}<HooksEditor />
  {:else if activeSection === "sandbox"}<SandboxEditor />
  {:else if activeSection === "environment"}<EnvVarEditor />
  {:else if activeSection === "statusline"}<StatusLineEditor />
  {:else if activeSection === "runtime"}<RuntimeEditor />
  {:else if activeSection === "mcpPolicy"}<McpPolicyEditor />
  {:else if activeSection === "pluginsMarketplace"}<PluginsMarketplaceEditor />
  {:else if activeSection === "advanced"}<AdvancedJsonEditor />
  {:else}<p class="text-sm" style="color: var(--text-muted)">Unknown section: {activeSection}</p>{/if}
</div>
```

(This intentionally keeps `SettingsEditor` Account-side identical to today; the Project side will call `SectionedSettings` directly with its own section list and sub-editors in Task 10.)

If the Account side already has a parent that supplies the section list to `SettingsEditor`, this refactor is purely a no-op preservation. The shared `SectionedSettings` is consumed only by `ProjectSettingsFacet` in Task 10 — Account-side adoption is intentionally deferred (out of scope for Stage 5) to keep the change surface small.

- [ ] **Step 3: Type check**

Run: `pnpm svelte-check`
Expected: 0 errors.

- [ ] **Step 4: Smoke-test Account Settings**

`pnpm tauri dev` → Account mode → Settings → click through all 10 sub-sections → verify each renders as before. Close dev session.

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/shared/SectionedSettings.svelte src/lib/components/settings/SettingsEditor.svelte
git commit -m "feat(stage5): shared SectionedSettings chrome; Account SettingsEditor unchanged (A1.1)"
```

---

## Task 9: Project sub-editors (A1.2)

**Files:**
- Create: `src/lib/components/project-mode/settings/ProjectAdvancedJsonEditor.svelte`
- Create: `src/lib/components/project-mode/settings/ProjectRuntimeEditor.svelte`
- Create: `src/lib/components/project-mode/settings/ProjectEnvVarEditor.svelte`
- Create: `src/lib/components/project-mode/settings/ProjectHooksEditor.svelte`

Each sub-editor receives `{ settings, onPatch, error }` as props. `settings: Settings` is the current draft, `onPatch: (partial: Partial<Settings>) => void` is the parent callback that merges into the draft.

- [ ] **Step 1: Create `ProjectAdvancedJsonEditor.svelte`**

This is the raw-JSON fallback (ports the current `ProjectSettingsFacet` textarea behavior):
```svelte
<script lang="ts">
  import { t } from "$lib/i18n";
  import type { Settings } from "$lib/api/types";

  let {
    settings,
    onPatch,
    error,
  }: {
    settings: Settings;
    onPatch: (partial: Partial<Settings>) => void;
    error: string | null;
  } = $props();

  let raw = $state(JSON.stringify(settings, null, 2));
  let lastSettingsKey = $state("");

  $effect(() => {
    const key = JSON.stringify(settings);
    if (key !== lastSettingsKey) {
      raw = JSON.stringify(settings, null, 2);
      lastSettingsKey = key;
    }
  });

  let localError = $state<string | null>(null);

  function onChange(e: Event) {
    raw = (e.target as HTMLTextAreaElement).value;
    try {
      const parsed = JSON.parse(raw);
      if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) {
        localError = t("projectMode.settings.notObject");
        return;
      }
      localError = null;
      onPatch(parsed as Partial<Settings>);
    } catch (e) {
      localError = (e as Error).message;
    }
  }
</script>

<div class="advanced">
  <textarea
    value={raw}
    oninput={onChange}
    spellcheck="false"
    aria-label={t("projectMode.settings.section.advanced")}
  ></textarea>
  {#if localError || error}
    <p class="err">{localError ?? error}</p>
  {/if}
</div>

<style>
  .advanced {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  textarea {
    flex: 1;
    min-height: 300px;
    font-family: ui-monospace, Menlo, monospace;
    font-size: 13px;
    line-height: 1.5;
    padding: 12px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-input, transparent);
    color: var(--text-primary);
    resize: vertical;
  }
  .err {
    color: var(--danger, #c44);
    font-family: ui-monospace, Menlo, monospace;
    font-size: 12px;
    margin: 8px 0 0;
  }
</style>
```

Note: `onPatch(parsed as Partial<Settings>)` replaces the entire draft when JSON is valid. The parent will merge by full-replacement in advanced mode (intentional — power users editing raw JSON expect their JSON to be canonical).

- [ ] **Step 2: Create `ProjectRuntimeEditor.svelte`**

Pick a small, commonly-overridden subset: `model` and `outputStyle`. Power users override the rest via Advanced JSON.
```svelte
<script lang="ts">
  import { t } from "$lib/i18n";
  import type { Settings } from "$lib/api/types";

  let {
    settings,
    onPatch,
  }: {
    settings: Settings;
    onPatch: (partial: Partial<Settings>) => void;
    error: string | null;
  } = $props();

  function setModel(e: Event) {
    const value = (e.target as HTMLInputElement).value.trim();
    onPatch({ model: value === "" ? undefined : value });
  }

  function setOutputStyle(e: Event) {
    const value = (e.target as HTMLInputElement).value.trim();
    onPatch({ outputStyle: value === "" ? undefined : value });
  }
</script>

<div class="runtime-fields">
  <label>
    <span>model</span>
    <input
      type="text"
      value={settings.model ?? ""}
      oninput={setModel}
      placeholder="claude-opus-4-7"
    />
  </label>
  <label>
    <span>outputStyle</span>
    <input
      type="text"
      value={settings.outputStyle ?? ""}
      oninput={setOutputStyle}
      placeholder=""
    />
  </label>
  <p class="hint">{t("projectMode.settings.section.runtimeHint")}</p>
</div>

<style>
  .runtime-fields {
    display: flex;
    flex-direction: column;
    gap: 12px;
    max-width: 480px;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  label span {
    color: var(--text-muted);
    font-size: 13px;
  }
  input {
    padding: 6px 10px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-input, transparent);
    color: var(--text-primary);
    font-family: ui-monospace, Menlo, monospace;
    font-size: 13px;
  }
  .hint {
    color: var(--text-muted);
    font-size: 12px;
    margin: 4px 0 0;
  }
</style>
```

If `Settings["model"]` or `Settings["outputStyle"]` does not exist or has a different name in `claude-types::Settings`, run `grep -n "pub model\|pub output_style\|pub outputStyle" crates/claude-types/src/settings.rs` and align the field names accordingly. CLAUDE.md gotcha #10 (TS↔Rust type drift) is the directive here.

- [ ] **Step 3: Create `ProjectEnvVarEditor.svelte`**

Key/value list with add/remove:
```svelte
<script lang="ts">
  import { t } from "$lib/i18n";
  import type { Settings } from "$lib/api/types";

  let {
    settings,
    onPatch,
  }: {
    settings: Settings;
    onPatch: (partial: Partial<Settings>) => void;
    error: string | null;
  } = $props();

  const entries = $derived(Object.entries(settings.env ?? {}));

  let newKey = $state("");
  let newValue = $state("");

  function update(key: string, value: string) {
    const next = { ...(settings.env ?? {}), [key]: value };
    onPatch({ env: next });
  }

  function remove(key: string) {
    const next = { ...(settings.env ?? {}) };
    delete next[key];
    onPatch({ env: Object.keys(next).length === 0 ? undefined : next });
  }

  function addNew() {
    if (!newKey || (settings.env ?? {})[newKey] != null) return;
    update(newKey, newValue);
    newKey = "";
    newValue = "";
  }
</script>

<div class="env-editor">
  <table>
    <thead>
      <tr><th>name</th><th>value</th><th></th></tr>
    </thead>
    <tbody>
      {#each entries as [key, value] (key)}
        <tr>
          <td><code>{key}</code></td>
          <td>
            <input
              type="text"
              value={value}
              oninput={(e) => update(key, (e.target as HTMLInputElement).value)}
            />
          </td>
          <td>
            <button type="button" onclick={() => remove(key)}>
              {t("common.remove")}
            </button>
          </td>
        </tr>
      {/each}
      <tr class="add-row">
        <td><input type="text" placeholder="NAME" bind:value={newKey} /></td>
        <td><input type="text" placeholder="value" bind:value={newValue} /></td>
        <td>
          <button type="button" onclick={addNew} disabled={!newKey}>
            {t("common.add")}
          </button>
        </td>
      </tr>
    </tbody>
  </table>
</div>

<style>
  .env-editor { max-width: 720px; }
  table { width: 100%; border-collapse: collapse; }
  th { text-align: left; color: var(--text-muted); font-weight: 500; font-size: 12px; padding: 4px 8px; }
  td { padding: 4px 8px; }
  input {
    width: 100%;
    padding: 4px 8px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-input, transparent);
    color: var(--text-primary);
    font-family: ui-monospace, Menlo, monospace;
    font-size: 13px;
  }
  code { font-family: ui-monospace, Menlo, monospace; font-size: 13px; }
  button {
    padding: 2px 10px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: transparent;
    cursor: pointer;
    color: inherit;
  }
  button[disabled] { opacity: 0.5; cursor: not-allowed; }
  .add-row { border-top: 1px solid var(--border); }
</style>
```

Verify `Settings.env` field exists and is `Record<string, string>`-shaped in `src/lib/api/types.ts`. If the field name or shape differs (CLAUDE.md gotcha #10), align names.

- [ ] **Step 4: Create `ProjectHooksEditor.svelte`**

For project layer, hooks are typically rarely overridden — list existing hooks and offer raw-JSON view of each hook group, keeping the implementation tight:
```svelte
<script lang="ts">
  import { t } from "$lib/i18n";
  import type { Settings } from "$lib/api/types";

  let {
    settings,
    onPatch,
  }: {
    settings: Settings;
    onPatch: (partial: Partial<Settings>) => void;
    error: string | null;
  } = $props();

  let raw = $state(JSON.stringify(settings.hooks ?? {}, null, 2));
  let lastSettingsKey = $state("");
  let localError = $state<string | null>(null);

  $effect(() => {
    const key = JSON.stringify(settings.hooks);
    if (key !== lastSettingsKey) {
      raw = JSON.stringify(settings.hooks ?? {}, null, 2);
      lastSettingsKey = key;
    }
  });

  function onChange(e: Event) {
    raw = (e.target as HTMLTextAreaElement).value;
    try {
      const parsed = JSON.parse(raw);
      if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) {
        localError = t("projectMode.settings.notObject");
        return;
      }
      localError = null;
      onPatch({ hooks: Object.keys(parsed).length === 0 ? undefined : parsed as Settings["hooks"] });
    } catch (e) {
      localError = (e as Error).message;
    }
  }
</script>

<div class="hooks">
  <p class="hint">{t("projectMode.settings.section.hooksHint")}</p>
  <textarea
    value={raw}
    oninput={onChange}
    spellcheck="false"
    aria-label={t("projectMode.settings.section.hooks")}
  ></textarea>
  {#if localError}<p class="err">{localError}</p>{/if}
</div>

<style>
  .hooks { display: flex; flex-direction: column; height: 100%; }
  .hint { color: var(--text-muted); font-size: 12px; margin: 0 0 8px; }
  textarea {
    flex: 1;
    min-height: 240px;
    font-family: ui-monospace, Menlo, monospace;
    font-size: 13px;
    line-height: 1.5;
    padding: 12px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-input, transparent);
    color: var(--text-primary);
    resize: vertical;
  }
  .err {
    color: var(--danger, #c44);
    font-family: ui-monospace, Menlo, monospace;
    font-size: 12px;
    margin: 8px 0 0;
  }
</style>
```

- [ ] **Step 5: Type check**

Run: `pnpm svelte-check`
Expected: 0 errors. If errors appear about `Settings.env` / `Settings.model` / `Settings.outputStyle` / `Settings.hooks` field names not matching, follow CLAUDE.md gotcha #10 — read `src/lib/api/types.ts` and align.

- [ ] **Step 6: Commit**

```bash
git add src/lib/components/project-mode/settings/
git commit -m "feat(stage5): project sub-editors (Runtime/EnvVar/Hooks/AdvancedJson) (A1.2)"
```

---

## Task 10: Rewrite `ProjectSettingsFacet` to use `SectionedSettings` (A1.3)

**Files:**
- Modify: `src/lib/components/project-mode/ProjectSettingsFacet.svelte` (full rewrite)
- Modify: `src/lib/locales/{zh-CN,en-US,ja-JP,ko-KR,fr-FR,es-ES}.json` (add new i18n keys)

- [ ] **Step 1: Add new i18n keys**

In `en-US.json`, add under existing `projectMode.settings.*`:
```json
  "projectMode.settings.section.runtime": "Runtime",
  "projectMode.settings.section.environment": "Environment",
  "projectMode.settings.section.hooks": "Hooks",
  "projectMode.settings.section.advanced": "Advanced (JSON)",
  "projectMode.settings.section.runtimeHint": "Project-layer overrides for model and output style.",
  "projectMode.settings.section.hooksHint": "Project-layer hook overrides. Edit as JSON.",
```

In `zh-CN.json`:
```json
  "projectMode.settings.section.runtime": "运行时",
  "projectMode.settings.section.environment": "环境变量",
  "projectMode.settings.section.hooks": "钩子",
  "projectMode.settings.section.advanced": "高级 (JSON)",
  "projectMode.settings.section.runtimeHint": "项目层覆盖：模型与输出样式。",
  "projectMode.settings.section.hooksHint": "项目层钩子覆盖。以 JSON 编辑。",
```

In `ja-JP.json`:
```json
  "projectMode.settings.section.runtime": "ランタイム",
  "projectMode.settings.section.environment": "環境変数",
  "projectMode.settings.section.hooks": "フック",
  "projectMode.settings.section.advanced": "詳細 (JSON)",
  "projectMode.settings.section.runtimeHint": "プロジェクト層のモデル・出力スタイル上書き。",
  "projectMode.settings.section.hooksHint": "プロジェクト層のフック上書き。JSON で編集します。",
```

In `ko-KR.json`, `fr-FR.json`, `es-ES.json`: add the same 6 keys with empty-string values for parity.

- [ ] **Step 2: Rewrite `ProjectSettingsFacet.svelte`**

Replace entire file with:
```svelte
<script lang="ts">
  import { t } from "$lib/i18n";
  import { ipcClient } from "$lib/ipc/client";
  import { toastStore } from "$lib/stores/toast.svelte";
  import type { Settings } from "$lib/api/types";
  import SectionedSettings from "$lib/components/shared/SectionedSettings.svelte";
  import ProjectRuntimeEditor from "./settings/ProjectRuntimeEditor.svelte";
  import ProjectEnvVarEditor from "./settings/ProjectEnvVarEditor.svelte";
  import ProjectHooksEditor from "./settings/ProjectHooksEditor.svelte";
  import ProjectAdvancedJsonEditor from "./settings/ProjectAdvancedJsonEditor.svelte";

  let { path }: { path: string } = $props();

  let original = $state<Settings>({});
  let current = $state<Settings>({});
  let activeSection = $state("runtime");
  let error = $state<string | null>(null);
  let loading = $state(true);
  let saving = $state(false);

  let isDirty = $derived(JSON.stringify(current) !== JSON.stringify(original));

  const sections = $derived([
    { id: "runtime",     label: t("projectMode.settings.section.runtime") },
    { id: "environment", label: t("projectMode.settings.section.environment") },
    { id: "hooks",       label: t("projectMode.settings.section.hooks") },
    { id: "advanced",    label: t("projectMode.settings.section.advanced") },
  ]);

  async function load() {
    loading = true;
    error = null;
    try {
      const resp = await ipcClient.projectReadSettings(path);
      original = (resp.settings ?? {}) as Settings;
      current = JSON.parse(JSON.stringify(original));
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  $effect(() => { void path; load(); });

  function onPatch(partial: Partial<Settings>) {
    current = { ...current, ...partial };
  }

  function onReplace(next: Partial<Settings>) {
    // Used by AdvancedJsonEditor to canonicalize the entire draft.
    current = next as Settings;
  }

  async function save() {
    saving = true;
    try {
      await ipcClient.projectWriteSettings(path, current);
      original = JSON.parse(JSON.stringify(current));
      toastStore.success(t("projectMode.settings.saved"));
    } catch (e) {
      toastStore.error(String(e));
    } finally {
      saving = false;
    }
  }

  function revert() {
    current = JSON.parse(JSON.stringify(original));
    error = null;
  }
</script>

<section class="settings-facet">
  <header>
    <h2>{t("projectMode.settings.title")}</h2>
    <p class="hint">{t("projectMode.settings.hint", { path: `${path}/.claude/settings.json` })}</p>
  </header>

  {#if loading}
    <div class="empty">{t("projectMode.settings.loading")}</div>
  {:else}
    <SectionedSettings {sections} bind:activeSection {isDirty} {error}>
      {#snippet content(section)}
        {#if section === "runtime"}
          <ProjectRuntimeEditor settings={current} onPatch={onPatch} {error} />
        {:else if section === "environment"}
          <ProjectEnvVarEditor settings={current} onPatch={onPatch} {error} />
        {:else if section === "hooks"}
          <ProjectHooksEditor settings={current} onPatch={onPatch} {error} />
        {:else}
          <ProjectAdvancedJsonEditor settings={current} onPatch={onReplace} {error} />
        {/if}
      {/snippet}
    </SectionedSettings>

    <div class="actions">
      <button
        type="button"
        onclick={save}
        disabled={!isDirty || saving || error !== null}
        class="primary"
      >{t("projectMode.settings.saveBtn")}</button>
      <button type="button" onclick={revert} disabled={!isDirty}>
        {t("projectMode.settings.revertBtn")}
      </button>
    </div>
  {/if}
</section>

<style>
  .settings-facet {
    padding: 0;
    height: 100%;
    display: flex;
    flex-direction: column;
    color: var(--text-primary);
  }
  header {
    padding: 16px 16px 8px;
  }
  h2 {
    margin: 0 0 8px;
    font-size: 16px;
    font-weight: 600;
  }
  .hint {
    color: var(--text-muted);
    font-size: 12px;
    margin: 0 0 8px;
  }
  .actions {
    display: flex;
    gap: 8px;
    padding: 12px 16px;
    border-top: 1px solid var(--border, transparent);
  }
  button {
    padding: 4px 12px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: transparent;
    color: inherit;
    cursor: pointer;
  }
  button[disabled] { opacity: 0.5; cursor: not-allowed; }
  button:hover:not([disabled]) { background: var(--bg-hover, rgba(0,0,0,0.05)); }
  button.primary {
    background: var(--accent, #2c6cff);
    border-color: var(--accent, #2c6cff);
    color: white;
  }
  .empty {
    padding: 32px;
    text-align: center;
    color: var(--text-muted);
  }
</style>
```

The pattern above uses Svelte 5's named child snippet form (`{#snippet content(section)}...{/snippet}` inside the `<SectionedSettings>` tag) — the snippet receives `activeSection` as its parameter and dispatches via `{#if}`. This is the idiomatic way to pass a `Snippet<[string]>` content prop in Svelte 5. If for some reason this doesn't compile, the alternative is to declare the snippet outside the tag and pass it explicitly: `<SectionedSettings ... {content} />` with `{#snippet content(section)}...{/snippet}` at top level.

- [ ] **Step 3: Type check**

Run: `pnpm svelte-check`
Expected: 0 errors.

- [ ] **Step 4: Manual smoke-test**

`pnpm tauri dev` → Project mode → bound project → Settings facet → verify:
- 4 section tabs: Runtime / Environment / Hooks / Advanced (JSON)
- Edit a runtime field → dirty bar appears → Save → close and reopen project → field persists
- Switch to Advanced → entire JSON visible → edit JSON → switch back to Runtime → fields reflect JSON edits
- Revert button discards unsaved changes

Close dev session.

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/project-mode/ProjectSettingsFacet.svelte src/lib/locales/
git commit -m "feat(stage5): ProjectSettingsFacet sectioned UI + 4 sub-editors + i18n (A1.3)"
```

---

## Task 11: i18n seed sweep (B1)

**Files:**
- Modify (as audit surfaces): `src/lib/locales/zh-CN.json`, `ja-JP.json` (or whichever locales the audit flags)
- Modify (if needed): `scripts/audit-i18n.whitelist.json`

- [ ] **Step 1: Run the audit**

Run: `pnpm run audit:i18n`

- If it exits 0: skip to Step 4 (commit the empty sweep).
- If it exits 1: continue.

- [ ] **Step 2: Triage each flagged entry**

For each line in the report `<locale>: <key> = <value>`:
- If the value SHOULD be that English (it's a brand, identifier, CLI command, or model name): add the key to `scripts/audit-i18n.whitelist.json#keys`.
- Otherwise: translate the value to the target locale. For zh-CN, find the corresponding `en-US` value for context and translate naturally. Same for ja-JP.

- [ ] **Step 3: Re-run until clean**

Run: `pnpm run audit:i18n`
Expected: exit 0.

- [ ] **Step 4: Commit**

```bash
git add src/lib/locales/ scripts/audit-i18n.whitelist.json
git commit -m "i18n(stage5): audit-i18n seed sweep — translate English-as-placeholder values (B1)"
```

(If nothing changed, this commit can be skipped — the audit script's existence and pnpm command are sufficient infrastructure.)

---

## Task 12: Stage 5 exit gate

**Files:** (no code changes; verification only)

- [ ] **Step 1: Full type check**

Run: `pnpm svelte-check`
Expected: 0 errors.

- [ ] **Step 2: Full Rust test sweep**

Run: `cargo test --workspace`
Expected: 159 passed + 1 ignored (Stage 4 baseline of 155 passed + 4 new A2 tests = 159).

- [ ] **Step 3: i18n audit**

Run: `pnpm run audit:i18n`
Expected: exit 0.

- [ ] **Step 4: Production build**

Run: `pnpm build`
Expected: clean exit, no warnings about missing imports / unresolved modules.

- [ ] **Step 5: B2 documentation reference check**

The Stage 5 spec includes the plugin tri-state E2E prereq doc under § Verification (E2E prerequisites). Confirm the spec file at `docs/superpowers/specs/2026-05-13-phase8-stage5-design.md` has this content; B2 has no implementation step, only the spec doc. No code commit required.

- [ ] **Step 6: Manual E2E flow**

Follow the Phase 8 spec `2026-05-11-phase8-mode-based-redesign-design.md` § Verification seven-step E2E end-to-end. Pre-requisite: complete the plugin install steps from the Stage 5 spec § Verification "E2E prerequisites" section first (B2).

If any step fails, file follow-up work as a Stage 6 candidate; do not retroactively expand Stage 5.

- [ ] **Step 7: Final commit (only if anything in this task surfaced fixes)**

If verification surfaced a fix (e.g. a missing register, an i18n typo): commit it as `fix(stage5): ...` and re-run the gate. Otherwise no commit needed for this task.

---

## Self-Review Notes

**Spec coverage:** A1 covered by Tasks 8/9/10. A2 covered by Tasks 6/7. B1 covered by Tasks 5/11. B2 (doc-only) covered by Stage 5 spec § Verification, verified in Task 12 Step 5. B3 covered by Task 4. C1/C2/C3 covered by Tasks 3/2/1 respectively. All eight items have at least one task.

**Type consistency:** `update_project_path` Rust command name matches the IPC string passed from `ipcClient.updateProjectPath`. `UpdateProjectPathRequest.old_path` (Rust snake_case) serializes to `oldPath` via `#[serde(rename_all = "camelCase")]`; client wrapper passes `{ req: { oldPath, newPath } }` accordingly. `ProjectEntry.stale` (already exists) is consumed unchanged.

**Placeholder scan:** Every code block is concrete. The phrases "follow CLAUDE.md gotcha #10" and "align field names accordingly" assume the engineer reads `src/lib/api/types.ts` if a name mismatch surfaces — this is rote alignment, not a placeholder.

**Risk callouts:**
- Task 8 / 10: The Svelte 5 `Snippet<[string]>` content-prop pattern uses named child snippets — if compilation fails, the alternative explicit-prop form is documented inline in Task 10. Test by running `pnpm svelte-check` after Task 8 (with a temporary consumer) before committing.
- Task 9: `Settings.model` / `Settings.outputStyle` / `Settings.env` / `Settings.hooks` field names are assumed. CLAUDE.md gotcha #10 directs to verify against `src/lib/api/types.ts` and `crates/claude-types/src/settings.rs` before writing the editor components.
- Task 11: the seed sweep may surface zero, few, or many English-as-placeholder values. The task is sized for "few" — if the audit surfaces dozens, consider splitting translation into a follow-up Stage 5.5 commit.
