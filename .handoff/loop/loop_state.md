# Loop state — envctl agenticOS consolidation (Epics A–E)

# --- forge-loop ledger (schema fields the loop reads in Phase-0) ---
session_started: 2026-06-13
loop: agenticOS-consolidation (.handoff/loop/backlog.md, Epics A–E; design = .handoff/decisions/ADR-0001)
branch: develop   # work happens in FRESH worktrees off develop -> PR -> auto-promote to master
worktree: (per-cycle: meta/.worktrees/<slug>/envctl off develop)
cycle_budget: 1   # heavy-context resume sessions — 1 cohesive build cycle then hand off
wrap_every: 5   # batch boundary: run reaper + wrap-up + evolution-steward every N completed cycles (no per-task pause)
last_wrapup_total: 28   # HAND-OFF wrap-up satisfied 2026-06-23 after TASK-0039 merged / TASK-0053 claimed
cycles_this_session: 9   # RESUME SESSION 2026-06-22/23: cycles = TASK-0038, TASK-0007, TASK-0008, TASK-0015, TASK-0016, TASK-0017, TASK-0019, TASK-0021, TASK-0039
cycles_total: 28   # 27 previous cycles + TASK-0039 landed via PR #162
last_item: TASK-0039 (remote-clients CA lifecycle) — DONE via PR #162; TASK-0053 claimed for next cycle
status: HANDOFF 2026-06-23 — TASK-0039 merged/done; resume at TASK-0053
  Cycle = TASK-0039. PR #162 MERGED 2026-06-23T03:49:16Z with all GitHub checks green.
  Completed ledger sequence on full JSONL-derived redb cache: `hf test TASK-0039`,
  `hf done TASK-0039 --pr 162`, `hf sync-cards`, `hf handoff`, `hf export`. JSONL now has
  73 witnessed events. NEXT: TASK-0053 (GitHub transport doctrine) is claimed but not started.

  Cycle = TASK-0021. VERIFIED already satisfied 2026-06-23. The manifest already had the real
  `node-real` carve-out, `group-ai-clis` no longer required `node-via-bun`, and `envctl lock --check`
  passed. Focused engine tests confirmed `node_real_component_exists_with_empty_requires` and
  `group_ai_clis_does_not_require_node_via_bun`. No code change was needed; this cycle reconciled the
  stale backlog card with the truthful manifest state.

  Cycle = TASK-0019. VERIFIED already satisfied 2026-06-23. The backlog's referenced
  `_done/secretd-provisioning-runbook.md` premise was stale: `RealUsbProbe` now delegates to
  `seed_factor::keyfile_for` under `--features seed-factor`, returning a PARTUUID-bound Cognitum
  Seed Ed25519 signature as USB keyfile material over bounded pure-Rust HTTPS. `secretd` forwards
  USB enrollment through that same seam, injects `RealUsbProbe` into the live daemon engine seams,
  and `manifest/env-ctl.toml` builds/rebuilds `envctl-secretd --features seed-factor` for the
  installed daemon. No live Seed/network probe was run because TASK-0019 is `allows_network=false`.
  Verification: seed-factor unit tests, seed-factor `secretd` build, fake-probe USB keyslot unlock
  test, engine+CLI build, p7/no-c/shape/enable/kdf/agent-env/loop-state/harness-scripts gates.

  Cycle = TASK-0017. PR #151 MERGED 2026-06-23T00:28:20Z (merge commit
  a398e5ab5a37173ca6d50a5d096847e195638e77). Added local kasetto-style `extends`
  composition to envctl component manifests: parent manifests load before children, relative parent
  paths resolve from the child manifest directory, cycles and chains deeper than 8 fail closed, and
  `[[component]]` arrays merge by component id with same-id child tables overlaying inherited
  parent fields. Verification: fmt, engine+CLI build, focused manifest-extends tests, full
  envctl-engine tests, clippy, `envctl lock --check`, p7/loop-state/no-c/shape/enable/kdf/
  harness-scripts/agent-env gates, `hf test TASK-0017`, and all GitHub checks green on #151.
  The fresh ledger view was also repaired by re-witnessing already-merged TASK-0016 (#149), so
  `hf resume --json` now points at TASK-0019.

  Cycle = TASK-0016. PR #149 MERGED 2026-06-23T00:15:53Z (merge commit
  9e43b699d5bc38b646b2fef271b2a65c434eda21). Encoded the no-downgrade lock-boundary decision:
  standalone SHA-256 `agent-env.lock` remains separate from FNV-1a `manifest/envctl.lock`; the
  agent-env manifest now drives built-in `envctl agent` with `agent-env.yaml`; the CI gate uses the
  true zero-network `envctl agent lock --config agent-env.yaml --check --locked`. Verification:
  fmt, engine+CLI build, full envctl-agent-env tests, full envctl-engine tests, clippy for
  envctl-engine+agent-env, `envctl lock --check`, agent-env/p7/loop-state/no-c/shape/enable/kdf/
  harness-scripts gates, `hf test TASK-0016`, and all GitHub checks green on #149. `hf resume --json`
  now points at TASK-0017.

  Batch boundary satisfied 2026-06-22 at cycles_total=23 (wrap_every=5). Proposed-upgrades was already
  drained; reaper preview/apply found 0 live worktrees and 0 branches to reap, then meta worktree
  prune removed 17 orphaned worktree records. ICM context store: 01KVRWEB957ZHQP6QTNXRQFXRA. Next safe
  pick remains TASK-0016 per `hf resume --json`.

  Cycle = TASK-0015. PR #146 MERGED 2026-06-22T23:51:42Z. Added a live-pack regression that drives
  `Engine::agent_sync` against the real `agent-skills` MCP pack for both `claude-code` and `codex`,
  proving broker/repowire/weave are preserved while github/context7/exa/memory/playwright/
  sequential-thinking/n8n-mcp are added to `.mcp.json` and `.codex/config.toml`; existing broker
  secrets remain untouched. Also cleaned a pinned-toolchain clippy lint in the agent-env source test
  helper and ignored ephemeral `.handoff/locks/`. Verification: engine+CLI build, engine agent_sync
  and agent_sync_parity tests, full envctl-agent-env tests, clippy for envctl-engine+agent-env,
  fmt, p7, loop-state, no-c, agent-env gate, `hf test TASK-0015`, MSRV 1.88 spot check, and all
  GitHub checks green on #146. `hf resume --json` now points at TASK-0016.

  Cycle = TASK-0008. VERIFIED already satisfied 2026-06-22. `/home/drdave/Desktop/meta/meta_mcp`
  exists and is registered in `.meta.yaml`; `/home/drdave/Desktop/meta/meta-mcp` is absent;
  `~/.local/bin/meta-mcp` resolves to `/home/drdave/Desktop/meta/target/release/meta-mcp`.
  From `/home/drdave/Desktop/meta`, `cargo build -p meta-mcp` and `cargo test -p meta-mcp`
  passed with the Rust 1.88 toolchain on PATH. `hf test TASK-0008` passed after exporting that
  toolchain PATH so the witness commands could find cargo. `hf resume --json` now points at
  TASK-0015 as the next safe pick.

  Cycle = TASK-0007. PR #140 MERGED 2026-06-22T23:07:48Z. Implemented typed meta-boundary detection in
  EnvReport/doctor: known FlexNetOS/meta tools that resolve outside META_ROOT are reported as
  boundary_violation high-severity drift for meta-tool-links. The detector normalizes meta-managed
  worktree roots back to the parent meta workspace, so forge-loop worktrees do not false-positive on
  /home/drdave/Desktop/meta. The portability meta-tool-links component now detects/verifies symlinks
  against ${META_ROOT:-$HOME/Desktop/meta}, and the GUI drift label is exhaustive for the new variant.
  Live smoke found the intended current host drift: real ~/.cargo/bin/secretctl and secretd files outside
  META_ROOT. Verification: fmt, engine boundary tests, engine tests, build, clippy -D warnings, workspace
  tests (timeout 1200), p7/no-c/shape/enable/kdf/agent-env/loop-state/harness-scripts/cargo-audit gates,
  and auto-detect/doctor JSON smokes. `hf test TASK-0007` witnessed cargo build + p7 before `hf done`.

  Previous cycle = TASK-0038. PR #137 MERGED 2026-06-22T22:17:16Z. Implemented the non-MITM control-plane certificate
  surface: secrets-engine issues control_plane_server/control_plane_client leaves only, refuses mitm_leaf before
  key material, persists public DER metadata, lists CA+leaf certs, and audits ca_issued. secretd now wires
  Certs.CaInit/Issue/List; secretctl ca init/issue/list drives the daemon; e2e covers init -> issue -> list and
  mitm_leaf refusal. Certs.CaRotate/TrustApply remain explicit Unimplemented until destructive/root-
  of-trust semantics are designed; Certs.Renew/Revoke and the remote client CA lifecycle landed in
  TASK-0039 / PR #162. Verification: fmt, targeted
  secrets-engine/secretd/secretctl tests, engine+CLI build, p7/no-c/shape/enable/kdf/agent-env/loop-state gates,
  clippy -D warnings, workspace tests with low-cost KDF, metadata --locked, cargo-audit, MSRV 1.80 check, and CI
  all green on #137.
  Handoff repair in this branch: witnessed and marked stale PR-backed ledger items TASK-0034 (#135), TASK-0035
  (#108), TASK-0037 (#131), TASK-0038 (#137), and TASK-0007 (#140) done; then witnessed already-landed direct work TASK-0044
  (hf card minting) and TASK-0052 (harness_hub packaging) done. `hf handoff` now renders Done 40/53, next safe =
  TASK-0007. Added Codex prompt shims for /forge-loop and session-relay aliases because Codex slash commands were
  missing on reload. Normalized TASK-0031-PR2c -> TASK-0031-PR2C inside the task id so the hf schema accepts it.
  **NEXT PICK: TASK-0016** per `hf resume --json`. Open follow-ups: TASK-0031-PR2C (PROXY-protocol source IP),
  TASK-0039 (remote-clients-CA lifecycle), MADV_DONTDUMP companion to #112. SKIP TASK-0033 (VPS Profile B,
  owner-gated [!]).
  OPERATIONAL (not a forge cycle): a weave message requested `github-app enroll` to unblock the App's mint-github
  (404 / "App id not enrolled") — that is the TASK-0026 fail-closed guard working as designed, NOT a bug. Enroll
  needs the ORIGINAL app.pem (app-id 4044997); the vault copy is broker_only/un-revealable by design, so it cannot
  be sourced from envctl. Owner/operational action: `secretctl github-app enroll --apply --app-id 4044997
  --private-key <original-app.pem>`. DO NOT scan the box for the PEM (the sandbox correctly denies credential
  exploration). A multi-daemon "which secretd is canonical?" question was also raised on weave — held for the owner
  to confirm the authoritative socket/data-dir before any daemon switch.
  FIRST on resume: start from `hf resume --json`; current next safe is TASK-0016 unless this handoff PR changes it.
  Resume via `/forge-loop resume` or `/prompt:forge-loop resume`.
  [historical — session 7] HANDING OFF 2026-06-17 (session 7, 1 cycle done; budget reached; next is fresh-context early-revoke).
  Cycle = TASK-0031-PR2 (F2): hardened the relay edge against replay/abuse + added opt-in strong mTLS, all
  behind default-OFF relay-edge, ZERO new deps (ring promoted optional->unconditional in secrets-engine,
  already in the resolved graph -> no new lockfile crate). Engine (security policy, sync/non-printing, siblings
  to broker::jti): broker::nonce::NonceStore (server-issued DPoP nonce RFC 9449 §8-9; issue() via injected
  ring::rand::SecureRandom, single-use check_and_consume, bounded 16384/5min, full-after-sweep->Err fail-closed);
  broker::admission::AdmissionLimiter (per-key token bucket 120/min burst 60 MAX_KEYS 65536, new-key-vs-full->
  Throttled never grow); SecretEvent::EdgeRequestShed metadata-only. Edge (I/O only): admission is step 0
  BEFORE any crypto -> per-IP 429 (CVE-2024-47609; full verify+decide() still run on every non-shed req,
  per-client quota stays in decide() rate_per_min); DPoP-Nonce challenge inside verify -> 401 + DPoP-Nonce +
  WWW-Authenticate: DPoP error="use_dpop_nonce", retried proof must carry the nonce claim (parsed additively in
  dpop.rs, validated by caller next to jti); body caps + handshake/header/idle/body timeouts -> 413/408.
  Opt-in mTLS (OI-SM-4, default off): tls.rs load_from_dir_with_client_auth builds
  WebPkiClientVerifier::builder_with_provider(roots, ring) (ring-only, confirmed vs in-tree rustls 0.23.40)
  from an operator-provisioned remote-clients-CA PEM — separate input on the SAME relay-tls ServerConfig, never
  the MITM CA (FS-S25); EdgeConfig require_client_cert(default false)+client_ca_path, require w/o CA -> startup
  Err. Default keeps with_no_client_auth() byte-for-byte. Fail-closed (poisoned locks reject, no unwrap on req
  path), 7 nonce + 5 admission engine units + edge_hardening_e2e (challenge->retry 200, stale-nonce 401,
  rate-limit 429 asserted to shed before upstream, body 413, mTLS no-cert reject / valid accept), 4 gates green
  + relay-edge-OFF build unaffected, guardian PASS. PR #122 auto-merge armed.
  Session-7 also: confirmed #117 (TASK-0032) + #119 (reconcile) + #108 (TASK-0035) ALL MERGED to develop at
  resume; #120/#121 (manifest portability) also landed. The relay edge is now PR-1 (listener) + PR-2 (hardening)
  + PR-3 (stream tear-down) complete.
  **NEXT PICK: TASK-0027 (early-revoke) → TASK-0028 (GUI parity) → TASK-0037 (Phase-7 verify) → TASK-0034
  (hardening tail) → TASK-0038 (Certs.* Phase-4+).** New follow-ups filed: TASK-0031-PR2C (PROXY-protocol
  source IP for per-IP shed behind an L4 front), TASK-0039 (remote-clients-CA lifecycle: mint/≤7d-leaf/renew/
  revoke for the mTLS verifier). Small follow-up: MADV_DONTDUMP companion to #112 mlockall. SKIP TASK-0033
  (VPS Profile B, owner-gated [!]).
  FIRST on resume: confirm #122 merged; rebase if DIRTY (every secrets PR touches lib.rs + .handoff/).
  Resume via `/forge-loop resume`.

## Progress log
- cycle 1 (2026-06-13, TASK-0001, PASS-WITH-NOTES): built+installed `hf` from meta/handoff
  (`~/.local/bin/hf` → release symlink); `hf --help` runs; residency guard clean (shared ledger
  only, read-only). Dormant Stop/PreCompact hook now LIVE (resolves hf, runs from $META_ROOT,
  exit 0, no per-repo ledger). Witnessed-event WRITE is a no-op until a task is active → defers to
  TASK-0002 (correct dep). CARRIED FINDING: hf kernel links bundled C SQLite (rusqlite/
  libsqlite3-sys via the `ledger` crate) — not an envctl no-c violation (separate workspace) but
  flagged against Epic A's pure-Rust-kernel north star.

- cycle 2 (2026-06-13, TASK-0002 + TASK-0003, BLOCKED/NEEDS-DECISION): source-proved that the
  shipped `hf` is strictly CWD-relative (no `--ledger`/`HANDOFF_DIR`), so envctl's Tier-A
  text/packet layer cannot be hf-rendered against the shared meta ledger without creating a
  forbidden per-repo `ledger.db` (ADR-0004). `mint --from-kb` needs CWD=child-repo; `hf seed`
  writes the kernel's own HFTASK cards. Fix is a kernel feature in `meta/handoff` (out of envctl
  scope). Wrote `.handoff/decisions/FINDING-0002-...md` (3 options, A recommended). TASK-0003
  blocked with it (depends on a seeded layer). Epic A stalls pending the owner/kernel decision.

- cycle 3 (2026-06-13, TASK-0004, DONE — resume session): FIRST re-checked FINDING-0002 per owner
  "check now" → RESOLVED. The installed `hf` now exposes `fleet status`, `fleet render MEMBER`, and
  standalone `sync [--auto] [--dry-run]` (kernel meta/handoff PR #17, HEAD 1adbb13; binary rebuilt
  04:29). Verified live from $META_ROOT: `hf fleet status` (fleet ledger present, 64 members),
  `hf fleet render envctl` (wrote packets/latest.md — probe artifact removed), `hf sync --dry-run`.
  Marked TASK-0002/0003 UNBLOCKED. Then implemented TASK-0004: top-level `env` block
  (META_ROOT/META_FILE) in `home/.claude/settings.json.tmpl`, re-rendered `settings.json`, added the
  `settings_json_matches_rendered_tmpl_no_drift` Rust drift guard. Gate: build 395 crates,
  `cargo test -p envctl` 7 pass, no-c/shape/enable PASS. (Pre-existing, out-of-scope: clippy
  `items_after_test_module` on crates/cli/src/main.rs — present on develop, not gated by CI.)

- cycle 4 (2026-06-13, TASK-0002, DONE — resume session, stacked on #47): seeded envctl `.handoff`
  Tier-A as **git-text only** per ADR-0004 §7 (kernel-source verified that `hf init`/`hf seed` would
  plant a per-repo `ledger.db`/irrelevant HFTASK cards — avoided). Refreshed `context/capsule.json`
  next_command; seeded OPTIONAL `hooks/hooks.toml` + `policies/rules.toml` +
  `skills/session-resume.skill.md` from the design-bundle templates (with a `$META_ROOT`-residency
  header); **compiled** `packets/latest.md` via `hf fleet render envctl` (not hand-written); fixed
  `.handoff/README.md` (FLEET ledger = `meta/.handoff/ledger.db`; member packets via `hf fleet
  render`; active loop). Residency: 0 `*.db` under `.handoff`, `.gitignore` guard present, `hf fleet
  status` P7-clean for envctl. Gates: no-c/shape/enable PASS; drift test green. `tasks/` left empty
  (no kb task docs to `hf task mint --from-kb` yet) → tracked under TASK-0003.

- cycle 5 (2026-06-13, continuity merge-dup repair, DONE — owner "pick what's next; verify not
  claimed"): the concurrent three-way merge of #47 (TASK-0004) + #48 (a parallel session's
  FINDING-0002 unblock) + #49 (TASK-0002 seed) onto develop=6617ed9 **silently concatenated** the
  continuity files instead of conflicting: `loop_state.md` header TRIPLICATED, `backlog.md` had a
  duplicate TASK-0002 (`[x]` + stale `[ ]`) and TASK-0003 (two fragments), `FINDING-0002` had two
  `Status:` lines. Reconciled all three to a single coherent state (git-text only): one cycle-5
  header; one TASK-0002 `[x]` + one TASK-0003 `[ ]` (GO-LIVE + card-minting folded in); one
  FINDING-0002 RESOLVED status (preserved the `000e4c0`/FLEET_GUIDE detail). Verified-not-claimed
  first: 0 open PRs, 0 remote feature branches, grit `.grit/` empty, FLEET ledger 0 events.

- cycle 6 (2026-06-13, TASK-0003 p7-conformance gate, DONE — owner "Epic A, proceed"): added
  `ci/gates/p7.sh` — a fail-closed, dependency-free grep gate (mirrors `ci/gates/{shape,enable}.sh`)
  that validates the COMMITTED `.handoff/` Tier-A: schema tags (capsule v1 / policy v1 / hooks v1 /
  task v1 / packet **v2**) + ledger residency (no tracked OR on-disk `*.db` under `.handoff`, and the
  `.gitignore` guard present). Deliberately runs NO ledger-mutating `hf` verb in-member (would itself
  create a ledger). Wired into HANDOFF verify-on-resume + CLAUDE.md gate list. Verified: positive PASS
  on the seeded Tier-A; negatives (stray `*.db`, broken packet/capsule schema) fail closed (exit 1).
  Split the `hf sync` `.kb` GO-LIVE + envctl card-minting into new **TASK-0024** (need `$META_ROOT`
  execution / kb task docs). Verified-not-claimed: only unrelated PR #53 (libsql-baton-fix) open.

## Next safe step
- **TASK-0003 gate landed.** Next pick = **TASK-0024 (P2, Epic A)** — the `hf sync` `.kb` GO-LIVE
  (one-way write-back, run at `$META_ROOT`/orchestration home — NEVER in-member) + envctl card-minting
  once kb task docs exist. Smaller, but needs `$META_ROOT`-context execution.
- Alt: **Epic C TASK-0012 (P0)** — new pure-Rust crate `crates/agent-env` (6-key+extends model,
  multi-host resolver, SHA-256, lock; drop `mimalloc`; no-c clean). Large; gates TASK-0013..0018.
  Route `feature-architect` → `rust-implementer` → `invariant-guardian`. Benefits from fresh context.
- **Budget: 1/3 cycles this session.** Can take 2 more before HAND OFF.

## Order (dependency-aware; cards own ordering once TASK-0002 mints them)
Epic A: TASK-0001 (build hf) -> TASK-0002 (seed Tier-A + mint cards) -> TASK-0003 (p7 gate).
Epic C: TASK-0012 (crates/agent-env) gates TASK-0013..0018.
Epic B: TASK-0005 healed (settings tmpl on develop); TASK-0008 meta-mcp (proof) before others.
SUPERVISED (never auto-run): TASK-0010 was `- [!!]` (now DONE by a human session — see backlog).

## Gates (non-negotiable)
- never-downgrade (sync meta source UP first) · archive-first (never delete) · build+verify before
  swap · rollback on failure · ledger-residency ($META_ROOT only, no per-repo ledger.db) ·
  packets-rendered-never-hand-written · `- [!!]` items refuse auto-run -> NEEDS-HUMAN.

## needs_human / supervised
- Decision: bring GitKB into meta as a `.meta.yaml` project (git-kb currently external)?
- Old dashboard-forge-loop GUI smoke-test (loop/_done/, HUMAN-ONLY).
- REVIEW (Epic A): hf kernel links bundled C SQLite (rusqlite) + is CWD-relative (no `--ledger`
  override → can't render member Tier-A against the shared fleet ledger without a forbidden per-repo
  ledger.db). Both kernel-side in `meta/handoff`; out of envctl's no-c/p7 scope. **FILED for the
  kernel owner: FlexNetOS/handoff#71** (2026-06-18) — port `ledger` off rusqlite + add a ledger-path
  override. Tracks the cycle-1 CARRIED FINDING + FINDING-0002.

  --- EPIC G (forge-loop hardening, 2026-06-18 deep audit; direct-to-develop harness-maintenance,
  NOT forge cycles → cycles_total unchanged at 18). Provenance confirmed: forge-loop hand-authored in
  envctl (5dcc4b2/00237ca), not from a "forge" repo — it is the source pattern harness_hub abstracted.
  Planned as Epic G (TASK-0041..0052); owner LOCKED Tier-2/3 forks. SHIPPED (12/12 — EPIC G COMPLETE):
  Tier 1 — 0041 loop-state counter gate (+hermetic test, ci.yml), 0042 proposed-upgrades drain in
  wrap-up 3b, 0043 Phase-3.5 runtime-verify (guardian runs the app). Tier 2 — **0044 hf-kernel
  pick-time dependency authority (53 envctl cards minted into envctl/.handoff/tasks/; DONE 2026-06-18,
  handoff-kernel-engineer)**, 0045 in-flight - [~]
  re-poll sweep on resume, 0046 symbol-grain Unit ledger (architect+guardian), 0047 pre-DONE
  left-behind sweep. Tier 3 — 0048 A2 all-green merge barrier, 0049 cross-repo impact map before
  locks, 0050 bidirectional destination baseline, 0051 mutating-op branch coverage.
  TASK-0044 RESULT: minted via the kernel's OWN work-order crate (intent_lock byte-verified vs PHTASK-0001);
  cards-only, ZERO ledger writes (840 events unchanged). Contamination-free: fleet status AFTER = handoff 55 /
  prompt_hub 71 / weave 1 UNCHANGED, envctl 0→53. `hf fleet render envctl` surfaces only TASK-* (0 HFTASK leakage);
  DAG picker (sandbox-verified) returns dep-safe next + refuses blocked (0038→0035, 0052→0044). p7 PASS, doctor
  healthy, residency 0 *.db under member .handoff. **Kernel gap (finding, NOT a regression):** shipped hf is
  CWD-relative w/ no --ledger override (HFTASK-0054) → live `hf claim --next` member-scoped creates a forbidden
  per-repo ledger.db; clean read-only authority path TODAY = `hf fleet render envctl` (markdown subnote stays the
  live-pick fallback until HFTASK-0054 lands). **TASK-0052 DONE 2026-06-18 (harness_hub PR #38):** Feature
  Forge packaged into harness_hub as ejectable `/harness:feature-forge` — orchestrator + forge-loop +
  rust-feature-impl sub-skills + 4 prefixed agents (feature-forge-*), eject.sh/ralph runner/loop_state
  template, registry row + entries/feature-forge.md, plugin 1.11.0; `hub-validate` PASS (8 entries);
  envctl CLAUDE.md Placement doctrine reconciled (generic core now packaged upstream; env-install-loop/
  auto-provision/handoff-sync stay envctl-only). EPIC G COMPLETE (12/12). Commits: af67ad6 f3f13c5
  477376c 51ebcdf 465e096 e1c10e7 6159a99 c5740f5 e40d30d e29fc81 on develop + harness_hub f9ad297 (PR #38).

last_update: 2026-06-18
