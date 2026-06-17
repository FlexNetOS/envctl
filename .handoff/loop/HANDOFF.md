# HANDOFF — forge-loop (envctl secrets / Epic-F build) · 2026-06-17 (session 4)

closed_utc: 2026-06-17   branch: develop (work in FRESH worktrees off develop)
cycle_budget: 1   cycles_this_session: 1   cycles_total: 13
last_item: TASK-0031 PR-1 (DONE, PR #111, guardian PASS)   next_item: TASK-0032 (F5 stream tear-down)
orchestrator_phase: handoff (cycle budget reached)   gate_status: PASS   pr_url: https://github.com/FlexNetOS/envctl/pull/111
resume_command: /forge-loop resume   (reads this file + backlog "⏭ NEXT PICK")

## State (authoritative = Git/merged PRs; this file is a companion view)
MERGED to develop: #106 (TASK-0026 enroll), #109 (TASK-0030 jti store + OI-SM-1 + CI timeout 20→30).
IN FLIGHT (auto-merge armed, both run under the 30m CI timeout):
  - **#111** (TASK-0031 PR-1 — remote relay-edge listener). guardian PASS, 4 gates green.
  - **#108** (TASK-0035 gRPC surface gaps). Rebased clean onto develop (twice); test passed at 27m.
Earlier merged: #102/#103/#104/#105/#107.

**Epic F is now serving remote clients end-to-end:** the #111 edge (`crates/secretd/src/edge/`, default-OFF
`relay-edge` feature, `POST /v1/relay/swap`) does TLS-terminate (relay-tls only) → RFC 9449 DPoP verify
(Ed25519/ring) → EKM bind (FS-S20) → F6 jti replay check → existing `relay_swap`/`decide()` (untouched).

## ⚠ FIRST on resume (baseline verify)
1. From an envctl worktree (NOT meta root — gh resolves the wrong repo there):
   `git -C envctl fetch origin develop && gh pr list --state open`.
2. Confirm **#108** and **#111** merged. If either is DIRTY (a sibling merged first and touched the same
   files), rebase it onto develop: resolve the `.handoff/loop/` cycle-artifact conflicts by taking the
   PR's own side; take develop's side for `loop_state.md`/`HANDOFF.md` (newest wins) and re-apply the PR's
   real backlog ticks; `cargo check -p envctl-secretd` to confirm; `git push --force-with-lease`. (This
   churn recurs because every PR touches `lib.rs` + `.handoff/` — expected.)

## NEXT (dep order — buildable)
1. **TASK-0032 (F5, P0)** — streaming-revocation tear-down (the architect's PR-3 for the edge): support
   long-lived HTTP/2 streams on the #111 edge + a periodic in-stream `decide()` re-check that actively
   tears down an in-flight stream on revoke/lock/USB-pull (FS-S5). Builds ON `crates/secretd/src/edge/`.
2. **TASK-0031-PR2 (F2 hardening, parallel)** — server-issued DPoP-Nonce challenge (OI-SM-1 nonce half) +
   per-IP/per-client rate-limit + body caps + timeouts + admission shedding (CVE-2024-47609) + opt-in
   hardened-mode mTLS `ClientCertVerifier` (OI-SM-4). Stacks on #111.
3. Then **TASK-0027** (early-revoke) → **TASK-0028** (GUI parity) → **TASK-0036** (mlockall) →
   **TASK-0037** (Phase-7 verify) → **TASK-0034** (hardening tail) → **TASK-0038** (Certs.* Phase-4+).
SKIP **TASK-0033** (VPS Profile B) — owner-gated `[!]`.

## decisions_and_dead_ends (don't re-litigate)
- The edge MUST terminate TLS in-process (no external TLS-terminating proxy) — EKM channel binding (FS-S20)
  is uncomputable behind a terminating front, and an uncomputable binding is fail-closed 403.
- `decide()` is the single Allow authority and re-asserts `dpop_verified` + the binding — the edge cannot
  forge an Allow by skipping a check. Keep policy in the engine, I/O in the edge.
- EKM accessor (confirmed): `tls_stream.get_ref().1.export_keying_material(out, label, None)` on
  tokio-rustls 0.26 → rustls 0.23. Verify against source before reusing, don't assume.
- CI `test` job timeout is now 30m (was 20m, was canceling green runs). If it creeps again, the suite is
  genuinely growing — don't just rerun; widen further or split the job (still << 6h hang-catch default).
- Stacked-branch pattern works: base a dependent feature on the dependency's branch, then rebase onto
  develop once the dependency merges (the duplicate commits drop as "already upstream").

## Invariants (carry forward — non-negotiable)
no-C trust boundary (reuse rustls-ring, no new dep) · fail-closed (reject on any uncertainty) · no secret
in logs/audit (Zeroizing, metadata-only) · engine the single sync non-printing authority (MINT/DECIDE in
the engine, I/O in front-ends) · relay-tls path only, never the MITM CA (FS-S25) · EKM bind (FS-S20).

## verify_on_resume (exact)
- `git -C envctl fetch origin develop && gh pr list --state open` (from a worktree)
- rebase #108/#111 if either still DIRTY (steps above); confirm both merged
- new worktree: `git -C envctl worktree add ../.worktrees/task-0032-stream/envctl -b task-0032-stream origin/develop`
