# HANDOFF — forge-loop (envctl secrets / Epic-F build) · 2026-06-17 (session 6)

closed_utc: 2026-06-17   branch: develop (work in FRESH worktrees off develop)
cycle_budget: 1   cycles_this_session: 1   cycles_total: 15
last_item: TASK-0032 (DONE, PR #117, guardian PASS)   next_item: TASK-0031-PR2 (F2 hardening)
orchestrator_phase: handoff (cycle budget reached)   gate_status: PASS   pr_url: https://github.com/FlexNetOS/envctl/pull/117
resume_command: /forge-loop resume   (reads this file + backlog "⏭ NEXT PICK")

## State (authoritative = Git/merged PRs; this file is a companion view)
MERGED to develop: #106 (TASK-0026 enroll), #109 (TASK-0030 jti + OI-SM-1), #111 (TASK-0031 PR-1 edge
  listener), **#112 (TASK-0036 mlockall FS-S4)**, plus infra **#113 (low-cost-kdf-tests — ~11x faster CI
  `test` job, production crypto unchanged)**, #114/#115 (Seed manifest reproducible USB-unlock).
IN FLIGHT (auto-merge armed, run under the CI gate):
  - **#117** (TASK-0032 — F5 streaming-revocation tear-down). guardian PASS, 4 gates green, zero new deps.
  - **#108** (TASK-0035 gRPC surface gaps). Rebased clean onto the new develop (DIRTY from #112/#113/#114/#115);
    build green; auto-merge armed.
Earlier merged: #102/#103/#104/#105/#107.

**Epic F status:** the remote edge serves clients end-to-end (TLS+DPoP/EKM+jti → relay_swap, #111), the
daemon is mlock-hardened (#112), AND long-lived streams now tear down on revoke/lock/USB-pull (#117). The
edge's PR-1 (listener) and PR-3 (stream tear-down) are done; PR-2 (hardening) is next.

## ⚠ FIRST on resume (baseline verify)
1. From an envctl worktree (NOT meta root): `git -C envctl fetch origin develop && gh pr list --state open`.
2. Confirm **#117** and **#108** merged. If either is DIRTY (a sibling merged first, touching the same
   `lib.rs`/`.handoff` lines), rebase onto develop: resolve `.handoff/loop/cycle/*` conflicts by taking the
   PR's own side (`--theirs`); take develop's side for `loop_state.md`/`HANDOFF.md`/backlog NEXT-PICK
   (`--ours`) but KEEP the PR's real per-item checkbox ticks; `cargo check -p envctl-secretd` (and
   `--features relay-edge`); `git push --force-with-lease`. This churn recurs because every secrets PR
   touches `lib.rs` + `.handoff/` — expected, not a problem.

## NEXT (dep order — the edge listener + tear-down are merged/in-flight)
1. **TASK-0031-PR2 (F2 hardening, P0)** — on the merged `crates/secretd/src/edge/`: server-issued
   **DPoP-Nonce** challenge (OI-SM-1 nonce half — the `jti` store is already in; add the nonce issuance +
   `DPoP-Nonce`/`use_dpop_nonce` 401 challenge loop) + **per-IP/per-client rate-limit** + **body caps** +
   **timeouts** + **pre-`decide()` admission shedding** (CVE-2024-47609 — shed before the expensive verify) +
   opt-in **hardened-mode mTLS `ClientCertVerifier`** (OI-SM-4). Keep it default-OFF behind `relay-edge`,
   zero new C deps, fail-closed.
2. Then **TASK-0027** (early-revoke) → **TASK-0028** (GUI parity) → **TASK-0037** (Phase-7 verify) →
   **TASK-0034** (hardening tail) → **TASK-0038** (Certs.* Phase-4+). Small follow-up: **MADV_DONTDUMP**
   companion to the merged #112 mlockall (named alongside it in THREAT-MODEL.md).
SKIP **TASK-0033** (VPS Profile B) — owner-gated `[!]`.

## decisions_and_dead_ends (don't re-litigate)
- The re-check must NOT reuse `relay_swap`/`relay_swap_prepare` — they fetch the key and `broker.bump()` the
  usage counters every call. `Broker::peek` (read-only) + `authorize_relay(bump=false)` + `bytes_out=0` give a
  budget/rate-enforcing re-check that consumes nothing and fetches no key. The swap path stays bump=true,
  byte-for-byte unchanged (proxy_swap_e2e/decide tests pass).
- `decide()` stays the single Allow authority — the re-check re-runs it with the open-time `RemotePeer`
  (clause 11a re-asserts dpop_verified/jkt); the edge never keeps a stream alive by its own judgment.
- Tear-down is fail-closed: every uncertainty (decide() Deny, locked vault, poisoned lock via `map_err`,
  store err, vanished bearer, absent USB gate, dropped engine handle, max-duration) → drop the downstream
  sender → clean HTTP/2 close. No `unwrap`/panic on the periodic hot path.
- Interval-poll (2s) chosen over watch-channel push for PR-3: decide()'s inputs (USB gate, bearer.revoked,
  policy) are re-read each call, so a 2s poll gives a ≤2s bound with zero new wiring. The sub-second
  `tokio::sync::watch` push needs an engine broadcast seam keyed by client/token → deferred to PR-4.
- CI `test` job is now ~11x faster via the **#113 low-cost-kdf-tests** feature (argon2 params dialed down for
  tests only; production crypto unchanged) — the 30m timeout fragility from sessions 2–5 is largely resolved.
- Loop discipline: pick the top *unblocked* item. TASK-0032 was unblocked once #111 merged; built it this
  session. TASK-0031-PR2 is next and is independent (hardening on the existing listener).
- Recurring rebase churn on the integration branch (every secrets PR touches `lib.rs` + `.handoff/`): resolve
  by taking the PR's code/cycle-artifacts and develop's loop_state/HANDOFF/backlog-narrative.

## Invariants (carry forward — non-negotiable)
no-C trust boundary (reuse rustls-ring / libc-FFI, no banned dep) · fail-closed / fail-safe (never panic on a
hardening/re-check path; strict modes refuse) · no secret in logs/audit (metadata-only) · engine the single
sync non-printing authority (policy in engine via decide(), I/O in front-ends) · relay-tls only never MITM CA
(FS-S25) · EKM bind (FS-S20) · default-OFF behind `relay-edge`.

## verify_on_resume (exact)
- `git -C envctl fetch origin develop && gh pr list --state open` (from a worktree)
- rebase #117/#108 if either still DIRTY (steps above); confirm both merged
- new worktree: `git -C envctl worktree add ../.worktrees/task-0031-pr2/envctl -b task-0031-pr2 origin/develop`
