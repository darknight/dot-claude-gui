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
