# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

dot-claude-gui — a desktop app for managing Claude Code's `~/.claude/` configuration. Tauri 2.0 shell wrapping a Svelte 5 frontend that talks to a Rust backend daemon over REST + WebSocket.

## Commands

```bash
pnpm install                              # Install frontend dependencies
cargo build -p claude-daemon              # Build daemon binary (needed before tauri dev)
mkdir -p src-tauri/binaries && cp target/debug/claude-daemon src-tauri/binaries/claude-daemon-aarch64-apple-darwin
pnpm tauri dev                            # Run full app with hot reload (builds daemon sidecar)
cargo run -p claude-daemon -- --port 7890  # Run daemon standalone
cargo test --workspace                    # Run all Rust tests
cargo test -p claude-config               # Run tests for a single crate
cargo test -p claude-daemon               # Run tests for daemon crate
pnpm build                                # Build frontend only
pnpm tauri build                          # Build production .app and .dmg
```

**First-time dev setup:** Build the daemon binary and place it as a sidecar before running `pnpm tauri dev`. The Tauri build script expects `src-tauri/binaries/claude-daemon-aarch64-apple-darwin` to exist.

## Architecture

**Three layers:**
- **Svelte 5 frontend** (`src/`) — UI with rune-based reactivity (`$state`, `$effect`, `$derived`)
- **Tauri shell** (`src-tauri/`) — manages sidecar daemon lifecycle, IPC commands for config dir access
- **Rust daemon** (`crates/claude-daemon/`) — axum REST/WebSocket server with file watching

**Rust workspace crates** (`crates/`):
- `claude-types` — shared types (settings, API, events, plugins, skills, memory, MCP)
- `claude-config` — config file parsing, merge engine, atomic writes (temp file → rename)
- `claude-daemon` — axum server, route handlers, file watcher, app state

**Frontend structure** (`src/lib/`):
- `api/client.ts` — `DaemonClient` REST wrapper
- `api/ws.ts` — `DaemonWsClient` with auto-reconnect and exponential backoff
- `api/types.ts` — API request/response TypeScript types
- `stores/*.svelte.ts` — singleton Svelte 5 rune stores (connection, connections, config, projects, plugins, skills, memory, mcp, appsettings)
- `components/` — organized by module (settings, plugins, skills, memory, mcp, effective, launcher, appsettings, shared)

**Config hierarchy** (merged bottom-up): Managed defaults → User (`~/.claude/settings.json`) → Project (`.claude/settings.json`) → Local (ephemeral)

**Real-time sync:** Backend watches `~/.claude/` and project dirs with `notify` crate, debounces changes, broadcasts `configChanged` events over WebSocket to all connected clients.

## App Config Directory

`~/.dot-claude-gui/` stores application-level state (not Claude Code config):
- `config.json` — GUI preferences (theme, language, font size, panel widths)
- `connections.json` — daemon connection registry (local sidecar + remote connections, tokens)

## Multi-Daemon Connections

The GUI supports connecting to multiple daemon instances (local + Docker/remote):
- **Local sidecar** — auto-started by Tauri on launch, auto-assigned port and token
- **Remote daemons** — user-configured in App Settings > Connections
- Header shows: `[Environment selector] → [Project selector]`
- Switching environment disconnects, resets all stores, reconnects to new daemon
- All stores have `reset()` methods for clean environment switching

## Key Conventions

- **Svelte 5 runes only** — no legacy `$:` reactive statements, use `$state`, `$effect`, `$derived`
- **Tailwind CSS 4** via `@tailwindcss/vite` plugin
- **CSS variable theming** — colors defined in `app.css` as `:root` (light) and `.dark` (dark) variables; use `var(--bg-primary)` etc. instead of hardcoded Tailwind color classes in layout components
- **Three-panel layout** in App.svelte: sidebar (resizable, 56-180px) → sub-panel (resizable, 160-400px) → detail panel
- **Sidebar** uses SVG icons (Heroicons outline) + text labels; text auto-hides at narrow widths
- **8 navigation modules:** Settings, Plugins, Skills, Memory, MCP, Effective Config, Launcher, App Settings
- **TypeScript strict mode** enabled
- **pnpm** as package manager (not npm/yarn)
- Daemon CLI: `--port`, `--bind` (default 127.0.0.1, use 0.0.0.0 for Docker), `--token`
- Frontend has no test suite; all tests are Rust-side with `cargo test`
