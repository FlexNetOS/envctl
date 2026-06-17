# Run evaluation — G2 (native GitHub App installation-token minting via secretd)

Run: branch `g2-native-mint`, PR #102 → develop. Crew: feature-architect → rust-implementer →
invariant-guardian (Feature Forge, sequential single-crew). Verdict chain: GO → GREEN →
PASS-WITH-NOTES (0 blocking findings).

## Friction — LOW
- Zero design↔build or build↔verify retries. The plan landed GO first pass; the implementer
  reported GREEN first pass; the guardian PASSed with 0 blocking findings. No item bounced backward.
- One latent gap (`MintReq.mode` absent ⇒ `NativeSubtoken` unreachable via `Mint`) was discovered
  by the architect *during design*, not at build/verify time — folded into U4 before any code was
  written. This is friction *avoided*, not incurred.
- The triggering claim (#116: `inject.rs`/`run_child = todo!()`) was **stale/false at HEAD**; the
  orchestrator verified against source before designing rather than building on the false premise.
  No cycle was wasted, but only because of an unencoded instinct (see Lesson 2).

## Gate quality — HIGH
- The guardian ran all 3 CI gates (no-c/shape/enable) + p7 + fmt + clippy (gate form) + every test
  suite, with exact exit codes. No defect slipped past; nothing was false-blocked.
- The pre-existing `--all-targets` clippy error (`gui/main.rs:1997`, untouched file) was correctly
  classified as inherited-red, not a G2 regression — both the implementer (Deviation #1) and the
  guardian (Finding #1, severity none) handled the baseline-vs-introduced distinction cleanly. The
  guardian additionally verified `git diff --name-only` shows no gui files. This worked, but the
  *method* for it lives only in the verification recipe's toolchain axis, not its CI-mirror axis
  (see Lesson 3).
- Fail-closed refusal path (HTTP error ⇒ durable Refused + no token) is unit-tested AND e2e-tested.
  No-secret-on-wire is asserted by a wire-capture e2e. Gate did its job.

## Coverage — COMPLETE (one documented, justified deferral)
- All 6 units (U1–U6) landed GREEN; nothing capped or silently dropped.
- GUI relay-mint parity is the one deferred surface — explicitly justified (no GUI relay-mint
  surface exists yet; all logic is engine-side via `resolve_injection`, so the follow-up is pure
  wiring). Flagged in the plan (R5), the log (Deviation #2), and the guardian report — not silent.
- Enrollment verb (`secretctl github-app enroll`) deferred to immediate follow-up (R2) — documented.

## Human walls — NONE
- No NEEDS-HUMAN, no manual intervention. The architect's "Open questions" was empty (GO, no owner
  decision required). The run was fully autonomous end to end.

## Net
A clean, low-friction run. The lessons below are about *encoding the instincts that made it clean*
so they're guaranteed next time — not about fixing breakage.
