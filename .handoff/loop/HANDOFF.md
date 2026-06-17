# HANDOFF — forge-loop (envctl secrets / Epic-F build) · 2026-06-17

closed_utc: 2026-06-17T17:50Z   branch: develop (work in fresh worktrees off develop)
cycle_budget: 1 (this session was very long — multiple full forge runs already spent)
cycles_this_session: 1   last_item: TASK-0026 (DONE, PR #106)   next_item: TASK-0035
resume_command: /forge-loop  (reads .handoff/loop/backlog.md — see "⏭ NEXT PICK")

## State (authoritative = Git/merged PRs; this file is a companion view)
Merged to develop this session: #102 (G2 native-mint primitive), #103 (G2 retro), #104 (backlog
reconcile + anti-drift wrap-up gate), #105 (TASK-0020-COMPLETE frozen `mint-github` surface).
Auto-merge armed, awaiting CI green: #106 (TASK-0026 `github-app enroll`).

**The downstream App is now unblocked end-to-end:** `secretctl github-app enroll` (once) →
`secretctl mint-github --installation-id 140063898 --output json` → `{"token","expires_at_unix"}`,
the exact frozen contract `flexnetos_github_app/crates/app-core/src/mint.rs` shells.

## NEXT (dep order — buildable)
1. **TASK-0035** — secretd gRPC surface gaps: `Vault.List`/`Rm`/`Rotate`, `Relay.Create`/`List`,
   `Audit.Query`, `GetSecret.meta` (all currently `Status::unimplemented`; engine lacks public read
   paths). Largest single non-edge item; can be split per-RPC.
2. **OI-SM-1 spec** (DPoP jti store) — WRITE FIRST; TASK-0030 is blocked on it.
3. **TASK-0030** (F6 jti replay store) → **TASK-0031** (F2 edge listener, NEW `secretd/src/edge`) →
   **TASK-0032** (F5 stream tear-down). Epic F / Phase 8 — multi-session.
4. **TASK-0027** (early-revoke) → **TASK-0028** (GUI parity) → **TASK-0036** (mlockall) →
   **TASK-0037** (Phase-7 verify-don't-rebuild).
SKIP **TASK-0033** (VPS Profile B) — owner-gated `[!]`, never auto-run.

## How to resume
- Fresh session: `/forge-loop` (reads this file + the backlog NEXT PICK; one architect→implementer→
  guardian cycle per item, commit + PR + auto-merge, reconcile the backlog via the wrap-up gate,
  hand off at budget).
- For TRULY unattended completion of Epic F: `/auto-provision` (fresh `claude -p` per cycle — the
  reliable "do not stop until done" mechanism given cron is session-only in this runtime).

## Invariants (carry forward — non-negotiable)
no-C trust boundary (reuse rustls-ring, no new dep) · fail-closed · no secret in logs/audit
(Zeroizing, metadata-only) · engine the single non-printing lib · build-to-the-frozen-contract check
before marking any task done (wrap-up step 3b).
