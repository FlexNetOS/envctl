# HANDOFF — forge-loop (envctl secrets / Epic-F build) · 2026-06-18 (session 9)

closed_utc: 2026-06-18   branch: develop (work in FRESH worktrees off develop)
cycle_budget: 1   cycles_this_session: 1   cycles_total: 18
last_item: TASK-0028 (DONE, PR #126, guardian PASS-WITH-NOTES)   next_item: TASK-0037 (Phase-7 verify-don't-rebuild)
orchestrator_phase: handoff (cycle budget reached)   gate_status: PASS-WITH-NOTES   pr_url: https://github.com/FlexNetOS/envctl/pull/126
resume_command: /forge-loop resume   (reads this file + backlog "⏭ NEXT PICK")

## State (authoritative = Git/merged PRs; this file is a companion view)
MERGED to develop: #106 (TASK-0026 enroll), #105 (mint), #109 (TASK-0030 jti+OI-SM-1), #111 (TASK-0031 PR-1
  listener), #112 (TASK-0036 mlockall), #117 (TASK-0032 stream tear-down), #108 (TASK-0035 gRPC), #122
  (TASK-0031-PR2 edge hardening), **#124 (TASK-0027 early-revoke)**, plus infra #113/#114/#115/#116/#120/#121.
IN FLIGHT (auto-merge armed): **#126 (TASK-0028 — GUI parity)**. Guardian PASS-WITH-NOTES, 4 gates green,
  ZERO new GUI deps. A separate chore PR carries this session's reconcile/handoff (fast CI squash-lands the
  feature before a bundled reconcile commit — proven sessions 6/7/8 — so bookkeeping ships as its own PR).
RETIRED: **#125** (session-8 reconcile) — this session-9 reconcile is a SUPERSET that subsumes it (ticked
  TASK-0027 + TASK-0028, cycles_total 16→18); #125 was closed as superseded so the two reconciles don't race.
Earlier merged: #102/#103/#104/#107/#118/#119/#123.

**Epic F status:** the GitHub App mint path is now FULL-LIFECYCLE + GUI-surfaced — **enroll (#106) → mint
(#105) → early-revoke (#124) → GUI parity (#126)**. The remote relay edge is feature-complete (listener #111 +
hardening #122 + stream tear-down #117), mlock-hardened (#112), with a real gRPC surface (#108). What remains
in Epic F is the Phase-7 verify-don't-rebuild pass, the hardening tail, and Certs.* (Phase 4+).

## ⚠ FIRST on resume (baseline verify)
1. From an envctl worktree (NOT meta root): `git -C envctl fetch origin develop && gh pr list --state open`.
2. Confirm **#126** merged. If DIRTY (a sibling merged first touching `lib.rs`/`engine`/`.handoff`), rebase onto
   develop: resolve `.handoff/loop/cycle/*` by taking the PR's own side (`--theirs`); take develop's side for
   `loop_state.md`/`HANDOFF.md`/backlog NEXT-PICK (`--ours`) but KEEP the PR's per-item ticks;
   `cargo build -p envctl-engine -p envctl-gui`; `git push --force-with-lease`.

## NEXT (dep order)
1. **TASK-0037 — Phase-7 verify-don't-rebuild.** This is a VERIFY pass, not a fresh build: confirm the secrets
   verbs are folded onto the `envctl` umbrella (not just standalone `secretctl`/`secretd`), confirm an
   `install secretd` manifest component exists (or file it), and fix stale ROADMAP/doc lines that claim
   unbuilt state. Architect should lead with a gap inventory (what the docs claim vs what Git shows merged) so
   the cycle confirms/corrects rather than re-implements. Mind the no-fabricate rule: verify each claim against
   source/merged PRs, don't re-port what's already done.
2. Then **TASK-0034** (hardening tail: F10 tonic version-pin + cargo-audit in CI, F11 MSRV-1.80 check job,
   F18 audit-log fsync) → **TASK-0038** (Certs.* Phase-4+). Small follow-up: **MADV_DONTDUMP** companion to the
   merged #112 mlockall. Open: **TASK-0031-PR2c** (PROXY-protocol source IP), **TASK-0039** (remote-clients-CA
   lifecycle for the mTLS verifier).
SKIP **TASK-0033** (VPS Profile B) — owner-gated `[!]`.

## OPERATIONAL NOTE (not a forge cycle — for the owner / an operational session)
A weave message requested `secretctl github-app enroll` to unblock the App's `mint-github` (it currently 404s /
"App id not enrolled"). This is the **TASK-0026 fail-closed guard working as designed**, NOT a code bug. The
enroll needs the **ORIGINAL `app.pem`** (app-id 4044997) — the vault's copy is `broker_only` / un-revealable by
design, so it cannot be sourced from envctl. This is an **owner/operational action**, not a loop task:
`secretctl github-app enroll --apply --app-id 4044997 --private-key <original-app.pem>` then
`secretctl mint-github --installation-id 140063898 --output json`. **DO NOT scan the box for the PEM** (the
sandbox correctly denies credential exploration; both the implementer and guardian sub-agents this session
correctly held rather than hunting for it). A second weave question — "which secretd is canonical / what is the
authoritative socket+data-dir?" — is also **held for the owner**; do not switch daemons or add a socket override
without that confirmation. If a socket override is ever wanted it is a NEW task (teach the GUI/CLI seam to honor
`--socket`/`$ENVCTL_SECRETD_SOCKET`), not part of TASK-0028.

## decisions_and_dead_ends (don't re-litigate)
- **TASK-0028 architecture = B (subprocess `secretctl`), NOT an embedded gRPC client.** `envctl-gui` drives the
  env-manager `Engine` in-process and has zero secrets-stack path; the secrets verbs reach only via `secretd`
  gRPC, which `secretctl` (a thin async client) already drives. Embedding a tonic/tokio `VaultClient` in the GUI
  (Option A) would add a runtime + deps + re-implement secretctl's request builders = the exact CLI↔GUI
  divergence the invariants forbid. Option B (GUI builds argv → engine shells `secretctl` → parses `--json`)
  adds ZERO deps, keeps the GUI pure-sync, and cannot diverge. Rejected A; do not revisit unless the GUI needs
  streaming daemon events the CLI doesn't expose.
- The architect's first pass returned NEEDS-DECISION (correctly — it lacked the owner blanket-approval +
  CLAUDE.md parity-goal context). The orchestrator resolved all 4 questions in-scope (GUI→daemon seam approved;
  installation-token revoke is the required one; mirror the REAL JSON shapes; daemon-down = graceful no-false-
  success) and re-spawned for a GO. Lesson: the architect flags decisions; the orchestrator RESOLVES the
  in-scope ones it has context for.
- Under Architecture B the GUI does NOT compile-depend on `secretctl`, so TASK-0028 built off develop with no
  rebase-on-#124 needed (revoke argv is pure strings; parity tests use verbatim replication, no secretctl
  import). #124 happened to merge mid-cycle so the runtime dependency is now satisfied anyway.
- LOOP MECHANICS: land the per-cycle reconcile/handoff as a SEPARATE chore PR after the feature merges — the
  fast CI (low-cost-kdf #113) squash-lands the feature before a reconcile commit bundled into the same branch
  can land (orphaned in sessions 6/7; #125 from session 8 never merged → folded into this superset). The owner
  noted meta/handoff is addressing this kernel-side.
- Recurring rebase churn: every secrets/loop PR touches `lib.rs`/`engine`/`.handoff/` → siblings go DIRTY;
  resolve by taking the PR's code/cycle-artifacts (`--theirs`) and develop's loop_state/HANDOFF/backlog-narrative.

## Invariants (carry forward — non-negotiable)
no-C trust boundary (zero new GUI deps; reuse the subprocess/HttpTransport seams) · fail-closed / fail-safe
(dry-run default for mutating/outward ops; never panic on the request/spawn path; never a false success) ·
no secret in logs/audit/persistence (Zeroizing, metadata-only, eframe persistence off) · engine the single
sync non-printing authority (policy in engine, I/O in front-ends, CLI+GUI drive the identical surface) ·
relay-tls only never MITM CA (FS-S25) · EKM bind (FS-S20) · frozen contracts (mint-github) stay byte-stable;
new work is additive.

## verify_on_resume (exact)
- `git -C envctl fetch origin develop && gh pr list --state open` (from a worktree)
- rebase #126 if still DIRTY (steps above); confirm it merged
- new worktree: `git -C envctl worktree add ../.worktrees/task-0037-phase7-verify/envctl -b task-0037-phase7-verify origin/develop`
