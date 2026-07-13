# GitHub execution policy

Apply this policy to every branch, commit, worktree, pull request, CI run, merge, or cleanup operation performed by `/agent-env-codex`.

## Invariants

1. **Strict upgrade only.** Preserve every working capability and integrate stronger behavior. Do not delete, disable, comment out, weaken, or replace a capability merely to resolve a conflict, satisfy a gate, or reduce scope.
2. **Never cherry-pick.** Inspect each source commit and its parent diff, map the behavior to current owners, and manually implement the required delta in the current branch. Record why any source hunk is already represented, superseded by a stronger implementation, or unrelated.
3. **No stranded commits.** Every intended task change must be committed, pushed, reviewed through a PR, and merged. A local commit, pushed branch, open PR, enabled auto-merge, or pending CI run is intermediate state, not completion.
4. **Unfinished-work closure.** A surfaced stale branch, orphaned or conflicted worktree, superseded PR, failed check, unpushed commit, dirty generated state, or unmerged task is part of the active task. Archive it, inspect it, integrate all required work, prove equivalence or supersession, then settle its GitHub and local state.
5. **Permission integrity.** Never change `/permissions`, an approval policy, sandbox mode, network policy, capability toggle, or an `Allow`/approval value to bypass a failure. Do not weaken tests or policies to make the run green. Repair the owning implementation or report the exact external blocker.
6. **Meta worktree authority.** Create and inspect task worktrees through `rtk meta git worktree ...`. Use `rtk meta git <adapted-command>` or, only for an unlisted operation, `rtk meta exec --include <repo> -- git <command>`. Never invoke raw `git`, bypass Meta worktree ownership, or edit a stale main checkout.
7. **Personal and organization SSH proof.** Run `rtk meta git setup-ssh`, require FlexNetOS remotes to use the organization SSH form, and verify fetch/push identity before mutation. Prove the personal SSH principal is `drdave-flexnetos`, then separately prove active FlexNetOS membership/role and SSH repository access. A personal SSH greeting alone is not organization authorization. SSH authenticates Git transport; organization administration still uses `gh`, REST, GraphQL, or the GitHub UI.
8. **Linux-only automation.** GitHub workflows may target Ubuntu/Linux hosted runners or FlexNetOS Linux self-hosted runners. Do not add or retain macOS or Windows jobs in a touched workflow.
9. **Protected trunks and disposable task state.** Never remove `main`, `master`, or `develop`. Every other merged task branch and its worktree is disposable after archive and representation proof; delete it locally and remotely, then prune.
10. **Non-destructive fork sync.** Keep `origin` on the FlexNetOS fork and `upstream` on its source when a repository is forked. Fetch both, merge upstream into the clean local protected trunk, resolve by preserving local and upstream capabilities, and push through SSH. Never force-sync, hard-reset, or discard local commits.
11. **Branch/origin/worktree convergence.** Before completion, fetch/prune, fast-forward clean protected trunks, verify the merged PR contains every task delta, remove merged task worktrees/branches, and prove local branches, origin, and worktrees agree.
12. **No destructive shortcuts.** Do not force-push, hard-reset, delete unarchived task state, or remove a branch/worktree before proving its commits and uncommitted changes are represented. Preserve secret protections, rulesets, and branch protections.

## Execution sequence

1. Run the Meta SSH setup/check, fetch/prune through RTK/Meta, and inspect the current base, task branch, related commits, worktrees, PRs, and checks.
2. Archive dirty or unique state before editing or cleanup.
3. Build a commit reconciliation ledger:

```text
source_commit | affected_owner | required_behavior | current_representation | action | proof
```

4. Re-implement missing deltas with ordinary source edits in the current worktree. Do not cherry-pick.
5. Run focused tests, complete harness validation, repository gates, and live frontdoor probes.
6. Commit every intended file through RTK/Meta, push the SSH task branch, create or update the PR, enable auto-merge, and wait for required checks and the actual merge.
7. Fix failures at their owning source. Do not sidestep them through permissions, approvals, test removal, or policy weakening.
8. Merge the PR. Then verify the merge commit or squash contains every intended change.
9. Close superseded PRs only after their required behavior is proven present in the merged replacement.
10. Archive and remove task worktrees and merged branches only after representation proof. Leave unrelated active worktrees and PRs untouched, but report them accurately.
11. Prove the final state with:

```text
rtk meta git status --short --sequential
rtk meta exec --include <repo> -- git worktree list --porcelain
gh pr view <touched-pr> --json state,mergeable,mergeStateStatus,statusCheckRollup
gh pr list --state open
```

Completion requires clean protected trunks, a merged task PR, no stale task worktree or branch, no task-created stash, no unresolved task check, no superseded task PR left open, and no touched workflow containing a macOS or Windows runner.
