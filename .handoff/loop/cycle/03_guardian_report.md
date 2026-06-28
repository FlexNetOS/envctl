# Guardian report — worktree slug vs main checkout

Date: 2026-06-28

## Runtime surface

Observable behavior is the helper CLI surface in `scripts/reap-worktrees.sh`:

- Managed worktree checkout prints a slug.
- Main checkout prints nothing and returns nonzero.
- Status wrapper invokes `meta git worktree status` only with a derived slug.

## Verification results

PASS:

- `bash -n scripts/reap-worktrees.sh scripts/tests/test-reaper.sh scripts/tests/test-skill-contract.sh ci/gates/harness-scripts.sh`
- `bash scripts/tests/test-reaper.sh`
  - `PASS: reaper reaped merged+clean and squash-equivalent branches, preserved local-only/dirty/.handoff work, protected master/develop, FF-synced trunk, cleaned husks`
- `bash scripts/tests/test-skill-contract.sh`
  - `SKILL-CONTRACT TEST PASS`
- `bash ci/gates/harness-scripts.sh`
  - `HARNESS-SCRIPTS GATE PASS`
- `bash ci/gates/loop-state.sh`
  - `LOOP-STATE GATE PASS` for `.handoff/loop/loop_state.md`
  - `LOOP-STATE GATE PASS` for `.handoff/loop/plan/loop_state.md`
- `bash ci/gates/meta-local-policy.sh`
  - `meta-local-policy: active install sources target META_ROOT FHS/XDG; only the single real-home .local bridge is allowed`
- Runtime proof:
  - `bash scripts/reap-worktrees.sh --managed-worktree-slug "$PWD" envctl` printed `fix-worktree-slug-main-checkout` from the managed worktree.
  - `bash scripts/reap-worktrees.sh --managed-worktree-slug /home/drdave/Desktop/meta/envctl envctl` returned nonzero/no output for the main checkout.
- `git diff --check`

## Status

PASS. The change is docs/shell-harness only; no Rust trust-boundary or dependency changes.
