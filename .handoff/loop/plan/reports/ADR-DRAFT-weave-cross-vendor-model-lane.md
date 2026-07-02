# ADR-DRAFT — weave: cross-vendor model lane (MiniMax guardian) governance

- Status: **DRAFT — proposed** (owner-wall; not written into weave's `.handoff/decisions/`)
- Date: 2026-06-26 · cycle 4 · target: weave (A2A transport plane)
- Relates: existing ADRs cover surfaces/transport (ADR-0001..0005); **none covers model routing**.
  Flagged ADR-uncovered by prompt-architecture (top ADR candidate) + rules-policy (CLAIM-P3).
- Traces to: CLAIM-P3 (CONFIRMED), prompt-arch §3 model-lanes (CONFIRMED), `ralph-weave.sh:18-21`,
  `.claude/agents/weave-guardian.md:16`, `weave-orchestrator/SKILL.md:56`.

## Context

weave's autonomous loop runs a **dual-model / cross-vendor** strategy:

- **Standalone lane = all-Opus.** The orchestrator hard-codes `model: opus` for every agent;
  planner/implementer/verifier/guardian all carry `model: opus`.
- **Autonomous loop lane = dual-model.** Local Opus runs plan→implement→verify→deliver, but the
  Phase-4 invariant/drift/docs **guardian is delegated to MiniMax `minimax-m3:cloud`** (a non-Anthropic
  model) as the external gate. The loop runner defaults the guardian to MiniMax
  (`MODEL="${WEAVE_MODEL:-minimax-m3:cloud}"`) and the worker to `claude`, overridable by env. Guardian
  BLOCK wins over verifier GREEN ("never ship RED or BLOCK").

So **a non-Anthropic model is the final correctness/security gate before auto-merge.** weave itself
carries no model-routing logic (consistent with transport-not-interpreter) — MiniMax writes its verdict
into the shared `.handoff/loop/` ledger that weave-transported sessions read. The trust boundary,
fallback, and availability of that cross-vendor lane are architectural and currently **undocumented**:
`WEAVE_SKIP_GUARDIAN=1` disables the gate entirely, and there is no stated fallback when the MiniMax
lane is unavailable.

## Decision

Record the cross-vendor model lane as a governed architectural decision:

1. **Affirm the dual-model split** — an independent, non-builder, cross-vendor model (MiniMax
   `minimax-m3:cloud`) is the final invariant/drift/docs guardian before auto-merge; guardian BLOCK is
   authoritative for invariants/drift. This is a deliberate "reviewer model ≠ builder model" design.
2. **Define the fallback / availability posture** — state what happens when the MiniMax lane is
   unavailable: the merge must **fail closed** (no auto-merge without a recorded guardian verdict), and
   `WEAVE_SKIP_GUARDIAN=1` is an owner-only escape, never a silent default. (Owner to set the exact
   fallback model/behavior — this ADR records that the decision is required and must be fail-closed.)
3. **Pin the trust boundary** — the guardian model sees the diff/ledger but holds no write credentials;
   its verdict is advisory-to-the-gate, enforced by the loop, not by the model directly.

## Consequences

- **Positive:** an external, cross-vendor reviewer reduces single-vendor blind spots on the final gate;
  the decision becomes auditable rather than implicit in a shell script's env default.
- **Cost / risk (SUPERVISED — see `risk-policy.md`):** a non-Anthropic model gating auto-merge is the
  one item touching the **model** risk dimension; availability/latency/cost of the external lane and the
  `WEAVE_SKIP_GUARDIAN` escape are governance-sensitive. Owner-decided.
- **No-Downgrades:** the fallback must never weaken the gate to "merge without a guardian verdict."

## Alternatives considered

- **All-Opus everywhere (drop the cross-vendor guardian)** — rejected: loses the independent-reviewer
  benefit; the dual-model split is a deliberate design (CLAIM-P3).
- **Leave it undocumented** — rejected: a non-Anthropic model gating auto-merge with an implicit
  skip-flag and no stated fallback is exactly the kind of architectural decision an ADR exists to record.
- **Make MiniMax the builder too** — rejected: violates reviewer ≠ builder separation.
