# HANDOFF — forge-loop (envctl secrets / Epic-F build) · 2026-06-17 (session 7)

closed_utc: 2026-06-17   branch: develop (work in FRESH worktrees off develop)
cycle_budget: 1   cycles_this_session: 1   cycles_total: 16
last_item: TASK-0031-PR2 (DONE, PR #122, guardian PASS)   next_item: TASK-0027 (early-revoke)
orchestrator_phase: handoff (cycle budget reached)   gate_status: PASS   pr_url: https://github.com/FlexNetOS/envctl/pull/122
resume_command: /forge-loop resume   (reads this file + backlog "⏭ NEXT PICK")

## State (authoritative = Git/merged PRs; this file is a companion view)
MERGED to develop: #106 (TASK-0026 enroll), #109 (TASK-0030 jti + OI-SM-1), #111 (TASK-0031 PR-1 edge
  listener), #112 (TASK-0036 mlockall), **#117 (TASK-0032 F5 stream tear-down)**, **#108 (TASK-0035 gRPC
  gaps)**, #119 (session-6 reconcile), plus infra #113 (low-cost-kdf-tests) / #114/#115/#120/#121 (Seed +
  manifest portability).
IN FLIGHT (auto-merge armed): **#122 (TASK-0031-PR2 — F2 edge hardening)**. guardian PASS, 4 gates green,
  zero new deps, relay-edge-OFF build unaffected.
Earlier merged: #102/#103/#104/#105/#107/#118.

**Epic F / relay edge is now feature-complete on the listener path:** PR-1 (listener, TLS+DPoP/EKM+jti →
relay_swap, #111) + PR-2 (hardening: nonce + admission/rate-limit + body-caps/timeouts + opt-in mTLS, #122)
+ PR-3 (streaming-revocation tear-down, #117). The daemon is mlock-hardened (#112) and the gRPC surface is
real (#108). What remains in Epic F is the revoke/UX/verify tail, not the edge core.

## ⚠ FIRST on resume (baseline verify)
1. From an envctl worktree (NOT meta root): `git -C envctl fetch origin develop && gh pr list --state open`.
2. Confirm **#122** merged. If DIRTY (a sibling merged first, touching `lib.rs`/`broker/mod.rs`/`.handoff`),
   rebase onto develop: resolve `.handoff/loop/cycle/*` conflicts by taking the PR's own side (`--theirs`);
   take develop's side for `loop_state.md`/`HANDOFF.md`/backlog NEXT-PICK (`--ours`) but KEEP the PR's real
   per-item checkbox ticks; `cargo check -p envctl-secretd --features relay-edge`; `git push --force-with-lease`.
   NOTE: this session bundled the reconcile commit INTO the feature branch BEFORE CI finished, so the
   squash-merge carried the bookkeeping (last session's separate chore PR #119 was needed because auto-merge
   squash-landed the feature before the reconcile — bundling avoids that orphan).

## NEXT (dep order)
1. **TASK-0027 (early-revoke)** — the revoke path: surface/strengthen relay+bearer early-revocation so a
   revoke takes effect immediately across the gRPC surface and the edge (the stream tear-down #117 already
   reacts to it within ≤2s; this is the revoke *issuance/propagation* side). Read `docs/secrets/SERVER-MODE.md`
   + `THREAT-MODEL.md` for the F-id, and the existing `relay_revoke`/`relay_revoke_bearer` engine methods.
2. Then **TASK-0028** (GUI parity for the new secrets verbs) → **TASK-0037** (Phase-7 verify-don't-rebuild:
   confirm secrets verbs folded onto `envctl`, manifest component, stale ROADMAP lines) → **TASK-0034**
   (hardening tail: F10 tonic pin + cargo-audit CI, F11 MSRV check, F18 audit-fsync) → **TASK-0038** (Certs.*
   Phase-4+). Small follow-up: **MADV_DONTDUMP** companion to the merged #112 mlockall.
   New follow-ups filed this session: **TASK-0031-PR2c** (PROXY-protocol source IP), **TASK-0039**
   (remote-clients-CA lifecycle for the mTLS verifier).
SKIP **TASK-0033** (VPS Profile B) — owner-gated `[!]`.

## decisions_and_dead_ends (don't re-litigate)
- Nonce + admission are ENGINE security policy (siblings to broker::jti), not edge logic — the edge only
  emits the 401/`DPoP-Nonce` header, 429, 413, 408. `ring` is now an unconditional secrets-engine dep
  (NonceStore.issue needs `ring::rand` on the always-built path); it was already in the resolved graph via
  rustls, so NO new lockfile crate and no-c stays green.
- Admission sheds per-IP BEFORE any crypto (CVE-2024-47609); per-CLIENT quota stays in `decide()`
  `rate_per_min` (client_id is unauthenticated pre-verify). Admission can only reject early — the full
  verify ladder + `decide()` still run on every non-shed request (an e2e asserts a 429'd req never reaches
  the recording upstream). decide() remains the sole Allow authority; mTLS is additive, never a replacement.
- Nonces are single-use (consume on accept); a genuine retry re-challenges → fresh nonce → fresh proof+jti.
  Windowed (TTL without removal) is a one-line fallback if HTTP/2 coalescing ever flakes (it shouldn't —
  each request carries its own proof).
- mTLS verifier built `WebPkiClientVerifier::builder_with_provider(roots, ring::default_provider())` —
  ring-only (confirmed vs in-tree rustls 0.23.40, NOT context7 which returned stale 0.20 docs). The
  client-CA is a SEPARATE operator input on the SAME relay-tls ServerConfig — never the MITM CA (FS-S25).
- Bundle the loop reconcile commit into the feature branch before CI finishes (don't rely on a separate
  chore PR) — auto-merge squash-lands the feature the instant CI is green and would orphan a late reconcile.
- Recurring rebase churn: every secrets PR touches `lib.rs`/`broker/mod.rs` + `.handoff/` → siblings go
  DIRTY. Resolve by taking the PR's code/cycle-artifacts and develop's loop_state/HANDOFF/backlog-narrative.

## Invariants (carry forward — non-negotiable)
no-C trust boundary (reuse rustls-ring / ring / libc-FFI, no banned dep) · fail-closed / fail-safe (never
panic on a hardening/verify/admission path; strict/require modes refuse) · no secret in logs/audit
(metadata-only) · engine the single sync non-printing authority (policy in engine via decide()/nonce/
admission, I/O in front-ends) · relay-tls only never MITM CA (FS-S25) · EKM bind (FS-S20) · default-OFF
behind `relay-edge`, mTLS additionally opt-in.

## verify_on_resume (exact)
- `git -C envctl fetch origin develop && gh pr list --state open` (from a worktree)
- rebase #122 if still DIRTY (steps above); confirm it merged
- new worktree: `git -C envctl worktree add ../.worktrees/task-0027-early-revoke/envctl -b task-0027-early-revoke origin/develop`
