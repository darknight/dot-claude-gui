# Phase 8 — Mode-based Redesign · Design Spec

Date: 2026-05-11
Status: Draft (pending user approval)
Supersedes: `2026-04-30-account-switch-design.md` (only the launcher-bolt-on subset; the multi-account capabilities are kept)
Aligns with: `2026-04-10-plan-d-account-workspace-design.md` (revives the core abstraction; explicitly drops shared plugin pool and `dotclaude-launch` CLI)

> **Phase numbering**: this doc is "Phase 8" in the project's overall sequence (1–6 done; 7.1 done; 7.2–7.5 from Plan D skipped in favour of the lighter 4-30 account-switch route). Inside this doc the 4 implementation chunks are called **Stages 1–4** to avoid "phase within phase" confusion. Future docs continue the outer Phase numbering.

## Context

After implementing the bolted-on multi-account flow (2026-04-30 spec) the product feels structurally broken in four ways:

1. **Scope selector is half-respected.** A global project/scope picker takes effect in some modules (Memory) and is ignored in others (MCP, Skills). The abstraction itself isn't defensible.
2. **Multi-account was added last.** Account became a field on `launcherProjectEnv` rather than a first-class dimension. Cross-module flows don't treat it consistently.
3. **Cross-dimensional integration fails.** Different projects need different accounts AND different plugins/skills. Plugins install only at the account level and there's no per-project override — so the workflow is impossible.
4. **UI doesn't read clearly.** Module split feels like traditional config management, not designed for the multi-account × multi-project reality.

This spec lays out a structural redesign that makes account and project explicit top-level perspectives (mirroring Claude Desktop's mode-tab pattern), eliminates the global scope selector, and gives every facet exactly one obvious home.

## Confirmed user decisions

1. Drop the global scope selector at top; replace with **mode tabs (Accounts / Projects)** + right-corner App settings.
2. **Two-panel layout** (sidebar = item list, main = facets-as-tabs). Drop the existing middle list panel.
3. **No shared plugin pool** — each account installs independently (deferred from Plan D, not relitigated).
4. **Project binding stored centrally** in `~/.dot-claude-gui/config.json`, not in per-project `.claude/dotclaude.json`.
5. **Native `~/.claude/` registered as `default` account** — explicit entry in config, non-deletable.
6. **Data directory kept as `~/.dot-claude-gui/`** (no rename to `~/.dot-claude/`).
7. **No backwards compatibility / field doubling** during migration. Direct switch (pre-release product, no users to protect).
8. Per-facet placement decisions (see Information Architecture).

## Information Architecture

### Top-level navigation

- `👤 Accounts` mode tab
- `📂 Projects` mode tab
- `⚙ App` settings — right-corner gear button (not a mode)

### Account Mode

| Component | Content |
|---|---|
| Sidebar | List of accounts (default + GUI-created); `+ Add Account` button pinned bottom |
| Main | Selected account's facets as a top-tab strip |

Account facets:
- **Overview** — status card (logged-in, project/plugin/skill counts), actions (Re-login / Open dir / Delete)
- **Settings** — edits `<account-dir>/settings.json` (user layer)
- **Plugins** — installed list + enablement toggle; marketplace browse and management
- **Skills** — list (built-in + plugin-contributed + user-created)
- **CLAUDE.md** — edits `<account-dir>/CLAUDE.md`
- **Memory** — index of projects under this account; click into a project to see its memory files
- **MCP** — server CRUD

### Project Mode

| Component | Content |
|---|---|
| Sidebar | Projects grouped by bound account + Unbound section; `+ Add Project` button |
| Main | Selected project's facets |

Project facets:
- **Binding** — bound-account picker, path readonly, `[Open Terminal] [Unbind] [Remove]`. Always available.
- **Launch** — env / args editors with `claude --help` autocomplete, `[Launch Claude Code]` button. Absorbs current Launcher.
- **Plugins ↓** — tri-state enablement override (Disable / Inherit / Enable) per plugin, writing to `<project>/.claude/settings.json`'s `enabledPlugins`
- **Settings** — edits `<project>/.claude/settings.json` (project layer; all keys allowed, no whitelist)
- **Memory** — `<account-dir>/projects/<encoded-path>/memory/` CRUD
- **CLAUDE.md** — edits `<project>/.claude/CLAUDE.md`
- **Effective** — read-only merged view (managed → user → project → local)

### Edge cases

- **Unbound projects**: only **Binding** tab clickable; others greyed with `select an account first` hint.
- **Path-stale projects** (path in `knownProjects` but missing on disk): all tabs disabled; top banner offers `Update path / Remove`.
- **`default` account**: Delete button disabled with tooltip "default 账号不可删除"; otherwise behaves like any other account.

### Layout choices

- Sidebar fixed width ~240px, resizable.
- Main facet tabs are top-tabs in a horizontal strip (7 fits comfortably).
- App settings panel (gear) reuses the same two-column layout but item list is preference categories (Appearance / Language / Terminal / About).

## Data Model

### File layout

```
~/.dot-claude-gui/
├── config.json               # single source of truth
└── accounts/
    ├── work/                 # GUI-created account = CLAUDE_CONFIG_DIR target
    │   ├── settings.json
    │   ├── .claude.json      # OAuth (Claude Code writes)
    │   ├── plugins/, projects/, sessions/, ...
    └── me/
        └── ...
```

`~/.claude/` is **not moved** — only referenced by the `default` account entry in `config.json`.

### `config.json` schema (v2)

```jsonc
{
  // ── App preferences ───────────────────────────────────────────────
  "theme": "auto" | "light" | "dark",
  "language": "zh-CN" | "en-US" | "ja-JP",
  "fontSize": 14,
  "sidebarWidth": 240,
  "preferredTerminal": "terminal" | "iterm2",

  // ── Account registry — includes default explicitly ───────────────
  "accounts": [
    { "name": "default", "displayName": "Native ~/.claude/",
      "isNative": true, "createdAt": "<auto-injected>" },
    { "name": "work",    "displayName": "Company Team Plan",
      "isNative": false, "createdAt": "..." },
    { "name": "me",      "displayName": "Personal Pro",
      "isNative": false, "createdAt": "..." }
  ],

  // ── Project bindings (path → { account, launch }) ────────────────
  "projects": {
    "/Users/eric/code/my-app": {
      "account": "work",
      "launch": {
        "env":  { "API_ENDPOINT": "staging" },
        "args": ["--effort", "high"]
      }
    }
  },

  // ── Project registry — paths user has explicitly added ───────────
  "knownProjects": [
    "/Users/eric/code/my-app",
    "/Users/eric/tmp/experiment-x"
    // path in knownProjects but missing from projects = Unbound
  ]
}
```

### Default account semantics

- On app startup: if `~/.claude/` exists and `accounts` doesn't already contain `default`, write `{ name: "default", isNative: true, displayName: "Native ~/.claude/", createdAt: <now> }` to config.
- When launching: `account === "default"` → **do not** inject `CLAUDE_CONFIG_DIR` (let claude use its native `~/.claude/`).
- For non-default accounts: inject `CLAUDE_CONFIG_DIR=~/.dot-claude-gui/accounts/<name>`.
- UI: Delete button on default is disabled; the Overview displays config dir as `~/.claude/`.

### Stale-path detection

On startup, scan `knownProjects` and mark entries whose path doesn't resolve to an existing directory. Don't auto-remove; surface in UI for explicit user action.

## Module migration matrix

Every existing module finds a home in the new structure. No feature is lost.

| Existing module | New location |
|---|---|
| Settings | Account > Settings (user layer) **+** Project > Settings (project layer) |
| Plugins | Account > Plugins (install + enable) **+** Project > Plugins ↓ (tri-state override — new) |
| Skills | Account > Skills |
| Memory | Account > Memory (per-project index) **+** Project > Memory |
| CLAUDE.md | Account > CLAUDE.md **+** Project > CLAUDE.md |
| MCP | Account > MCP |
| Effective Config | Project > Effective (not in Account mode) |
| Launcher | Project > Launch **+** Project > Binding (account selection) |
| App Settings | Right-corner ⚙ panel |

Newly introduced facets: **Account > Overview**, **Project > Binding**, **Project > Plugins ↓**.

## Migration Path

One-time on first launch of the new version:

1. Read old `config.json`.
2. For each `launcherProjectEnv[<path>]`:
   - Create `projects[<path>] = { account: <accountName ?? "default">, launch: { env: <customEnv>, args: <customArgs> } }`
   - Add `<path>` to `knownProjects`
3. Re-shape each entry in old `accounts`: add `displayName` (default = `name`), `isNative: false`.
4. If `~/.claude/` exists, append the `default` account entry.
5. Drop `subpanelWidth` (two-panel layout has no middle column).
6. **Snapshot order**: copy old `config.json` → `config.json.bak.<unix>`; then atomically write new config (temp file + rename). The `.bak` is the pre-migration state, not the new one.
7. Show toast on app open: `Config migrated · N projects bound · [View changes]`.

Account directories under `~/.dot-claude-gui/accounts/` are untouched. `~/.claude/` is untouched.

## Implementation stages

Single design, four stages. Each stage ends in a usable state.

### Stage 1 — Data layer + migration (medium risk)

- New `config.json` schema (Rust types + frontend types)
- Migration code + backup snapshot
- Default account auto-injection on startup
- New IPC commands: `list_projects`, `add_project`, `bind_project`, `unbind_project`, `remove_project`, `update_project_launch`
- Existing IPC kept where it still maps (account CRUD, plugins, skills, memory, mcp, claudemd)
- **No old-field double-write.** Old fields removed in this stage.
- UI temporarily broken — accept this since Stage 2 immediately follows

Acceptance: existing `~/.dot-claude-gui/` migrates cleanly; bak file exists; new schema validates; default account appears.

### Stage 2 — New shell + Accounts mode (medium risk)

- Top mode-tab bar + right-corner gear
- Mode-aware sidebar component (renders accounts list / projects list / app prefs list)
- Two-panel layout (drop middle column)
- Account mode: all 7 facets wired to existing backends
  - Overview: new
  - Settings/Plugins/Skills/CLAUDE.md/Memory/MCP: existing module contents moved into facet containers
- Project mode: sidebar renders project list; main shows `Coming in Stage 3` placeholder

Acceptance: all account facets render real data; toggle widgets persist; mode switching is instant; existing operations (login, install plugin, edit CLAUDE.md) all still work.

### Stage 3 — Projects mode complete (low-medium risk)

- All 7 Project facets functional
- Launcher module code relocated to Project > Launch
- Effective Config code relocated to Project > Effective
- New: Project > Binding UI (account selector + actions)
- New: Project > Plugins ↓ (tri-state override, writes to project `settings.json`'s `enabledPlugins`)
- Unbound-project degradation: only Binding tab clickable

Acceptance: project facets all functional; Launcher and Effective Config also accessible via Projects mode (old sidebar routes still there for safety until Stage 4); tri-state plugin override writes to `<project>/.claude/settings.json` and is reflected in Project > Effective.

### Stage 4 — Cleanup (low risk)

- Delete old top-level routes: Settings/Plugins/Skills/Memory/CLAUDE.md/MCP/Effective Config/Launcher/AppSettings (sidebar entry)
- Complete the right-corner gear panel (Appearance / Language / Terminal / About)
- i18n: complete zh-CN / en-US / ja-JP strings for all new copy
- Final migration toast / changelog summary view
- Remove unused IPC commands

Acceptance: no old module routes accessible; gear panel covers all previous App Settings; no English fallbacks in zh-CN UI; no dead Rust code.

## Out of scope (explicit non-goals)

- Shared plugin pool with symlinks (deferred indefinitely; revisit only if disk footprint becomes a problem)
- `dotclaude-launch` CLI binary (deferred; central binding storage keeps it feasible later)
- All-accounts bulk-edit settings mode
- Per-project `.claude/dotclaude.json` binding file
- Migration from existing `ccs` instances
- Backwards-compat shims / field-doubling

## Verification

### Acceptance per stage

See each stage above.

### End-to-end check after Stage 4

1. Launch new version on a machine with existing data → toast confirms migration.
2. Account mode → `@work` → Plugins → enable `typescript-lsp` → toggle ON.
3. Project mode → `my-app` (bound to @work) → Plugins ↓ → override `typescript-lsp` to Disable.
4. Project > Effective shows `typescript-lsp` disabled (from project layer).
5. Project > Launch → `[Launch Claude Code]` → terminal opens with `CLAUDE_CONFIG_DIR=~/.dot-claude-gui/accounts/work`, project cwd, and the env/args set in Launch tab.
6. App settings gear → switch language to en-US → all UI strings re-render.
7. Unbound project (e.g. `experiment-x`) → all facets greyed except Binding; banner explains.

## Reference

Interactive prototype reflecting this design is generated under `.superpowers/brainstorm/<session>/content/interactive-prototype.html` (gitignored). It validates the structure, interaction rhythm, and visual density. Rebuild from this spec if the prototype is lost.
