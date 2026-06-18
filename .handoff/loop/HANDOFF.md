# HANDOFF — forge-loop (envctl secrets / Epic-F build) · 2026-06-17 (session 8)

closed_utc: 2026-06-17   branch: develop (work in FRESH worktrees off develop)
cycle_budget: 1   cycles_this_session: 1   cycles_total: 17
last_item: TASK-0027 (DONE, PR #124, guardian PASS)   next_item: TASK-0028 (GUI parity)
orchestrator_phase: handoff (cycle budget reached)   gate_status: PASS   pr_url: https://github.com/FlexNetOS/envctl/pull/124
resume_command: /forge-loop resume   (reads this file + backlog "⏭ NEXT PICK")

## State (authoritative = Git/merged PRs; this file is a companion view)
MERGED to develop: #106 (TASK-0026 enroll), #105 (mint), #109 (TASK-0030 jti+OI-SM-1), #111 (TASK-0031 PR-1
  listener), #112 (TASK-0036 mlockall), #117 (TASK-0032 stream tear-down), #108 (TASK-0035 gRPC), #122
  (TASK-0031-PR2 edge hardening), #123 (session-7 reconcile), plus infra #113/#114/#115/#116/#120/#121.
IN FLIGHT (auto-merge armed): **#124 (TASK-0027 — installation-token early-revoke)**. guardian PASS, 4 gates
  green, zero new deps. A separate chore PR carries this session's reconcile/handoff (the fast CI squash-lands
  the feature before a bundled reconcile commit — proven sessions 6/7 — so bookkeeping ships as its own PR).
Earlier merged: #102/#103/#104/#107/#118/#119.

**Epic F status:** the GitHub App mint path is now full-lifecycle — **enroll (#106) → mint (#105) →
early-revoke (#124)** — and the remote relay edge is feature-complete (listener #111 + hardening #122 +
stream tear-down #117), mlock-hardened (#112), with a real gRPC surface (#108). What remains in Epic F is
GUI parity, the Phase-7 verify-don't-rebuild pass, the hardening tail, and Certs.* (Phase 4+).

## ⚠ FIRST on resume (baseline verify)
1. From an envctl worktree (NOT meta root): `git -C envctl fetch origin develop && gh pr list --state open`.
2. Confirm **#124** merged. If DIRTY (a sibling merged first touching `lib.rs`/`.handoff`), rebase onto
   develop: resolve `.handoff/loop/cycle/*` by taking the PR's own side (`--theirs`); take develop's side for
   `loop_state.md`/`HANDOFF.md`/backlog NEXT-PICK (`--ours`) but KEEP the PR's per-item ticks;
   `cargo check -p envctl-secretd`; `git push --force-with-lease`.

## NEXT (dep order)
1. **TASK-0028 (G2) — GUI parity.** Surface relay-mint / mint-github / **revoke-token** in `envctl-gui`.
   The mint+revoke logic is entirely engine-side (`Engine::mint_github*`, `Engine::revoke_github_token`); the
   CLI (`secretctl`) drives it today. The GUI must drive the SAME Engine API (no logic in the GUI) so the two
   front-ends can't diverge — read how the GUI already calls the Engine for other verbs and mirror it. Mind
   the secret-handling: a minted token / a revoke token must never be rendered to a log or persisted by the
   GUI; show only metadata + a copy-once affordance.
2. Then **TASK-0037** (Phase-7 verify-don't-rebuild: confirm secrets verbs folded onto `envctl`, the
   `install secretd` manifest component exists, fix stale ROADMAP lines) → **TASK-0034** (hardening tail: F10
   tonic pin + cargo-audit CI, F11 MSRV check, F18 audit-fsync) → **TASK-0038** (Certs.* Phase-4+).
   Small follow-up: **MADV_DONTDUMP** companion to #112. Open: **TASK-0031-PR2c** (PROXY-protocol source IP),
   **TASK-0039** (remote-clients-CA lifecycle for the mTLS verifier).
SKIP **TASK-0033** (VPS Profile B) — owner-gated `[!]`.

## OPERATIONAL NOTE (not a forge cycle — for the owner / an operational session)
weave #126 requests `secretctl github-app enroll` to unblock the App's `mint-github` (it currently 404s /
"App id not enrolled"). This is the **TASK-0026 fail-closed guard working as designed**, NOT a code bug. The
enroll needs the **ORIGINAL `app.pem`** (app-id 4044997) — the vault's copy is `broker_only` / un-revealable
by design, so it cannot be sourced from envctl. This is an **owner/operational action**, not a loop task:
`secretctl github-app enroll --apply --app-id 4044997 --private-key <original-app.pem>` then
`secretctl mint-github --installation-id 140063898 --output json`. DO NOT scan the box for the PEM (the
sandbox correctly denies credential exploration). If the original PEM is lost, generate a fresh App private
key in GitHub settings and enroll that.

## decisions_and_dead_ends (don't re-litigate)
- GitHub `DELETE /installation/token` authenticates with the TOKEN ITSELF as bearer (not the App-JWT), 204 on
  success. The daemon does NOT persist minted tokens (by design) → the primary kill-switch is the explicit
  `revoke-token` verb (holder supplies the token); `relay_revoke` auto-revokes only the relay's last
  engine-minted NATIVE token (the one path where the engine still holds it in-process). Full token-tracking
  was rejected (worse at-rest posture, larger change) — it's TASK-0039-adjacent if the owner ever wants it.
- Revoke reuses the existing HttpTransport/DaemonHttpTransport seam → zero new deps; reads
  ENVCTL_GITHUB_API_BASE like mint (GHES parity). Fail-closed: non-204/transport ⇒ Err, never a false success;
  dry-run by default; token only in the Authorization header, never logged/audited/Debug-printed.
- LOOP MECHANICS: land the per-cycle reconcile/handoff as a SEPARATE chore PR after the feature merges — the
  fast CI (low-cost-kdf #113) squash-lands the feature before a reconcile commit bundled into the same branch
  can land (orphaned in sessions 6 and 7; the owner noted meta/handoff is addressing this kernel-side).
- Recurring rebase churn: every secrets PR touches `lib.rs` + `.handoff/` → siblings go DIRTY; resolve by
  taking the PR's code/cycle-artifacts and develop's loop_state/HANDOFF/backlog-narrative.

## Invariants (carry forward — non-negotiable)
no-C trust boundary (reuse rustls-ring / ring / libc-FFI / the HttpTransport seam, no banned dep) ·
fail-closed / fail-safe (dry-run default for mutating/outward ops; never panic on the request path; never a
false success) · no secret in logs/audit (Zeroizing, metadata-only) · engine the single sync non-printing
authority (policy in engine, I/O in front-ends, CLI+GUI drive the identical Engine API) · relay-tls only never
MITM CA (FS-S25) · EKM bind (FS-S20) · frozen contracts (mint-github) stay byte-stable; new work is additive.

## verify_on_resume (exact)
- `git -C envctl fetch origin develop && gh pr list --state open` (from a worktree)
- rebase #124 if still DIRTY (steps above); confirm it merged
- new worktree: `git -C envctl worktree add ../.worktrees/task-0028-gui-parity/envctl -b task-0028-gui-parity origin/develop`
