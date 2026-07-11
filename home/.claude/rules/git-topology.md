# Git topology invariants (FlexNetOS workspace)

- Only `main`/`master` and `develop` are long-lived branches, in every repo. Everything else is a short-lived feature/worktree branch that gets merged and reaped.
- PRs target `develop`. `develop` propagates to the protected `main`/`master` by fast-forward/superset merge only (org automation where wired; local `git merge --ff-only` otherwise).
- **Superset / upgrade-only merges:** a merge into a long-lived branch must never remove capability. If a merge would delete files or regress config, stop and surface it.
- Never force-push, force-delete (`-D`), rebase, or filter-branch a long-lived branch (hook-enforced).
- Session ritual for meta-workspace repos: work in a fresh git worktree off freshly-fetched `develop`, never on a shared checkout; prune worktrees after merge.
- Commit subjects are area-prefixed (repo conventions) and cite the task/phase item that motivated them.

## GitHub management execution policy (META DEMAND — always in force)

- **ORG SSH FOR ALL GITHUB MANAGEMENT.** Every fleet remote is `git@github.com:FlexNetOS/<repo>.git`; auth is the org account (`admin:org`); `gh config git_protocol` = `ssh`. Never switch a remote to HTTPS or push fleet management over a personal-token HTTPS path.
- **Never cherry-pick** — onto a long-lived branch, between worktrees, or to rescue a commit. History moves by superset merge or fast-forward only.
- **Strict upgrade-only, no sidestepping:** no removal of capability, no commenting-out to pass a gate, no permission change (adding an `Allow` rule / widening settings) to get past a blocked action. Fix the cause or surface the blocker.
- **Stale or orphaned work = unfinished work:** an abandoned branch, unpushed worktree commit, or unmerged-but-mergeable PR you surface is driven to `MERGED` or explicitly backlogged with owner+reason — never left dangling. Another live session's dirty worktree is recorded, not seized.
- **Route fleet git through `rtk meta git`** (never bare `git` cross-repo; unlisted verbs via `rtk meta exec -- git <cmd>`; raw `git` only when unsummarized proof output is required, and tee it).
- **Land everything:** commit ALL changes, push, PR to `develop` with auto-merge armed; DONE only when `gh pr view` returns `MERGED` (tick-on-merged). **Merged ⇒ reap** the branch + worktree immediately (dry-run reaper first, never `-D`/force, never a dirty worktree).
- **Branches ↔ origin ↔ worktrees stay in sync;** divergence a probe finds is unfinished work. `master`/`main` and `develop` are the ONLY never-removed branches.
- **GitHub workflows are Linux-only** — no `macos-*`/`windows-*` runners in any `runs-on` or matrix.
- **Forks sync as supersets:** merge upstream into the fork (keep local updates); never force-reset a fork to upstream.
- Operational toolbox: the `github` skill (the law + runnable flows) and the loaded `github-*` skills (`npx`→`bunx` normalized). The skill binds the toolbox to this policy.
