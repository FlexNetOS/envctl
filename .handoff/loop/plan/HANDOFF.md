# HANDOFF — Planning Engineer Loop (fleet-convergence-first-run)

Cold-start resume pointer. State precedence: Git > this markdown. Read `loop_state.md` first.

## Where we are
- **Cycle 1 of the capped first run is COMPLETE and HANDED OFF.** cycle_budget=1 reached.
- Target planned: **rusty-idd** → `- [~]` planned-with-gaps (artifact gate PASS; 8/12 dimensions verified).
- Run-from: `meta/.worktrees/plan-fleet-convergence/envctl` (branch `plan/fleet-convergence-first-run`).
- RED tests live in a SEPARATE worktree: `meta/.worktrees/plan-rusty-idd-red/rusty-idd`
  (branch `plan/rusty-idd-red-tests`, commit `2f8a42f`).
- **Owner review gate:** do NOT continue unattended until the owner approves (first-run brief §8).

## To resume (cycle 2)
1. Reap first: `cd meta/envctl && bash scripts/reap-worktrees.sh` then `--apply`.
2. Verify-on-resume baseline; if it fails → write NEEDS-HUMAN, reset cycles_this_session.
3. Pick the **ready-set top = `weave`** (the nervous system — unblocks rusty-idd, envctl, harness, the
   agents; see `graph/target-dag.md`). Reset cycles_this_session=0.
4. Run one planning cycle on weave via the same crew.

## rusty-idd residual (cycle-2 follow-up on rusty-idd, not blocking the fleet)
- 4 dimensions analysed-not-verified (gate them next): **performance** (needs measured build/binary/runtime
  delta, not asserted), **autoresearch**, **rules-policy-org**, **prompt-architecture** (open + gate their
  own CLAIM rows). See `dimensions.md` + `findings/verdicts.md`.
- Re-run per-crate `git-kb code symbols` to lift the 500-row truncation (public-API/dead-code are lower bounds).
- Cross-repo fabric edges remain UNCONFIRMED (fleet-map marked them so).

## The 3 decision-findings awaiting owner verdict (the loop's deliverable, NOT pre-answered)
1. **Where the shared fleet north-star lives.** There is NO single north-star every repo can read — two
   competing docs (meta-root `NORTH-STAR.md` + a different one in `handoff/`), neither propagated to
   member repos. Recommend: one fleet-level artifact, bound by repos AS DATA. (fleet-north-star-map.md)
2. **Run-from / residency / transport.** §2 placement (run from envctl, weave as transport found via
   META_ROOT) VALIDATED this run — plan state landed correctly under the envctl worktree; weave resolved
   but wasn't exercised (Claude foreground → direct Opus sub-agents). Confirm as the standing model.
3. **harness_hub audience** (internal-only vs shareable marketplace) — determines whether the north-star
   binds in the skill or in a meta-level layer the skill reads. Unresolved; needs owner intent.

## Headline plan finding (rusty-idd)
rusty-idd attaches to the fabric by **filesystem + JSON-schema contracts only** — `weave`/`icm`/`grit`/`hf`
have **0 library/IPC deps** (CONFIRMED). The decision-relevant gap = a **typed, C-free live binding** to
those organs that KEEPS the filesystem `.handoff/` contract as the required fallback (DRAFT ADR:
`reports/ADR-DRAFT-rusty-idd-convergence-boundary.md`). Concrete first step: make the `work-order`
(`handoff.task.v1`) card-load **fail-closed** — the authored RED suite already proves it's fail-open today.

## Proposed harness upgrades (propose-only this run — owner approves before apply)
See `proposed-upgrades.md`: P1 single-source artifact-name contract (eject↔gate naming drift),
P2 gate strengthening (feasibility verdict required), P3 dimension-flip authority, P4 DAG/backlog scaling,
plus the slug-regex constraint (gate rejects snake_case repo names — relax or canonicalize).
