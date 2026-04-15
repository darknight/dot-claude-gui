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
- **Three-panel layout** in App.svelte: sidebar (resizable, 56-180px) → sub-panel (resizable, 160-400px) → detail panel
- **Sidebar** uses SVG icons (Heroicons outline) + text labels; text auto-hides at narrow widths
- **9 navigation modules:** Settings, Plugins, Skills, Memory, CLAUDE.md, MCP, Effective Config, Launcher, App Settings
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

6. **Project path decoding is ambiguous.** Claude Code encodes `/` as `-` in `~/.claude/projects/<dirname>`, so `whoishiring-insight` and `whoishiring/insight` encode to the same string. When you need the real path, read `cwd` from any session `.jsonl` file inside the directory (see `src-tauri/src/commands/memory.rs::read_cwd_from_sessions`).

7. **Validation lists drift from Claude Code's schema.** Hook event names, settings keys, etc. Source of truth is `https://json.schemastore.org/claude-code-settings.json`. If a save fails with "unknown X", check the schema before assuming the user's config is wrong.

8. **`config-changed` events carry `source: "file-watcher"`** (not `"user"`). Filter handlers accordingly — overly strict source checks silently break live reload.
