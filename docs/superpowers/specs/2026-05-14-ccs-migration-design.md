# ccs → dot-claude-gui Account Data Migration

**Date:** 2026-05-14
**Status:** Design — pending implementation plan

## Goal

Migrate the user's two ccs instance directories (`~/.ccs/instances/work` and `~/.ccs/instances/me`) into the corresponding dot-claude-gui account directories so that the GUI becomes the user's daily entry point for Claude Code. The ccs CLI itself stays installed as a fallback; cliproxy and unrelated subsystems are untouched.

The user explicitly opted out of multi-provider routing (cliproxy) because they rarely use it. They also chose to keep the GUI account named `myself` (not rename to `me`) as the personal-account label, and to let ccs data fully overwrite the current GUI account contents.

## Background

dot-claude-gui's Phase 8 (Stages 1-5) finished 2026-05-14 with the seven-step E2E flow passing. The GUI now covers account isolation, per-project binding, plugin tri-state, settings editing, and launch flow. It does not cover cliproxy's multi-provider routing, custom CLI management (`ccs auth add`, `ccs config`), or ccs-shipped commands/skills.

The user's two ccs instances:

| Instance | Size | Last used |
|---|---|---|
| `~/.ccs/instances/work` | 200 MB | 2026-05-14 |
| `~/.ccs/instances/me` | 108 MB | 2026-05-10 |

The user has been working out of ccs for ~2 months. The settings, OAuth state, session history, projects, plugins, and CLAUDE.md files in these instances are the canonical source of truth. The GUI's current account directories (`~/.dot-claude-gui/accounts/{work,myself}`) hold only the Stage 5 test fixture (typescript-lsp plugin installed against `work`).

## Scope

**In scope:**
- rsync `~/.ccs/instances/work` → `~/.dot-claude-gui/accounts/work` (overwrite)
- rsync `~/.ccs/instances/me` → `~/.dot-claude-gui/accounts/myself` (overwrite, note destination name change)
- Pre-migration backup of both GUI account directories + the GUI top-level config
- Post-migration verification checklist
- Idempotent migration script in the project repo

**Out of scope:**
- Renaming `myself` to `me` (user opted out)
- Migrating cliproxy, OAuth proxy state, or `~/.ccs/shared/`
- Migrating `~/.ccs/.claude/` (ccs CLI's bundled `ccs.md` slash command and `ccs-delegation` skill — these target the ccs CLI itself and become meaningless when ccs is dropped from the user's workflow)
- Uninstalling the ccs CLI binary or removing `~/.ccs/`
- Adding an "Import from ccs" button to the GUI
- Merging conflicting fields field-by-field (ccs wins entirely on every conflict; the rsync source of truth model is total)

## Architecture

A single shell script (`scripts/migrate-from-ccs.sh`) drives the migration. The script is idempotent: re-running it does another full rsync (which is incremental in practice). The user runs the script manually after closing the GUI.

No code lands in the Tauri backend or the Svelte frontend. No new IPC. No new UI surface. The migration is a one-shot operational task; the GUI is the consumer of the resulting account directories.

## Migration Steps

### Step 0 — Preflight (script enforces)

- Verify both ccs source directories exist; abort with a clear error otherwise.
- Verify both GUI destination directories exist; abort with a clear error otherwise.
- Detect whether the GUI process is running. If yes, instruct the user to quit it (`Cmd+Q`) and re-run the script. Detection: `pgrep -f "dot-claude-gui"` or look for a Tauri lock file; whichever is reliable on macOS.
- Compute a timestamp `TS=$(date +%s)`.
- Create three backups before any write:
  - `tar -czf ~/.dot-claude-gui/accounts/work.pre-ccs-migration.${TS}.tar.gz -C ~/.dot-claude-gui/accounts work`
  - `tar -czf ~/.dot-claude-gui/accounts/myself.pre-ccs-migration.${TS}.tar.gz -C ~/.dot-claude-gui/accounts myself`
  - `cp ~/.dot-claude-gui/config.json ~/.dot-claude-gui/config.json.pre-ccs-migration.${TS}`
- Print the three backup paths to stdout so the user can copy them somewhere safe if desired.

### Step 1 — rsync work

```sh
rsync -av --delete \
  "$HOME/.ccs/instances/work/" \
  "$HOME/.dot-claude-gui/accounts/work/"
```

Key flag semantics:
- `-a`: archive mode — preserves permissions, timestamps, symlinks, ownership.
- `-v`: verbose so the user sees what's transferred.
- `--delete`: any file in the destination not present in the source is removed. This is the "total source of truth" model: the destination becomes a byte-for-byte (modulo `--delete`'s purpose) reflection of the source.
- Trailing slashes are required: `<src>/` copies contents, not the directory itself.

This step replaces the Stage 5 test data (the `typescript-lsp@claude-plugins-official` plugin under `~/.dot-claude-gui/accounts/work/plugins/`) since the ccs work instance has never installed that plugin.

### Step 2 — rsync me → myself

```sh
rsync -av --delete \
  "$HOME/.ccs/instances/me/" \
  "$HOME/.dot-claude-gui/accounts/myself/"
```

Note the asymmetric naming: ccs source is `me`, GUI destination is `myself`. This is intentional (user opted to keep `myself` as the account label).

### Step 3 — Launch the GUI

The user reopens `pnpm tauri dev` (or the production binary). The GUI's top-level `~/.dot-claude-gui/config.json` was not touched, so all bindings, project lists, and settings stay intact. The accounts now contain the full ccs history.

### Step 4 — (Optional) Re-install Stage 5 test plugin

If the user wants to keep the Stage 5 E2E fixture around for future verification:

```sh
CLAUDE_CONFIG_DIR=~/.dot-claude-gui/accounts/work \
  claude plugin install typescript-lsp
```

The marketplace entry was preserved by the rsync (ccs work has `claude-plugins-official` registered already), so only the plugin install step is needed.

## Risks and Edge Cases

1. **`.claude.json` overwrite.** Both directories contain a `.claude.json` (recent projects list, OAuth state). rsync replaces GUI's version with ccs's. This is the desired behavior under the "ccs wins" model; the user has been using ccs as their daily driver, so its `.claude.json` is the more recent reality.

2. **`policy-limits.json` overwrite.** Both have this file (token-rate-limit tracking). ccs wins. The GUI's version was empty (no usage), so no information is lost.

3. **Plugin marketplaces in the new account directory.** rsync brings over `plugins/marketplaces/claude-plugins-official` from ccs work. The GUI will see this marketplace on next launch — desired effect, no schema mismatch since both ccs and the GUI store plugins identically.

4. **OAuth tokens (`.anthropic/` directory).** Present in ccs work, not in GUI work. After rsync the GUI inherits the ccs OAuth state. The user does NOT need to re-run `claude /login`.

5. **Session history (`sessions/*.jsonl`, `projects/<dir>/*.jsonl`).** The Stage 5 E2E session file in GUI work gets deleted by rsync (it's not in the ccs source). This is acceptable: it was test data.

6. **Extra subdirectories from ccs (`agents`, `commands`, `skills`, `plans`, `todos`, `image-cache`, `paste-cache`, `session-env`, `file-history`, `shell-snapshots`, `debug`, `logs`).** These don't exist in the GUI account today. After rsync they are simply additional files on disk. dot-claude-gui's facets read only known subdirectories (`projects/`, `sessions/`, `plugins/`, `settings.json`, `CLAUDE.md`, etc.), so the extra directories are ignored — no GUI crash. They remain available to the `claude` CLI process running inside the launched terminal, which is the point of keeping them.

7. **Disk usage.** GUI accounts grow from ~5 MB total to ~308 MB total. This is expected and acceptable (the data is the same data; only the on-disk location moves).

8. **GUI 's `~/.dot-claude-gui/config.json` is NOT touched.** Bindings (`projects` map, `known_projects` list, gear settings, account list) remain intact. The GUI continues to see accounts named `default` (native `~/.claude`), `work`, and `myself`.

9. **Concurrent GUI process.** If a stale GUI instance is running, files under `~/.dot-claude-gui/accounts/work/` may be open (e.g., a session jsonl). The script aborts on running-process detection to avoid silently corrupting data. The user must quit the GUI explicitly.

10. **`--delete` semantics.** Any file present only in the GUI destination is removed. This is by design (clean overwrite). The pre-migration tar.gz backup is the recovery path if the user later realizes they needed something the GUI had.

## Rollback

If post-migration the GUI behaves wrong:

```sh
# 1. Quit the GUI (Cmd+Q)
# 2. Replace the migrated directories with the backups:
TS=<the timestamp printed by Step 0>
mv ~/.dot-claude-gui/accounts/work        ~/.dot-claude-gui/accounts/work.failed-${TS}
mv ~/.dot-claude-gui/accounts/myself      ~/.dot-claude-gui/accounts/myself.failed-${TS}
tar -xzf ~/.dot-claude-gui/accounts/work.pre-ccs-migration.${TS}.tar.gz   -C ~/.dot-claude-gui/accounts/
tar -xzf ~/.dot-claude-gui/accounts/myself.pre-ccs-migration.${TS}.tar.gz -C ~/.dot-claude-gui/accounts/
cp ~/.dot-claude-gui/config.json.pre-ccs-migration.${TS} ~/.dot-claude-gui/config.json
# 3. Restart the GUI.
# 4. Use `ccs work` / `ccs me` until the underlying issue is understood.
```

`~/.ccs/` is not touched by the migration, so `ccs` commands continue to work exactly as before regardless of the migration outcome.

## Verification

After migration:

1. GUI starts cleanly. No migration toast (the top-level config was not modified).
2. Account mode > @work > Settings: the `permissions.allow` list contains the 50+ Bash entries from ccs (was an empty `permissions` block in GUI's pre-migration `settings.json`).
3. Account mode > @work > Plugins: shows ccs's installed plugins (at minimum `andrej-karpathy-skills@karpathy-skills` and `atlassian@claude-plugins-official`, both enabled). The Stage 5 test fixture `typescript-lsp@claude-plugins-official` is **gone** — it was overwritten by rsync's `--delete`. Re-install it via Step 4 if you want it back as a verification fixture.
4. Account mode > @myself > Plugins: shows ccs's installed plugins for the `me` instance (at minimum `andrej-karpathy-skills@karpathy-skills` and `atlassian@claude-plugins-official`).
5. Project mode > sidebar: project list unchanged. Existing bindings still work (the top-level config was not touched).
6. Project mode > any project bound to @work > Launch → new terminal opens → inside the terminal `claude --resume` shows the ccs work session history.
7. `du -sh ~/.dot-claude-gui/accounts/{work,myself}` reports a total size close to 200 MB + 108 MB.
8. (Optional) After re-installing typescript-lsp: Plugin tri-state still works as Stage 5 verified.

## Implementation Notes

The migration script is the only artifact:

- File: `scripts/migrate-from-ccs.sh`
- Permissions: chmod 755 (executable)
- Shebang: `#!/usr/bin/env bash`
- Strict mode: `set -euo pipefail`
- Idempotent: re-running after a partial run is safe (rsync is incremental; the backup step uses a per-run timestamp).
- Output: clear sectioned progress messages with the backup paths echoed at the end.

No changes to `crates/`, `src/`, `src-tauri/`, or `package.json`. The script can be committed independently and re-used months later if the user wants to refresh from ccs.

## Out of Scope (Future Work)

- An "Import from ccs" UI inside the GUI (currently the script is sufficient and one-shot).
- Migration of cliproxy oauth tokens (user opted out).
- A `dotclaude-launch` CLI binary that would replace `ccs work` shell command pattern entirely.
- Uninstalling ccs (`bun rm -g ccs` and removing `~/.ccs/`). User will do this manually once GUI is proven over a usage window.
- Migrating ccs's `~/.ccs/.claude/` (bundled commands/skills) to `~/.claude/` user-level. Out of scope because the `ccs.md` slash command and `ccs-delegation` skill target the ccs CLI itself.
