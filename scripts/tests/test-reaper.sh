#!/usr/bin/env bash
# test-reaper.sh — proves scripts/reap-worktrees.sh upholds its destructive-safety invariants:
#   * REAPS a merged/clean per-cycle worktree+branch (upstream [gone] after origin deleted the head)
#   * REAPS a squash-equivalent branch whose PR-head SHA is not an ancestor of master
#   * PRESERVES a [gone] branch with local-only patches (upstream gone is not proof of merge)
#   * SKIPS a dirty worktree (uncommitted work is NEVER destroyed) even when its upstream is [gone]
#   * PROTECTS master/develop (never reaped)
#   * FF-syncs a protected trunk that is strictly behind origin (FF-only, clean-only)
#
# Hermetic: builds a bare "origin" + a clone with real worktrees in a tmpdir, exercises the REAL
# reaper with --apply, and asserts the outcome. No network, no real workspace touched. A `meta` stub
# neutralises the best-effort `meta git worktree prune` tail so the test never touches the live tree.
set -euo pipefail

REPO_ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
REAPER="$REPO_ROOT/scripts/reap-worktrees.sh"
[ -x "$REAPER" ] || { echo "FAIL: $REAPER missing/not executable" >&2; exit 1; }
fail() { echo "FAIL: $*" >&2; exit 1; }

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

# neutralise the reaper's optional `meta git worktree prune` tail (it must not touch the real tree).
mkdir -p "$tmp/bin"; printf '#!/bin/sh\nexit 0\n' > "$tmp/bin/meta"; chmod +x "$tmp/bin/meta"
# Stub `gh` to FAIL: the squash-merge oracle in is_reapable() is fail-closed, so against this
# hermetic (non-GitHub) repo it must fall through to the [gone]/ancestor predicates. Stubbing gh
# to exit 1 makes that deterministic (independent of whether the host has a real, authed gh).
printf '#!/bin/sh\nexit 1\n' > "$tmp/bin/gh"; chmod +x "$tmp/bin/gh"
export PATH="$tmp/bin:$PATH"

gitc() { git -c user.email=t@example.com -c user.name=test -c commit.gpgsign=false "$@"; }

# bare origin
git init -q --bare "$tmp/origin.git"
# main checkout (origin is empty here — the clone warning is expected, silence it)
gitc clone -q "$tmp/origin.git" "$tmp/main" 2>/dev/null
cd "$tmp/main"
echo seed > f; gitc add f; gitc commit -qm seed
gitc branch -M master
gitc push -qu origin master
gitc checkout -qb develop; gitc push -qu origin develop
gitc checkout -q master

mk_gone_worktree() { # $1=branch $2=worktree-dir  -> branch with a [gone] upstream + its own worktree
  gitc checkout -qb "$1"
  gitc push -qu origin "$1"
  gitc checkout -q master
  gitc worktree add -q "$2" "$1"
  gitc push -q origin --delete "$1"      # simulate the PR merge: origin deletes the head
}

mk_gone_worktree feat-merged "$tmp/wt-merged"
mk_gone_worktree feat-dirty  "$tmp/wt-dirty"
echo "uncommitted change" > "$tmp/wt-dirty/dirtyfile"   # make wt-dirty DIRTY (untracked file)
# A gone branch with local-only patches must be preserved. This is the fail-closed correction for
# the confusing "upstream gone" case: remote branch deletion is not the same thing as merged work.
mk_gone_worktree feat-local-only "$tmp/wt-local-only"
echo "local-only" > "$tmp/wt-local-only/local-only.txt"
gitc -C "$tmp/wt-local-only" add local-only.txt
gitc -C "$tmp/wt-local-only" commit -qm local-only
# Husk-cleanup case: a merged/clean worktree UNDER a meta/.worktrees/<slug>/ layout — after the
# worktree is removed, the now-empty <slug> husk dir must be rmdir'd (FIX: empty husks piled up).
mk_gone_worktree feat-husk "$tmp/.worktrees/huskslug/envctl"
# .handoff source-of-truth guard: a [gone] (merged) worktree whose ONLY change is uncommitted
# `.handoff` state must be REFUSED, never reaped (owner FIX #4).
mk_gone_worktree feat-handoff "$tmp/wt-handoff"
mkdir -p "$tmp/wt-handoff/.handoff"; echo "loop state" > "$tmp/wt-handoff/.handoff/state.md"

# Squash-equivalent branch: its branch tip is not an ancestor of master, but its patch is already
# represented on origin/master, matching GitHub squash merge behavior.
gitc checkout -qb feat-squash
printf 'squash
' > squash.txt; gitc add squash.txt; gitc commit -qm 'squash branch patch'
gitc push -qu origin feat-squash
gitc checkout -q master
printf 'squash
' > squash.txt; gitc add squash.txt; gitc commit -qm 'squashed equivalent patch on master'
gitc push -qu origin master
gitc worktree add -q "$tmp/wt-squash" feat-squash
gitc push -q origin --delete feat-squash

# put local master strictly BEHIND origin/master so step-1b FF-sync has something to do
gitc fetch -q --prune origin
adv="$(mktemp -d)"; gitc clone -q "$tmp/origin.git" "$adv/c"; cd "$adv/c"
gitc checkout -q master; echo more >> f; gitc commit -qam advance; gitc push -q origin master
cd "$tmp/main"

# run the reaper for real
bash "$REAPER" --apply >/dev/null 2>&1 || fail "reaper exited non-zero"

# assertions
[ ! -d "$tmp/wt-merged" ]                                   || fail "merged/clean worktree was NOT reaped"
gitc show-ref --verify --quiet refs/heads/feat-merged       && fail "merged branch was NOT reaped" || true
[ ! -d "$tmp/wt-squash" ]                                   || fail "squash-equivalent worktree was NOT reaped"
gitc show-ref --verify --quiet refs/heads/feat-squash       && fail "squash-equivalent branch was NOT reaped" || true
[ -d "$tmp/wt-local-only" ]                                 || fail "[gone] branch with local-only patch was destroyed"
gitc show-ref --verify --quiet refs/heads/feat-local-only   || fail "local-only gone branch was reaped (must preserve)"
[ -f "$tmp/wt-local-only/local-only.txt" ]                  || fail "local-only committed file was lost"
[ -d "$tmp/wt-dirty" ]                                      || fail "DIRTY worktree was destroyed (must skip)"
gitc show-ref --verify --quiet refs/heads/feat-dirty        || fail "branch of dirty worktree was reaped (must protect)"
[ -f "$tmp/wt-dirty/dirtyfile" ]                            || fail "uncommitted file in dirty worktree was lost"
gitc show-ref --verify --quiet refs/heads/master           || fail "master was reaped (must protect)"
gitc show-ref --verify --quiet refs/heads/develop          || fail "develop was reaped (must protect)"
# step-1b FF-sync: local master must now equal origin/master
[ "$(gitc rev-parse master)" = "$(gitc rev-parse origin/master)" ] || fail "master was not FF-synced to origin"
# husk-cleanup: the worktree under .worktrees/huskslug/ was reaped AND its empty slug dir removed
[ ! -d "$tmp/.worktrees/huskslug/envctl" ]                 || fail "husk worktree was NOT reaped"
[ ! -d "$tmp/.worktrees/huskslug" ]                        || fail "empty husk dir was NOT removed (rmdir)"
# .handoff guard: the worktree with uncommitted .handoff state must be PRESERVED (refused)
[ -d "$tmp/wt-handoff" ]                                    || fail ".handoff worktree was destroyed (must refuse)"
[ -f "$tmp/wt-handoff/.handoff/state.md" ]                 || fail "uncommitted .handoff state was lost"
gitc show-ref --verify --quiet refs/heads/feat-handoff     || fail "branch of .handoff worktree was reaped (must protect)"

echo "PASS: reaper reaped merged+clean and squash-equivalent branches, preserved local-only/dirty/.handoff work, protected master/develop, FF-synced trunk, cleaned husks"
