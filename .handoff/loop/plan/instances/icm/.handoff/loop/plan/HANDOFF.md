# HANDOFF — fleet-convergence planning loop → cycle 8 (Codex lane)

> Written at the cycle-7 close under a hard token ceiling (~5% Opus remaining). Owner directive:
> **"strictly swap codex for opus — Claude tokens exhausted."** Cycle 8 must run on the **Codex**
> foreground lane, NOT by spawning Opus sub-agents. This is a model-lane swap, not a scope change.

## State at handoff (Git is source of truth)
- **Cycle 7 (icm) = COMPLETE + SHIPPED.** envctl PR **#287** (base master, auto-merge armed) ·
  icm RED PR **#5** (base develop, acceptance contract — NOT auto-merged). Lease `plan:claim:icm`
  released. ICM store `01KW407HX8HC8XDXA1Y44JRXTV`.
- Verdict: icm = canonical agent-memory plane, PEER of handoff ledger + git-kb. Convergence = **SIDECAR**
  (unconditional C-floor vs handoff no-C redb kernel) + **bind-as-data** via a typed `memory` pointer in
  `handoff.context_capsule.v1`. RED GREEN target = time-aware recency/decay (5 tests @ 258667e).
- Planned organs so far: rusty-idd · handoff · weave · grit · prompt_hub · **icm**. Front-door pair:
  prompt_hub (STORE) done; **harness_hub (INTERPRETER) still unplanned**.
- Counters: cycles_this_session=1, cycles_total=7, cycle_budget=1 (per-instance), wrap_every=1.

## Cycle 8 — recommended target + why
**Recommended: `harness_hub`** — the Front-Door INTERPRETER (owner D3 north-star: transforms user
intent → model-ready language). It is the last unplanned half of the two-layer front door (prompt_hub
STORE is planned). Alternatives in the ready-set: `lane` (execution/model-lane organ). Owner may
override; if unsure, auto-claim the top unclaimed ready node in `graph/target-dag.md`.

## How to run cycle 8 on Codex (resume instructions)
1. From `meta/envctl`, export `META_ROOT=/home/drdave/Desktop/meta`.
2. **Reap** (mandatory): `bash scripts/reap-worktrees.sh` then `--apply`.
3. **Claim**: `HF_LEASE_HOLDER="plan-harness-hub-<date>" weave lease reserve --resource "plan:claim:harness-hub" --ttl 1800 --note "plan-loop cycle 8 (codex)"`.
4. **Worktree**: `git worktree add meta/.worktrees/plan-harness-hub/envctl -b plan/loop-harness-hub origin/master`. Seed targets.md (single kebab row `harness-hub`), loop_state.md, dimensions.md (mirror this cycle's skeleton; note targets.md prose lines MUST be `#`-prefixed or blank — the gate parser rejects bare prose rows).
5. **Drive with Codex** (the swap): run the crew via the Codex CLI as the foreground analyst, e.g.
   `codex` re-entering `/harness:plan-loop` against `prompt_hub/prompts/plan-loop-parallel-run.md`,
   target=`harness-hub`. Do NOT spawn Opus sub-agents. If a bg lane is needed, route via
   `scripts/plan-weave-dispatch.sh --target harness-hub --root meta/harness_hub ...` with
   `PLAN_OPUS_CMD` pointed at the available (non-Opus) runtime.
6. Gate (`scripts/plan-artifact-gate.sh`), ship (envctl PR base master + auto-merge; RED PR on the
   target repo — CHECK its PR-base rule first), release lease, `icm store`, notify envctl via weave.

## Gate gotchas carried forward (save a re-run)
- targets.md: every non-target line `#`-prefixed or blank; active rows = ONE kebab slug.
- No bare sentinel tokens (TODO/TBD/"placeholder evidence"/"citation needed") ANYWHERE in
  `require_contains` files — incl. meta-notes like "no TODO used"; hyphen-break them.
- Numeric cross-agent facts (dims/versions) must be verifier-adjudicated before becoming plan facts
  (L-icm-2, now 2nd recurrence — apply-eligible).

## Apply-eligible upgrades queued (steward)
- L-icm-2 (2nd recurrence): add "numeric facts are must-refute" to the verifier agent def.
- P1 (carried from cycle 6): scope git-kb index to member `src/`, excluding `vendor/`.
