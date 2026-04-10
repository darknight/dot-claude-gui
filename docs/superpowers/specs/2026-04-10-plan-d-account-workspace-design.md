# Plan D: Account-Centric Workspace Manager Design

## Problem

Current dot-claude-gui was designed around a multi-daemon remote management scenario that doesn't match the user's actual needs. The real pain points are:

1. **Multi-account Claude Code subscriptions**: The user has two Claude Code accounts — a company-paid team plan for work projects and a personal subscription for side projects. They need per-project credential isolation to maximize token utilization across both accounts.

2. **Plugin duplication across accounts**: The user currently uses CCS (`kaitranntt/ccs`) for account switching, but CCS creates separate `CLAUDE_CONFIG_DIR` instances where plugins must be installed twice. Investigation of CCS's source confirms this is a technical consequence of Claude Code writing absolute instance paths into `plugins/known_marketplaces.json`, not a CCS design limitation.

3. **Complex manual configuration**: Claude Code's configuration lives in `settings.json` and environment variables. Users must hand-edit JSON and manage shell exports. Hooks in particular are hard to observe — many skills install hooks but there's no unified view.

4. **CLI-only tooling**: User explicitly wants a GUI-first experience. Memorizing CLI commands for an infrequently-used configuration tool defeats the purpose.

Target: replace CCS entirely and provide a unified GUI for managing Claude Code configuration across multiple accounts and projects.

## Technical Verification

A spike confirmed the feasibility of this architecture:

1. **`CLAUDE_CONFIG_DIR` is officially supported** by Claude Code. Setting it redirects all of Claude Code's state (`.claude.json`, `backups/`, `projects/`, `sessions/`, etc.) to the specified directory. OAuth session is naturally isolated — two separate `CLAUDE_CONFIG_DIR` values produce two independent "Not logged in" states, neither of which reads the global `~/.claude/` or `~/.claude.json`.

2. **Symlinks survive Claude Code startup**. `$CLAUDE_CONFIG_DIR/settings.json → shared/settings.json` works: Claude Code reads through the symlink without overwriting it.

3. **`installed_plugins.json` accepts absolute `installPath` values pointing outside `$CLAUDE_CONFIG_DIR`**. Confirmed by inspecting CCS instances — all `installPath` entries point to `~/.claude/plugins/cache/...` regardless of the instance directory. This enables a shared physical plugin pool.

4. **`known_marketplaces.json` uses instance-relative `installLocation` paths**. This is the root cause of CCS's plugin duplication. We solve it by having our GUI act as the canonical installer, writing to all accounts' metadata files simultaneously.

## Scope

### In scope

- Account management (create, login, delete) using `CLAUDE_CONFIG_DIR` isolation
- Project-to-account binding via `.claude/dotclaude.json`
- Shared plugin pool with per-account and per-project enablement
- Per-project launch customization (CLI args + environment variables)
- GUI modules: Accounts, Projects, Settings (with "All accounts" mode), Plugins, Skills, Memory, CLAUDE.md, MCP, Effective Config
- `dotclaude-launch` CLI for terminal integration
- Migration path from existing `~/.claude/` setup

### Out of scope

- Remote daemon / multi-machine management
- Web UI / browser-based access
- Anthropic API integrations beyond what Claude Code itself provides
- Automatic OAuth token management — Claude Code owns all credential storage

### Not changing

- Phase 6 features (Toast notifications, Skill content preview, CLAUDE.md editor) are preserved
- Svelte 5 frontend framework and rune-based reactivity
- Tauri 2.0 as the desktop shell
- CSS variable theming, three-panel layout, Chinese UI labels

## Design

### 1. Directory Structure

```
~/.dot-claude/                          Data root owned by this tool
│
├── config.json                         GUI self-config (theme, language, window state)
│
├── accounts.json                       Account registry
│
├── accounts/                           Per-account data (= CLAUDE_CONFIG_DIR target)
│   ├── work/
│   │   ├── .claude.json                OAuth session (Claude Code owns)
│   │   ├── settings.json               Account settings (includes enabledPlugins)
│   │   ├── plugins/
│   │   │   ├── installed_plugins.json  Physical file; installPath → shared pool
│   │   │   ├── known_marketplaces.json Physical file; installLocation → shared pool via symlink
│   │   │   ├── cache → ../../shared/plugins/cache              (directory symlink)
│   │   │   ├── marketplaces → ../../shared/plugins/marketplaces (directory symlink)
│   │   │   └── blocklist.json → ../../shared/plugins/blocklist.json (file symlink)
│   │   ├── agents → ../../shared/agents           (directory symlink)
│   │   ├── commands → ../../shared/commands       (directory symlink)
│   │   ├── skills → ../../shared/skills           (directory symlink)
│   │   ├── CLAUDE.md → ../../shared/CLAUDE.md     (file symlink)
│   │   └── (session state: projects/, sessions/, history.jsonl,
│   │        file-history/, logs/, session-env/, shell-snapshots/,
│   │        tasks/, todos/, plans/, debug/, paste-cache/, backups/,
│   │        policy-limits.json, stats-cache.json)  [all per-account, no symlinks]
│   └── me/
│       └── (same structure)
│
├── shared/                             Shared data across accounts
│   ├── agents/                         Custom agents
│   ├── commands/                       Custom slash commands
│   ├── skills/                         User skills
│   ├── CLAUDE.md                       Global instructions
│   └── plugins/
│       ├── cache/                      Physical plugin files (version-scoped)
│       ├── marketplaces/               Cloned marketplace repos
│       └── blocklist.json              Global plugin blocklist
│
├── known_projects.json                 GUI-side bookmarks of managed projects
└── .lock                               File lock for concurrent GUI writes
```

**Key points:**
- `CLAUDE_CONFIG_DIR` for each account is `~/.dot-claude/accounts/<id>/`
- Session state directories are per-account (no symlinks). Sharing these would break session isolation.
- `policy-limits.json` and `stats-cache.json` are per-account so quota/usage tracking is independent.
- `installed_plugins.json` and `known_marketplaces.json` are physical files (not symlinks) because each account needs to independently record its plugin state, but their internal paths point to shared locations.

### 2. Account Model

#### 2.1 accounts.json schema

```json
{
  "accounts": [
    {
      "id": "work",
      "name": "Company (Team Plan)",
      "type": "oauth",
      "createdAt": "2026-04-10T...",
      "launch": {
        "args": {
          "verbose": true
        },
        "env": {
          "ANTHROPIC_LOG": "info"
        }
      }
    },
    {
      "id": "me",
      "name": "Personal",
      "type": "oauth",
      "createdAt": "2026-04-10T...",
      "launch": { "args": {}, "env": {} }
    }
  ],
  "defaultAccount": "me"
}
```

#### 2.2 Account lifecycle

**Create**: GUI prompts for display name and auto-generates an id. Creates `~/.dot-claude/accounts/<id>/` with all symlinks initialized. Seeds `settings.json` from `shared/settings.template.json` if present. Writes to `accounts.json`.

**Login**: GUI spawns `CLAUDE_CONFIG_DIR=~/.dot-claude/accounts/<id>/ claude /login` as a subprocess. Claude Code's native OAuth flow handles browser authentication and writes `.claude.json` to the account directory.

**Use**: `dotclaude-launch` reads bindings and sets `CLAUDE_CONFIG_DIR` before exec'ing `claude`.

**Edit**: GUI modifies display name or launch defaults. Writes to `accounts.json`.

**Delete**: GUI prompts confirmation, warns about session history / OAuth loss, `rm -rf`s the account directory, removes from `accounts.json`. All `.claude/dotclaude.json` files referencing the deleted account become stale — GUI marks them invalid on next scan.

Account `id` is immutable once created because it's referenced by directory names and dotclaude.json files.

#### 2.3 Credential storage

We do not store OAuth tokens. Claude Code writes them into `$CLAUDE_CONFIG_DIR/.claude.json` and (on recent versions) into macOS Keychain entries keyed by config dir. Deleting an account directory also invalidates its OAuth state.

### 3. Project Binding

#### 3.1 `.claude/dotclaude.json` schema

```json
{
  "account": "work",
  "launch": {
    "args": {
      "dangerously-skip-permissions": true,
      "effort": "high",
      "add-dir": ["../shared-lib", "../docs"]
    },
    "env": {
      "PROJECT_API_ENDPOINT": "https://staging.example.com"
    }
  }
}
```

- `account` is required. Refers to an account `id` in `accounts.json`.
- `launch` is optional. Everything under it is project-level override on top of account defaults.
- Future fields can be added under additional top-level keys without breaking existing files.

#### 3.2 Launch args serialization

`launch.args` is a dict (not array) for clean override semantics:

| Value type | CLI output |
|-----------|-----------|
| `true` | `--key` (bare flag) |
| `false` | omitted (allows project to disable an account default) |
| string or number | `--key value` |
| array | `--key value1 --key value2 ...` (repeated) |
| `null` | omitted |

#### 3.3 Launch merge rules

Priority order (lowest to highest):

1. Account defaults (`accounts.json[id].launch`)
2. Project overrides (`.claude/dotclaude.json.launch`)
3. Direct command-line args passed to `dotclaude-launch`

Per-key last-wins. `launch.env` merges the same way.

Example:
- Account `work`: `{ verbose: true, effort: "medium" }`
- Project `~/work/critical`: `{ dangerously-skip-permissions: true, effort: "high" }`
- Merged: `{ verbose: true, effort: "high", dangerously-skip-permissions: true }`
- Command: `claude --verbose --effort high --dangerously-skip-permissions`

#### 3.4 `settings.json env` vs `launch.env`

Both inject environment variables but at different layers:

- `settings.json env`: Claude Code reads and applies these internally. Good for long-term configuration values Claude Code itself consumes.
- `launch.env`: `dotclaude-launch` sets these in the subprocess environment before exec'ing `claude`. Good for project-specific runtime values like proxies and log levels.

GUI presents them in separate sections to avoid confusion.

#### 3.5 `dotclaude-launch` resolution algorithm

```
fn resolve_launch_config(cwd):
    config = find_upward(cwd, ".claude/dotclaude.json")
    if config exists:
        account = load_account(config.account)
        if account not found: error with suggestion to reconfigure
        return merge(account.launch, config.launch)
    else:
        if stdin is not a TTY:
            error "project not bound; run from an interactive shell or use GUI"
        account_id = interactive_prompt_for_account()
        write dotclaude.json with { "account": account_id }
        return accounts[account_id].launch
```

#### 3.6 First-launch interactive prompt

When a user runs `claude` in an unbound project:

```
Starting claude in an unbound project:
  /Users/eric.yao/new-project

Which account should this project use?
  1. work  — Company (Team Plan)
  2. me    — Personal  [default]

Select [1/2/Enter=default]: _
```

Selection writes `{ "account": "<id>" }` to `<project>/.claude/dotclaude.json`. Subsequent launches use the binding directly.

#### 3.7 Binding storage location

Bindings live in each project's `.claude/dotclaude.json`, not in a central registry. Rationale:

- Bindings travel with the project when it's moved
- Projects can optionally gitignore or commit the file based on team workflow
- No stale state when projects are renamed or deleted
- Aligns with Claude Code's existing `.claude/` project-level config convention

GUI maintains `~/.dot-claude/known_projects.json` purely as a UI bookmark list. `dotclaude-launch` never reads this file.

### 4. Plugin Sharing

#### 4.1 Three-layer model

```
Physical storage (one copy):
  ~/.dot-claude/shared/plugins/cache/<marketplace>/<plugin>/<version>/

Installation registry (per-account files, pointing to physical layer):
  ~/.dot-claude/accounts/<id>/plugins/installed_plugins.json
  ~/.dot-claude/accounts/<id>/plugins/known_marketplaces.json

Enablement (per-account + per-project via Claude Code native):
  Account: ~/.dot-claude/accounts/<id>/settings.json enabledPlugins
  Project: <project>/.claude/settings.json enabledPlugins
```

#### 4.2 Install flow

1. GUI reads the marketplace manifest to resolve the target version.
2. If the marketplace is not yet cloned, `git clone` it into `shared/plugins/marketplaces/<marketplace>/`.
3. `git clone`/`git checkout` the plugin at the target version into `shared/plugins/cache/<marketplace>/<plugin>/<version>/`.
4. Acquire the file lock `~/.dot-claude/.lock`.
5. For each account in `accounts.json`, atomically write (temp file + rename) an updated `installed_plugins.json` adding the plugin entry with `installPath` pointing to the shared location.
6. For each account, add the marketplace entry to `known_marketplaces.json` if not already present.
7. Release the lock.
8. Broadcast a Tauri event to the frontend.
9. Display success toast.

**Default enablement after install: `false` for all accounts.** Users must explicitly enable plugins per account or per project. Rationale: minimize Claude Code context window usage.

#### 4.3 Update flow

1. Fetch and check out the new version into `shared/plugins/cache/<marketplace>/<plugin>/<new-version>/` as a new sibling directory. The old version stays in place.
2. Acquire lock; update all accounts' `installed_plugins.json` to point at the new version.
3. Release lock; broadcast event.
4. Mark the old version directory as eligible for garbage collection.

Old versions are not deleted immediately. A running Claude Code CLI might be using files from the old version, and hot-reload would crash on vanished files.

#### 4.4 Garbage collection

On GUI startup (or on-demand):
1. Walk all accounts' `installed_plugins.json` and collect the set of referenced version directories.
2. Walk `shared/plugins/cache/` and list actual version directories.
3. Delete directories that exist on disk but are not referenced by any account's metadata.

This is safe because unreferenced versions are by definition not in use by any account.

#### 4.5 Uninstall flow

1. Acquire lock.
2. For each account, remove the plugin entry from `installed_plugins.json` and from `settings.json` `enabledPlugins`.
3. Release lock.
4. Next GC pass will delete the cache directory.

Marketplace directories are not auto-cleaned on plugin uninstall. The user may want to install other plugins from the same marketplace. A manual "Clean unused marketplaces" action is provided.

#### 4.6 Per-account and per-project enablement

Fully reuses Claude Code's native `enabledPlugins` merge hierarchy:

```
Account-level (~/.dot-claude/accounts/<id>/settings.json):
  {
    "enabledPlugins": {
      "plugin-x@marketplace": true,
      "plugin-y@marketplace": false
    }
  }

Project-level (<project>/.claude/settings.json):
  {
    "enabledPlugins": {
      "plugin-x@marketplace": false,   // override: disable for this project
      "plugin-z@marketplace": true     // override: enable for this project
    }
  }
```

Claude Code's existing config merge (managed → user → project → local) handles the layering. Project overrides account. We write no new merge code.

#### 4.7 Out-of-band install detection

Users may install plugins via `claude /plugin install ...` inside a running Claude Code session. In this case:

- Plugin files land in `$CLAUDE_CONFIG_DIR/plugins/cache/...` which is symlinked to `shared/plugins/cache/...`, so the physical files are shared.
- Only that specific account's `installed_plugins.json` and `known_marketplaces.json` are updated.
- Other accounts do not see the plugin in their metadata.

GUI detection: on opening the Plugins module, the backend computes the union of plugins across all accounts' `installed_plugins.json` and flags discrepancies. UI displays a sync prompt:

```
⚠ Plugin sync needed
  plugin-x was installed in 'work' but not in 'me'.
  [Sync to all accounts]  [Dismiss]
```

Clicking "Sync to all accounts" writes the plugin entry to every account's metadata. No physical files are moved because they are already in the shared pool.

#### 4.8 Concurrency protection

| Risk | Mitigation |
|------|-----------|
| GUI write + Claude CLI read racing on `installed_plugins.json` | Atomic rename: write to temp file, rename over target. Reader sees either old or new, never partial. |
| Multiple GUI processes writing concurrently | Advisory file lock on `~/.dot-claude/.lock` using `fd-lock` crate |
| Claude CLI hot-reload during plugin update | Version-scoped directories ensure the running process keeps its files; new version lives at a new path |
| Two accounts triggering install of the same new version simultaneously | `git clone` into an existing directory fails; we catch this and verify the expected version is present |

### 5. GUI Navigation

#### 5.1 Header

```
[Account Selector]  →  [Project Selector]
```

- Account selector includes `All accounts`, individual accounts, each with a status badge:
  - Green: logged in
  - Yellow: not logged in
  - Red: OAuth expired or error
- Project selector shows projects bound to the currently selected account. When `All accounts` is chosen, the selector shows all projects grouped by account.

`EnvironmentSelector` and `ConnectionsPanel` are removed.

#### 5.2 Sidebar modules

| ID | Name | Scope | Notes |
|----|------|-------|-------|
| `A` | 账号 (Accounts) | Global | Manage accounts, login, launch defaults |
| `J` | 项目 (Projects) | Global + Project | List, bind, override launch, per-project plugins |
| `S` | 设置 (Settings) | All accounts / Account / Project | Claude Code settings.json editor |
| `P` | 插件 (Plugins) | Global install + per-account/project enable | Install, marketplace, enablement matrix |
| `K` | 技能 (Skills) | Global (shared) | Shared skills |
| `M` | 记忆 (Memory) | Account | Memory files per account |
| `D` | 指令 (Instructions) | Global + Project | CLAUDE.md editor (preserved from phase 6) |
| `C` | MCP | Account | MCP servers per account |
| `E` | 配置 (Effective Config) | Account + Project | Merged view showing Claude Code's effective config |

Below the separator: `G` 应用设置 (App Settings), unchanged.

**Launcher module is removed.** Its function (show project + launch) is absorbed into the Projects module, which provides a "Launch" tab with command preview and an "Open in Terminal" action.

#### 5.3 Accounts module (A)

Sidebar lists accounts with login status. Main panel shows:

- Display name (editable)
- ID (readonly)
- Status (logged in / not logged in / error)
- Created timestamp
- Default `launch.args` form
- Default `launch.env` form
- Actions: Re-login, Delete account

"Add account" button opens a dialog: enter name, auto-generate id, create directory structure, prompt "Login now?" which triggers the `claude /login` flow.

#### 5.4 Projects module (J)

Sidebar lists projects grouped by bound account. Main panel has three tabs:

**Binding**
- Current bound account with a dropdown to re-bind
- Path to `.claude/dotclaude.json` with a link to view the raw file
- "Open in Terminal" button (opens Terminal.app at the project directory)
- "Unbind" button (deletes `.claude/dotclaude.json`)

**Launch**
- Inherited `launch.args` from account (shown dimmed)
- Project overrides (editable) with per-row disable/override controls
- Inherited `launch.env` from account (shown dimmed)
- Project env overrides (editable)
- Effective command preview showing the resolved `CLAUDE_CONFIG_DIR=... <env...> claude <args...>` that will be executed

**Plugins**
- List of plugins installed in the bound account
- Each plugin has a tri-state: inherited (from account), explicit enable, explicit disable
- Explicit states are written to `<project>/.claude/settings.json enabledPlugins`

"Add project" flow:
1. Open directory picker
2. Read `.claude/dotclaude.json`
3. If valid: add to project list, add to `known_projects.json`
4. If missing: prompt for account selection, write the file, add to list

#### 5.5 Settings module (S) scope behavior

```
All accounts mode:
  Reads all accounts' settings.json, displays common values
  Fields that differ between accounts are marked "differs per account"
  Edits write to all accounts atomically

Specific account mode:
  Shows that account's settings.json values
  Fields that differ from other accounts are marked with a warning indicator
  Edits write to only that account

Project mode (when project selected in header):
  Shows project's .claude/settings.json
  Edits write to only that project
```

`enabledPlugins` is hidden from the Settings module. It is managed exclusively through the Plugins module (account-level matrix) and the Projects module (project-level tab).

#### 5.6 Plugins module (P)

**Installed tab**
- All plugins installed across any account (union)
- Each plugin card shows:
  - Name, version, description
  - Account enablement matrix: one toggle per account
  - Update and Uninstall buttons (apply to all accounts)
- Out-of-band sync prompts appear at the top of this view when detected

**Marketplace tab**
- Browse plugins in registered marketplaces
- Install button installs to the shared pool and writes to all accounts' metadata

**Manage Marketplaces tab**
- Add or remove marketplaces
- Per-marketplace plugin count

Per-project plugin management is handled in the Projects module's Plugins tab, not duplicated here.

#### 5.7 Other modules scope behavior

| Module | Behavior |
|--------|---------|
| 技能 (K) | Shows shared `shared/skills/` only. Not affected by account/project selection. |
| 记忆 (M) | Shows the selected account's memory files, organized per project. Memory files live at `accounts/<selected>/projects/<path-encoded>/memory/<file>.md` following Claude Code's existing convention. |
| 指令 (D) | "Global" tab shows `shared/CLAUDE.md`. "Project" tab shows `<project>/.claude/CLAUDE.md`. |
| MCP (C) | Shows the selected account's MCP servers. |
| 配置 (E) | Shows the merged Claude Code config: managed → user (account) → project → local. |

### 6. Code Migration Strategy

Five phased sub-projects. Each phase ends in a usable state.

#### Phase 7.1: De-daemonization

Target: no user-visible changes; internal rewire from daemon to in-process Tauri commands.

- Delete `crates/claude-daemon/`
- Add Tauri commands in `src-tauri/src/commands/` mirroring existing REST endpoints
- Rewrite frontend `api/client.ts` as `ipc/client.ts` using `invoke()`
- Delete `ws.ts`, `connection.svelte.ts`, `connections.svelte.ts`, `EnvironmentSelector.svelte`, `ConnectionsPanel.svelte`
- Remove axum, tower, tokio-tungstenite, reqwest, portpicker from workspace dependencies
- Verify all 8 existing modules still function

Risk: low. Mechanical conversion with functional parity.

#### Phase 7.2: Workspace and Account Model

Target: introduce `~/.dot-claude/` and account concept.

- Create `src-tauri/src/workspace.rs` for directory initialization and symlink management
- Define `accounts.json` schema and CRUD Tauri commands
- New Accounts module (frontend UI + backend commands)
- New `AccountSelector` component replacing `EnvironmentSelector`
- Implement "create account → `claude /login`" subprocess flow
- On first launch, detect existing `~/.claude/` and prompt user to import it as an account
- Verify: can create multiple accounts, log in separately, switch between account views in GUI

Risk: medium. OAuth subprocess integration and directory bootstrapping are the sensitive parts.

#### Phase 7.3: Project Binding and dotclaude-launch

Target: terminal `claude` command auto-resolves the correct account.

- New crate `crates/dotclaude-launch` building a standalone CLI binary
- Implement `.claude/dotclaude.json` read/write
- Implement CWD upward lookup algorithm
- Implement interactive first-launch prompt (with TTY detection)
- Rewrite Projects module UI: Binding / Launch / Plugins tabs
- Implement launch.args/launch.env merge logic
- Provide install script and shell alias instructions
- Verify: `cd ~/work/project && claude` launches with the correct account

Risk: medium. TTY handling and shell integration edge cases.

#### Phase 7.4: Plugin Sharing

Target: install once, all accounts can use.

- Implement shared plugin pool in `shared/plugins/`
- Implement symlink initialization for new accounts
- Implement cross-account sync for `installed_plugins.json` and `known_marketplaces.json`
- Implement file lock protocol for concurrent writes
- Implement version-scoped cache and delayed GC
- Implement out-of-band install detection and sync prompt
- Rewrite Plugins module UI with account enablement matrix
- Implement Projects module Plugins tab
- Verify: launching from `work` and `me` both see shared plugins; independent enable states; project-level overrides work

Risk: high. Plugin metadata format is tightly coupled to Claude Code internals. Requires integration tests that lock in current behavior before implementation.

#### Phase 7.5: Polish and All-Accounts Mode

Target: complete scope semantics and polish.

- Settings module "All accounts" mode
- Account-scoped behavior for Memory and MCP modules
- Effective Config module shows the new four-layer hierarchy
- Final header and navigation polish
- Remove remaining deprecated code
- Update README and user documentation
- Verify: all modules behave correctly in All accounts / specific account / project views

Risk: low.

### 7. Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Claude Code plugin metadata format changes in a future version | Phase 7.4 implementation breaks | Write integration tests in Phase 7.4 that lock in current behavior; run against new Claude Code versions before upgrading users |
| OAuth login subprocess has unintended side effects on existing `~/.claude/` | Phase 7.2 could damage user's current config | Back up `~/.claude/` before the first account-creation flow; document the migration path clearly |
| Advisory file lock fails in unusual filesystem scenarios (NFS, container volumes) | Phase 7.4 data corruption | Use `fd-lock` crate; provide a "force unlock" diagnostic command |
| `dotclaude-launch` shell integration fails in non-standard shells | Phase 7.3 users can't launch | Provide `dotclaude-launch doctor` subcommand for diagnostics |
| `CLAUDE_CONFIG_DIR` semantics change in a future Claude Code version | Entire architecture assumption invalidated | Document the dependency explicitly; run compatibility check before each Claude Code upgrade |
| User hand-edits `installed_plugins.json` causing sync drift | Phase 7.4 cross-account state diverges | Detection pass on GUI open identifies and surfaces drift with a sync prompt |

## Verification

### Technical feasibility (already verified)

- `CLAUDE_CONFIG_DIR` isolation confirmed by spike
- Symlink survival confirmed by spike
- `installPath` absolute path acceptance confirmed by inspecting CCS instances

### Per-phase acceptance

- **Phase 7.1**: All existing modules function identically after daemon removal. No new user-visible bugs.
- **Phase 7.2**: Create work and me accounts, log in both, switch between them in GUI, confirm settings and OAuth state are fully isolated.
- **Phase 7.3**: From a fresh terminal, `cd ~/work/project && claude` spawns Claude Code with the correct `CLAUDE_CONFIG_DIR`. First-launch prompt works in TTY. Non-TTY invocation fails gracefully.
- **Phase 7.4**: Install a plugin via GUI; both accounts see it in their metadata; shared cache has one physical copy; per-account enablement toggles work; per-project overrides work; out-of-band install is detected and syncable.
- **Phase 7.5**: "All accounts" settings mode writes atomically to all accounts. Account-scoped modules respond to header selection. Effective Config shows the correct merge hierarchy.

### End-to-end acceptance (after all phases)

1. User installs the app and runs it for the first time
2. GUI detects existing `~/.claude/` and offers to import as `default` account
3. User creates a second account `work` and logs in
4. User installs Skill `test-driven-development` — appears in shared/skills
5. User installs Plugin `typescript-lsp` via GUI — physical files in shared pool, both accounts' metadata updated
6. User opens a new terminal, `cd ~/work/some-project`, runs `claude` — prompted to choose account, selects `work`
7. User enables `typescript-lsp` only for `work` account in GUI — me account's session does not load it
8. User opens `~/side/my-app` in terminal, runs `claude` — prompted, selects `me`
9. User adds a project-level override to enable `typescript-lsp` in `my-app` specifically — works
10. User edits a shared setting (language) in GUI's "All accounts" mode — both accounts' `settings.json` updated atomically

## Open Questions

None at design time. Remaining decisions are implementation details to be resolved during each phase.
