#!/usr/bin/env bash
# reap-worktrees.sh — keep THIS repo's local worktrees / branches / remote-tracking refs in
# sync with origin, so the forge-loop's per-cycle worktrees don't pile up after their PRs merge.
#
# Why this exists: every loop cycle creates a fresh `meta/.worktrees/<slug>/envctl` worktree off
# develop, pushes a PR, and auto-merges on green. origin auto-deletes the merged head branch
# (`delete_branch_on_merge=true`), but nothing locally reaped the worktree, the local branch, or
# the now-dangling remote-tracking ref — so they accumulated (46 worktrees / 85 branches once).
#
# Design (mirrors envctl invariants):
#   * DRY-RUN BY DEFAULT — destructive ops are fail-closed + preview unless you pass --apply.
#   * NEVER --force — git refuses to remove a dirty worktree; we also skip+warn on dirty.
#   * NEVER touches remotes — origin self-cleans on merge; we only `fetch --prune` to mirror it.
#   * PROTECTS master, develop, the current branch, the current worktree, and the main checkout.
#
# A branch is REAPABLE only when it is safely resolved:
#   (a) its upstream is gone — `[gone]` — i.e. origin deleted the head after the PR merged
#       (this is the squash-merge–robust signal; squash tips are never ancestors of master), OR
#   (b) it is an ancestor of origin/master — a true/FF merge.
# A branch with unpushed local commits (no upstream, not an ancestor) is NEVER reaped.
#
# Usage:  scripts/reap-worktrees.sh            # preview (dry-run)
#         scripts/reap-worktrees.sh --apply    # actually reap
set -euo pipefail

APPLY=0
[ "${1:-}" = "--apply" ] && APPLY=1
run() { if [ "$APPLY" = 1 ]; then "$@"; else printf '    DRY-RUN: %s\n' "$*"; fi; }

CUR_WT="$(git rev-parse --show-toplevel)"
MAIN_WT="$(git worktree list --porcelain | awk 'NR==1 && /^worktree /{print $2; exit}')"
CUR_BR="$(git symbolic-ref --quiet --short HEAD || echo '')"
PROTECT=" master develop ${CUR_BR} "
is_protected() { case "$PROTECT" in *" $1 "*) return 0 ;; esac; return 1; }

# Branch is reapable iff upstream is gone OR it is already in origin/master.
is_reapable() {
  local b="$1"
  [ "$(git for-each-ref --format='%(upstream:track)' "refs/heads/$b")" = '[gone]' ] && return 0
  git merge-base --is-ancestor "refs/heads/$b" origin/master 2>/dev/null && return 0
  return 1
}

# 1. Mirror origin's merge-time branch deletions into local tracking refs (non-destructive).
git fetch --prune origin >/dev/null 2>&1 || true

# 2. Reap merged/clean per-cycle worktrees (skip main, develop, current, and anything dirty).
echo "== worktrees =="
reaped_wt=0
while IFS= read -r wt; do
  [ "$wt" = "$CUR_WT" ] && continue
  [ "$wt" = "$MAIN_WT" ] && continue
  br="$(git -C "$wt" symbolic-ref --quiet --short HEAD 2>/dev/null || echo '')"
  [ -z "$br" ] && continue
  is_protected "$br" && continue
  if [ -n "$(git -C "$wt" status --porcelain 2>/dev/null)" ]; then
    echo "    SKIP (dirty — has uncommitted work): $wt [$br]"
    continue
  fi
  if is_reapable "$br"; then
    echo "  reap worktree: $wt [$br]"
    run git worktree remove "$wt"
    reaped_wt=$((reaped_wt + 1))
  fi
done < <(git worktree list --porcelain | awk '/^worktree /{print $2}')
echo "  worktrees reaped: $reaped_wt"

# 3. Reap merged local branches not checked out in any remaining worktree.
echo "== branches =="
git worktree prune
checked_out="$(git worktree list --porcelain | awk '/^branch /{sub("refs/heads/","",$2); print $2}')"
reaped_br=0
for b in $(git for-each-ref --format='%(refname:short)' refs/heads/); do
  is_protected "$b" && continue
  printf '%s\n' "$checked_out" | grep -qx "$b" && continue
  if is_reapable "$b"; then
    echo "  reap branch: $b"
    run git branch -D "$b"
    reaped_br=$((reaped_br + 1))
  fi
done
echo "  branches reaped: $reaped_br"

# 4. Best-effort: let meta drop now-orphaned worktree-set metadata (non-destructive to commits).
if command -v meta >/dev/null 2>&1; then
  echo "== meta worktree prune =="
  run meta git worktree prune
fi

[ "$APPLY" = 1 ] || echo $'\n(dry-run — re-run with --apply to perform the above)'
