# Account Switching — Design Spec

Date: 2026-04-30
Status: Approved
Related plan: `~/.ccs/instances/me/plans/iterm2-playful-petal.md`

## Context

User holds two Claude accounts (a company team plan and a personal max plan) and currently switches them with the `ccs` CLI. dot-claude-gui already owns the project launch flow (terminal selection, env vars, CLI args, per-project persistence). Account switching is the last missing piece. The goal is for dot-claude-gui to natively support multiple accounts so the user can eventually drop `ccs` entirely. We borrow only the core mechanism from `ccs` — point `CLAUDE_CONFIG_DIR` at an isolated home directory — but keep our data fully independent.

## Confirmed user decisions

1. **Scope:** per-project, persisted alongside the existing `launcherProjectEnv`.
2. **Onboarding:** GUI-created accounts only (re-OAuth in-app); no `ccs` migration.
3. **No-account fallback:** native `~/.claude/` (do not inject `CLAUDE_CONFIG_DIR`).
4. **UI placement:** management lives in App Settings; selection lives in the Launcher.

## Architecture / data model

Storage layout:

```
~/.dot-claude-gui/
├── config.json                  # existing app preferences
└── accounts/
    ├── work/                    # one account = one isolated ~/.claude/ mirror
    │   ├── settings.json
    │   ├── projects/
    │   ├── sessions/
    │   └── ...                  # owned by Claude Code
    └── me/
        └── ...
```

`AppConfig` extension:

```ts
interface Account {
  name: string;       // directory name; only [a-z0-9_-]{1,32}
  createdAt: string;  // ISO timestamp
}
interface AppConfig {
  // ...existing fields
  accounts?: Account[];
}
interface LauncherProjectEnv {
  // ...existing (customEnv / envChecks / customArgs)
  accountName?: string;  // undefined => fallback to ~/.claude/
}
```

Launch logic: if `accountName` exists in `accountsStore`, inject `CLAUDE_CONFIG_DIR=~/.dot-claude-gui/accounts/<name>` into `LaunchRequest.env`; otherwise inject nothing.

## UI components

### App Settings → Accounts section (below the Launcher section)

- List, one row per account: `name` + `createdAt` + Login button + Delete button.
- Add: name input (live regex validation + dedupe) + Add Account button.
- After successful add, automatically call `launchClaude` to trigger OAuth: `projectPath: $HOME, env: { CLAUDE_CONFIG_DIR: <newPath> }, args: ["/login"], preferredTerminal`.
- The per-row Login button reuses the same launch flow for re-login / re-auth.
- Delete shows a confirmation dialog (emphasising that all local data, including OAuth credentials, will be removed) and then calls `delete_account`.

### LauncherView → details pane Account row (between Config Summary and the env section)

- Dropdown with `(default — ~/.claude/)` plus all user-created accounts.
- onChange → `launcherStore.setAccount(projectPath, name | undefined)` → existing `persistFor` writes to disk.
- A dangling reference (account deleted but the project still names it) is rendered as `⚠ <name> (deleted)` in muted text.

### accountsStore (new)

```ts
class AccountsStore {
  accounts = $state<Account[]>([]);
  async loadAccounts(): Promise<void>;     // calls list_accounts IPC
  async createAccount(name: string): Promise<Account>;
  async deleteAccount(name: string): Promise<void>;
}
```

`loadAccounts()` is awaited at app startup alongside `launcherStore.loadClaudeArgs()`.

## Data flow

**Add account**: input name → `create_account` IPC (mkdir + persist config) → push into `accountsStore` → automatically call `launchClaude` with `CLAUDE_CONFIG_DIR + /login` → user finishes OAuth in the terminal and exits claude. The GUI does not observe completion.

**Pick account + launch**: dropdown onChange → `launcherStore.setAccount` persists the choice. On launch, `LauncherView.launch()` consults `accountsStore`: if the account exists, inject `CLAUDE_CONFIG_DIR`; otherwise fall back and toast.

**Delete account**: click Delete → confirm → `delete_account` IPC (rm -rf + remove from config) → `accountsStore` refreshes. Projects already bound to it keep their dangling `accountName` until the user picks something else.

## Error handling

| Situation | Behaviour |
| --- | --- |
| Invalid name (empty / `/` / `..` / too long / non-ASCII) | Frontend disables Add; Rust re-validates and returns `invalid_name` |
| Duplicate name | Rust returns `account_exists` |
| `accounts/<name>/` exists on disk but missing from config (orphan) | `list_accounts` prefers disk and merges config metadata, preserving any existing OAuth |
| `accounts/` dir doesn't exist | `list_accounts` returns `[]`; `create_account` mkdirs on demand |
| Path-traversal attempt on delete | Rejected at the `validate_name` stage; Rust additionally `canonicalize`s and checks `starts_with(accounts_dir)` |
| Account dir wiped externally between loads | Frontend rechecks `accountsStore` before launch; toasts and falls back if missing |
| User aborts OAuth mid-flow | GUI doesn't notice; the next launch lets Claude Code re-prompt for login |
| Add succeeded but launch failed (no terminal / claude not in PATH) | Toast the error; the account directory exists, so the user can hit Login to retry |

## Testing

### Rust unit tests (`src-tauri/src/commands/accounts.rs::tests`)

- `validate_name_accepts_valid` / `_rejects_invalid` covering empty / `/` / `..` / over-length / valid
- `create_account_creates_dir_and_appends_config` (tempdir-injected fake home)
- `create_account_rejects_duplicate`
- `delete_account_removes_dir_and_config_entry`
- `delete_account_rejects_path_traversal`
- `list_accounts_includes_orphan_dirs`

### Manual verification (after restarting `pnpm tauri dev`)

1. App Settings: add `work` and `me`, log in to each.
2. LauncherView: pick `work` for project A and `me` for project B. After restarting the GUI both selections should still be set.
3. After launching, run `echo $CLAUDE_CONFIG_DIR` in the terminal to confirm the right value.
4. Delete `me` → project B's dropdown shows `⚠ me (deleted)` → choosing default clears it.
5. Inspect `~/.dot-claude-gui/config.json` to confirm the persisted shape.
