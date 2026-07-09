# ADR-0004: Two-plane model routing — operator-pinned session model, RuvLTRA-tiered subagent dispatch

- Status: draft        <!-- draft → accepted (owner) → superseded; never self-accept -->
- Date: 2026-07-09
- Target: agentic-os-blueprint
- Plan: ../loop/plan/reports/agentic-os-blueprint-plan.md

## Context

Two doctrines looked like they collided, and the collision shaped a wrong "not recommended" call in
the first blueprint audit:

- **Law 8** (`home/.claude/rules/laws.md`): "MODEL ROUTING IS AN OPERATOR DECISION. Everything runs
  on Fable unless the operator says otherwise" — enforced live by the statusline reroute alarm
  (verdict V18).
- **The operator said otherwise for a specific plane**: commit `79c2f91f`
  (`meta-ruvector-router-wt`, 2026-07-09) wires a RuvLTRA FastGRNN complexity tier into the harness
  router; the router source carries "Operator directive 2026-07-09: RuvLTRA is installed and
  proven"; three RuvLTRA GGUFs were pulled the same morning (V18). The tier runs real local WASM
  inference (`backend: "ruvltra-fastgrnn"`) but is currently non-discriminating — both a
  BFT-consensus prompt and "fix typo in README" route to `opus` at ~0.55 confidence (V7); the
  commit's "fix typo → haiku PROVEN" does not reproduce live.

These are not in conflict: Law 8 assigns the decision; the directive *is* the decision. What is
missing is the doctrine that makes both planes explicit and gates the tier on evidence.

## Decision

Adopt **two-plane routing**:

1. **Session plane (interactive main loop): operator-pinned.** The session model stays Fable;
   any reroute (safety classifier or otherwise) trips the statusline alarm and requires operator
   acknowledgment. Nothing automated may change the session model. (Law 8, unchanged.)
2. **Worker plane (subagent/tier dispatch): RuvLTRA-tiered, calibration-gated.** The harness
   router may assign Claude tiers (haiku/sonnet/opus) to spawned workers using local RuvLTRA
   FastGRNN complexity inference — **only after** the discrimination acceptance passes (plan row
   R4 / test T4: a 10-prompt fixture routes trivial→haiku ∧ complex→opus, reproducible ×3). Until
   then the tier block must remain unmerged; the router's fail-closed shape (tier absent ⇒ pure
   keyword routing, observed live in the main checkout) is retained permanently.

## Consequences

- Positive: local intelligence routes worker cost/capability without ever touching the operator's
  session plane; the blueprint's "complexity routing head" lands as an *upgrade* instead of a Law-8
  exception; a constant-opus tier-inflator is structurally blocked by the merge gate.
- Negative/commitments: the router becomes a governed prompt-architecture component (every
  UserPromptSubmit flows through it — fleet-wide blast, V7), so its fixtures live in CI (T4) and
  future tier changes need the same evidence; RuvLTRA model files become a managed runtime
  dependency (§4 tool-eval: calibrate).
- Forecloses: fully-automatic session-model routing (explicitly out — Law 8), and merging
  `codex-ruvltra-router` on its current "PROVEN" claim (refuted live, V7).

## Alternatives considered

- **Status quo (keyword-only router, operator-only everything):** loses the local complexity
  signal the operator already built and directed; leaves the worktree branch to rot.
- **Full auto-routing including the session plane:** violates Law 8; rejected.
- **ruvllm's built-in `claude_flow` model_router:** routes only among cloud Claude tiers inside the
  crate with no local-arm or harness integration (audit finding); the harness router + local
  FastGRNN is the operator's chosen surface.
- **Merge now, calibrate later:** rejected — live evidence shows a constant router (V7); merging
  would tier-inflate every routed task to opus.

## Invariants check

- **No C in the trust boundary:** untouched — FastGRNN runs as WASM under bun in the var-runtime
  lane, outside the Rust trust boundary; no engine dependency changes.
- **One rustls / ring-only:** untouched (no Rust dep changes).
- **Engine is the single shared library:** untouched (router is harness-side JS).
- **Destructive ops fail-closed:** strengthened — the tier is additive with an observed fail-closed
  fallback (absent ⇒ keyword routing), and the merge itself is gated on T4 evidence.
