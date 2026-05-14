# ccs → dot-claude-gui Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a single shell script that backs up the GUI account directories, then rsync's ccs's `work` and `me` instance data into the GUI's `work` and `myself` account directories. After the script runs, the GUI becomes the user's daily Claude Code entry point.

**Architecture:** One bash script (`scripts/migrate-from-ccs.sh`) with `--dry-run` support, idempotent re-run, and clear rollback path documented in the spec. No Tauri/Svelte/Rust code changes. The script is a one-shot operational artifact, run manually after the user closes the GUI.

**Tech Stack:** bash 4+, rsync, tar, standard POSIX shell utilities. No additional dependencies.

**Spec:** `docs/superpowers/specs/2026-05-14-ccs-migration-design.md` (commit `28ede3b`).

---

## File Structure

**New:**
- `scripts/migrate-from-ccs.sh` — the migration script. Shebang `#!/usr/bin/env bash`, `set -euo pipefail`, executable bit set. Supports `--dry-run` flag.

**Not modified:** anything else. No `package.json` script added (the migration is one-shot; not worth a pnpm alias).

**Created at runtime (user environment, not committed):**
- `~/.dot-claude-gui/accounts/work.pre-ccs-migration.<ts>.tar.gz`
- `~/.dot-claude-gui/accounts/myself.pre-ccs-migration.<ts>.tar.gz`
- `~/.dot-claude-gui/config.json.pre-ccs-migration.<ts>`

---

## Task 1: Write `scripts/migrate-from-ccs.sh`

**Files:**
- Create: `scripts/migrate-from-ccs.sh`

- [ ] **Step 1: Write the script skeleton + preflight checks**

Create `scripts/migrate-from-ccs.sh` with:
```bash
#!/usr/bin/env bash
# Migrate ccs instance data (work, me) into dot-claude-gui account directories.
# Spec: docs/superpowers/specs/2026-05-14-ccs-migration-design.md
#
# Usage:
#   bash scripts/migrate-from-ccs.sh [--dry-run]
#
# Preflight aborts if:
#   - ccs source dirs missing
#   - GUI destination dirs missing
#   - GUI process appears to be running
#
# All operations are idempotent; safe to re-run.

set -euo pipefail

DRY_RUN=0
if [[ "${1:-}" == "--dry-run" ]]; then
  DRY_RUN=1
fi

CCS_WORK="$HOME/.ccs/instances/work"
CCS_ME="$HOME/.ccs/instances/me"
GUI_WORK="$HOME/.dot-claude-gui/accounts/work"
GUI_MYSELF="$HOME/.dot-claude-gui/accounts/myself"
GUI_CONFIG="$HOME/.dot-claude-gui/config.json"

abort() {
  echo "ERROR: $*" >&2
  exit 1
}

section() {
  echo
  echo "=== $* ==="
}

section "Preflight"

[[ -d "$CCS_WORK"   ]] || abort "ccs work source missing: $CCS_WORK"
[[ -d "$CCS_ME"     ]] || abort "ccs me source missing: $CCS_ME"
[[ -d "$GUI_WORK"   ]] || abort "GUI work destination missing: $GUI_WORK"
[[ -d "$GUI_MYSELF" ]] || abort "GUI myself destination missing: $GUI_MYSELF"
[[ -f "$GUI_CONFIG" ]] || abort "GUI config missing: $GUI_CONFIG"

if pgrep -if "dot-claude-gui" >/dev/null; then
  abort "dot-claude-gui appears to be running. Quit it (Cmd+Q) and re-run."
fi

echo "All preflight checks passed."
if (( DRY_RUN )); then
  echo "DRY RUN mode — no files will be modified."
fi
```

- [ ] **Step 2: Run the script with --dry-run to verify preflight**

Run: `bash scripts/migrate-from-ccs.sh --dry-run`

Expected: prints `=== Preflight ===`, then `All preflight checks passed.`, then `DRY RUN mode — no files will be modified.`. Exit code 0.

If GUI is currently running, the script aborts at this point with `dot-claude-gui appears to be running...`. Quit GUI and re-run.

- [ ] **Step 3: Append the backup phase**

Append to `scripts/migrate-from-ccs.sh`:
```bash

section "Backup current GUI account state"

TS=$(date +%s)
WORK_BACKUP="$HOME/.dot-claude-gui/accounts/work.pre-ccs-migration.${TS}.tar.gz"
MYSELF_BACKUP="$HOME/.dot-claude-gui/accounts/myself.pre-ccs-migration.${TS}.tar.gz"
CONFIG_BACKUP="${GUI_CONFIG}.pre-ccs-migration.${TS}"

if (( DRY_RUN )); then
  echo "[dry-run] would tar.gz $GUI_WORK   -> $WORK_BACKUP"
  echo "[dry-run] would tar.gz $GUI_MYSELF -> $MYSELF_BACKUP"
  echo "[dry-run] would cp     $GUI_CONFIG -> $CONFIG_BACKUP"
else
  tar -czf "$WORK_BACKUP"   -C "$HOME/.dot-claude-gui/accounts" work
  tar -czf "$MYSELF_BACKUP" -C "$HOME/.dot-claude-gui/accounts" myself
  cp "$GUI_CONFIG" "$CONFIG_BACKUP"
  echo "Backed up:"
  echo "  $WORK_BACKUP"
  echo "  $MYSELF_BACKUP"
  echo "  $CONFIG_BACKUP"
fi
```

- [ ] **Step 4: Verify backup phase with --dry-run**

Run: `bash scripts/migrate-from-ccs.sh --dry-run`

Expected output (after preflight):
```
=== Backup current GUI account state ===
[dry-run] would tar.gz /Users/.../accounts/work   -> /Users/.../accounts/work.pre-ccs-migration.<ts>.tar.gz
[dry-run] would tar.gz /Users/.../accounts/myself -> /Users/.../accounts/myself.pre-ccs-migration.<ts>.tar.gz
[dry-run] would cp     /Users/.../config.json     -> /Users/.../config.json.pre-ccs-migration.<ts>
```

Exit code 0. No files created (dry-run).

- [ ] **Step 5: Append the rsync phases**

Append to `scripts/migrate-from-ccs.sh`:
```bash

section "rsync ccs work -> GUI work"

RSYNC_OPTS=(-av --delete)
if (( DRY_RUN )); then
  RSYNC_OPTS+=(--dry-run)
fi

rsync "${RSYNC_OPTS[@]}" "$CCS_WORK/" "$GUI_WORK/"

section "rsync ccs me -> GUI myself"

rsync "${RSYNC_OPTS[@]}" "$CCS_ME/" "$GUI_MYSELF/"
```

Note the trailing slashes on source and destination — required by rsync to copy *contents* of the directory rather than the directory itself.

- [ ] **Step 6: Verify rsync phases with --dry-run**

Run: `bash scripts/migrate-from-ccs.sh --dry-run`

Expected: prints the file lists rsync would transfer (each row is a relative path). Far into the output you'll see rsync's typical summary line `sent X bytes  received Y bytes  ...`. Exit code 0. No actual file changes.

If the dry-run output is suspicious (e.g., transfers far fewer files than expected, or fails with a permissions error), stop and investigate before running for real.

- [ ] **Step 7: Append the final-message section**

Append to `scripts/migrate-from-ccs.sh`:
```bash

section "Done"

if (( DRY_RUN )); then
  echo "Dry run complete. Re-run without --dry-run to apply the migration."
else
  cat <<EOF
Migration complete. Next steps:

  1. Restart dot-claude-gui (pnpm tauri dev, or the production binary).
  2. Verify per spec § Verification:
       - Account mode > @work > Settings: permissions allow list has ccs's 50+ entries
       - Account mode > @work > Plugins: shows andrej-karpathy-skills + atlassian
       - Account mode > @myself > Plugins: same
       - Project mode > any bound project > Launch: claude --resume shows ccs history
  3. (Optional) Re-install Stage 5 test fixture:
       CLAUDE_CONFIG_DIR=~/.dot-claude-gui/accounts/work \\
         claude plugin install typescript-lsp

Backups (delete when satisfied):
  $WORK_BACKUP
  $MYSELF_BACKUP
  $CONFIG_BACKUP

ccs is untouched. \`ccs work\` / \`ccs me\` still work as fallback.
EOF
fi
```

- [ ] **Step 8: Make the script executable**

Run: `chmod +x scripts/migrate-from-ccs.sh`

- [ ] **Step 9: Final dry-run smoke test**

Run: `bash scripts/migrate-from-ccs.sh --dry-run`

Expected: full pipeline runs end-to-end:
1. `=== Preflight ===` + "All preflight checks passed." + dry-run notice
2. `=== Backup current GUI account state ===` with three `[dry-run] would ...` lines
3. `=== rsync ccs work -> GUI work ===` with rsync's file list and summary
4. `=== rsync ccs me -> GUI myself ===` with rsync's file list and summary
5. `=== Done ===` + "Dry run complete. Re-run without --dry-run to apply the migration."

Exit code 0. No files modified on disk.

If any step errors, fix and re-run dry-run.

- [ ] **Step 10: Commit**

```bash
git add scripts/migrate-from-ccs.sh
git commit -m "feat(ccs-migration): rsync-based migration script (work + me)"
```

---

## Task 2: User executes the migration

**Files:** None modified by Claude. This is a user-driven operational task.

The implementer should hand control back to the user with these instructions:

- [ ] **Step 1: Quit the GUI**

`Cmd+Q` in the dot-claude-gui window (or quit the dev server if running). Confirm with `pgrep -if dot-claude-gui` (should return nothing).

- [ ] **Step 2: Final dry-run review**

```bash
bash scripts/migrate-from-ccs.sh --dry-run
```

Review the output. Look at the rsync file list — confirm it lists files you expect (sessions, projects, settings.json, .anthropic, plugins, etc.). If anything looks wrong, stop and report.

- [ ] **Step 3: Run the migration for real**

```bash
bash scripts/migrate-from-ccs.sh
```

Note the three backup paths printed. Save them somewhere (e.g., paste into a sticky note) in case rollback is needed.

- [ ] **Step 4: Relaunch the GUI**

```bash
pnpm tauri dev
```

The GUI starts. No migration toast should appear (the top-level `config.json` was not modified). The login state, accounts list, and bindings are unchanged.

If anything looks visibly broken, see § Rollback in the spec.

---

## Task 3: Post-migration verification

**Files:** None. This is the spec's § Verification checklist executed end-to-end after Task 2.

Run each verification step. Hand any failure back to the implementer.

- [ ] **Step 1: GUI starts cleanly**

Confirm: GUI window opens, no migration toast, sidebar shows accounts (`work` + `myself` + native `default` if shown).

- [ ] **Step 2: Account > @work > Settings has ccs permissions**

Navigate: Account mode → @work → Settings (whichever section renders the JSON). The `permissions.allow` list should include ccs's 50+ Bash entries (e.g. `Bash(git:*)`, `Bash(gh:*)`, `Bash(pnpm:*)`, etc.).

If empty or only has GUI's pre-migration values, the rsync didn't land. Check `cat ~/.dot-claude-gui/accounts/work/settings.json | head -60`.

- [ ] **Step 3: Account > @work > Plugins shows ccs's plugins**

Navigate: Account mode → @work → Plugins. Should see at minimum:
- `andrej-karpathy-skills@karpathy-skills` (enabled)
- `atlassian@claude-plugins-official` (enabled)

The Stage 5 test fixture `typescript-lsp@claude-plugins-official` is **gone** (deleted by `rsync --delete`). This is expected.

- [ ] **Step 4: Account > @myself > Plugins**

Navigate: Account mode → @myself → Plugins. Same minimum plugin set as work.

- [ ] **Step 5: Project mode bindings still work**

Navigate: Project mode. The sidebar project list is unchanged (top-level config not touched). Open a bound project (e.g., x-simplify bound to work). All facets clickable.

- [ ] **Step 6: Launch + history resumes**

Project mode → x-simplify → Launch → click Launch. New terminal opens with `CLAUDE_CONFIG_DIR=~/.dot-claude-gui/accounts/work` exported (Fix 3 from Stage 5 polish). In the terminal, run:

```bash
claude --resume
```

You should see ccs's session list for this project (or for the user's broader history). Pick one and confirm content is there.

- [ ] **Step 7: Disk size check**

```bash
du -sh ~/.dot-claude-gui/accounts/work ~/.dot-claude-gui/accounts/myself
```

Expected: roughly 200 MB and 108 MB respectively (matching ccs's `~/.ccs/instances/{work,me}` sizes from the spec).

- [ ] **Step 8: (Optional) Re-install Stage 5 test fixture**

If you want typescript-lsp back as a verification fixture:

```bash
CLAUDE_CONFIG_DIR=~/.dot-claude-gui/accounts/work \
  claude plugin install typescript-lsp
```

Confirm in GUI: Account mode → @work → Plugins → typescript-lsp appears, enabled.

- [ ] **Step 9: Decide on backup cleanup**

After a week or two of confidence:

```bash
TS=<the timestamp from Task 2 Step 3>
rm ~/.dot-claude-gui/accounts/work.pre-ccs-migration.${TS}.tar.gz
rm ~/.dot-claude-gui/accounts/myself.pre-ccs-migration.${TS}.tar.gz
rm ~/.dot-claude-gui/config.json.pre-ccs-migration.${TS}
```

Or move them to long-term archive. Not required to keep on the primary disk.

---

## Self-Review Notes

**Spec coverage:**
- Spec § Migration Steps 0-4 → Task 1 (preflight, backups, rsync, optional plugin reinstall), Task 2 (user executes).
- Spec § Risks → covered by spec; the script's `--delete` behavior + the backup-before-write contract is exactly as specified.
- Spec § Rollback → documented in spec; Task 3 Step 1 surfaces rollback if GUI startup is broken. The rollback steps live in the spec file, not duplicated here.
- Spec § Verification → Task 3 Steps 1-8.
- Spec § Implementation Notes → Task 1 (the script follows: shebang, set -euo pipefail, idempotent, sectioned output, prints backup paths).

**Placeholder scan:** All steps contain concrete code, commands, and expected outputs. No "TBD", no "implement later", no vague "verify it works". Each rsync expectation is described as "file list + summary line".

**Type consistency:** The script uses these names consistently throughout: `DRY_RUN`, `CCS_WORK`, `CCS_ME`, `GUI_WORK`, `GUI_MYSELF`, `GUI_CONFIG`, `TS`, `WORK_BACKUP`, `MYSELF_BACKUP`, `CONFIG_BACKUP`, `RSYNC_OPTS`, `abort`, `section`. No drift.

**Risk callouts:**
- The script aborts if it detects a running GUI. The detection (`pgrep -if "dot-claude-gui"`) is heuristic and may false-positive on substring matches (e.g., another shell with that string in argv). If it false-positives, the user re-runs after confirming no GUI is open. Acceptable.
- The script's `pgrep` check is the only safeguard against concurrent file access. If the user starts the GUI between preflight and rsync, weird things can happen. Acceptable risk: the user is the operator and knows not to do this.
- `rsync --delete` is destructive. The backup phase runs *before* rsync to mitigate. Backups are tar.gz so they're space-efficient and easy to extract.
- macOS-specific: `pgrep -if` is supported on macOS. `tar -czf -C dir name` is portable. `rsync -av --delete` is portable.
