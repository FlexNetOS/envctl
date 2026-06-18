# Proposed harness upgrades — 2026-06-18 hygiene/audit retro (owner approval needed)

These are ESCALATED, not auto-applied. Each touches test/CI surface or invocation cadence — out of
the low-risk-doc auto-apply lane. Fail-closed: none weakens a guard; all would *add* coverage.

## P1 — No automated test for the `handoff-reconcile` merge driver
**Gap.** `scripts/handoff-merge-guard.sh` (the driver that forces a visible conflict on
`loop_state.md`/`backlog.md` instead of git's silent concatenate) and `install-handoff-merge-driver.sh`
have **zero tests**. The driver is the only thing standing between us and a repeat of the cycle-5
triplicated-header corruption, and it is shell, registered per-clone — exactly the kind of thing that
silently stops working (e.g. a `git config` key typo) without anyone noticing until corruption recurs.
**Proposed.** Add a hermetic test (a `tests/` shell harness or a small `#[test]` that shells out):
construct two divergent commits that both append to a fixture `loop_state.md`, run a merge with the
driver installed, assert the merge HALTS with a conflict (non-zero) rather than producing a
concatenated file. Wire into the CI gates job so a broken driver fails CI.
**Risk.** Low logic risk (additive test) but it is CI surface → escalated per policy. **Strengthens**
the guard (proves it still fires); never weakens it.

## P2 — No automated test for the reaper's protect/skip-dirty invariants
**Gap.** `scripts/reap-worktrees.sh` is destructive automation whose entire safety story is its
guards (protect master/develop/current, skip-dirty, reap only `[gone]`/ancestor, FF-only sync). Those
invariants are verified only by hand ("verified both paths" in the change log). A future edit could
silently break "skip-dirty" and we would not know until it ate uncommitted work.
**Proposed.** A hermetic test over a throwaway temp repo + worktrees: assert (a) a dirty worktree is
SKIPPED, (b) master/develop/current are never deleted, (c) a `[gone]` clean branch IS reaped under
`--apply`, (d) an ahead/diverged protected branch is NOT FF'd. Dry-run assertions need no `--apply`.
**Risk.** Low; additive. CI surface → escalated. **Strengthens** the destructive-automation guards.

## P3 — The reaper has no scheduled/automatic invocation (drift can re-accrue between sessions)
**Observation, NOT a clear fix.** Today the reaper runs only when `session-relay-resume`/`-wrap-up`
or a human calls it. That is **deliberate** (mid-cycle reaping is unsafe — a PR may still be merging),
and it is correct for the loop's own boundaries. But if the loop goes idle for a long stretch with
PRs merging via other paths, worktrees/branches can re-accrue with no trigger to reap them.
**Options (owner to choose, or decline):**
  - (a) Do nothing — accept that reap is loop-boundary-only; the next resume cleans up. (Lowest risk;
    the pileup only matters when it gets large, and resume always runs first.)
  - (b) A scheduled GitHub Action / cron that runs `reap-worktrees.sh` (dry-run report only — never
    `--apply` from CI, since CI has no business mutating a developer's local worktrees; this would
    only be meaningful on a long-lived shared checkout, which we do not have).
**Recommendation:** (a). A scheduled reaper is a solution looking for a problem here — the reaper is
inherently a *local-workspace* tool and CI/cron has no local workspace to clean. Documenting that the
loop boundaries ARE the cadence (already in forge-loop "Worktree hygiene") is sufficient. Escalated
only so the owner can confirm "loop-boundary-only is the intended cadence" rather than discover it.

## Not proposed (considered, declined)
- **A scheduled backlog status-truth reconcile.** The markdown backlog has stale `[ ]` on done items
  (TASK-0012/13/14, 0018, 0020). This is already owned by wrap-up step 3b (TICK-ON-MERGED reconcile);
  the fix is to RUN a forge-loop wrap-up, not to add new machinery. Noted in evaluation.md, left to
  the next loop wrap-up.
