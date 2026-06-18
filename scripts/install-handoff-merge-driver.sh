#!/usr/bin/env bash
# install-handoff-merge-driver.sh — idempotently register the `handoff-reconcile` merge driver
# in THIS repo's git config, so the `.gitattributes` `merge=handoff-reconcile` mapping takes
# effect. Git does NOT auto-trust merge drivers named in .gitattributes (a security boundary —
# a clone can't run arbitrary commands from a tracked file), so each clone/worktree must register
# the driver locally. The forge-loop runs this at session start (session-relay-resume); it is
# idempotent, so it self-heals on every fresh clone or resume.
set -euo pipefail
root="$(git rev-parse --show-toplevel)"
guard="$root/scripts/handoff-merge-guard.sh"
[ -x "$guard" ] || chmod +x "$guard" 2>/dev/null || true
git config merge.handoff-reconcile.name \
  'handoff loop-state guard — force conflict, never silently concatenate'
git config merge.handoff-reconcile.driver "$guard %O %A %B %L %P"
echo "handoff-reconcile merge driver registered for $root"
