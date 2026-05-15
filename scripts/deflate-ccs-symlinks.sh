#!/usr/bin/env bash
# Materialize any ccs-pointing symlinks inside ~/.dot-claude-gui/accounts/.
#
# Background: an earlier ccs migration (scripts/migrate-from-ccs.sh) used
# `rsync -av` instead of `-avL`, which preserved ccs's
#   <instance>/{agents,commands,skills,settings.json} → ~/.ccs/shared/*
# symlinks verbatim. Result: GUI accounts that look independent on the
# surface but secretly share state with ccs (and with each other) for
# these four items. Deleting ~/.ccs breaks the GUI; editing a shared
# skill from one account is visible in every other account.
#
# This script finds those symlinks and replaces each with a real copy of
# its target, so every account owns its own files.
#
# Usage:
#   bash scripts/deflate-ccs-symlinks.sh            # dry-run (default)
#   bash scripts/deflate-ccs-symlinks.sh --apply    # actually deflate
#
# Idempotent: re-running on an already-deflated tree is a no-op.

set -euo pipefail

APPLY=0
case "${1:-}" in
  ""|"--dry-run") APPLY=0 ;;
  "--apply")      APPLY=1 ;;
  *)
    echo "usage: $0 [--dry-run|--apply]" >&2
    exit 2
    ;;
esac

ACCOUNTS_DIR="$HOME/.dot-claude-gui/accounts"
CCS_PREFIX="$HOME/.ccs/"
ITEMS=(agents commands skills settings.json)

if [ ! -d "$ACCOUNTS_DIR" ]; then
  echo "ERROR: $ACCOUNTS_DIR does not exist." >&2
  exit 1
fi

section() {
  echo
  echo "=== $* ==="
}

# Count of operations performed (or planned, in dry-run mode).
planned=0
skipped_external=0
skipped_missing=0
skipped_not_symlink=0

deflate_one() {
  local link="$1"
  if [ ! -L "$link" ]; then
    skipped_not_symlink=$((skipped_not_symlink + 1))
    return
  fi
  local target
  target=$(readlink "$link")
  # Only touch symlinks that point into ~/.ccs/. Leave anything else alone
  # (e.g. a deliberate symlink the user set up for some other reason).
  case "$target" in
    "$CCS_PREFIX"*) ;;
    *)
      echo "  skip  (target not under ~/.ccs/): $link -> $target"
      skipped_external=$((skipped_external + 1))
      return
      ;;
  esac
  if [ ! -e "$target" ]; then
    echo "  skip  (target missing): $link -> $target"
    skipped_missing=$((skipped_missing + 1))
    return
  fi
  planned=$((planned + 1))
  if (( APPLY == 0 )); then
    echo "  plan  deflate: $link -> $target"
    return
  fi
  rm "$link"
  if [ -d "$target" ]; then
    # rsync -aL: archive + dereference links (in case target itself has
    # nested symlinks like ccs-delegation -> ~/.ccs/.claude/skills/...).
    rsync -aL "$target/" "$link/"
  else
    cp -L "$target" "$link"
  fi
  echo "  done  $link  (copied from $target)"
}

for acct_dir in "$ACCOUNTS_DIR"/*/; do
  [ -d "$acct_dir" ] || continue
  acct=$(basename "$acct_dir")
  section "Account: $acct"
  for item in "${ITEMS[@]}"; do
    deflate_one "${acct_dir%/}/$item"
  done
done

section "Summary"
if (( APPLY == 0 )); then
  echo "Dry run. Would deflate: $planned symlink(s)."
  echo "  not-a-symlink (already deflated or never linked): $skipped_not_symlink"
  echo "  external (not under ~/.ccs/, left alone): $skipped_external"
  echo "  target-missing (dangling, left alone): $skipped_missing"
  if (( planned > 0 )); then
    echo
    echo "Re-run with --apply to perform the deflation."
  fi
else
  echo "Applied. Deflated: $planned symlink(s)."
  echo "  not-a-symlink: $skipped_not_symlink"
  echo "  external: $skipped_external"
  echo "  target-missing: $skipped_missing"
fi
