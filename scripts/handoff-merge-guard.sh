#!/usr/bin/env bash
# handoff-merge-guard.sh — git MERGE DRIVER for the high-churn loop-state files
# (.handoff/loop/loop_state.md, .handoff/loop/backlog.md).
#
# Why: git's default 3-way merge silently CONCATENATES these append-heavy files when two
# branches add to different regions — it reports a clean merge while actually producing a
# triplicated loop_state header / duplicated TASK cards (forge-loop cycle 5, 2026-06-13).
# This driver refuses to merge them silently: it writes standard conflict markers (ours over
# theirs) and exits non-zero, so git flags a CONFLICT that session-relay-wrap-up step 3b must
# reconcile explicitly. No content is ever lost or silently duplicated.
#
# git invokes it as:  handoff-merge-guard.sh %O %A %B %L %P
#   %O ancestor(base)  %A ours (driver MUST write the result here)  %B theirs
#   %L conflict-marker length (default 7)  %P pathname
set -euo pipefail
ours="${2:?ours path}"
theirs="${3:?theirs path}"
path="${5:-.handoff/loop state}"

tmp="$(mktemp)"
{
  printf '<<<<<<< ours (%s)\n' "$path"
  cat "$ours"
  printf '\n=======\n'
  cat "$theirs"
  printf '\n>>>>>>> theirs (%s)\n' "$path"
} >"$tmp"
mv "$tmp" "$ours"

# Non-zero => git marks the path CONFLICTED rather than auto-merging. Reconcile per
# session-relay-wrap-up step 3b (one coherent header; one row per TASK; no dup/triplication).
exit 1
