# Implementer log — worktree slug vs main checkout

Date: 2026-06-28

## Implemented

- Extended `scripts/reap-worktrees.sh` with read-only helper modes before any
  fetch/prune/reap logic:
  - `--managed-worktree-slug [path] [repo]`
  - `--meta-worktree-status [path] [repo]`
- The helper resolves the checkout root and succeeds only for the path shape
  `.worktrees/<slug>/<repo>`. It rejects the main checkout and malformed paths,
  while allowing the valid edge where slug equals repo name:
  `.worktrees/envctl/envctl`.
- Added regression coverage in `scripts/tests/test-reaper.sh` for main checkout,
  managed checkout, slug-equals-repo, malformed paths, and the status wrapper's
  no-call behavior on main checkout.
- Updated `AGENTS.md`, `CLAUDE.md`, forge-loop skills, session-relay skills, and
  continuity-steward docs to explain that `<slug>` is a worktree-set slug, not
  the repo name.
- Added `scripts/tests/test-skill-contract.sh` guard against reintroducing stale
  `[gone] (merged)` or `meta git worktree status envctl` doctrine in active
  harness docs/agent config.

## Invariant notes

This is a shell/docs/harness safety change only. It does not alter Rust engine
logic, dependency graphs, trust-boundary dependencies, or install targets.
