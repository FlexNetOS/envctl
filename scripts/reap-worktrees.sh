#!/usr/bin/env bash
# reap-worktrees.sh — keep THIS repo's local worktrees / branches / remote-tracking refs in
# sync with origin, so the forge-loop's per-cycle worktrees don't pile up after their PRs merge.
#
# Why this exists: every loop cycle creates a fresh `meta/.worktrees/<slug>/envctl` worktree off
# develop, pushes a PR, and auto-merges on green. origin auto-deletes the merged head branch
# (`delete_branch_on_merge=true`), but nothing locally reaped the worktree, the local branch, or
# the now-dangling remote-tracking ref — so they accumulated (46 worktrees / 85 branches once).
#
# It also keeps the protected trunk branches (master, develop) fast-forwarded to origin, so
# `develop ↔ master ↔ origin` stay consistent locally without a manual merge — the main checkout's
# `master` mirror in particular does NOT auto-FF when a develop push fast-forwards origin/master.
#
# Design (mirrors envctl invariants):
#   * DRY-RUN BY DEFAULT — destructive ops are fail-closed + preview unless you pass --apply.
#   * NEVER --force — git refuses to remove a dirty worktree; we also skip+warn on dirty.
#   * NEVER touches remotes — origin self-cleans on merge; we only `fetch --prune` to mirror it.
#   * FF-ONLY, CLEAN-ONLY sync — protected branches are only fast-forwarded (never merged/rebased)
#     and only when their worktree is clean; an ahead/diverged/dirty branch is left untouched.
#   * PROTECTS master, develop, the current branch, the current worktree, and the main checkout
#     from REAPING (they are never deleted; they are kept in sync via FF instead).
#
# A branch is REAPABLE only when its content is safely resolved into origin/master:
#   (a) it is an ancestor of origin/master — a true/FF merge, OR
#   (b) every local commit is patch-equivalent to origin/master — the normal squash-merge case
#       where the PR head SHA is not an ancestor, but `git cherry` proves no unique patch remains.
#
# "upstream gone" is NOT a repo failure and NOT proof of merge. It only means the local branch was
# tracking a temporary remote branch that origin no longer has (usually because GitHub deleted the
# PR head after merge, but it can also happen after close/delete). A `[gone]` branch with local-only
# commits is NEVER reaped; it is preserved and surfaced for human review.
#
# For IRREVERSIBLE actions against a REMOTE, `[gone]`/ancestor/patch-equivalence are still not
# sufficient — confirm the PR actually MERGED via the GitHub oracle before deleting remote state.
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

upstream_track() { git for-each-ref --format='%(upstream:track)' "refs/heads/$1"; }

has_unique_patch() {
  local b="$1"
  # `git cherry origin/master branch` prints `+` for patches not present upstream and `-` for
  # patch-equivalent commits. Any `+` means local-only work remains, so fail closed.
  git cherry origin/master "refs/heads/$b" 2>/dev/null | awk '$1 == "+" { found=1 } END { exit found ? 0 : 1 }'
}

is_patch_equivalent_to_master() {
  local b="$1"
  git show-ref --verify --quiet "refs/heads/$b" || return 1
  # Fast path: every commit is individually patch-present upstream (normal/rebase/cherry-pick
  # merges, and SINGLE-commit squashes — one commit's patch-id == the squash's patch-id).
  has_unique_patch "$b" || return 0
  # Squash path (the >1-commit case `git cherry` MISSES): a multi-commit branch collapses to ONE
  # commit on master, so per-commit patch-id matching sees every branch commit as unique (`+`) and
  # would refuse to reap it forever (the husk-pileup the owner flagged). Detect it by the branch's
  # COMBINED patch-id (merge-base..branch) matching some master commit's patch-id since that
  # merge-base — i.e. the squash commit. Robust to master advancing past the squash (unrelated
  # commits simply don't match) and fail-closed (no match => not reapable).
  local mb bpid c
  mb="$(git merge-base "refs/heads/$b" origin/master 2>/dev/null)" || return 1
  [ -n "$mb" ] || return 1
  bpid="$(git diff "$mb" "refs/heads/$b" | git patch-id --stable 2>/dev/null | awk '{print $1}')"
  [ -n "$bpid" ] || return 1
  for c in $(git rev-list "$mb"..origin/master 2>/dev/null); do
    [ "$(git diff "$c^" "$c" 2>/dev/null | git patch-id --stable 2>/dev/null | awk '{print $1}')" = "$bpid" ] && return 0
  done
  return 1
}

# Branch is reapable iff it is already in origin/master OR all remaining commits are
# patch-equivalent to origin/master (squash-merge safe). `[gone]` alone is only a diagnostic.
is_reapable() {
  local b="$1"
  git merge-base --is-ancestor "refs/heads/$b" origin/master 2>/dev/null && return 0
  is_patch_equivalent_to_master "$b" && return 0
  return 1
}

explain_gone_branch() {
  local b="$1" role="$2"
  [ "$(upstream_track "$b")" = '[gone]' ] || return 0
  echo "  NOTE: $role branch '$b' tracks an upstream that is gone."
  echo "        That means the temporary remote branch was deleted; the repo/origin is not gone."
  if is_reapable "$b"; then
    echo "        Its patches are already represented on origin/master. Switch to master and rerun this reaper to remove the local husk."
  else
    echo "        It still has local-only patches; preserving it for review instead of reaping."
  fi
}

# 1. Mirror origin's merge-time branch deletions into local tracking refs (non-destructive).
git fetch --prune origin >/dev/null 2>&1 || true

# 1a. If the *current* checkout is on a gone upstream, explain it. The current branch is protected
#     from deletion, so the operator fix is to switch to master/develop, then rerun this script.
[ -n "$CUR_BR" ] && explain_gone_branch "$CUR_BR" "current"

# 1b. Fast-forward the protected trunk branches to origin (FF-only, clean-only). Keeps the main
#     checkout's `master` mirror and `develop` in lockstep with origin without a manual merge.
echo "== sync protected branches (FF-only) =="
# Map each checked-out branch -> its worktree path (so we can FF a branch that lives elsewhere).
declare -A WT_OF
_wt=""
while IFS= read -r line; do
  case "$line" in
    "worktree "*) _wt="${line#worktree }" ;;
    "branch refs/heads/"*) WT_OF["${line#branch refs/heads/}"]="$_wt" ;;
  esac
done < <(git worktree list --porcelain)
synced=0
for b in master develop; do
  git show-ref --verify --quiet "refs/heads/$b" || continue
  git show-ref --verify --quiet "refs/remotes/origin/$b" || continue
  # Only when strictly BEHIND (local is an ancestor of origin and not equal) => fast-forwardable.
  git merge-base --is-ancestor "refs/heads/$b" "refs/remotes/origin/$b" 2>/dev/null || { echo "  skip $b (ahead/diverged — not FF)"; continue; }
  [ "$(git rev-parse "refs/heads/$b")" = "$(git rev-parse "refs/remotes/origin/$b")" ] && continue  # already current
  wt="${WT_OF[$b]:-}"
  if [ -n "$wt" ]; then
    if [ -n "$(git -C "$wt" status --porcelain 2>/dev/null)" ]; then
      echo "  SKIP (dirty): $b @ $wt"; continue
    fi
    echo "  FF $b -> origin/$b (in $wt)"
    run git -C "$wt" merge --ff-only "origin/$b"
  else
    # Not checked out anywhere: FF the ref directly (git rejects a non-FF, so this is safe).
    echo "  FF $b -> origin/$b (ref update)"
    run git fetch origin "$b:$b"
  fi
  synced=$((synced + 1))
done
echo "  protected branches FF'd: $synced"

# 2. Reap merged/clean per-cycle worktrees (skip main, develop, current, and anything dirty).
echo "== worktrees =="
reaped_wt=0
while IFS= read -r wt; do
  [ "$wt" = "$CUR_WT" ] && continue
  [ "$wt" = "$MAIN_WT" ] && continue
  br="$(git -C "$wt" symbolic-ref --quiet --short HEAD 2>/dev/null || echo '')"
  [ -z "$br" ] && continue
  is_protected "$br" && continue
  # Explicit source-of-truth guard (owner FIX #4): NEVER reap a worktree with uncommitted/untracked
  # `.handoff` state — it is the loop's durable source of truth. (The generic dirty-skip below also
  # catches this, but make the refusal specific + legible so it can't be "optimized away".)
  if [ -n "$(git -C "$wt" status --porcelain -- '.handoff' 2>/dev/null)" ]; then
    echo "    REFUSE (uncommitted .handoff state — source of truth): $wt [$br]"
    continue
  fi
  if [ -n "$(git -C "$wt" status --porcelain 2>/dev/null)" ]; then
    echo "    SKIP (dirty — has uncommitted work): $wt [$br]"
    continue
  fi
  if is_reapable "$br"; then
    echo "  reap worktree: $wt [$br]"
    run git worktree remove "$wt"
    # Remove the now-empty meta/.worktrees/<slug> husk dir (rmdir refuses non-empty -> safe).
    parent="$(dirname "$wt")"
    case "$parent" in */.worktrees/*) [ -z "$(ls -A "$parent" 2>/dev/null)" ] && run rmdir "$parent" ;; esac
    reaped_wt=$((reaped_wt + 1))
  fi
done < <(git worktree list --porcelain | awk '/^worktree /{print $2}')
echo "  worktrees reaped: $reaped_wt"

# 2b. Sweep husk dirs left behind by PRIOR sessions (empty meta/.worktrees/<slug> whose worktree
#     was already removed but the slug parent dir lingered). rmdir is inherently safe (refuses
#     non-empty), so a slug dir that still holds a live `envctl/` worktree is never touched.
ws="$(dirname "$MAIN_WT")/.worktrees"
if [ -d "$ws" ]; then
  for d in "$ws"/*/; do
    [ -d "$d" ] || continue
    [ -z "$(ls -A "$d" 2>/dev/null)" ] && { echo "  reap husk dir: $d"; run rmdir "$d"; }
  done
fi

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
  elif [ "$(upstream_track "$b")" = '[gone]' ]; then
    explain_gone_branch "$b" "local"
  fi
done
echo "  branches reaped: $reaped_br"

# 4. Best-effort: let meta drop now-orphaned worktree-set metadata (non-destructive to commits).
if command -v meta >/dev/null 2>&1; then
  echo "== meta worktree prune =="
  run meta git worktree prune
fi

[ "$APPLY" = 1 ] || echo $'\n(dry-run — re-run with --apply to perform the above)'
