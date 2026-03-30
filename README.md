# dot-claude-gui

> Your `.claude/` all-in-one manager GUI

A desktop application for managing Claude Code configuration, plugins, skills, memory, and MCP servers.

## Architecture

- **Frontend:** Svelte 5 + TypeScript + Tailwind CSS 4
- **GUI Shell:** Tauri 2.0
- **Backend Daemon:** Rust (axum + tokio + notify)

## Development

### Prerequisites

- Rust (latest stable)
- Node.js 20+
- pnpm

### Setup

```bash
pnpm install
```

### Run daemon (standalone)

```bash
cargo run -p claude-daemon -- --port 7890
```

### Run Tauri dev

```bash
pnpm tauri dev
```

### Run tests

```bash
cargo test --workspace
```

## Project Structure

```
dot-claude-gui/
├── crates/
│   ├── claude-types/    # Shared type definitions (Settings, API, Events)
│   ├── claude-config/   # Config parsing, merge engine, validation, file watcher
│   └── claude-daemon/   # REST/WebSocket API server with file watching
├── src-tauri/           # Tauri 2 app shell
├── src/                 # Svelte 5 frontend
│   ├── lib/api/         # REST + WebSocket clients
│   ├── lib/stores/      # Reactive state stores
│   └── lib/components/  # UI components
└── docs/                # Design specs and implementation plans
```

See [Design Spec](docs/superpowers/specs/2026-03-30-dot-claude-gui-design.md) for full architecture details.
