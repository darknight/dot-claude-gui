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
  tar -czf "$WORK_BACKUP" -C "$HOME/.dot-claude-gui/accounts" work \
    || { rm -f "$WORK_BACKUP"; abort "tar failed for work backup (disk full?). Aborted before rsync."; }
  tar -czf "$MYSELF_BACKUP" -C "$HOME/.dot-claude-gui/accounts" myself \
    || { rm -f "$MYSELF_BACKUP"; abort "tar failed for myself backup (disk full?). Aborted before rsync."; }
  cp "$GUI_CONFIG" "$CONFIG_BACKUP"
  echo "Backed up:"
  echo "  $WORK_BACKUP"
  echo "  $MYSELF_BACKUP"
  echo "  $CONFIG_BACKUP"
fi

section "rsync ccs work -> GUI work"

# -L (--copy-links): dereference symlinks during transfer. ccs sets up
# {agents,commands,skills,settings.json} as symlinks into ~/.ccs/shared/;
# without -L those symlinks land in the GUI account dirs verbatim and the
# accounts stay tied to ccs (delete ~/.ccs and everything dangles, change
# one account's skill and the other account sees it too). With -L the
# rsync transfer materializes real files/dirs and each GUI account owns
# its own copy.
RSYNC_OPTS=(-avL --delete)
if (( DRY_RUN )); then
  RSYNC_OPTS+=(--dry-run)
fi

rsync "${RSYNC_OPTS[@]}" "$CCS_WORK/" "$GUI_WORK/"

section "rsync ccs me -> GUI myself"

rsync "${RSYNC_OPTS[@]}" "$CCS_ME/" "$GUI_MYSELF/"

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
