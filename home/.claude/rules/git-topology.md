# Git topology invariants (FlexNetOS workspace)

- Only `main`/`master` and `develop` are long-lived branches, in every repo. Everything else is a short-lived feature/worktree branch that gets merged and reaped.
- PRs target `develop`. `develop` propagates to the protected `main`/`master` by fast-forward/superset merge only (org automation where wired; local `git merge --ff-only` otherwise).
- **Superset / upgrade-only merges:** a merge into a long-lived branch must never remove capability. If a merge would delete files or regress config, stop and surface it.
- Never force-push, force-delete (`-D`), rebase, or filter-branch a long-lived branch (hook-enforced).
- Session ritual for meta-workspace repos: work in a fresh git worktree off freshly-fetched `develop`, never on a shared checkout; prune worktrees after merge.
- Commit subjects are area-prefixed (repo conventions) and cite the task/phase item that motivated them.
