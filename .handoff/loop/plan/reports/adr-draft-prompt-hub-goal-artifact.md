# ADR-DRAFT — Typed versioned GoalArtifact envelope (harness_hub → prompt_hub → rusty-idd)

- Status: DRAFT (proposed). Not yet a numbered prompt_hub ADR — the next free number must be chosen in
  prompt_hub's own `docs/adr/` (NOT 0007, which is the existing Plugin System ADR — see Context).
- Date: 2026-06-27. Origin: plan-architect, cycle 6. Lives under the plan dir only; never written into
  prompt_hub's tree by planning.
- Supersedes the dangling "(ADR-0007)" citation in `docs/plans/lifeos-meta-front-door.md:36` for the
  intent-store boundary.

## Context

prompt_hub is the durable Front-Door intent STORE (owner decision D3): the harness_hub interpreter
produces intent, prompt_hub stores/governs it, and it must hand a provenance-stamped GOAL ARTIFACT to
rusty-idd, which owns lifecycle. This cycle CONFIRMED (verdicts.md VERDICT 1, + a 7-RED probe) that the
contract is ASPIRATIONAL / doc-only:

- `grep -rniE 'goal_artifact|provenance|rusty-idd' prompt-hub/src prompthub/src prompthub-server/src` → 0 hits.
- `hub.rs:981-999` `get()` is a "Simplified" retrieval; `models.rs:388-408,558-566` `Prompt`/`Intent`
  carry no `schema_version`/`provenance`; `Intent` is transient.
- Provenance is fragmented across three half-primitives that never compose: the audit SHA-256 chain
  (`audit.rs`, log-only), lineage (`lineage.rs`, in-memory/test-only, `created_at:"now"` sentinel), and
  the unsigned `Prompt.author`.

Two governance defects make an ADR necessary now:

1. **Number collision** (VERDICT 2): the front-door plan cites "(ADR-0007)" for "prompt_hub = intent
   store/boundary", but `docs/adr/0007-plugin-system.md` is the unrelated Plugin System ADR. The
   authoritative boundary ADR does not exist in prompt_hub.
2. **Unbound wire format risk**: the analyst's proposed fields are a plausible-but-unbound guess. The
   gate STRENGTHENED feasibility with a non-negotiable condition — the envelope must bind to rusty-idd's
   ACTUAL consumer schema (`rusty-idd/.handoff/loop/plan/`), not be invented in prompt_hub.

## Decision

Record a new prompt_hub ADR that:

1. Names prompt_hub the **durable, provenance-stamped intent STORE + boundary** in the two-layer front
   door, and rusty-idd the consumer; prompt_hub never owns rusty-idd lifecycle.
2. Defines a **typed, versioned `GoalArtifact` envelope** (new feature-gated `prompt-hub/src/goal_artifact.rs`)
   as the wire format prompt_hub emits and rusty-idd consumes. Proposed shape (to be RECONCILED with
   rusty-idd's consumer schema before it is canonical):
   - `schema_version: String` (e.g. `"goal-artifact/1"`) — stable, identical across prompt versions.
   - `artifact_kind: "goal_artifact"`, `target: "rusty-idd"`, `origin_prompt_id`.
   - `goal` — the intent payload (derived from `Intent`/the selected prompt).
   - `provenance: { produced_by: "prompt_hub", produced_at, prompt_hub_version, audit_hash, author,
     sources: [..non-empty citations..], lineage_path }` — composes the three existing fragments into
     one stamped object.
3. Fixes the dangling citation: the front-door plan must reference THIS ADR's number, not local 0007.
4. Treats the **plugin native-code trust boundary** as a separate amendment to the real ADR-0007
   (Plugin System): loaded `.so` objects are outside the `#![forbid(unsafe_code)]` guarantee.

## Binding condition (non-negotiable)

The envelope field set MUST be derived from rusty-idd's actual goal-file consumer schema. Sequence:
**step 0** read `rusty-idd/.handoff/loop/plan/` → **step 1** implement the typed envelope serializing to
it → **step 2** add the emit CLI verb/route → the RED suite `prompt-hub/tests/goal_artifact_contract.rs`
flips GREEN. An unbound envelope must NOT land as canonical (verdicts.md UPGRADE A/B/H conditions).

## Consequences

- Positive: the store's defining job becomes a typed, falsifiable, version-pinned contract; provenance
  is one stamped object; rusty-idd can consume it safely; the 7-RED suite becomes the regression gate.
- Cost: touches the `hub.rs` god-object + the 194KB `routes.rs` — bounded by first extracting a
  `provenance` sub-facade (roadmap PROV-facade) before the emit surface lands.
- Risk: a guessed schema would couple rusty-idd to a wrong wire format — mitigated by the step-0 binding.

## Alternatives considered

- **Reuse `export`** (generic `serde_json::to_string(prompt)` dump): rejected — no provenance, no schema
  version, no rusty-idd framing (`export.rs:42-52`). EXPORT-1 adds an audit-hash to export as a cheap
  step, but it is NOT a substitute for the typed envelope.
- **Emit the bare `Intent`/`Prompt`**: rejected — transient, no id/timestamp/author/provenance
  (`models.rs:558-566`); cannot be a durable, addressable goal artifact.
- **No ADR / keep prose**: rejected — a cross-repo runtime contract between two authoritative engines
  must be a recorded decision, and the current citation mis-resolves.
