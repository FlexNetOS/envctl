---
name: github
description: >-
  Govern ALL GitHub management on the FlexNetOS fleet — PRs, merges, branches, worktrees, forks,
  workflows, releases, issues — under the operator's execution policy over org SSH. ALWAYS use when
  asked to open/merge a PR, push a branch, manage a fork, edit a GitHub workflow, cut a release, or
  "do github work". Binds the loaded claude-flow github skills (multi-repo, workflow-automation,
  release-management, code-review, project-management) to the policy; those are the toolbox, this is
  the law.
---

# github — FlexNetOS GitHub execution policy (the law for all GitHub management)

This skill is the **governing contract** for every GitHub action on the fleet. The five loaded
operational skills — `github-multi-repo`, `github-workflow-automation`, `github-release-management`,
`github-code-review`, `github-project-management` (claude-flow catalog, `npx`→`bunx` normalized) —
are the **toolbox**. This skill is the **law**: where a toolbox recipe conflicts with a rule below,
the rule wins. The always-on law surface is `~/.claude/rules/git-topology.md`.

## Non-negotiable policy (operator-stated, binding)

1. **Org SSH for ALL GitHub management.** Every remote is `git@github.com:FlexNetOS/<repo>.git`;
   auth is the org account (`drdave-flexnetos`, `admin:org`). `gh config git_protocol` = `ssh`. Never
   switch a remote to HTTPS or use a personal-token HTTPS push for fleet management.
2. **Never cherry-pick.** Not onto long-lived branches, not between worktrees, not to rescue one
   commit. History moves by superset merge or fast-forward only. A change worth keeping is merged whole.
3. **Strict upgrade-only — no sidestepping.** No removal of capability, no commenting-out code to pass
   a gate, no permission change (adding an `Allow` rule / widening settings) to get past a blocked
   action. Fix the cause or surface the blocker.
4. **Stale or orphaned work = unfinished work.** An abandoned branch, an unpushed worktree commit, an
   unmerged-but-mergeable PR you surface is driven to `MERGED` or explicitly routed to the backlog with
   owner + reason — never left dangling. A dirty worktree owned by another live session is *recorded*,
   not seized.
5. **Meta worktree ritual, always.** Work in a fresh worktree off freshly-fetched `develop`
   (`rtk meta git worktree create <slug>` / `rtk git worktree add … origin/develop`) — never on a
   shared checkout.
6. **Route fleet git through `rtk meta git`** — never bare `git` for cross-repo ops; unlisted verbs via
   `rtk meta exec -- git <cmd>`; raw `git` only when unsummarized output is required for proof (tee it).
7. **Land everything.** Commit ALL changes, push, open the PR against `develop` with auto-merge armed.
   `DONE` = `gh pr view <N>` returns `MERGED` (tick-on-merged); armed-but-unmerged stays in-flight.
8. **Merged ⇒ reap, immediately.** Delete the feature branch and remove its worktree after merge
   verification. `scripts/reap-worktrees.sh` is the tool — dry-run first, never `-D`/force, never a
   dirty worktree.
9. **`master`/`main` and `develop` are the ONLY never-removed branches.** Everything else is
   short-lived. Branches ↔ origin ↔ worktrees stay in sync — divergence a probe finds is unfinished
   work (rule 4).
10. **GitHub workflows are Linux-only.** No `macos-*`/`windows-*` in any `runs-on` or matrix. A
    workflow carrying one is a blocking finding.
11. **Forks sync as supersets.** A fork pulls upstream changes WITHOUT removing local updates —
    merge upstream into the fork, never force-reset the fork to upstream.
12. **`npx` = `bunx`.** Every loaded skill's `npx` invocation runs as `bunx` (bun-first doctrine;
    `bun-rewrite.sh` enforces it live). e.g. `bunx ruv-swarm …`, `bunx claude-flow@alpha …`.

## Runnable flows (the policy as commands)

```bash
# One-time / verify org SSH is the GitHub-management identity
gh auth status                                  # expect: account drdave-flexnetos, protocol ssh
gh config set git_protocol ssh                  # default clone/create over SSH
ssh -T git@github.com                           # expect: Hi drdave-flexnetos!
git -C <repo> remote get-url origin             # expect: git@github.com:FlexNetOS/<repo>.git

# Standard change (single repo) — worktree ritual → PR → auto-merge → reap
rtk git fetch
rtk git worktree add ../<repo>-<slug> -b <area>/<slug> origin/develop
# … edit; commit ALL …
rtk git add -A && rtk git commit -m "<area>: <subject> (cites <task>)"
rtk git push -u origin <area>/<slug>
gh pr create --base develop --title "…" --body "…"
gh pr merge --auto --squash                     # auto-merge armed; DONE only when state==MERGED
# after MERGED:
rtk git fetch && rtk git merge --ff-only origin/develop
bash scripts/reap-worktrees.sh --apply          # dry-run first; reaps merged branch+worktree

# Cross-repo / fleet
rtk meta git status                             # all repos at once
rtk meta git worktree create <slug>             # meta-managed, multi-repo aware
rtk meta exec -- git <cmd>                      # unlisted fleet git verb

# Fork superset sync (never force-reset)
rtk git fetch upstream
rtk git merge upstream/main                      # merge upstream INTO the fork; keep local updates
```

## Toolbox skills (loaded, bunx-normalized) — use under the law above

| Loaded skill | Use for |
|---|---|
| `github-multi-repo` | cross-repo coordination / sync / architecture (fleet, meta-aware) |
| `github-workflow-automation` | GitHub Actions / CI-CD — **enforce Linux-only runners (rule 10)** |
| `github-release-management` | versioning, release orchestration, rollback |
| `github-code-review` | PR review passes |
| `github-project-management` | issues, boards, sprint planning |

Each carries a policy-binding preamble; if a recipe would cherry-pick, remove capability, widen
permissions, add a macOS/Windows runner, or force-reset a fork, **do not run it** — that is a
policy violation, not a step.

## Guardrails

- Never automate around an irreversible external action (remote branch delete, fork force-push,
  release publish/delete) — those are owner walls; confirm `MERGED`/intended state first.
- Never read/print/commit secrets or tokens.
- `gap`/`unsupported`/`not_run` are honest states — record them; they never justify a policy bypass.
