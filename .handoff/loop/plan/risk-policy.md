# risk_policy — weave (cycle 4)

Per-upgrade risk classification for the weave plan (`reports/weave-plan.md`). Each upgrade is graded
**APPLY** (additive/no-prod-code; auto-applyable in scope), **PROPOSE** (lands via PR/owner review),
or **SUPERVISED** (large blast / control-plane / security; human-in-the-loop, never auto-applied).
Graded across five dimensions: **trust-boundary** (no-C / pure-Rust / new-dep), **secrets**,
**destructive** (irreversible/data-loss/state mutation), **provider** (external runtime/vendor),
**model** (LLM lane/routing). Frame: Upgrade Only, No Downgrades, fail-closed; gates only strengthen.

`risk_policy` version: weave.v1 · built from CONFIRMED/QUALIFIED verdicts only.

---

## Classification table

| upgrade | tier | trust-boundary | secrets | destructive | provider | model | rationale |
|---|---|---|---|---|---|---|---|
| **R1 — A2A v1.0 interop adapter** | **PROPOSE** (additive) | clean — pure Rust over existing `serde_json` + `ed25519-dalek`; NO new dep, NO C enters the boundary (if gRPC ever added, pure-Rust `tonic`/`prost` only) | none — AgentCard signing reuses default-off `sign` keys already in `Intent.sig` | none — additive `to_a2a`/`from_a2a`; never mutates `Intent` serde; SQLite-mailbox transport untouched | new external interop edge (A2A agents) — default-off `a2a` feature | none — transport-only; carries no model routing | new public protocol surface at the highest-blast schema (1238), but additive + default-off keeps effective blast contained; RED suite is the acceptance contract |
| **R2 — dual-backend conformance harness** | **APPLY** | clean — test crate, pure Rust, touches no production code | none | none — read/exercise only | none | none | additive `tests/store_conformance.rs`; lowest-risk; locks the verified `guard_writable` asymmetry; re-scoped to ~90 methods |
| **R3 — single-source CLI↔MCP verb mirror** | **PROPOSE** | clean — additive cross-guard test (low-risk path); declarative-registry derive deferred | none | none — enumerates surfaces, no behavior change | none | none | touches the control-plane verb surface of two crates; test enumerates measured 71/72/76 |
| **R4a — documented-gate 6→7 alignment** | **PROPOSE** | clean — doc edit | none | none | none | none | `CLAUDE.md`/`policy.toml` are protected canon (`rules.toml:40-50`); pure truthfulness tightening |
| **R4b — Python-out-of-CI (Rust xtask)** | **PROPOSE** | clean — restores Rust-native invariant; pure Rust | none | none — must reproduce same `target-smoke.json` schema + `cargo deny` posture (a port, never a relaxation) | removes Node/Python runtime from build plane | none | CI + build-tooling; never weaken — gate outputs must be identical |
| **R5 — memory-organ separation ADR + doc** | **PROPOSE** | clean — docs/ADR, no code deletion | none | none | option (a) ICM wiring is feasibility-constrained (cross-binary weave-core→ICM coupling) → default to option (b) doc-contract | none | governance decision; provenance/opt-out RED test + no-vector regression fence |
| **R6 — `main.rs` dispatch extraction** | **SUPERVISED** | clean — pure behavior-identical move, pure Rust | none | none — pure move, but on the **highest-blast bin** (427); a wrong move silently changes dispatch | none | none | large structural refactor on the dispatch god-file; one verb-group per PR; fenced behind R2+R3; `main_rs_line_cap` gate + per-handler unit test |
| R-aux — repo-native git-kb freshness / renovate (autoresearch U1/U2) | PROPOSE | clean — additive CI, no trust-boundary code | none | none | renovate/dependabot is an external bot edge | none | low-priority observability; bot PR noise is the only cost |
| R-aux — user-global residency exemption ADR (WV-FSL-3) | PROPOSE | n/a — doc/ADR (option a) or env redirection (option b) | none | option (b) env redirection touches the broker rendezvous | envctl ownership if option (b) | none | OWNER-WALL; XDG-correct but unmanaged w.r.t. meta-residency; no exemption ADR exists |
| R-aux — PreToolUse document + optionally arm (U-GOV-009) | **SUPERVISED** | n/a | n/a — approver config | none | peer/owner approver over weave's own mailbox | none | changes default behavior of a SECURITY gate; security TIGHTENING only — must never weaken deny-by-default (`main.rs:8896-8919`) |
| R-aux — cross-vendor model lane governance (ADR) | **SUPERVISED** | n/a | none | none | MiniMax `minimax-m3:cloud` external vendor | **YES — a non-Anthropic model is the pre-auto-merge guardian** | the model dimension's only material item; availability/fallback (`WEAVE_SKIP_GUARDIAN=1` disables the gate) is architectural → ADR draft |

---

## Dimension notes

- **trust-boundary:** the headline R1 A2A adapter is feasible precisely because it adds NO C and NO new
  dependency — it rides the already-present pure-Rust `serde_json` and `ed25519-dalek` (verdicts
  U-ARCH-2). Any future gRPC binding is gated on pure-Rust `tonic`/`prost`, never a C protobuf.
- **secrets:** no upgrade introduces a new secret. R1's AgentCard signing reuses the existing default-off
  `sign` (ed25519) keys; push-bearer-token handling is unchanged.
- **destructive:** no upgrade deletes data or mutates on-disk state irreversibly. R6 is a pure code move
  but is graded **SUPERVISED** because the blast lands on the dispatch path (a behavior-identical move
  that goes wrong silently mis-dispatches verbs) — hence one-group-per-PR + the line-cap gate + the
  R2/R3 test fences as prerequisites.
- **provider:** R4b removes the Node/Python build-plane runtime; the autoresearch bot and the A2A edge
  add opt-in external edges; the MiniMax guardian is the standing external provider coupling.
- **model:** only the cross-vendor guardian lane touches the model dimension — a non-Anthropic model
  gates auto-merge — which is why its governance ADR is **SUPERVISED** and owner-decided.

## SUPERVISED items (human-in-the-loop, never auto-applied)

1. **R6 — `main.rs` dispatch extraction** — large blast on the highest-blast bin; SUPERVISED until
   R2 (parity harness) and R3 (verb-parity test) are GREEN as fences.
2. **PreToolUse document + arm (U-GOV-009)** — changes a security gate's default posture; SUPERVISED,
   tightening-only, deny-by-default semantics must be preserved.
3. **Cross-vendor model-lane governance ADR** — a non-Anthropic model is the final gate before
   auto-merge; SUPERVISED owner decision (`ADR-DRAFT-weave-cross-vendor-model-lane.md`).

Everything else is APPLY (R2 only) or PROPOSE. No upgrade is auto-applied that touches the trust
boundary, secrets, a destructive op, or the model lane without supervision.
