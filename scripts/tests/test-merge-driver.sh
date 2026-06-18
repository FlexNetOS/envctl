#!/usr/bin/env bash
# test-merge-driver.sh — proves the `handoff-reconcile` merge driver forces a VISIBLE conflict on
# the high-churn loop-state files instead of silently concatenating (the forge-loop cycle-5 hazard,
# 2026-06-13: a 3-way merge of non-overlapping regions auto-merged and triplicated loop_state.md).
#
# Hermetic: builds a throwaway git repo in a tmpdir, exercises the REAL scripts/handoff-merge-guard.sh,
# and asserts (a) WITHOUT the driver git auto-merges non-overlapping edits clean (the hazard), and
# (b) WITH the driver registered the same merge CONFLICTS with markers. No network, no real repo touched.
set -euo pipefail

REPO_ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
GUARD="$REPO_ROOT/scripts/handoff-merge-guard.sh"
[ -x "$GUARD" ] || { echo "FAIL: $GUARD missing/not executable" >&2; exit 1; }
fail() { echo "FAIL: $*" >&2; exit 1; }

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
cd "$tmp"

git init -q
git config user.email t@example.com; git config user.name test
git config commit.gpgsign false
mkdir -p scripts .handoff/loop
cp "$GUARD" scripts/handoff-merge-guard.sh
chmod +x scripts/handoff-merge-guard.sh

# Base file with well-separated regions so a default 3-way merge of edits to DIFFERENT regions
# merges CLEANLY (that clean auto-merge is exactly the silent-concatenation hazard for loop-state).
{ echo "# loop_state header"; for i in $(seq 1 20); do echo "line $i"; done; } > .handoff/loop/loop_state.md
git add -A; git commit -qm base
BASE_BR="$(git symbolic-ref --short HEAD)"

mk_divergence() {
  # branch A edits near the TOP; branch B appends at the BOTTOM -> non-overlapping -> clean by default.
  git checkout -q -b a "$BASE_BR"
  sed -i 's/^line 2$/line 2 (edited on a)/' .handoff/loop/loop_state.md
  git commit -qam "a: edit top"
  git checkout -q -b b "$BASE_BR"
  printf 'line 21 (appended on b)\n' >> .handoff/loop/loop_state.md
  git commit -qam "b: append bottom"
}

# (a) CONTROL — no driver registered, no gitattributes mapping: default merge should be CLEAN.
mk_divergence
if ! git merge --no-edit a >/dev/null 2>&1; then
  fail "control: expected a clean default merge of non-overlapping edits (setup wrong)"
fi
grep -q "line 2 (edited on a)" .handoff/loop/loop_state.md \
  && grep -q "line 21 (appended on b)" .handoff/loop/loop_state.md \
  || fail "control: default merge did not keep both edits (setup wrong)"
echo "  control: default 3-way merge silently combined both edits (the hazard) — confirmed"

# Reset to a clean base and redo the divergence, this time WITH the driver wired.
git checkout -q "$BASE_BR"
git branch -qD a b
printf '.handoff/loop/loop_state.md merge=handoff-reconcile\n' > .gitattributes
git add .gitattributes; git commit -qm "wire merge attr"
git config merge.handoff-reconcile.name 'handoff loop-state guard (test)'
git config merge.handoff-reconcile.driver "$tmp/scripts/handoff-merge-guard.sh %O %A %B %L %P"
mk_divergence

# (b) WITH the driver: the SAME merge must now CONFLICT (driver exits non-zero) and leave markers.
if git merge --no-edit a >/dev/null 2>&1; then
  fail "driver did NOT force a conflict — silent concatenation hazard is LIVE"
fi
grep -q '^<<<<<<< ours' .handoff/loop/loop_state.md || fail "driver wrote no '<<<<<<< ours' marker"
grep -q '^>>>>>>> theirs' .handoff/loop/loop_state.md || fail "driver wrote no '>>>>>>> theirs' marker"
# Both sides' content must be preserved between the markers (nothing lost, just flagged).
grep -q "line 2 (edited on a)" .handoff/loop/loop_state.md || fail "ours content lost"
grep -q "line 21 (appended on b)" .handoff/loop/loop_state.md || fail "theirs content lost"

echo "PASS: handoff-reconcile driver flips a silent clean-merge into a visible conflict (both sides preserved)"
