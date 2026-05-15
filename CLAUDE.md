# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

dot-claude-gui — a desktop app for managing Claude Code's `~/.claude/` configuration. Tauri 2.0 shell wrapping a Svelte 5 frontend that talks to a Rust backend via Tauri IPC commands.

## Commands

```bash
pnpm install                              # Install frontend dependencies
pnpm tauri dev                            # Run full app with hot reload
cargo test --workspace                    # Run all Rust tests
cargo test -p claude-config               # Run tests for a single crate
pnpm build                                # Build frontend only
pnpm tauri build                          # Build production .app and .dmg
```

## Architecture

**Two layers:**
- **Svelte 5 frontend** (`src/`) — UI with rune-based reactivity (`$state`, `$effect`, `$derived`)
- **Tauri shell** (`src-tauri/`) — Rust backend with in-process IPC commands, file watcher, and subprocess executor

**Rust workspace crates** (`crates/`):
- `claude-types` — shared types (settings, API, events, plugins, skills, memory, MCP)
- `claude-config` — config file parsing, merge engine, atomic writes (temp file → rename)

**Tauri backend modules** (`src-tauri/src/`):
- `commands/` — IPC command handlers per domain (config, projects, plugins, mcp, skills, claudemd, memory, launcher, health)
- `state.rs` — shared AppState (settings caches, project registry)
- `watcher.rs` — notify-based file watcher emitting `config-changed` / `validation-error` Tauri events
- `executor.rs` — streaming subprocess runner for `claude` CLI invocations (plugin install, mcp add, etc.), emits `command-output` / `command-completed` Tauri events
- `events.rs` — event payload type definitions

**Frontend structure** (`src/lib/`):
- `ipc/client.ts` — `IpcClient` wrapping Tauri `invoke()` calls (32 methods mirroring the backend commands)
- `ipc/events.ts` — Tauri event listeners (`onConfigChanged`, `onCommandOutput`, `onCommandCompleted`, `onValidationError`)
- `api/types.ts` — Shared TypeScript type definitions (Settings, ConfigResponse, etc.)
- `stores/*.svelte.ts` — singleton Svelte 5 rune stores (config, projects, plugins, skills, memory, mcp, claudemd, appsettings, toast)
- `components/` — organized by module (settings, plugins, skills, memory, mcp, effective, launcher, appsettings, claudemd, shared)

**Config hierarchy** (merged bottom-up): Managed defaults → User (`~/.claude/settings.json`) → Project (`.claude/settings.json`) → Local (ephemeral)

**Real-time sync:** Tauri backend watches `~/.claude/` and project dirs with the `notify` crate, reads updated settings, updates the in-memory cache, and emits `config-changed` Tauri events. The frontend subscribes via `onConfigChanged` helper in `src/lib/ipc/events.ts`.

## App Config Directory

`~/.dot-claude-gui/` stores application-level state (not Claude Code config):
- `config.json` — GUI preferences (theme, language, font size, panel widths)

## Key Conventions

- **Svelte 5 runes only** — no legacy `$:` reactive statements, use `$state`, `$effect`, `$derived`
- **Tailwind CSS 4** via `@tailwindcss/vite` plugin
- **CSS variable theming** — colors defined in `app.css` as `:root` (light) and `.dark` (dark) variables; use `var(--bg-primary)` etc. instead of hardcoded Tailwind color classes in layout components
- **Mode-based shell** in App.svelte: top mode tabs (Accounts / Projects) + right-corner gear; per-mode 2-panel layout (mode-aware sidebar → main facet area)
- **Account mode** (sidebar = accounts list) renders 7 facets: Overview, Settings, Plugins, Skills, CLAUDE.md, Memory, MCP
- **Project mode** (sidebar = projects list) renders 7 facets: Binding, Launch, Plugins↓ (tri-state override), Settings, CLAUDE.md, Memory, Effective
- **Gear panel** (modal): Appearance / Language / Terminal / About
- **TypeScript strict mode** enabled
- **pnpm** as package manager (not npm/yarn)
- Frontend has no test suite; all tests are Rust-side with `cargo test`
- **i18n** — user-facing strings go through `src/lib/i18n.ts` with `t("key", params)`. Supports `zh-CN` / `en-US` from `appSettingsStore.preferences.language`. Do NOT hardcode Chinese or English text in components.

## Svelte 5 Gotchas (hard-learned)

These caused multi-round debugging sessions. Check here FIRST when UI doesn't update as expected.

1. **HMR does not rebuild the reactive graph for script changes.** Adding/removing `$state`, `$derived`, `$effect` in `<script>` often looks like it applied (Vite logs "hmr update") but the running component keeps the stale graph. Template-only edits HMR reliably. **When you add new runes or change effect bodies, kill and restart `pnpm tauri dev`** — don't trust HMR.

2. **`onDestroy` must be called synchronously during component setup.** Calling it inside an `await`-ed callback (e.g. after `onConfigChanged()` resolves) throws `lifecycle_outside_component` and **silently corrupts the component's reactive state** — `{#if}` chains stop re-evaluating, events fire but UI doesn't update. Use the cleanup function returned from `onMount(() => { ...; return () => {...} })` and store async unlisteners in a module-level variable.

3. **`{#each}` keys must be globally unique.** `(item.id)` fails with `each_key_duplicate` when two items share an id from different sources (e.g. plugin-contributed skills with the same name). Use compound keys: `(item.id + ':' + item.source)`.

4. **Prefer direct state comparisons in `{#if}` over helper functions.** `{#if activeNav === "S"}` is reliable; `{#if isSettings()}` or `{#if isSettingsDerived}` can fail to re-render in `{:else if}` chains. When in doubt, inline the comparison.

5. **Open Tauri DevTools (`Cmd+Option+I`) before debugging UI bugs.** The Console almost always has the real error — Svelte lifecycle errors, each-key duplicates, null IPC params. Grepping source is slower than reading one error line.

6. **Project path encoding/decoding is ambiguous — and `.` counts too.** Claude Code encodes BOTH `/` and `.` as `-` in `~/.claude/projects/<dirname>`, so `/Users/eric.yao/...` lands on disk as `-Users-eric-yao-...`. Collisions: `whoishiring-insight` vs `whoishiring/insight`, and `eric.yao` vs `eric-yao`. When you need the real path, read `cwd` from any session `.jsonl` file inside the directory (see `src-tauri/src/commands/memory.rs::read_cwd_from_sessions`). Two gotchas hit in practice: (a) recent Claude Code writes a summary/index header as the **first** jsonl line with no `cwd` — scan every line, don't bail on line 1; (b) when encoding path → dirname, replace `.` along with `/` (see `encode_project_path` in `commands/project_facets.rs`). `project_path.replace('/', "-")` alone is wrong.

7. **Validation lists drift from Claude Code's schema.** Hook event names, settings keys, etc. Source of truth is `https://json.schemastore.org/claude-code-settings.json`. If a save fails with "unknown X", check the schema before assuming the user's config is wrong.

8. **`config-changed` events carry `source: "file-watcher"`** (not `"user"`). Filter handlers accordingly — overly strict source checks silently break live reload.

9. **Tauri IPC request structs need explicit `#[serde(rename_all = "camelCase")]`.** Top-level command params (`fn cmd(project_path: String)` → `{ projectPath }` from JS) auto-rename, but inner struct fields do NOT. A `WriteFooRequest { project_path }` without `rename_all` requires JS to send `{ project_path }` snake_case or the call fails at runtime with "missing field". Stage 3 hit this — landed `feat(stage3): IPC client wrappers...` then needed `fix(stage3): camelCase rename_all...` immediately after.

10. **TS ↔ Rust type drift on shared shapes is silent until used.** `Settings.enabledPlugins` was `string[]` in `src/lib/api/types.ts` long after Rust changed to `HashMap<String, bool>`. Since old code only read the field as a free-form, tsc didn't complain. New code assuming the Rust shape will type-error or worse, work-then-fail at IPC boundary. When adding a new consumer of a `claude-types::Settings` field, **diff the Rust definition against `src/lib/api/types.ts`** first.

11. **`tauri::App::setup` runs BEFORE the WebView exists — `app.emit(...)` from inside `setup` is silently lost.** Frontend listeners attach in `onMount`, which runs after the JS bundle loads (hundreds of ms after setup returns). Tauri events are not buffered for late subscribers. For one-shot setup-time signals (migration reports, first-run flags, etc.), stash the payload in `AppState` and expose a one-shot IPC the frontend pulls on mount. Stage 4 hit this: `feat(stage4): emit and surface v1→v2 migration toast` (commit `144bb3e`) used emit; the toast never fired in practice; the fix `fix(stage4): migration report — switch from emit to IPC pull` (commit `ff019dc`) introduced `take_migration_report`. Use IPC-pull, not emit, for anything fired during setup.

12. **Tauri 2 split `emit()` off `Manager` into a separate `Emitter` trait.** `use tauri::Manager;` is no longer enough to call `app.emit("...")` — you also need `use tauri::Emitter;`. Tauri 1 had this on `Manager`. Cargo's error is `method not found in '&App'` (or `&AppHandle`), which doesn't point at the missing import.

13. **Svelte `{...}` inside double-quoted HTML attributes is JS interpolation, not literal text.** `placeholder="https://x.com/{owner}/{repo}/pull/{number}"` triggers `Cannot find name 'owner'` / `'number' only refers to a type, but is being used as a value here` from svelte-check, because Svelte parses the bracketed names as JS expressions. To put literal `{token}` placeholders into an attribute, wrap the whole string as a JS expression: `placeholder={"https://x.com/{owner}/{repo}/pull/{number}"}`. Stage 4 spent multiple debugging cycles on this one before realizing it was an attribute-parse issue.

14. **`initialized` flags don't work as same-batch guards between `$effect`s.** A common-looking pattern — effect A syncs local state from a store and flips `initialized = true`; effect B watches that state and does `if (initialized) markDirty()` — silently fires `markDirty()` on every mount. Svelte 5 runs both effects in the same microtask batch in declaration order: by the time B runs for the first time, A has already set `initialized = true`, so the "skip first run" intent fails. Symptom in this repo: switching into the Permissions / Sandbox settings sub-tab flagged "unsaved changes" with zero user input. Fix: don't proxy through `length` + a flag; have B compare local state against the store's real value (`!arraysEqual(local, store)`) so the post-sync equality short-circuits.
