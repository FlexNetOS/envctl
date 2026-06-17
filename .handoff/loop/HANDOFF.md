# HANDOFF — forge-loop (envctl secrets / Epic-F build) · 2026-06-17 (session 3)

closed_utc: 2026-06-17   branch: develop (work in FRESH worktrees off develop)
cycle_budget: 1   cycles_this_session: 1   cycles_total: 12
last_item: TASK-0030 (DONE, PR #109, guardian PASS)   next_item: TASK-0031 (F2 edge listener)
orchestrator_phase: handoff (cycle budget reached)   gate_status: PASS   pr_url: https://github.com/FlexNetOS/envctl/pull/109
resume_command: /forge-loop resume   (reads this file + backlog "⏭ NEXT PICK")

## State (authoritative = Git/merged PRs; this file is a companion view)
landed_this_session:
  - PR #109 (task-0030-jti): F6 jti replay store + OI-SM-1 spec + CI timeout 20→30. guardian PASS, auto-merge armed.
MERGED this session: **#106** (TASK-0026 enroll — its earlier `test` fail was a flaky 20m CI timeout; passed on rerun).
IN FLIGHT (auto-merge armed): **#109** (TASK-0030, rebased onto develop, has the CI fix → runs under 30m).
OPEN, NEEDS REBASE: **#108** (TASK-0035 gRPC surface gaps, guardian PASS) — must rebase onto develop
  AFTER #109 merges: it conflicts with #106 (grpc.rs/conv.rs/proto/lib.rs) and needs the 30m test timeout.

Earlier merged: #102/#103/#104/#105 (G2 + retro + anti-drift gate + frozen mint-github), #107 (prior handoff).

## ⚠ FIRST on resume (baseline verify — do BEFORE picking TASK-0031)
1. `cd $META_ROOT && git -C envctl fetch origin develop` then `gh pr list --state open` (run from an
   envctl worktree, not the meta root — gh resolves the wrong repo at the meta root).
2. If **#109** still open: let auto-merge finish (test runs under the new 30m timeout). If red for a NON-
   timeout reason, investigate.
3. If **#108** still open: rebase it onto the now-current develop —
   `cd .worktrees/task-0035-grpc/envctl && git fetch origin develop && git rebase origin/develop`
   (resolve cycle-artifact + backlog conflicts by taking the TASK-0035 side; lib.rs/grpc.rs/conv.rs
   auto-merge — `cargo check -p envctl-secretd` to confirm), `git push --force-with-lease`,
   `gh pr merge 108 --auto --squash`. Then it lands on green.

## NEXT (dep order — buildable)
1. **TASK-0031 (F2, P0)** — in-process TLS-terminating HTTPS + DPoP/EKM relay-edge listener (NEW
   `crates/secretd/src/edge`). The only thing that actually serves remote clients. It CALLS the F6
   `JtiReplayStore` (TASK-0030, now `pub` in secrets-engine) right after proof verification, before
   `decide()`. rustls ServerConfig from the `relay-tls` path ONLY (never the MITM CA, FS-S25); RFC 9449
   DPoP verify; EKM channel binding (FS-S20). LARGE + security-critical → fresh-context cycle.
   Engine F3/F4/F12/F14/F15 foundation already built (`relay_mint_remote`, `register_remote_client`,
   `broker/decide.rs` remote DenyReasons, `broker/gate.rs` PresenceGate) — do NOT rebuild.
2. → **TASK-0032** (F5 stream tear-down) → **TASK-0027** (early-revoke) → **TASK-0028** (GUI parity)
   → **TASK-0036** (mlockall) → **TASK-0037** (Phase-7 verify) → **TASK-0034** (hardening tail)
   → **TASK-0038** (Certs.* Phase-4+).
SKIP **TASK-0033** (VPS Profile B) — owner-gated `[!]`, never auto-run.

## decisions_and_dead_ends (don't re-litigate)
- The `test` CI job was being CANCELED at `timeout-minutes:20` on GREEN runs — the workspace suite
  (cold build + memory-hard argon2 + daemon e2e; secrets-engine alone ~294s) crept to ~18-20m. Fixed
  to 30m in #109. NOT a hang and NOT a code bug — tests were still completing at the cancel instant.
- `hf resume` from $META_ROOT reports the kernel's HFTASK project, NOT the envctl forge backlog
  (TASK-00xx) — use the markdown backlog as the pick authority for this loop.
- TASK-0030 store is engine-side (`broker/jti.rs`) on purpose: it's a security-policy decision (like
  `decide`/`gate`), pure-unit-testable, one shared authority for the future edge. Don't move it to secretd.
- Integration-branch churn: PRs touching lib.rs/grpc.rs/proto/.handoff conflict on the integration
  branch as siblings merge; rebase-onto-develop per the FIRST-on-resume steps. Expected, not a problem.

## Invariants (carry forward — non-negotiable)
no-C trust boundary (reuse rustls-ring, no new dep) · fail-closed + dry-run for destructive ops · no
secret in logs/audit (Zeroizing, metadata-only) · engine the single sync non-printing lib ·
build-to-the-frozen-contract check before marking done (wrap-up step 3b).

## verify_on_resume (exact)
- `git -C envctl fetch origin develop && gh pr list --state open` (from a worktree)
- rebase #108 if still open (steps above); confirm #109 merged
- new worktree for TASK-0031: `git -C envctl worktree add ../.worktrees/task-0031-edge/envctl -b task-0031-edge origin/develop`
