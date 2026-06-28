# Architect plan — worktree slug vs main checkout

Date: 2026-06-28
Branch/worktree: `fix-worktree-slug-main-checkout` at `meta/.worktrees/fix-worktree-slug-main-checkout/envctl`

## Problem

`meta git worktree status <arg>` takes a meta-managed worktree-set slug from
`$META_ROOT/.worktrees/<slug>`, not a repo/project name. The main checkout
`/home/drdave/Desktop/meta/envctl` has no managed slug, while a managed checkout
looks like `/home/drdave/Desktop/meta/.worktrees/<slug>/envctl`.

The dangerous ambiguity is that both the main checkout and managed envctl
worktrees end in `envctl`, and a valid managed set may itself be named `envctl`
(`.worktrees/envctl/envctl`). Therefore slug identity must be derived from path
shape, never from basename or from a string blacklist.

## Planned change

- Add a read-only helper mode to `scripts/reap-worktrees.sh`:
  - `--managed-worktree-slug [path] [repo]` prints the set slug only when the
    checkout path has shape `.worktrees/<slug>/<repo>`.
  - `--meta-worktree-status [path] [repo]` calls `meta git worktree status <slug>`
    only after the helper derives a slug; main/unmanaged checkouts fail closed.
- Add hermetic tests to `scripts/tests/test-reaper.sh` for:
  - main checkout has no managed slug;
  - managed checkout derives the set slug;
  - valid `.worktrees/envctl/envctl` derives slug `envctl`;
  - malformed `.worktrees/envctl` and wrong repo leaf fail;
  - status wrapper does not call `meta` for main checkout.
- Update forge-loop/session-relay/continuity-steward docs to helper-gate
  `meta git worktree status` and correct stale `[gone]` merge-proof wording.
- Add a skill-contract grep guard so stale worktree doctrine does not return.

## Scanner notes folded in

The read-only scanner found no literal `meta git worktree status envctl` command,
but identified stale documentation wording and missing edge tests for slug
`envctl` and main-checkout-vs-managed-checkout path identity. Those findings are
included in this plan.
