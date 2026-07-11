# GitHub execution policy

Apply this policy to every branch, commit, worktree, pull request, CI run, merge, or cleanup operation performed by `/agent-env-codex`.

## Invariants

1. **Strict upgrade only.** Preserve every working capability and integrate stronger behavior. Do not delete, disable, comment out, weaken, or replace a capability merely to resolve a conflict, satisfy a gate, or reduce scope.
2. **Never cherry-pick.** Inspect each source commit and its parent diff, map the behavior to current owners, and manually implement the required delta in the current branch. Record why any source hunk is already represented, superseded by a stronger implementation, or unrelated.
3. **No stranded commits.** Every intended task change must be committed, pushed, reviewed through a PR, and merged. A local commit, pushed branch, open PR, enabled auto-merge, or pending CI run is intermediate state, not completion.
4. **Unfinished-work closure.** A surfaced stale branch, orphaned or conflicted worktree, superseded PR, failed check, unpushed commit, dirty generated state, or unmerged task is part of the active task. Archive it, inspect it, integrate all required work, prove equivalence or supersession, then settle its GitHub and local state.
5. **Permission integrity.** Never change `/permissions`, an approval policy, sandbox mode, network policy, capability toggle, or an `Allow`/approval value to bypass a failure. Do not weaken tests or policies to make the run green. Repair the owning implementation or report the exact external blocker.
6. **No destructive shortcuts.** Do not force-push, hard-reset, delete unarchived task state, or remove a branch/worktree before proving its commits and uncommitted changes are represented. Preserve secret protections and branch protections.

## Execution sequence

1. Fetch and inspect the current base, task branch, all related commits, worktrees, PRs, and checks.
2. Archive dirty or unique state before editing or cleanup.
3. Build a commit reconciliation ledger:

```text
source_commit | affected_owner | required_behavior | current_representation | action | proof
```

4. Re-implement missing deltas with ordinary source edits in the current worktree. Do not cherry-pick.
5. Run focused tests, complete harness validation, repository gates, and live frontdoor probes.
6. Commit every intended file, push the task branch, create or update the PR, and wait for required checks.
7. Fix failures at their owning source. Do not sidestep them through permissions, approvals, test removal, or policy weakening.
8. Merge the PR. Then verify the merge commit or squash contains every intended change.
9. Close superseded PRs only after their required behavior is proven present in the merged replacement.
10. Archive and remove task worktrees and merged branches only after representation proof. Leave unrelated active worktrees and PRs untouched, but report them accurately.
11. Prove the final state with:

```text
git status --short --branch
git worktree list --porcelain
gh pr view <touched-pr> --json state,mergeable,mergeStateStatus,statusCheckRollup
gh pr list --state open
```

Completion requires a clean target branch, merged task PR, no stale task worktree or branch, no task-created stash, no unresolved task check, and no superseded task PR left open.
