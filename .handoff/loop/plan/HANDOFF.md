# HANDOFF — Planning Engineer Loop (fleet-convergence-first-run)

Cold-start resume pointer. State precedence: Git > this markdown. Read `loop_state.md` first.


## Root roll-up — TASK-0078 planning-harness convergence (2026-06-28)

**Status: HAND_OFF, not COMPLETE.** This root artifact is now the latest truth for the envctl
planning-harness reconciliation lane. The historical cycle-1/2 narrative below remains useful
provenance, but it is superseded by this roll-up until all root targets are terminal.

Evidence captured so far:
- **Task id:** `TASK-0078/envctl` planning-harness convergence / upgrade-only review slice.
- **envctl branch/worktree:** `plan-convergence-envctl-20260628` at
  `/home/drdave/Desktop/meta/.worktrees/plan-convergence-20260628/envctl`.
- **Canonical package branch/worktree:** `plan-convergence-harness-20260628` at
  `/home/drdave/Desktop/meta/.worktrees/plan-convergence-20260628/harness_hub`.
- **Canonical package PR:** `harness_hub` PR #64
  (`https://github.com/FlexNetOS/harness_hub/pull/64`) merged with commit `d31fb44`.
- **Upgrade-only invariant:** no production Rust code changed in this lane; changes are harness/docs/
  planning-state hardening only. A blind re-eject attempted to replace richer envctl ejected docs/tests
  with the package's thinner copy; that downgrade was rejected, the rich envctl contract was preserved,
  and only additive hardening was kept.

What changed in the current envctl slice:
1. `plan-artifact-gate.sh` now fails closed when a terminal root state (`status: COMPLETE`/`DONE` or
   `DONE`) has zero target rows, a nonterminal target row, or no `[x]` completed target. This closes the
   root-roll-up ambiguity that let a status document say complete while `targets.md` still said `[~]`.
2. `plan-weave-dispatch.sh` keeps the five-lane Opus/weave transport contract and removes the
   envctl-specific orchestrator prefix so the packaged harness remains reusable.
3. `test-plan-contract.sh`, `test-plan-artifact-gate.sh`, `test-plan-evals.sh`, and
   `test-plan-weave-dispatch.sh` lock the source-of-truth, PromptHub intent, Opus transport,
   terminal-roll-up, eval-fixture, and P9 pending-vs-missing contracts.
4. P9 is applied in the axis-agent prompts: an expected peer artifact absent during parallel fan-out is
   `PENDING` until its producer lane reports done; it becomes a hard missing-artifact finding only after
   producer completion.
5. P8 is accepted as the direction, partially satisfied by `instances/<target>/` namespace plus the new
   root terminal gate, and left open for a non-destructive `runs/<target>/` compatibility/migration PR.

Known open coordination items (not silently claimed done):
- `rusty-idd` PRs #130/#131 remain separate planning-run outputs to triage/merge independently.
- `prompt_hub` PRs #184/#188 remain upstream prompt/contract PRs to reconcile independently.
- The Odysseus service note reported source at `/home/drdave/Desktop/meta/.local/share/odysseus/src`
  and a live service, but the address was truncated; no exact endpoint is asserted here.

## Historical baseline (after cycle 2) — superseded by root roll-up above
- **Cycles 1 + 2 COMPLETE and HANDED OFF.** cycle_budget=2 reached.
- Cycle 1 = **rusty-idd** (`- [~]` planned-with-gaps; 8/12 dims verified; gate PASS).
- Cycle 2 = **handoff**, planned as the **union with rusty-idd** (`- [~]` planned-with-gaps; 12/13 dims
  verified, only performance `[~]`; gate PASS). Verifier 57 CONFIRMED / 3 QUALIFIED / 0 REFUTED.
- Run-from: `meta/.worktrees/plan-fleet-convergence/envctl` (branch `plan/fleet-convergence-first-run`).
- RED worktrees (separate, isolated): `plan-rusty-idd-red/rusty-idd` (@ `2f8a42f`), `plan-handoff-cycle2/handoff` (@ `d74ad4b`).
- **Owner review gate:** do NOT continue unattended until the owner approves.

## Owner verdicts (RESOLVED — binding north-star data; findings/resolved-decisions.md)
- **D1:** north-star lives @ `$META_ROOT` + handoff; **goal = handoff + rusty-idd UNION** (one continuity+intent control plane).
- **D2:** run from envctl (confirmed).
- **D3:** harness_hub = the **Front-Door interpreter** (transforms user intent → model-ready language).
- Three-way fit: **harness_hub interprets → handoff witnesses (deterministic classifier + ledger) → rusty-idd specifies (OpenSpec)**; weave = transport (distinct plane).

## The union verdict (cycle 2)
**MERGE.** handoff + rusty-idd's `crates/{cli,core,runner,spec,tui}` are **~95% shared-lineage forks**
(work-order originated in handoff). handoff is the production-hardened kernel with **real-teeth policy
gates** (`exit(1)`); rusty-idd is the intent/OpenSpec superset with the CLI handoff stripped. Fold
rusty-idd's CLI UNDER handoff's gates into one workspace. Sequenced union (reports/union-plan-handoff-rusty-idd.md):
1. **Resolve the RuVector `../../RuVector/*` path-dep** (A-U1) — SUPERVISED; the union is provably
   non-standalone today (fails at workspace manifest-load). **Everything sequences behind this.**
2. Dedup the shared crates (rusty-idd superset wins + re-apply handoff HFTASK-0082 lint) — SUPERVISED.
3. rusty-idd depends on handoff `work-order` + `validate_card` (kill the mirrored schema) — PROPOSE.
4. Design the missing **ledger read API** (intent plane reads witnessed state) — PROPOSE (gated on #1).
5. Bridge `hooks.toml` block-gates → Claude `PreToolUse` (closes the fail-OPEN seam for BOTH forks) — PROPOSE.

## To resume (cycle 3) — owner choice
Reap first (`cd meta/envctl && bash scripts/reap-worktrees.sh` → `--apply`). Then EITHER:
- **(a) Plan `weave`** — the A2A transport plane (ready-set; the union's distinct transport; A2A v1.0 is the interop target), OR
- **(b) Begin executing the union** — start with **RuVector A-U1** (SUPERVISED owner wall: vendor/publish/git-pin the `../../RuVector/*` deps so handoff builds standalone), the gate on which all other union steps depend.
Recommendation: (b)-A-U1 first if the owner wants to move the union forward; (a) if continuing breadth-first fleet mapping.

## Key corrections carried forward (verify against source, don't trust framing — lesson L5)
- Witness chain = **SHAKE-256 hash-link, UNSIGNED** (not blake3+ed25519). blake3 is only work-order intent_lock; ed25519-dalek compiles but never signs.
- RVF "semantic recall" is **dead** (0 callers, SHA3 pseudo-embeddings, write-amplified) — same class as rusty-idd's dead vector store.
- Fleet has **5 disjoint memory surfaces, no unified recall** (handoff ledger, RVF-dead, ICM, git-kb, rusty-idd .idd/knowledge 47MB).

## Residuals (cycle-3 follow-ups, non-blocking)
- rusty-idd: 4 dims [~] (performance/autoresearch/rules-policy-org/prompt-architecture); 500-row graph truncation.
- handoff: performance [~] (no measured delta); below-leaf tests blocked by RuVector; ledger read API unbuilt.

## North-star DRAFT (owner D1 — PROPOSE, not yet canon)
`reports/north-star-DRAFT.md` — proposed home: `$META_ROOT` + handoff, carried as DATA in the witnessed
`.handoff/context/capsule.json` `northstar` field (drift-invalidated by `handoff-drift`). Owner approves before it becomes canon.

## Proposed harness upgrades (propose-only; proposed-upgrades.md)
P1 (single-source artifact naming — VALIDATED this cycle, APPLY first when free-running enabled),
P2 (gate: feasibility verdict required), P3 (dimension-flip authority), P4 (DAG/backlog scaling),
P5 (clean-clone standalone-build gate — L6), P6 (cross-repo schema-drift gate — L7),
P7 (verifier must source-derive security-critical framing — L5, recurred). Plus the slug-regex relax/canonicalize item.
