# Loop backlog — envctl agenticOS consolidation (2026-06-12)

> Source: owner directive (2026-06-12) — handoff full-sync + meta-portability + **kasetto
> full-feature unification into envctl** (no downgrades, no feature lost, upgrade-only), plus the
> follow-up work surfaced when consolidating all WIP branches into `develop`. Design + research
> cross-references: `.handoff/decisions/ADR-0001-kasetto-handoff-portability-unification.md`.
>
> Workflow: `develop` is the integration branch; `master` is its protected mirror (auto-synced).
> Each item below is picked up in a FRESH worktree off `develop` (`git worktree add … -b <slug> develop`).
>
> Legend: `- [ ]` todo · `- [x]` done (MERGED-confirmed) · `- [~]` in-flight (guardian PASS +
> auto-merge armed, PR NOT yet merged — re-poll `gh pr view <N>` next session) · `- [!]` blocked
> (reason) · `- [?]` needs investigation · `- [!!]` SUPERVISED/CRITICAL (never auto-run).

## North star

envctl is the **agenticOS**: it owns the meta environment boundary (PATH, dotfiles, `~/.local`,
the canonical `home/` tree), holds + auto-injects secrets, provisions the agent environment, and
carries the handoff continuity kernel. Everything meta consumes resolves inside meta; user-global
holds ONLY symlinks into meta; configs reference meta via PATH (bare names) or `$META_ROOT` (from
the `.meta.yaml` marker) — never hardcoded paths. **HEAL not harm · NEVER delete (archive) ·
NEVER downgrade (sync meta source UP first) · pure-Rust, no C in the trust boundary.**

---

## ⚠ 2026-06-17 RECONCILE — read this FIRST (drift sweep)

A full completeness sweep (owner flag: *"during forge loop we drift and forget the remaining parts;
this repo is holding up the rest of the project"*) found the backlog badly out of sync with ground
truth AND missing the single biggest remaining blocker. Corrected here; details flow into the Epics.

### Status truth (verified vs merged PRs / live code — these are DONE, ignore stale `[ ]` below)
| Task | Backlog said | TRUTH | Evidence |
|------|-------------|-------|----------|
| TASK-0011 | `[ ]` | **DONE** | PR #97 MERGED (KASETTO-FEATURES v3.2.0) |
| TASK-0012..0014 | `[ ]`/IN-PROGRESS | **DONE** | PRs #71/#90/#91/#93/#94 MERGED; Epic C parity 102/0/13 |
| TASK-0006 / TASK-0018 | `[ ]` | **DONE** | PR #98 MERGED (retire ext kasetto + localize agent-env) |
| TASK-0023 | `[ ]` | **DONE** | `.github/workflows/sync-master.yml` present |

### ‼ TASK-0020 (github-app-mint) — PARTIAL, the FROZEN CONTRACT IS NOT MET (this is the holdup)
G2/PR #102 (`g2-native-mint` → develop) wired the native-mint **primitive** end-to-end (GitHubAppMint
install-on-unlock, DaemonHttpTransport, `resolve_injection`→`mint_scoped`) and exposed it as
`secretctl relay mint --mode native`. **But that is the WRONG surface.** TASK-0020 froze a downstream
contract that `flexnetos_github_app/crates/app-core/src/mint.rs` already shells:
`secretctl mint-github --installation-id <N> [--repository-ids …] [--permissions …] --ttl-secs <T>
--output json` → `{"token":"…","expires_at_unix":<i64>}`, backed by a new `rpc MintGithub(MintGithubReq)
returns (MintGithubResp)` on `service Vault`. **PR #102 added NONE of these** (no `mint-github`
subcommand, no `MintGithub` RPC, no `expires_at_unix`). **The App still 404s → the autonomous
reviewer chain (.github_org G1→G5) is STILL blocked.** Root cause of the drift: G2 was designed from a
weave message + source, the loop never surfaced TASK-0020's frozen contract. PR #102 is sound,
additive infra — let it merge — but it does **not** close TASK-0020.

### ✅ 2026-06-17 BUILD SESSION RESULTS (forge + forge-loop cycle 1)
- **TASK-0020-COMPLETE DONE** — frozen `mint-github` surface shipped: `MintGithub` RPC + `secretctl mint-github` → `{"token","expires_at_unix"}` (PR #105 MERGED). The App's frozen contract is now met.
- **TASK-0026 DONE** — `secretctl github-app enroll` + `Vault.SetGithubAppId` RPC (PR #106, auto-merge armed). Enroll→mint round-trip e2e green. The App can now mint end-to-end (enroll once, then mint-github).
- **Anti-drift wrap-up gate** live (PR #104 MERGED): backlog is now a written-back artifact.

### ✅ 2026-06-18 HARNESS-MAINTENANCE SESSION (forge-loop audit — direct-to-develop, not a forge cycle)
- **Workspace reaper** — `scripts/reap-worktrees.sh` + wired into resume/wrap-up; reaped the 46-worktree/85-branch/17-remote pileup; `.handoff` now fully git-tracked (ledger.db guard kept).
- **Forge-loop audit upgrades** U1 (TICK-ON-MERGED status gate), U3 (handoff-reconcile merge driver — no silent concat), U4 (frozen-contract pick-time check), U6 (auto-provision for unattended).
- **TASK-0040 DONE** — completed the kasetto integration: migrated `kasetto.yaml`/`.lock` → `agent-env.yaml`/`agent-env.lock` (absorbed-CLI default names) and wired the previously claimed-but-absent drift gate `ci/gates/agent-env.sh` (`envctl agent lock --check`) into CI. Closes the operational loose end after the v3.2.0 absorption (parity already 102/0/13).
- **Dependabot** GHSA-8m95-fffc-h4c5 (libsql-sqlite3-parser, low) dismissed as unreachable+unfixable (rationale in `crates/secrets-store-libsql/Cargo.toml`).


### ⏭ NEXT PICK (updated 2026-06-22): **TASK-0016** (agent-asset lock unification)
MERGED: TASK-0026 (#106), TASK-0030+OI-SM-1 (#109), **TASK-0031 PR-1 edge listener (#111)**, **TASK-0036 mlockall (#112)**, **TASK-0032 F5 stream tear-down (#117)**, **TASK-0035 gRPC gaps (#108)**, **TASK-0031-PR2 F2 edge hardening (#122)**, **TASK-0027 early-revoke (#124)**, **TASK-0037 Phase-7 verify-don't-rebuild (#131)**, plus infra #113 (low-cost-kdf-tests) / #114/#115/#116/#120/#121 (Seed + manifest portability + kasetto --help).
IN FLIGHT: None. Guardian report notes are informational only (socket pass-through intentional). Next, in dep order:
0. **TASK-0053** (P0, inserted 2026-06-23): route the verified meta GitHub transport doctrine into envctl.
   Source proof is `tasks/envctl-github-transport-truth` in the meta KB plus TASK-0053's card. Doctrine:
   SSH-backed `git` is repository truth; `gh`/GitHub API is workflow orchestration and must be read-back
   verified against git refs, PR state, and required checks; raw API/connector output is advisory. Envctl owns
   the scoped/broker-only GitHub App token path (`mint-github`, enroll/revoke, policy-drift token equivalent).
   Keep continuity wording current: handoff uses the redb-backed ledger plus deterministic JSONL export, not SQLite.
1. **TASK-0034** (hardening tail: F10 tonic pin + cargo-audit CI, F11 MSRV check, F18 audit-fsync) → **TASK-0038** (Certs.* Phase-4+).
   Small follow-up: **MADV_DONTDUMP** companion to the merged #112 mlockall.
   Open follow-ups: **TASK-0031-PR2C** (PROXY-protocol source IP for the per-IP shed behind an L4 front) +
   **TASK-0039** (remote-clients-CA lifecycle: mint/≤7d-leaf/renew/revoke for the mTLS verifier).
SKIP **TASK-0033** (VPS Profile B — owner-gated `[!]`). Resume with `/forge-loop`; for unattended completion use `/auto-provision`.
completion use `/auto-provision`. FIRST on resume: confirm #126 merged; rebase if DIRTY (every
secrets PR touches `lib.rs` + `.handoff/` so siblings recur DIRTY — expected, not a problem).
NOTE (session 9): this reconcile is a SUPERSET that subsumes the session-8 reconcile PR #125 (TASK-0027 tick) —
#125 retired as superseded; its bookkeeping is folded here.

### GitHub transport doctrine (TASK-0053, delivered 2026-06-23)

Verified meta GitHub transport/automation doctrine, routed into envctl as
`docs/secrets/GITHUB-TRANSPORT-DOCTRINE.md` (every cite read-back-verified from source):
- **SSH `git` is the repository source of truth** for the FlexNetOS fleet (`.meta.yaml` → SSH
  remotes; `git ls-remote --symref origin HEAD`). `gh config get git_protocol` reports `https`, so
  **`gh` is not the git transport truth.**
- **`gh`/GitHub API is workflow orchestration — advisory until read-back verified.** `gh` mutations
  can silently succeed; re-query against git refs / PR state (`gh pr view <PR> --json
  state,mergeStateStatus`) / required checks before trusting them. Always assert `--owner/--repo`
  (the `gh repo view` wrong-CWD hazard).
- **Envctl owns the broker-only, scoped GitHub App token path.** Frozen contract (TASK-0020) stays
  byte-stable: `secretctl mint-github … --output json` → `{token, expires_at_unix}` /
  `Vault.MintGithub`. Tokens broker-only/scoped/short-lived/never-logged; mutating App ops are
  `--apply`-gated dry-runs.
- **POLICY_DRIFT_TOKEN** is the **existing** `mint-github --permissions
  administration:write,metadata:read` path (live consumer
  `.github_org/scripts/rotate-policy-drift-token.sh:39,90-95`) — **no new envctl surface.** Pinned
  by the additive `policy_drift_permissions_scope_serializes` regression test (parse-level / fake
  transport; no real credential).
- **Merge-gate cross-check** (`../flexnetos_github_app/crates/app-core/src/merge_gate.rs`): the App
  posts a verdict as a REQUIRED check-run and arms native auto-merge only after green; never a bot
  APPROVE; `UnwiredMergeGate` fails closed. Agents hold no broad merge token, never native-APPROVE
  their own PRs, never force-merge red.
- Continuity wording stays current: handoff uses the **redb-backed ledger plus deterministic JSONL
  export**, not SQLite.

- [x] **TASK-0053 (P0) — DONE 2026-06-23 via PR #164 (MERGED 04:37:54Z):** doctrine note
  `docs/secrets/GITHUB-TRANSPORT-DOCTRINE.md` + README index entry + additive
  `policy_drift_permissions_scope_serializes` regression test in `crates/secretctl/src/main.rs`.
  Zero new Engine method / RPC / CLI flag / crate dep (the existing `mint-github` path already
  covers POLICY_DRIFT). Guardian PASS, runtime fail-closed verified.

<details><summary>TASK-0020-COMPLETE original spec (DONE — kept for reference)</summary>

Build the FROZEN contract on top of the G2 primitive (small; the mint engine is done):
1. proto: `rpc MintGithub(MintGithubReq) returns (MintGithubResp)` on `service Vault`;
   `MintGithubReq{ uint64 installation_id; repeated string repository_ids; repeated string permissions;
   int64 ttl_secs; }` · `MintGithubResp{ string token; int64 expires_at_unix; }`.
2. secretd handler: build `MintRequest{Github, repos, perms, ttl}`, call the installed provider's
   `mint_scoped` (already wired by #102), return `{token, expires_at_unix}`; witnessed event, NEVER log token.
3. secretctl `mint-github` subcommand: exact frozen flags; `--output json` prints `{"token","expires_at_unix"}` only.
4. Confirm App secrets sealed (TASK-0020 says already done): `github-app-private-key` (broker-only),
   `github-app-id`=4044997, `github-app-installation-id`=140063898 (org FlexNetOS). If absent → the
   `secretctl github-app enroll` verb (G2 follow-up) is a prerequisite.
Acceptance: `secretctl mint-github --installation-id 140063898 --output json` returns a real token;
`fxapp mint-token` (flexnetos_github_app) succeeds end-to-end. DO NOT change the flag/JSON shape.
</details>

### G2 follow-ups (were ONLY in PR #102 body — the drift the owner flagged; now tracked)
- [x] **TASK-0026 (G2) — DONE 2026-06-17 (PR #106):** `secretctl github-app enroll` + `Vault.SetGithubAppId`
  RPC seal the broker-only App PEM + `github-app-id` meta. Round-trip e2e (enroll→mint) green.
- [x] **TASK-0027 (G2) — DONE 2026-06-17 (PR #124, MERGED):** `DELETE /installation/token` early-revoke
  via the existing HttpTransport seam (zero new deps); `RevokeGithubToken` RPC + `secretctl github-app
  revoke-token` + best-effort `relay_revoke` tie-in for the relay's last engine-minted native token.
- [x] **TASK-0028 (G2) — DONE 2026-06-18 (PR #126, auto-merge armed):** GUI mint-github / relay-mint /
  revoke parity in `envctl-gui`. Architecture B — the GUI builds an argv and a new sync, non-printing
  `EngineCommand::Secrets` shells out to the installed `secretctl` binary (zero new GUI deps, no-c untouched;
  divergence structurally impossible — drives the identical clap surface). Metadata-only render, dry-run
  default, revoke token via stdin in `Zeroizing`, eframe persistence stays off. Guardian PASS-WITH-NOTES.

### Home-tree portability (were ONLY in ICM — now tracked)
- [ ] **TASK-0029:** `portability-links.toml` branch fork — `usrlocal-script-links` present on master,
  absent on develop; `home/` tree hash diverges. Reconcile so promote can't silently drop a component.
  (GAP1 `~/.claude/settings.json` real-file drift = FIXED 2026-06-17; GAP2 gitconfig leak = FIXED.)

### Promoted from the namespaced rust-port loop (was invisible to this flat backlog)
- [!] **KBTASK-SEED-UNLOCK:** Seed-USB live-hardware unlock — code-complete, OWNER-GATED (live-hardware
  test only). From `.handoff/loop/rust-port/HANDOFF.md`.

## Epic F — Secrets SERVER-MODE / Phase 8 remote edge (THE missing blocker cluster, was UNTRACKED)
The completeness sweep found the largest genuine remaining work — *"serve remote clients"* — was **not
in this backlog at all**. `crates/secretd/src/edge` does not exist. Engine-side F3/F4/F12/F14/F15
foundation IS built (`relay_mint_remote`, `register_remote_client`, `broker/decide.rs` remote DenyReasons,
`broker/gate.rs` PresenceGate) — do NOT rebuild. Sequence: spec spike → F6 → F2 → F5. Source:
`docs/secrets/SERVER-MODE.md`, `docs/secrets/audits/AUDIT-server-mode.md`.
- [x] **TASK-0030 (F6, P0) — DONE (PR #109, guardian PASS):** Bounded DPoP `jti` replay store
  (`crates/secrets-engine/src/broker/jti.rs`, `JtiReplayStore::check_and_record`). OI-SM-1 spec
  written first (`docs/secrets/OI-SM-1-jti-replay-store.md`). Fail-closed cap (16384, no live-eviction
  hole), in-memory, zero new deps. F2 edge listener that calls it = TASK-0031 (next). PR #109 also
  carries a CI fix: test-job `timeout-minutes` 20→30 (the workspace suite grew past the 20m wall and
  green runs were being canceled — surfaced by #106/#108).
- [x] **TASK-0031 (F2, P0) — PR-1 DONE (PR #111, guardian PASS):** In-process TLS-terminating
  HTTPS+DPoP/EKM relay-edge listener (`crates/secretd/src/edge/{mod,dpop,tls,listener}.rs`, default-OFF
  `relay-edge` feature, route `POST /v1/relay/swap`). rustls ServerConfig from `relay-tls` ONLY (FS-S25,
  structural + shape.sh grep); RFC 9449 DPoP verify (Ed25519/ring); EKM binding (FS-S20, accessor confirmed
  vs source); F6 jti check; drives EXISTING `relay_swap`/`decide()` (untouched). Engine seam additive only
  (`EgressReq.remote` + `relay_swap_prepare` + `load_remote_client` + `Paths::relay_tls_dir()`). Fail-closed,
  zero new deps, 4 gates green. **PR-2 / PR-3 deferred (see TASK-0031-PR2 / TASK-0032 below).**
  - [x] **TASK-0031-PR2 (F2 hardening) — DONE (PR #122, guardian PASS):** `broker::nonce::NonceStore`
    (server-issued DPoP-Nonce, RFC 9449 §8-9, single-use, ring::rand) + `broker::admission::AdmissionLimiter`
    (per-IP token bucket, shed BEFORE crypto, CVE-2024-47609) + body caps/timeouts (413/408) + opt-in mTLS
    `WebPkiClientVerifier` (OI-SM-4, ring-only, default-OFF, separate client-CA never the MITM CA). Zero new
    deps; `SecretEvent::EdgeRequestShed` metadata-only; fail-closed; 7 nonce + 5 admission units +
    edge_hardening_e2e. Edge now PR-1+PR-2+PR-3 complete.
    - [x] **TASK-0031-PR2C:** parse the PROXY-protocol header to key the per-IP shed on the real client IP
      when the edge sits behind an L4 front. DONE 2026-06-23, PR #157: relay-edge now trusts the
      PROXY-protocol source IP before admission/rate limiting; `hf test TASK-0031-PR2C` witnessed
      build+p7 before `hf done`.
    - [ ] **TASK-0039 (remote-clients-CA lifecycle):** mint/≤7d-leaf/renew/revoke + revocation-set
      propagation for the mTLS client-CA. PARTIAL 2026-06-23, PR #158: hardened the verifier-side
      revocation propagation path (reload revocation set on each connection, reject revoked client
      leaves, compatible with PR #157 PROXY source-IP). Remaining: implement the lifecycle surface
      (`mint`/`<=7d leaf`/`renew`/`revoke`) instead of relying on operator-provisioned files.
- [x] **TASK-0032 (F5, P0) — DONE (PR #117 MERGED, guardian PASS):** Streaming-revocation
  tear-down. Engine `relay_stream_authorized` + `Broker::peek` (non-mutating re-check through the SAME
  `decide()`); edge `stream.rs` supervises long-lived HTTP/2 streams with a 2s in-stream re-check + max-stream
  deadline and tears the stream down (drops the `StreamBody` sender → clean HTTP/2 close) on revoke/lock/
  USB-pull (FS-S5). Fail-closed, metadata-only `RelayStreamTornDown` audit, zero new deps, default-OFF
  `relay-edge`. Detection ≤2s; sub-second watch-push deferred to PR-4.
- [!] **TASK-0033 (VPS Profile B, BLOCKED — gated non-shippable):** F7 install-time fail-closed gate +
  F8/OI-SM-2 operator-authorizer protocol + OI-SM-3 external trusted-time. Keep gated until designed.
- [x] **TASK-0034 (hardening tail — DONE, PR #135 MERGED, guardian PASS):** F10 (CVE-2024-47609
  tonic/hyper floors + cargo-audit CI), F11/OQ-1 (MSRV 1.80 `cargo +1.80 check --workspace --locked`
  CI), F18 (group-commit audit-fsync spec: N=100/T=100ms, barrier-before-response, whole-batch deny
  on barrier failure), F13/F17/F19/F23 defense-in-depth docs reconciled.

### secretd gRPC surface gaps (Phase-6 honest Unimplemented seams — engine lacks public read paths)
- [x] **TASK-0035 (in review — branch `task-0035-grpc`):** Vault `List`/`Rm`/`Rotate`,
  `Relay.Create`/`List`, `Audit.Query`, and `GetSecret.meta` are now WIRED engine-first (zero new deps,
  no proto change). Added engine `secret_list`/`SecretListItem`, `secret_meta`, `secret_rm`,
  `secret_rotate`, `relay_list`, `relay_create`, `audit_query` (+ `Store::delete_secret` default-body
  trait method, real `InMemStore`/libSQL impls); conv `secret_meta_to_proto`/`secret_list_item_to_proto`/
  `policy_to_proto`/`method_str`; replaced the 6 grpc `Unimplemented` bodies + Get-meta. Destructive
  verbs (Rm/Rotate) fail-closed + dry-run by default; reads gate on unlock. Tests: engine inline +
  conv inline + `secretd/tests/grpc_surface_e2e.rs`. The Certs.* / non-mitm ca_issue / secretctl ca /
  empty-features carve-out moved to TASK-0038.
- [x] **TASK-0038 (deferred from TASK-0035 — Phase 4+):** secretd `Certs.CaInit/Issue/List`
  + non-mitm `ca_issue` + `secretctl ca init/issue/list` landed in PR #137 (MERGED, guardian PASS).
  The general operator path mints only `control_plane_server` / `control_plane_client` leaves and
  refuses `mitm_leaf`; Certs.CaRotate/TrustApply remain explicit Unimplemented until
  destructive/root-of-trust semantics are designed. Certs.Renew/Revoke and the remote client CA
  lifecycle landed in TASK-0039 / PR #162.
- [x] **TASK-0036 — DONE (PR #112, guardian PASS):** secretd in-process `mlockall(MCL_CURRENT|MCL_FUTURE)`
  in `harden_process()` via libc (pure-Rust FFI, zero new lockfile crates), best-effort + `require_mlock`
  strict opt-in (fail-closed). Linux-cfg-gated, never panics, metadata-only WARN.
  - [ ] follow-up: `MADV_DONTDUMP` companion (named alongside mlockall in THREAT-MODEL.md) — not widened here.
- [x] **TASK-0037 (Phase-7 verify-don't-rebuild):** confirm secrets verbs are folded onto the `envctl`
  binary (today on `secretctl`) + an `envctl install secretd` manifest component exists. Update stale
  `docs/ROADMAP.md` lines 108-109/128 (contradict code).

## Epic A — Handoff continuity full-sync (bring `.handoff` to Tier-A)

Research: `meta/handoff` kernel vs `envctl/.handoff` (~30% Tier-B stub). Current continuity truth is
the committed `.handoff/ledger.events.jsonl` export; `.handoff/ledger.db` is an ignored local redb
cache rebuilt from JSONL (ADR-0018 D1). Packets are **rendered by `hf`, never hand-written**.

- [x] **TASK-0001 (P0):** Build & install the `hf` kernel binary from `meta/handoff` (not on PATH
  today — keystone blocker). Relocate per Epic B procedure (symlink into meta). Verify
  `hf resume/claim/checkpoint/handoff` run from envctl against `meta/.handoff/ledger.db`.
  - DONE 2026-06-13 (forge-loop cycle 1, `handoff-kernel-engineer` agent + `handoff-sync` Step 1).
    Built `cargo build --release -p hf` (3.6 MB ELF); installed `~/.local/bin/hf` → SYMLINK into
    `meta/handoff/target/release/hf` (meta convention; rebuilds propagate). `which hf` resolves the
    meta symlink; `hf --help` runs (verbs: init|seed|status|session|claim|release|checkpoint|
    sync-cards|done|task mint|ship|review|handoff|resume — no `hf drift`/`hf policy`). Residency
    guard PASSES before+after: no per-repo `ledger.db` under any envctl tree; `hf status` from
    `$META_ROOT` reads the shared `meta/.handoff/ledger.db` read-only (md5 unchanged).
  - GO-LIVE for the wired-but-DORMANT continuity hook: `.claude/settings.json` +
    `.claude/hooks/hf-checkpoint.sh` are already wired (Stop + PreCompact, fleet-ledger-resident,
    self-resolves `$META_ROOT`) but no-op until `hf` exists + supports `checkpoint --auto --quiet`.
    Acceptance: after `hf` lands, a Stop fires `hf checkpoint --auto` writing a witnessed event to
    `$META_ROOT/.handoff/ledger.db` (NOT a per-repo ledger), proving "auto-update .handoff after
    every task" (the `/verify` 2026-06-13 finding — currently FALSE, this makes it TRUE).
    - HOOK NOW LIVE: fired the Stop hook with `CLAUDE_PROJECT_DIR`=envctl worktree → exit 0,
      resolves `hf` via PATH, runs `hf checkpoint --auto --quiet` from `$META_ROOT`, creates NO
      per-repo ledger. The witnessed-event WRITE is correctly a no-op today (`hf checkpoint --auto`
      → "no task id … `--auto` with an active task"; 0 cards seeded). **End-to-end witnessed-event
      proof therefore defers to TASK-0002** (which seeds + mints + claims a task) — correct
      dependency ordering, not a regression. Hook go-live (resolve+run+residency-safe) = DONE here.
  - NOTE (carried → Open findings / Epic A): the `hf` binary's `ledger` crate pulls
    **`rusqlite`/`libsqlite3-sys` (bundled C SQLite, statically linked)**. Does NOT violate
    envctl's `no-c.sh` (separate `meta/handoff` workspace, not an envctl crate), but is relevant to
    Epic A's "pure-Rust, no C in the trust boundary" north star if the kernel itself must be C-free.
- [x] **TASK-0002 (P0):** Seed envctl `.handoff` via `hf` — render `policy.toml`, `hooks/hooks.toml`,
  `policies/rules.toml`, `active.md`, `packets/latest.md`, `skills/`. Do NOT create a per-repo
  `ledger.db`; do NOT hand-write packets.
  - **DONE 2026-06-13 (resume cycle 4).** Per ADR-0004 §7 the Tier-A layer is authored as git-text
    (NOT via `hf init`/`hf seed` — both would plant a per-repo `ledger.db` / irrelevant HFTASK cards;
    kernel-source verified). Landed: refreshed `context/capsule.json` `next_command`; seeded the
    OPTIONAL autonomous-loop descriptors `hooks/hooks.toml` + `policies/rules.toml` +
    `skills/session-resume.skill.md` from the design-bundle templates (residency-safe text, with a
    `$META_ROOT`-residency header so ledger-mutating verbs never run in-member); **compiled**
    `packets/latest.md` via `hf fleet render envctl` (rendered, not hand-written); corrected
    `.handoff/README.md` (FLEET ledger = `meta/.handoff/ledger.db`, member packets via `hf fleet
    render`, active loop). Residency verified: 0 `*.db` under `.handoff`, `.gitignore`
    `.handoff/**/ledger.db` guard present, `hf fleet status` shows envctl with **no `⚠ stray
    ledger.db (P7)`**. Gates: no-c/shape/enable PASS; drift test green. `tasks/` stays empty — cards
    are minted via `hf task mint --from-kb` once kb task docs exist for envctl (packet degrades to
    "no open cards"); that + the `hf sync` `.kb` GO-LIVE (run at `$META_ROOT`, never in-member) are
    follow-ups, tracked under TASK-0003.
  - **BLOCKED 2026-06-13 (cycle 2, REVISED): installed `hf` is the S1 spike, missing the fleet
    verbs → NEEDS-DECISION.** The design is SETTLED (ADR-0004 §2/§3/§4 + PRD v2): per-repo
    `.handoff/` is **text-only, no `ledger.db`**; events live in the **fleet** ledger
    (`meta/.handoff/ledger.db` — cycle-1's target was correct; `meta/handoff/.handoff/ledger.db` is
    the separate KERNEL ledger w/ 23 HFTASK cards); per-repo packets/cards are joined centrally via
    **`hf fleet status`**. The blocker: the installed binary (S1 spike) lacks **`fleet`/`policy`/
    `drift`/`sync`** (only `sync-cards`), which ADR-0001 §22 documents and ADR-0004 §76 cards as
    "to implement" (HFTASK-0007 `session`+`policy.toml`, HFTASK-0011 `hf sync` `.kb` mirror, plus
    `hf fleet status` + fleet-aware packet render). Fix = **build those verbs in `meta/handoff`**
    (kernel scope), then re-run TASK-0002. NOTE: envctl's REQUIRED Tier-A text core
    (`context/capsule.json`+`README`+`tasks/`+`packets/`) already exists; only OPTIONAL
    `hooks/policies/skills` (residency-safe, no kernel dep) + the rendered/minted/synced parts
    remain. v1's "add a `--ledger`/`HANDOFF_DIR` flag" is RETRACTED. Full analysis + 3 options
    (A: build kernel fleet verbs [recommended]; B: seed the text subset now, defer the rest;
    C: rescope to required-text-core + central `hf fleet` render) →
    `.handoff/decisions/FINDING-0002-hf-ledger-residency-vs-repo-tier-a.md`.
  - **UNBLOCKED 2026-06-13 (resume, owner "check now"): FINDING-0002 RESOLVED via Option A.** The
    kernel built the fleet verbs — `meta/handoff` PR **#17** (`feat: fleet verbs hf fleet
    status/render, hf sync`); installed `hf` rebuilt 2026-06-13 04:29. Verified live from `$META_ROOT`:
    `hf fleet status` (fleet ledger present, 64 members enumerated), `hf fleet render envctl` (wrote
    `packets/latest.md`), `hf sync --dry-run` (one-way `.kb` mirror). TASK-0002 is now executable as
    written. Next Epic A cycle: seed the OPTIONAL `hooks/policies/skills` text + run
    `hf fleet render envctl` / `hf sync` properly inside a worktree cycle and commit the artifacts.
- [x] **TASK-0003 (P1) — DONE 2026-06-13 (cycle 6): `p7-conformance` gate landed.** Added
  `ci/gates/p7.sh`, a fail-closed grep-based bash gate mirroring `ci/gates/{shape,enable}.sh`
  (dependency-free; validates the COMMITTED Tier-A, never runs a ledger-mutating `hf` verb in-member).
  - **Schema validation ✓:** asserts the `schema` tag on `context/capsule.json`
    (`handoff.context_capsule.v1`), `policies/rules.toml` (`handoff.policy.rules.v1`),
    `hooks/hooks.toml` (`handoff.hooks.v1`), every `tasks/*.task.json` (`handoff.task.v1`), and that
    `packets/latest.md` (the `hf fleet render` artifact) is a `handoff.packet.v2`.
  - **Residency invariant ✓:** asserts **no per-repo `ledger.db`** is git-tracked OR present on disk
    under `.handoff`, and that `.gitignore` carries the `.handoff/**/ledger.db` guard. Fail-closed.
  - **Wired** into the loop verify-on-resume (`.handoff/loop/HANDOFF.md`) + `CLAUDE.md` gate list.
    Verified: positive PASS on the seeded Tier-A; negative tests (stray `*.db`, bad packet/capsule
    schema) all fail closed (exit 1). GO-LIVE + card-minting split to **TASK-0024**.
- [x] **TASK-0024 (P2, Epic A) — `hf sync` `.kb` GO-LIVE DONE 2026-06-13 (cycle 8); card-minting
  conditional-deferred** (split from TASK-0003):
  - **GO-LIVE ✓:** wired `hf sync --auto` into the Stop/PreCompact hook
    (`.claude/hooks/hf-checkpoint.sh`) right after `hf checkpoint --auto`, run at `$META_ROOT`
    (same residency — never a per-repo ledger), fail-soft. So every checkpoint now ALSO one-way
    mirrors the witnessed FLEET ledger → GitKB (ADR-0003 HFTASK-0011). Verified live from `$META_ROOT`:
    `hf sync --auto` → "mirrored context/overridable/{active,progress} (one-way ledger→kb)", exit 0
    (FLEET ledger now has 10 witnessed events). Refreshed the hook's stale "DORMANT" header → LIVE.
    The `/verify` finding's "auto-sync to .handoff and .kb" is now **TRUE**. (Broken `.kb` SessionStart
    hook was already FIXED upstream: `meta/.claude/settings.json` `git kb service`→`git kb serve`.)
  - **Card-minting (conditional-deferred, no actionable prereq):** `envctl/.handoff/tasks/` is empty
    and there are **no envctl `.kb` task docs** to `hf task mint --from-kb` (verified). When kb task
    docs are authored for the envctl backlog, mint them (packet degrades to "no open cards" until
    then — residency-correct). Ref `meta/handoff/FLEET_GUIDE.md`; use the installed `hf`.

## Epic B — Meta-portability / env-ownership (`$META_ROOT`)

`~/.local/bin` must hold ONLY symlinks into meta. Per-tool relocation procedure: (1) confirm
provenance, (2) build meta source `--release`, (3) **if meta < installed → UPGRADE meta source
FIRST** (never relocate to older), (4) smoke-test, (5) archive installed copy (timestamped, never
delete), (6) symlink `~/.local/bin/<tool>`→meta build, (7) re-verify + ROLLBACK on failure, (8)
verify env health.

- [x] `envctl env` — discover meta-root via `.meta.yaml` marker (`engine::dashboard::locate_meta_file`),
  emit `export META_ROOT=…` + meta tool dirs on PATH; `--toolchains`/`--materialize` (merged from
  feat/envctl-env, 2026-06-12).
- [x] **TASK-0004 (P0):** Wire `META_ROOT` into the env Claude inherits (login/session env envctl owns).
  - DONE 2026-06-13 (resume cycle): added a top-level `"env": { "META_ROOT", "META_FILE" }` block to
    `home/.claude/settings.json.tmpl` (rendered per-machine to absolute paths by the existing
    `claude-global-links` `sed` render — the same path TASK-0005 uses); re-rendered the committed
    `settings.json`. Claude Code applies settings `env` to every session, so every repo+meta session
    now inherits `META_ROOT`/`META_FILE` with no hardcoding. Added a Rust drift-guard test
    (`settings_json_matches_rendered_tmpl_no_drift`) asserting `settings.json == render(tmpl, root)`
    + the env-block wiring (host-independent via the statusline anchor) — a guard that did not exist
    before. Gate green: build (395 crates), `cargo test -p envctl` 7 pass, no-c/shape/enable PASS.
- [x] **TASK-0005 (P1):** Heal the 3 hardcoded `home/.claude/settings.json` refs via `$META_ROOT`/
  per-machine templating: statusline script + 2 plugin-marketplace dirs (HIGH — live source-of-truth file).
  - DONE 2026-06-13: `home/.claude/settings.json.tmpl` + `claude-global-links` per-machine render
    (byte-identical, non-breaking). PR **envctl#37 MERGED → develop** (`bf29acd`). (Git>backlog: confirmed merged.)
- [ ] **TASK-0006 (P2):** Point global `home/.config/kasetto/kasetto.yaml` mcps source at in-meta
  agent-skills (not `github.com/FlexNetOS/agent-skills`); genericize MED shell/nushell hardcodes
  (`shell_nu.nu`, `shell_bash.sh`, `config.nu`). Fix stale `Documentation=` URL in `manifest/env-ctl.toml`.
- [x] **TASK-0007 (P2):** `envctl doctor`/env boundary-refusal when a real FlexNetOS install is found
  outside meta; idempotent `~/.local/bin` symlink regen from `META_ROOT`. DONE in PR #140
  (merged 2026-06-22): typed `meta_boundary` report on `EnvReport`, high-severity
  `boundary_violation` drift for `meta-tool-links`, GUI drift label parity, and META_ROOT-aware
  detect/install/verify for `meta-tool-links`.
- [x] **TASK-0008 (P2):** Relocate **meta-mcp** → `meta/meta_mcp` (lowest risk; first proof of procedure).
  VERIFIED already satisfied 2026-06-22: `/home/drdave/Desktop/meta/meta_mcp` exists and is registered
  in `.meta.yaml`; no `meta-mcp/` directory remains; `~/.local/bin/meta-mcp` is a symlink to
  `/home/drdave/Desktop/meta/target/release/meta-mcp`; `meta-mcp` builds from the meta workspace and
  `cargo test -p meta-mcp` passes (10 tests) with toolchain cargo at Rust 1.88 path.
- [!] **TASK-0009 (P2):** Relocate **kasetto + kst** — superseded by Epic C (kasetto becomes built-in;
  no external binary to relocate once absorbed). Until C lands: meta source v3.0.0 < installed v3.1.0.
- [x] **TASK-0010 (P2):** Relocate **rtk + rtk-monitor** — DONE 2026-06-13 (human-supervised session,
  per rtk-tokenkill weave report). `FlexNetOS/rtk-tokenkill#1` (sync upstream 0.42.4, rusqlite 0.40 kept)
  MERGED → develop; rtk built canonically → `meta/target/release/rtk`; `~/.local/bin/rtk` now a SYMLINK
  into meta (0.42.4); live hook verified; old 0.42.2 archived; meta `Cargo.lock` locked to 0.42.4.
  (Was `- [!!]` SUPERVISED — correctly NOT auto-run by the loop; resolved by a human, as designed.)

## Epic C — Kasetto full-feature unification into envctl (no downgrade)

kasetto is already pure-Rust + passes no-c gate (only drop `mimalloc`). envctl already ported §2
lock / §16 runtime / doctor / lock --check. Absorb the rest as a pure-Rust crate. NO-DOWNGRADE
checklist in ADR-0001 (all 11 verbs incl v3.1 add/remove/lock; 6-key+extends schema; 21-agent
preset; multi-host resolver; 5 cmd + 4 MCP-merge additive transforms; 3 lock modes).

- [ ] **TASK-0011 (P1):** Refresh `docs/KASETTO-FEATURES.md` to v3.2.0 (full verb/schema inventory +
  no-downgrade checklist; current doc is stale at v3.0.0).
- [ ] **TASK-0012 (P0 of C):** New pure-Rust crate `crates/agent-env` — config model (6 keys +
  `extends`), multi-host source resolver, SHA-256 hash, lock. Drop `mimalloc`. no-c gate clean.
  - **IN PROGRESS 2026-06-13 (forge-loop cycle, owner-directed `/harness:rust-port`).** Owner
    resolved the no-downgrade fork: synced `meta/kasetto` source UP to **pivoshenko/kasetto v3.2.0**
    (canonical upstream; FlexNetOS v3.0.0 divergence archived → `flexnetos-divergence-backup-2026-06-13`).
    Crate `crates/agent-env` seeded + `model/*` ported (foundational config/extend/source/hash/lock +
    full 21-agent preset table + 4 MCP/5 command formats); 78 tests + no-c GREEN. **PR #71 → develop
    (auto-merge armed).** Now driven by the rust-port **parity ledger**
    (`.handoff/loop/rust-port/parity-ledger.md`: 55 ported `[~]` / 44 todo `[ ]` / 13 front-end `[≠]`
    / 0 verified `[x]`). Resume via `/harness:rust-port` (HANDOFF: `.handoff/loop/rust-port/HANDOFF.md`)
    — next: parity-verifier pass, then fsops/config_edit/MCP-merge/commands. NOT done until 100% parity.
    The ledger spans TASK-0012..0018 (Engine wiring = TASK-0013, CLI verbs = TASK-0014).
  - **FORK SYNC DONE 2026-06-13** (owner ran the force push): kasetto fork RENAMED
    `env_manager_agent` → `FlexNetOS/kasetto`; `origin/main` force-pushed (--force-with-lease) UP to
    upstream v3.2.0 (`ec01cca`, 0/0 in sync); divergence preserved (remote backup branch + git bundle
    in `.archives/`); `.meta.yaml` retargeted via meta PR #31. Fork == upstream == local.
- [ ] **TASK-0013:** Engine `agent_env` module + Engine methods + Events (agent_sync/add/remove/lock);
  non-printing, front-end parity.
- [ ] **TASK-0014:** CLI verbs `envctl agent {sync,add,remove,lock,list,clean}` (--dry-run/--json/--locked)
  + GUI parity.
- [x] **TASK-0015:** Provisioning fidelity — verbatim skill copy; 5 command-format transforms; 4
  MCP-merge formats (ADDITIVE, never-clobber — must preserve global broker/repowire/weave servers).
  DONE 2026-06-22, PR #146: added live `agent-skills` MCP-pack regression through
  `Engine::agent_sync` for `claude-code` + `codex`, proving broker/repowire/weave survive while
  github/context7/exa/memory/playwright/sequential-thinking/n8n-mcp are added to both `.mcp.json`
  and `.codex/config.toml`; kept mesh secrets untouched; cleaned a pinned-toolchain clippy lint in
  the agent-env source test helper; ignored ephemeral `.handoff/locks/`. CI all green on #146 and
  `hf test TASK-0015` witnessed build+p7 before `hf done`.
- [x] **TASK-0016:** Lock unification — fold agent assets into `envctl.lock` (SHA-256 section) or keep
  kasetto.lock owned by the subsystem; reframe `manifest/agent-env.toml` external-binary → built-in.
  DONE 2026-06-23, PR #149: recorded the no-downgrade decision to keep standalone SHA-256
  `agent-env.lock` separate from FNV-1a `manifest/envctl.lock`; reframed `manifest/agent-env.toml`
  around built-in `envctl agent` + `agent-env.yaml`; tightened `ci/gates/agent-env.sh` to use
  `envctl agent lock --config agent-env.yaml --check --locked`; regenerated `manifest/envctl.lock`.
  CI all green on #149 and `hf test TASK-0016` witnessed build+p7 before `hf done`.
- [x] **TASK-0017:** Adopt kasetto `extends` config composition for envctl component manifests.
  DONE 2026-06-23, PR #151: added local manifest `extends` support before component deserialization;
  parents load first, relative paths resolve from the child manifest, cycles/depth overflow fail
  closed, and `[[component]]` arrays merge by component `id` with same-id child tables overlaying
  parent tables. Added integration tests for inherited components, same-id overlay, cycle refusal,
  and depth refusal; updated docs. CI all green on #151 and `hf test TASK-0017` witnessed build+p7
  before `hf done`.
- [ ] **TASK-0018:** Retire the external `kasetto` binary dependency — only after the no-downgrade
  checklist passes end-to-end.

## Epic D — Follow-ups surfaced from the WIP-branch consolidation (2026-06-12)

All WIP branches were merged to develop + verified green (build, 197 tests, no-c/shape/enable,
fmt, clippy). Remaining follow-ups extracted from each:

- [x] **TASK-0019 (fix-secretd):** U1 USB-unlock path needs a real `RealUsbProbe` (crash-loop +
  durable store + passphrase path already fixed/merged). DONE 2026-06-23: verified at HEAD that
  the old `_done/secretd-provisioning-runbook.md` premise is stale. `RealUsbProbe` delegates to the
  pure-Rust Cognitum Seed `seed_factor` backend under `--features seed-factor`; `secretd` forwards
  USB enrollment through the same seam and injects `RealUsbProbe` into the live daemon engine; the
  `env-ctl` manifest install/fix hooks build `envctl-secretd --features seed-factor`. The same
  closeout fixed the envctl-owned rustup component after verification found a dangling hosted-runner
  cargo shim: rustup now lives under `$META_ROOT/.toolchains/{cargo,rustup}` and `$HOME/.cargo/bin`
  is compatibility links only. Verification: seed-factor unit tests, seed-factor `secretd` build,
  fake-probe USB keyslot unlock test, engine+CLI build, envctl lock check, envctl doctor, and
  p7/no-c/shape/enable/kdf/agent-env/loop-state/harness-scripts gates.
- [ ] **TASK-0020 (github-app-mint, P0 — unblocks the `flexnetos_github_app` e2e crown slice):** Expose
  the completed `provider-github` `ProviderMint` (`secrets-engine/src/mint_github.rs`, PR #35, fully
  unit-tested via `FakeTransport`) through `secretd` + `secretctl` so the trusted-writer App can mint
  short-lived installation tokens from the vault-sealed key. **The minting impl is DONE — this is the
  daemon plumbing + CLI surface only.** (Authored as a build-ready card by the cross-repo session that
  created+installed+sealed the live App, 2026-06-13.)
  - **Consumer contract (FROZEN — `flexnetos_github_app` already depends on it):** `app-core::mint::
    SecretctlInvoker` shells `secretctl mint-github --installation-id <N> [--repository-ids a,b]
    [--permissions name:access,...] --ttl-secs <T> --output json` and parses stdout exactly as
    `{"token":"<installation-token>","expires_at_unix":<i64>}`. Permission access maps Read→`"read"`,
    Write→`"write"`. Today this **404s** (no such subcommand) → the e2e token write-back (post
    check-run / merge-gate) is blocked. Do not change the flag/JSON shape.
  - **Build:**
    1. **Real `HttpTransport`** (the `mint_github::HttpTransport` seam): REUSE the PR #58 relay-proxy
       client — `reqwest` pinned to **`webpki-roots`, rustls-on-ring** (FS-S7), NOT native-tls/OS roots
       (else `no-c.sh` fails). Wrap it as the sync `execute(&HttpRequest)->HttpResponse`.
    2. **Internal unseal path:** the daemon reads `github-app-private-key` (**broker-only** — use the
       same internal key-extraction the relay proxy uses, NOT the `get --reveal` API, which refuses
       broker-only) + `github-app-id` from the unlocked vault, then
       `GitHubAppMint::new(app_id, installation_id, Zeroizing(pem), RealClock, RealTransport)`.
       Fail-closed when the vault is locked / key absent / the `provider-github` feature is off.
    3. **proto (`secrets-proto/proto/control.proto`):** add `rpc MintGithub(MintGithubReq) returns
       (MintGithubResp)` to `service Vault`. `MintGithubReq{ uint64 installation_id; repeated string
       repository_ids; repeated string permissions; int64 ttl_secs; }` ·
       `MintGithubResp{ string token; int64 expires_at_unix; }`.
    4. **secretd handler:** build `MintRequest{ provider: Github, repos, perms, ttl_secs }`, call
       `mint_scoped`, map `ScopedToken{token, expires_at}` → resp; emit a witnessed mint event but
       **NEVER log the token**.
    5. **secretctl `mint-github` subcommand:** flags per the frozen contract; `--output json` prints
       the JSON to stdout only.
    6. **Feature:** enable `provider-github` in the `secretd` build (cargo feature, same pattern as
       `seed-factor`); wire it through `secrets-engine`.
  - **Gates/tests:** `no-c.sh` (reqwest MUST stay `webpki-roots`/rustls-ring — already clean post #58),
    `shape.sh`, `enable.sh`; `cargo fmt` + `clippy -D warnings`; add a `secretd` RPC test + a CLI smoke
    (the JWT/request/parse logic is already unit-tested in `mint_github.rs`).
  - **Acceptance (LIVE — the App is already created+installed+sealed):** with the vault unlocked (Seed)
    and these secrets sealed — `github-app-private-key` (broker-only), `github-app-id`=**4044997**,
    `github-app-installation-id`=**140063898** (org **FlexNetOS**, app slug `flexnetos-github-app`) —
    `secretctl mint-github --installation-id 140063898 --output json` returns a real installation token
    from `POST /app/installations/140063898/access_tokens`, and `fxapp mint-token` (flexnetos_github_app
    app-core P1) then succeeds end to end. This completes the crown slice's token write-back; the
    webhook→dispatch→fork-gate half was already proven LIVE through a public tunnel (2026-06-13).
  - **Cross-refs:** ADR-0007/0008; `mint_github.rs` (PR #35); `seam.rs::ProviderMint`; **PR #58**
    (relay-proxy `reqwest`/`webpki-roots` transport to reuse); `flexnetos_github_app/crates/app-core/
    src/mint.rs` (`build_argv` contract) + `app-cli` `MintToken`. Sibling of TASK-0019 (RealUsbProbe,
    done via #61) — both are Epic-D secrets-egress.
- [x] **TASK-0021 (node-via-bun):** Manifest design follow-up — mark node not-applicable when a real
  node in the n8n range is present, or add a `node-real` component + drop the group-ai-clis edge
  (cosmetic detect-drift only; truth-telling fix already merged). DONE 2026-06-23: verified at HEAD
  that `node-real` already exists as a standalone carve-out, `group-ai-clis` no longer requires
  `node-via-bun`, `envctl lock --check` is clean, and focused engine tests prove the `node-real`
  component is empty-requires and the `group-ai-clis` edge is absent.
- [ ] **TASK-0022 (agent-web-access):** Phases 2–3 of the agent web-access ladder (Phase 1 n8n-mcp +
  kasetto wiring merged). `- [!]` n8n live smoke test is HUMAN-ONLY (see
  `.handoff/loop/_done/n8n-live-smoke-runbook.md`).

## Epic E — Workflow infrastructure

- [ ] **TASK-0023:** develop→master auto-sync GitHub Action (ff master on develop push) +
  enable branch protection on master (PR-only for humans; action token bypass). [in progress 2026-06-12]
- [x] **TASK-0025 (P1, Epic E) — DONE 2026-06-13 (cycle 7): CI required checks on `develop` so
  auto-merge gates fail-closed.** Added `.github/workflows/ci.yml` (4 jobs: **rustfmt · clippy
  (workspace, default features) · test (`--test-threads=1`) · gates (no-c/shape/enable/p7)**) — no
  `--all-features` (mutually-exclusive `remote`/`embedded`). Enabled repo `allow_auto_merge` +
  `develop` branch protection requiring those 4 contexts (strict=false so concurrent sessions aren't
  serialized; no required reviews; admins not enforced). Fixed a real isolation bug TASK-0004 exposed:
  `dashboard::tests::locate_walks_up` leaked the inherited `$META_FILE` → made hermetic. `test` runs
  serial in CI to kill the `XDG_CACHE_HOME`/`$META_FILE` parallel env-race flakiness. Verified all 4
  green locally before requiring them. (Master protection / develop→master mirror = TASK-0023, separate.)

## Epic G — Forge-loop hardening (2026-06-18 deep audit)

> Source: owner-requested deep audit of the forge-loop harness (provenance + gaps + adoptable
> patterns from the rust-port crew and harness_hub). Provenance confirmed: forge-loop is
> hand-authored bespoke in envctl (commits `5dcc4b2`/`00237ca`, 2026-06-04), NOT ported from an
> external "forge" repo — it is the *source pattern* harness_hub later abstracted. Findings recorded
> here as a tiered, dependency-ordered plan. **These items edit the hand-authored harness
> (`.claude/skills/*`, `.claude/agents/*`, `ci/gates/*`, `scripts/*`) — outside the agent-env
> pipeline, committed in place.** Build order: **Tier 1 → Tier 2 → Tier 3**, deps honored within.
>
> Audit dimensions & severities are carried per-item below. None weakens an existing guard
> (evolution-steward discipline); each is additive.

### Tier 1 — low-risk pure additions (build first, no design decision needed)

- [x] **TASK-0041 (T1.1, P0) — DONE 2026-06-18 (direct-to-develop):** `ci/gates/loop-state.sh`
  shipped — asserts the 5 counters parse as non-negative ints, `wrap_every`/`cycle_budget` >= 1,
  `cycles_total >= last_wrapup_total`, and `cycles_total` monotonic vs HEAD~1 (skip-if-unreadable,
  never false-block). Hermetic test `scripts/tests/test-loop-state-gate.sh` (7 scenarios) wired into
  `ci/gates/harness-scripts.sh` + a dedicated `ci.yml` step; CLAUDE.md gates list updated. Gate green
  on live state; full harness-scripts gate green. `ci/gates/loop-state.sh` — loop-counter integrity gate. The
  batch-boundary / hand-off / WRAP-UP-OWED logic all key off hand-edited integers in `loop_state.md`
  (`cycles_total`, `last_wrapup_total`, `wrap_every`); `cycles_total: 18` is reconstructed by free-text
  narration with no check that it matches ground truth.
  - **Files:** `ci/gates/loop-state.sh` (NEW, mirror the dependency-free grep gates), `.github/
    workflows/ci.yml` (+gate step), `CLAUDE.md` (CI-gates list row).
  - **Gate asserts:** the three counters parse as integers; `cycles_total >= last_wrapup_total`;
    `cycles_total` is monotonic vs the prior commit (non-decreasing). Read-only, zero-network, exits 1
    on drift.
  - **Acceptance:** gate green on current state; flips red on a hand-injected non-integer / decreased
    `cycles_total`. Add to `ci/gates/harness-scripts.sh` family or its own step.
  - **Deps:** none. **Risk:** very low (new gate, additive).

- [x] **TASK-0042 (T1.2, P1) — DONE 2026-06-18 (direct-to-develop):** Mechanism shipped + existing
  file drained. `session-relay-wrap-up` step 3b now drains `proposed-upgrades.md` fail-closed (open →
  `- [?]` backlog item; addressed → record resolved vs HEAD; declined → record disposition; then reset
  the file to its drained header — a non-empty file means wrap-up is INCOMPLETE).
  **Drained the 49-line 2026-06-18 backlog:**
  - *P1 (merge-driver test)* → **RESOLVED** — `scripts/tests/test-merge-driver.sh` shipped + wired
    into `ci/gates/harness-scripts.sh` (prior session). No new work.
  - *P2 (reaper test)* → **RESOLVED** — `scripts/tests/test-reaper.sh` shipped + wired into the same
    gate. No new work.
  - *P3 (scheduled reaper)* → **DECLINED, accept (a)** — reap stays loop-boundary-only (resume +
    wrap-up); a CI/cron reaper has no local workspace to clean. Documented in forge-loop "Worktree
    hygiene". `proposed-upgrades.md` reset to drained header.
  Original entry: Drain `proposed-upgrades.md` into the backlog. The evolution-steward
  writes structural proposals to `.handoff/loop/proposed-upgrades.md` (currently 49 lines, undrained);
  nothing tracks them to accept/reject closure, so escalations sit indefinitely.
  - **Files:** `.claude/skills/session-relay-wrap-up/SKILL.md` (extend step 3b), `.handoff/loop/
    backlog.md` (the drained items land here).
  - **Change:** wrap-up step 3b also drains `proposed-upgrades.md` → tracked `- [?]` harness-upgrade
    items in the backlog with origin + an owner-decision status; an undrained non-empty file makes
    wrap-up INCOMPLETE (same fail-closed shape as the follow-up drain).
  - **Acceptance:** running a wrap-up with a non-empty `proposed-upgrades.md` results in `- [?]`
    backlog rows citing it; the file is marked drained.
  - **Deps:** none. **Risk:** low (skill-text + backlog edits).

- [x] **TASK-0043 (T1.3, P0) — DONE 2026-06-18 (direct-to-develop):** Phase 3.5 runtime-verify
  shipped. feature-architect now emits a `## Runtime surface` section (the `runtime_verifiable?` flag
  + the exact drive path); feature-forge has a **Phase 3.5** between Verify and Synthesize; the
  invariant-guardian gained invariant #10 (Runtime behavior), a "Runtime verification" section that
  drives the declared surface via the `verify` skill (run the app, capture evidence, probe one
  off-happy-path), and a `## Runtime check` report line; `verification.md` §4.5 mirrors it. A clean
  PASS now requires a runtime observation (or a recorded SKIP for no-surface); static-gates-only is
  downgraded to PASS-WITH-NOTES. Closes the TASK-0028 "green but broken" class. (Behavioral-branch
  coverage deepening = TASK-0051, builds on this.) Runtime-verify phase in feature-forge — wire the bundled `verify`
  skill into the guardian gate. Today the guardian is **static-only** (gates + `cargo` + source-grep
  parity); the crew never runs the app. TASK-0028 shipped a GUI Secrets screen marked done on "argv
  round-trip vs a replica" — no `secretctl` invocation, no GUI launch. The "compiles + gates green but
  doesn't work at runtime" class escapes.
  - **Files:** `.claude/skills/feature-forge/SKILL.md` (new Phase 3.5 runtime-verify), `.claude/agents/
    feature-architect.md` (plan emits a `runtime_verifiable?` flag + the surface: CLI verb / GUI screen
    / daemon RPC), `.claude/agents/invariant-guardian.md` (verdict PASS only after the behavioral check
    when a surface exists).
  - **Change:** for any item whose plan declares an observable surface, the guardian invokes
    `verify`/`run` to drive that surface and capture evidence before PASS; SKIP only when the architect
    declares no runtime surface (docs/types/test-only), one line why.
  - **Acceptance:** a feature with a CLI/GUI/RPC surface cannot reach PASS on static gates alone — the
    guardian report cites a runtime observation. Smoke: re-run a representative past item.
  - **Deps:** none (T3.4 builds on this). **Risk:** medium (changes the gate contract — but strictly
    *stronger*, never weakens it).

### Tier 2 — pick-correctness + completeness (small design decision per item)

- [x] **TASK-0044 (T2.1, P1) — DONE 2026-06-18 (handoff-kernel-engineer cycle): envctl backlog
  minted into 53 fleet-scoped `handoff.task.v1` cards in the member store `envctl/.handoff/tasks/`;
  hf kernel is now envctl's pick-time dependency authority.**
  - **Mint path used (sanctioned, contamination-free, residency-safe):** cards generated by a
    throwaway tool that links the kernel's OWN `work-order` crate (`WorkOrder::compute_intent_lock`),
    so every `intent_lock` blake3 hash is byte-identical to what `hf` verifies — proven by
    re-deriving an existing `PHTASK-0001` card's 3 hashes (all matched). Cards written as
    `*.task.json` files only (the per-member card store, exactly how handoff/prompt_hub hold theirs);
    **zero ledger writes** (intake/file-write does not touch the ledger — verified: fleet ledger
    md5 + 840-event count unchanged before/after). NOT `hf intake`/`hf task mint`: intake is
    vibe-synthesis (can't carry 53 pre-specified ids/deps/status) and `task mint --from-kb` forces a
    `KBTASK-` prefix into the shared FLEET dir (would mix with HFTASK/KBTASK cards) — both wrong here.
  - **Card model (DAG-correct for the picker):** `dependencies` carries ONLY still-OPEN prerequisites
    (the picker's `next_safe` resolves a dep as satisfied via a ledger **Done** event; with the
    residency-mandated empty member ledger, an edge to an already-done task can never satisfy, so
    edges to done tasks are dropped from `dependencies` and preserved in `blocked_by` for provenance).
    Status mapping: `[x]`→done (34), `[ ]`/`[?]`→backlog, `[!]`→blocked, `[~]`→review, this task→active.
    No `[!!]` SUPERVISED items currently open (TASK-0010 was, now done).
  - **VERIFICATION (acceptance gate):**
    - *No contamination:* `hf fleet status --json` AFTER = **handoff 55 (unchanged), prompt_hub 71
      (unchanged), weave 1 (unchanged), envctl 53 (was 0)**; fleet_ledger events 840 unchanged.
    - *Member isolation, no HFTASK leakage:* `hf fleet render envctl` (run from `$META_ROOT`,
      read-only on the FLEET ledger) renders envctl's packet from its OWN cards; Remaining lists
      ONLY `TASK-*`; **0 `HFTASK-*` references**. Packet `Done: 34/53`.
    - *DAG / dep-safe pick (proven in an isolated sandbox so no member ledger is created in the repo):*
      `hf resume --json` → `next_task_id: TASK-0035` (the in-progress `review` task, correct step-1
      precedence). With 0035/0044 flipped to backlog, `next_task_id: TASK-0007` and `hf claim --next`
      routes to TASK-0007 — and **TASK-0038 (dep open TASK-0035) and TASK-0052 (dep open TASK-0044)
      are correctly NOT served** (dep-gating works). Spot-checks: TASK-0047 `blocked_by [TASK-0046]`,
      TASK-0038 `dependencies [TASK-0035]`, TASK-0052 `dependencies [TASK-0044]` ✓.
    - *Residency:* **0 `*.db` under any envctl `.handoff`** (card writes never touch a ledger);
      `bash ci/gates/p7.sh` → **P7 GATE PASS** on the 53 new cards. `hf doctor --json` healthy=true.
      `hf drift --json` shows only pre-existing `HFTASK-0022` kernel-scope drift (0 envctl `TASK-*`
      drift — minting added no ledger events).
  - **⚠ KERNEL GAP (the picker-VERB residency wall — finding, not a regression):** the shipped `hf`
    (S1 spike) is strictly **CWD-relative for the ledger** with **no `--ledger`/`HANDOFF_LEDGER`/
    `--member` override** (tracked kernel-side as **HFTASK-0054** in the FLEET ledger). Running the
    *picker verbs* `hf resume`/`hf claim --next` from a member dir (cwd = `envctl/.handoff/`) **creates
    a forbidden `envctl/.handoff/ledger.db`** (verified: even read-only `resume` opens/creates it) —
    an ADR-0004 residency violation. Running them from `$META_ROOT` reads the FLEET dir's cards
    (HFTASK-*), not envctl's. So with this binary the dep-safe picker CANNOT be run live, member-scoped,
    without either contamination or a per-repo ledger. The **read-only** authority path that IS clean
    today is `hf fleet render envctl` (the cards ARE the dependency authority; render proves the DAG).
    The live `hf claim --next` member-scoped picker is unblocked only by the kernel landing HFTASK-0054
    (a ledger-path override) — handoff-repo scope, like the C-SQLite #71 item. **The cards (the
    authority substrate) are delivered and DAG-correct now; the live picker-verb wiring waits on the
    kernel flag.** Forge-loop pick step should therefore: prefer `hf fleet render envctl` to read the
    authoritative open/DAG set, and keep the markdown subnote as the live-pick fallback until
    HFTASK-0054 lands a `--ledger` override (then `hf claim --next --ledger $META_ROOT/.handoff/ledger.db`
    from the member becomes residency-safe).
  - [ORIGINAL SPEC BELOW] Pick-time dependency authority via the **hf kernel** (LOCKED
  2026-06-18, owner). Cards were never minted (`.handoff/tasks/` is just `.gitkeep`), so ordering is
  *always* the markdown-subnote fallback while the skill/architect/steward assert "cards/`hf next_safe`
  are authoritative" — two ordering stories, only the prose one live (the TASK-0020 wrong-surface miss
  is exactly this). **LOCKED DECISION:** adopt the **kernel-backed loop** — `handoff-loop-init` →
  `handoff-loop-run` driving `hf init` + `hf next_safe` (the witnessed-ledger DAG picker) as the
  dependency authority, retiring the markdown fallback as the *primary* path. (Not the front-matter
  pick-check — owner chose the kernel.)
  - **Files:** adopt `handoff-loop-init`/`handoff-loop-run` (from harness_hub `harness/skills/`),
    `.claude/skills/forge-loop/SKILL.md` (pick step delegates to `hf next_safe`; markdown is fallback
    only when hf is absent/non-resident), `.handoff/tasks/*.task.json` (cards minted from backlog).
  - **Acceptance:** `hf resume --json` returns `next_task_id` from the real DAG; picking an item whose
    `blocked_by` is unmet is refused by the kernel, not by prose parsing.
  - **2026-06-18 finding (hf IS live):** `hf` is on PATH (`~/.local/bin/hf`) and the shared fleet
    ledger is resident (`$META_ROOT/.handoff/ledger.db`, 240KB). So 0044 is **NOT blocked on hf
    existing** — the gap is that envctl's `TASK-*` cards were never minted. **⚠ The real hazard:** the
    shared ledger currently holds the handoff kernel's OWN `HFTASK-*` loop (29 tasks). Minting envctl's
    `TASK-*` into the same fleet ledger risks **cross-loop contamination** (cf. `HFTASK-0026` = "fix
    kb-mint contamination" — a real, already-hit failure mode). So this is a **cross-repo, shared-ledger
    fleet-routing** task → route to **`handoff-kernel-engineer`** (owns ledger residency + fleet routing),
    NOT a quick doc edit. hf's internal C-SQLite (#71) is the handoff repo's concern, NOT an envctl
    link-boundary blocker (hf is an external tool binary, like grit/rtk) — so it does not gate 0044.
  - **Deps:** Epic A is effectively satisfied for *availability*; remaining work is fleet-scoped card
    minting (envctl member) + verifying `hf claim --next`/`hf resume` picks envctl items without
    surfacing `HFTASK-*`. Do as a dedicated cycle.
  - **Risk:** medium-high (couples the loop to the kernel; shared-ledger contamination risk).

- [x] **TASK-0045 (T2.2, P1) — DONE 2026-06-18 (direct-to-develop):** `session-relay-resume` gained
  **step 4d** — a fail-closed sweep that, before any pick, re-polls every `- [~]` item's `pr=<N>` via
  `gh pr view --json state`, promotes `MERGED`→`- [x]` (+`hf done` when present), leaves the rest `- [~]`
  with a REQUIRED `pr=<N> state=<status>` field, and **excludes all still-`- [~]` items from the pick
  set** (a CLOSED-unmerged one → NEEDS-HUMAN, not re-picked). New non-negotiable bullet added. Enforces
  the tick-on-merged promise resume previously didn't mandate (#125 class). Fail-closed in-flight re-poll/promote sweep on resume. Tick-on-merged
  leaves armed-not-merged work as `- [~]` and says "next session re-polls," but `session-relay-resume`
  does not *mandate* the sweep and the `pr=<N> state=<status>` field is specified-but-unpopulated — so
  a race/skip could re-pick an already-built `- [~]` item.
  - **Files:** `.claude/skills/session-relay-resume/SKILL.md` (new fail-closed step: for every `- [~]`,
    `gh pr view <N> --json state`; promote `MERGED`→`- [x]`, else leave `- [~]`, and **exclude all
    `- [~]` from the pick set**), `.handoff/loop/loop_state.md` (require the `pr=<N> state=<status>`
    field), optionally fold into `ci/gates/loop-state.sh`.
  - **Acceptance:** resume with an armed-not-merged `- [~]` either promotes it (if merged) or excludes
    it from the pick; the structured PR-state field is present.
  - **Deps:** complements TASK-0044; can land independently. **Risk:** low-medium.

- [x] **TASK-0046 (T2.3, P1, HIGH value) — DONE 2026-06-18 (direct-to-develop):** Symbol-grain
  completeness ledger shipped. feature-architect plan now has a **`## Unit ledger`** (one tagged `U#`
  row per concrete deliverable — Engine method / Event / type / CLI flag / RPC / GUI control /
  component / test — with `file::symbol` + how it is wired); invariant-guardian gained a
  **Completeness check** that proves each row PRESENT (AST) AND WIRED (has a caller) — any missing or
  unwired row is a FAIL, plus a `## Unit ledger` report table. Closes the backlog-grain "done ≠
  complete" drift at symbol grain. Symbol-grain completeness ledger (adopt the rust-port
  *cartographer* pattern). forge tracks completeness only at task-card grain in `backlog.md` — "done"
  ≠ "the surface area exists & is wired." Introduce a per-cycle ledger of the concrete units a task must
  produce (Engine method / `Event` / CLI flag / RPC / component) and whether each is *present* and
  *wired*.
  - **Files:** `.claude/agents/feature-architect.md` (emit the unit list into `.handoff/loop/cycle/
    01_architect_plan.md`), `.claude/agents/invariant-guardian.md` (verify each unit present+wired),
    a `.handoff/loop/cycle/` ledger surface; reference `rust-port-cartographer.md` as the pattern.
  - **Acceptance:** a cycle's guardian report enumerates each planned unit with present/wired/verified
    status; a missing unit blocks PASS.
  - **Deps:** none structurally; foundation for TASK-0047. **Risk:** medium.

- [x] **TASK-0047 (T2.4, P1, HIGH value) — DONE 2026-06-18 (direct-to-develop):** Pre-DONE
  left-behind sweep shipped. forge-loop's DONE sentinel write now requires, in addition to completion-
  confirmed, an **independent re-derivation** (a completeness-critic guardian pass) that derives the
  expected surface from the plans' `## Unit ledger` + goals + `docs/` and diffs vs **delivered code**
  (not vs the backlog's own `- [x]`); a surfaced un-built unit or a zero/partial harvest →
  **INCONCLUSIVE → NEEDS-HUMAN**, not DONE. "Clean" now requires a positive matching re-derivation.
  Builds on TASK-0046's ledger. Pre-DONE left-behind sweep (adopt cartographer's
  independent re-derivation). forge's DONE trusts the backlog is exhaustive; rust-port re-harvests scope
  and fails **INCONCLUSIVE** rather than "clean." Add a completeness critic before terminal DONE that
  re-derives the expected surface from the plan/spec and diffs vs what was built.
  - **Files:** `.claude/skills/session-relay-wrap-up/SKILL.md` (or a `forge-loop` DONE-gate step) — a
    sweep that re-derives scope independently and emits `INCONCLUSIVE → NEEDS-HUMAN` on a zero/partial
    re-derivation rather than silently passing.
  - **Acceptance:** DONE cannot be declared while the independent re-derivation surfaces an un-built
    planned unit; emits NEEDS-HUMAN with the gap.
  - **Deps:** **TASK-0046** (uses the unit ledger). **Risk:** medium.

### Tier 3 — structural (A2 maturity + behavioral coverage + hub conformance)

- [x] **TASK-0048 (T3.1, P1) — DONE 2026-06-18 (direct-to-develop):** A2 all-green barrier shipped — Phase 2-A2 step 6 now commits each repo on its own guardian PASS but DEFERS arming `gh pr merge --auto` on ALL N PRs until EVERY target repo's guardian PASSes; if one repo FAILs, no sibling was armed → half-landed is impossible, the failed repo is `- [!]`, cycle does not reach Done. Barrier waits on each repo's guardian/CI, not an OS matrix. A2 cross-repo merge atomicity — **all-green barrier** (LOCKED
  2026-06-18, owner). A2 commits/PRs each repo independently after that repo's guardian passes — repo A
  can MERGE before repo B's guardian FAILs, leaving a half-landed feature with no rollback path.
  **LOCKED DECISION:** arm auto-merge for **every** target repo ONLY after **all** repos' guardians
  PASS (the all-green barrier) — half-landed becomes impossible by construction. (Not the
  allow-partial + `partial-landed`-state option.)
  - **Caveat (owner):** the all-green barrier is across target **repos**, NOT across OS platforms —
    do **not** add macOS/Windows/Ubuntu cross-platform build matrices to the "all-green" gate right
    now. The barrier waits on each repo's existing guardian/CI, not a new OS matrix.
  - **Files:** `.claude/skills/feature-forge/SKILL.md` (Phase 2-A2: hold all per-repo auto-merge arming
    until the all-repos-green barrier), `.claude/agents/rust-implementer.md` (commit-on-PASS but
    arm-merge deferred to the barrier).
  - **Acceptance:** an A2 cycle where one repo FAILs leaves NO sibling repo merged (no auto-merge was
    armed before the barrier); the failed repo is `- [!]` blocked, the cycle does not reach Done.
  - **Deps:** none. **Risk:** medium-high (touches the cross-repo commit/merge flow).

- [x] **TASK-0049 (T3.2, P1) — DONE 2026-06-18 (direct-to-develop):** Phase 2-A2 step 2b now builds a cross-repo blast-radius map (`git-kb code callers/callees/impact` across target repos + shared protocol types) into `.handoff/loop/<repo>/00_impact_map.md` BEFORE locking; the grit lock scope (step 3) derives from it instead of an ad-hoc guess. Adopts the cross-repo-referencer discipline. Cross-repo impact map before A2 grit locks (adopt the
  *cross-repo-referencer* pattern). A2 takes grit symbol locks *blind* — no who-calls-what blast-radius
  map drives the lock scope.
  - **Files:** `.claude/skills/feature-forge/SKILL.md` (Phase 1.5 A2: build a `git-kb code
    callers/callees/impact` map across target repos before locking; lock scope derives from it),
    reference `rust-port-cross-repo-referencer.md`.
  - **Acceptance:** A2 lock scope is derived from an impact map artifact, not chosen ad hoc.
  - **Deps:** none; pairs with TASK-0050. **Risk:** medium.

- [x] **TASK-0050 (T3.3, P1) — DONE 2026-06-18 (direct-to-develop):** Phase 2-A2 step 2a captures each target repo's behavioral baseline (`00_baseline.md`: tests + runtime obs of touched surfaces) BEFORE any implementer writes; step 5's guardian diffs delivered behavior vs that baseline — a merge that lands new code but regresses the destination's prior behavior FAILs. Bidirectional no-downgrade. Builds on TASK-0049's map. Bidirectional "don't regress the destination" baseline in A2 (adopt the
  *merge-integrator* discipline). A2 proves each repo compiles + passes its own guardian but never
  captures the target's behavioral baseline nor proves the change didn't regress it.
  - **Files:** `.claude/skills/feature-forge/SKILL.md` (A2: capture destination behavioral baseline at
    DISCOVER, prove no-regression at the gate), `.claude/agents/invariant-guardian.md` (the
    no-regression diff), reference `rust-port-merge-integrator.md`.
  - **Acceptance:** an A2 merge that regresses the destination's prior behavior fails the gate.
  - **Deps:** **TASK-0049** (uses the impact map). **Risk:** medium.

- [x] **TASK-0051 (T3.4, P2) — DONE 2026-06-18 (direct-to-develop):** invariant-guardian Runtime verification + verification.md §5 now require driving EVERY refusal/error branch + the dry-run-vs-`--apply` split at the real surface for mutating ops (not just confirming a refusal unit test exists). Adopts the parity-verifier exercise-every-branch discipline, scoped to new features (differential golden testing N/A — no reference). Builds on TASK-0043. Behavioral-branch coverage in the guardian (scoped-down
  *parity-verifier* technique). Differential golden testing doesn't map to new features (no reference),
  but the *exercise-every-branch* discipline does — especially the fail-closed guard **refusal** paths
  the guardian today only checks structurally.
  - **Files:** `.claude/agents/invariant-guardian.md` (for mutating ops, exercise each error/refusal
    branch, not just the happy path), `.claude/skills/rust-feature-impl/references/verification.md`.
  - **Acceptance:** a mutating op's guardian report shows the fail-closed refusal path was driven and
    observed, not inferred from source.
  - **Deps:** **TASK-0043** (builds on runtime-verify). **Risk:** medium.

- [x] **TASK-0052 (T3.5, P2) — DONE 2026-06-18 (harness_hub PR #38):** Feature Forge packaged into
  harness_hub as ejectable **`/harness:feature-forge`** — `harness/skills/feature-forge/` (orchestrator,
  phases 0–4 incl. 3.5 + Phase E) + sub-skills `forge-loop`/`rust-feature-impl` + 4 prefixed specialists
  `harness/agents/feature-forge-{architect,implementer,guardian,kernel-engineer}.md` (reuses shared
  evolution/continuity/integration-qa/build-health); `eject.sh`+`references/eject.md`+`loop_state.
  template.md`+`ralph-feature-forge.sh`; `registry.json` row + `entries/feature-forge.md` + README;
  plugin 1.10.1→1.11.0. **`hub-validate` PASS (8 entries)**, 0 dangling old-agent refs. envctl CLAUDE.md
  Placement doctrine reconciled (supersedes never-packaged stance for the generic core; env-install-loop/
  auto-provision/handoff-sync stay envctl-only).
  [ORIGINAL SPEC] **Full eject/package** forge-loop into harness_hub (LOCKED 2026-06-18,
  owner). forge-loop is unregistered/unpackaged/unvalidated by the hub though it already *consumes* the
  hub's shared layer (byte-identical `harness-evolution`, the whole `rust-port`/`session-relay`/
  `icm-memory` families). **LOCKED DECISION:** convert the Feature-Forge family into the **factory-minted
  packaged-harness shape** (like `meta-plugin`/`rust-port`/`code-research`) — ejectable into other repos
  — NOT a catalog-row-only entry.
  - **⚠ Doctrine change:** this **overrides** envctl CLAUDE.md's "harness is hand-authored and
    git-tracked, intentionally OUTSIDE the kasetto/agent-env pipeline" rule for the Feature-Forge family.
    Reconcile that CLAUDE.md section + the change-history table as part of this task. Prefix the core
    specialists that would collide in the hub's shared pool (`feature-architect`, `rust-implementer`,
    `invariant-guardian`, `handoff-kernel-engineer`) per the Packaged-Harness Standard's agent-pool rule.
  - **Files:** harness_hub factory shape — `harness/skills/feature-forge/` + bundled `scripts/eject.sh`
    + `references/eject.md`; `registry.json` row + `entries/<id>.md`; run `scripts/register.sh` + the
    `hub-validate` crate; conform to `docs/packaged-harness-standard.md`. envctl side: CLAUDE.md
    reconcile + (if ejected back) the generated skills.
  - **Acceptance:** `harness_hub` `hub-validate` passes with the Feature-Forge harness packaged +
    registered + ejectable; envctl CLAUDE.md doctrine reconciled.
  - **Deps:** capstone — do last (the whole family must be stable first). **Risk:** medium (doctrine
    shift + cross-repo factory work). **Cross-repo** (envctl + harness_hub).

## Epic H — Eliminate system-depth installs (owner doctrine, 2026-06-23)

**Doctrine (owner):** meta and its peers use NO system-depth installs (apt `/usr`, `/usr/local`,
nix `/nix`, kernel). Every system-depth install has an upstream repo and MUST be meta-prefix-
installed (`$META_ROOT/.toolchains/<x>` via tarball / `cargo install --root` / runfile
`--toolkitpath`) or cloned+added (`.meta.yaml` peer / `add-repo`), or — only if physically
irreducible — declared an explicit `system:` component. Authoritative plan + per-item method:
`docs/adr-install-locations-and-local-state.md` (§System-depth convergence plan). Each card below
adds a real envctl component (new `manifest/*.toml`) + `.toolchains` install + `~/.local/bin`
symlink; dry-run-safe, applied via `envctl install`/`env-install-loop`. Sequenced EASY → HARD.

**Single-admin / full-agentic principle (owner, 2026-06-23):** this box has one admin and runs
no-human-in-the-loop, so convergence is NEVER gated by *permission* — only by *work*. Host-policy
knobs (e.g. `apparmor_restrict_unprivileged_userns`) and sudo-phase installs are owner-tunable and
**envctl declares + applies them as components** (reproducible), not treated as external walls.
"Can't be meta-owned" is reserved for physical impossibility, never for "needs root" or "needs a
policy change" — both are available and declarable here.

- [x] **TASK-0054 (H, EASY, owner-pref) — DONE 2026-06-23 via PR #182 (MERGED, squash `5c4f666`):**
  `wild-linker` component install (done earlier) + **WIRING now landed**: the component install ALSO
  writes the meta-root `$META_ROOT/.cargo/config.toml` (`linker = "clang"` +
  `rustflags = ["-Clink-arg=--ld-path=wild"]`) — meta-owned path (NOT `~/.cargo`/system-depth), a
  runtime artifact at the meta root so **CI never sees it** (CI clones repos standalone) → local-dev
  acceleration only. detect asserts binary symlink + config section; verify link-tests a throwaway
  `/tmp` crate proving `--ld-path=wild`; remove is marker-self-guarded (restores a pre-wild backup or
  removes the managed file; never clobbers a foreign config). **Build-verification gate GREEN:**
  `cargo clean -p envctl` + `cargo build -v` shows `linker=clang` + `-Clink-arg=--ld-path=wild`; the
  full secrets stack (engine/secretd/secretctl/libsql/tonic) links cleanly via wild (15.5s, exit 0);
  the wild-linked `envctl --version` runs. Lock content_hash regen (count 74, extended not added).
  `mold` drop deferred → TASK-0070. (Implementer agent hit the weekly model limit mid-cycle after
  writing the edits; orchestrator completed the build-verify gate + guardian + commit.)
- [x] **TASK-0055 (H, EASY) — DONE 2026-06-23 via PR #184 (MERGED):** `kache` wired as
  RUSTC_WRAPPER via a delimited `[build] rustc-wrapper = "kache"` block in the meta-root
  `$M/.cargo/config.toml` — CO-MANAGED with the wild-linker component (TASK-0054) via a
  non-clobbering block-upsert (each owns a `# >>> … <<<` block; wild migrated off its wholesale
  `cat >`). Chose the cargo config over the env-seam because the seam isn't sourced in
  non-interactive/login/build contexts; the config is read by cargo everywhere. Runtime-verified:
  a throwaway build with the config (isolated XDG_CACHE_HOME) is kache-intercepted (index.db
  written). `hurry` DROPPED; `zccache`/`sccache` noted as alts.
- [!] **TASK-0056 (H, EASY) — BLOCKED:** `archon` relink is NOT a free symlink — **`meta/Archon`
  is not cloned/built locally** (the `.meta.yaml` peer exists but no checkout; `meta/Archon/target/
  release/archon` absent). The live `/usr/local/bin/archon` is a 105 MB ELF of unknown provenance.
  PRE-REQ: `meta git update` (clone Archon) + build it, THEN add to `meta-tool-links` + remove the
  stray `/usr/local/bin/archon` (sudo). Reclassify EASY→MEDIUM (needs clone+build first).
- [x] **TASK-0057 (H, EASY) — DONE 2026-06-23:** `gh-cli` component authored + APPLIED + verified
  — release tarball → `.toolchains/gh` + `~/.local/bin/gh` symlink (gh 2.95.0, healthy). Remaining
  cleanup: drop the apt `gh` (sudo `apt remove gh`) — meta gh is already primary on PATH.
- [x] **TASK-0058 (H, EASY) — DONE 2026-06-23:** `nushell` + `zellij` components authored
  (`epic-h-toolchains.toml`) + APPLIED + verified — musl tarballs → `.toolchains/{nushell,zellij}` +
  `~/.local/bin/{nu,zellij}` symlinks (nu **0.113.1**, zellij **0.44.3**, both healthy). Removes nix
  as the *delivery path* for interactive nu/zellij; yazelix's own bundled copies (absolute nix path)
  are untouched (additive). NOTE skew: interactive nu 0.113.1 vs yazelix-bundled nu 0.112.2 —
  reconciled fully by TASK-0064 (yazelix de-nix). Repos: nushell/nushell, zellij-org/zellij.
- [x] **TASK-0059 (H, EASY) — DONE 2026-06-23:** `mise` component authored + APPLIED + verified —
  static binary → `.toolchains/mise/bin/mise` + `~/.local/bin/mise` (mise **2026.6.12**, healthy);
  replaced the prior real-file `~/.local/bin/mise`. `MISE_DATA_DIR` already meta. Repo: jdx/mise.
- [x] **TASK-0060 (H, MEDIUM) — DONE 2026-06-23 via PR #172 (MERGED 08:52:09Z, squash `1ad57a8`):**
  `ollama` component authored (`components.d/epic-h-toolchains.toml`) + APPLIED + verified on box —
  upstream zstd tarball (`ollama-linux-amd64.tar.zst`) → `.toolchains/ollama` (`bin/ollama` +
  `lib/ollama/` cuda_v12/cuda_v13/vulkan GPU runners) + `~/.local/bin/ollama` symlink (client
  **0.30.10**, auto-detect detected/healthy/wiring_present). GPU `.so` redirect wired via
  `OLLAMA_LIBRARY_PATH` in `envctl env --toolchains` (shell+json; the binary also resolves
  `../lib/ollama` from its real path). Lock regen 71→72; env integration test extended. Repo:
  ollama/ollama. **Remaining cleanup (separate, sudo):** remove the stale root `/usr/local/bin/ollama`
  real binary (shadowed by the `~/.local/bin` symlink on PATH already) + stop the old 0.30.6 root daemon.
- [x] **TASK-0061 (H, MEDIUM) — DONE 2026-06-23 via PR #175 (MERGED 09:39:19Z, squash `440faee`):**
  `llvm-clang` component (8th Epic-H tarball→`.toolchains`→`~/.local/bin`) — pins the latest **21.x**
  release via the GitHub releases API (the `/releases/latest` redirect now points at 22.x — avoided),
  downloads `LLVM-<ver>-Linux-X64.tar.xz` → `.toolchains/llvm` (`--strip-components=1`), symlinks a
  **probe-gated** curated set into `~/.local/bin`. REPLACED the pre-existing apt-based `llvm-clang` in
  `gpu.toml` (the system-depth install being eliminated), id-preserved so `cuda-oxide`/`gpu-stack`
  `requires` still resolve (last-wins → tarball def authoritative; lock `content_hash b66d8854ad82aa99`,
  count 72). `LIBCLANG_PATH` env seam added to `run_env` (JSON+shell) + env.rs test. Verified on-box:
  `✓ llvm-clang [healthy] wired`; clang/llc/llvm-config 21.1.8; `clang -print-resource-dir` →
  `.toolchains/llvm/lib/clang/21`. **Guardian-driven robustness fix:** the prebuilt `lld`/`ld.lld`
  link `libxml2.so.2` (box has `.so.16`, ABI-incompatible) → install loop now probes each candidate
  (`--version` exit 0) before symlinking and self-prunes stale owned symlinks, so `lld`/`ld.lld`
  auto-drop here (strategic linker is `wild`, TASK-0054; apt lld remains fallback) and `verify` gates
  on `clang && llvm-config` only. Curated set landed (10): clang, clang++, clang-21, clang-cpp, llc,
  llvm-ar, llvm-config, llvm-nm, llvm-objcopy, llvm-objdump.
  **Remaining cleanup (separate, sudo):** `apt remove clang clang-21 llvm-21 …` (meta clang already
  shadows `/usr/bin` on PATH via `~/.local/bin`).
- [x] **TASK-0062 (H, MEDIUM) — DONE 2026-06-23 via PR #177 (MERGED, squash `1cc08dc`):**
  `libgccjit` for `rustc_codegen_gcc` — meta-owned (9th Epic-H tarball→`.toolchains` component). Install
  reads the pinned commit from rustc_codegen_gcc's own `libgccjit.version`, downloads the `rust-lang/gcc`
  release asset `master-${COMMIT}/libgccjit.so` (commit `2f06e64…`, HTTP-200 verified) →
  `.toolchains/libgccjit/lib/libgccjit.so` (+ `.so.0` SONAME). Payload is a runtime `.so` consumed by the
  external rustc backend → **NO `~/.local/bin` symlink**; verify = file-exists + `file | grep 'shared
  object'`; self-guarded remove; idempotent re-install. `GCC_PATH` env seam added to `run_env`
  (JSON+shell, after `LIBCLANG_PATH`) — the seam rustc_codegen_gcc's `config.toml gcc-path` /
  `LIBRARY_PATH`+`LD_LIBRARY_PATH` consume. `crates/cli/tests/env.rs` extended (both toolchains tests).
  Lock regen **72→73** (additive net-new id — NOT id-preserved, no pre-existing libgccjit id). ADR row
  marked shipped. **no-C clean:** the `.so` is a `.toolchains/` runtime artifact, never a Cargo dep
  (Cargo.toml/lock diff vs develop empty) — `ci/gates/no-c.sh` provably unaffected. Verified on-box:
  `✓ libgccjit (meta-owned) [healthy] wired`; `GCC_PATH=…/.toolchains/libgccjit/lib`; libgccjit.so (426M
  ELF shared object) + `.so.0` present; absent from drift. Guardian PASS (all 8 gates green).
  **Remaining (separate, wiring):** rustc_codegen_gcc backend selection itself (nightly + `-Zcodegen-backend`
  / `config.toml gcc-path`) is the consumer-side wiring, out of this component's scope.
- [~] **TASK-0063 (H, HARD) — ARMED 2026-06-23 via PR #186 (auto-merge; re-poll → tick `- [x]`
  when MERGED):** CUDA toolkit relocated apt `cuda-toolkit-13-3` → `$M/.toolchains/cuda` via the
  runfile `cuda_13.3.0_610.43.02_linux.run` (md5 16d68669…, cached under `$M/.cache/cuda`,
  toolkit-only `--toolkit --toolkitpath … --override --no-opengl-libs --nox11`, NO sudo, NO driver).
  **KEY FIX:** the makeself self-extractor needs `--nox11` + `</dev/null` headless or it tries to
  spawn an xterm and dies `exec: -title` (exit 127). nvidia-open KERNEL driver + libcuda stay apt
  (irreducible floor). id preserved (cuda-oxide/gpu-stack edges resolve); dropped the unneeded
  `requires=["nvidia-cuda-repo"]`. Real **7.0G** install verified on-box: `✓ cuda-toolkit
  [healthy] wired`, nvcc 13.3 V13.3.33 from the meta prefix, libcublas present; live `~/.bashrc`
  CUDA_HOME converged meta-first (apt fallback). nsys/ncu included. **Owner sudo follow-up:**
  `sudo apt-get remove -y cuda-toolkit-13-3 && sudo apt-get autoremove -y`.
- [~] **TASK-0064 (H, HARD) — removes nix entirely (the real fix for `/nix`); U1 DONE, full de-nix
  RESERVED for the OWNER JOINT CLOSE-OUT:** yazelix is the SOLE reason nix exists on this box.
  **U1 fork-reconcile DONE 2026-06-23 (owner: "reconcile to FlexNetOS fork"):** FF
  `FlexNetOS/yazelix:main` ← `luccahuguet/yazelix:main` (was 0-ahead/65-behind, clean FF →
  `e09582da`, pushed; old pinned rev `e60d15e` preserved in branch `pin-meta-2026-06-12`). `yzx`
  confirmed a plain-cargo bin (`rust_core/yazelix_core` → `[[bin]] yzx`); it finds its tools via
  `YAZELIX_RUNTIME_DIR` (any dir with `runtime_identity.json` + `toolbin/`) — nix is only the
  current *builder* of that tree. **HARD PREREQ for the full de-nix (found this session):**
  `yazi` + `hx` resolve ONLY via `/nix/store` today (nu/zellij are already meta-prefix via
  TASK-0058) → meta-prefix `yazi` + `helix` components MUST be built first. **Remaining (owner
  joint close-out):** (a) build meta `yazi`+`helix`; (b) cargo-build `yzx`→`.toolchains/yazelix` +
  compose a meta-prefix `YAZELIX_RUNTIME_DIR` runtime tree (replaces `mk_runtime_tree.nix`);
  (c) verify `yzx` in a throwaway `env -i` shell (no `/nix` in resolved paths); (d) repoint the
  live yazelix-shell auto-enter (transitional fallback to nix until removed). Owner open Qs:
  ghostty GPU/nixGL replacement; pin `main` HEAD vs a tag. Then → TASK-0067 removes host `/nix`.
- [ ] **TASK-0065 (host prerequisites — DETECT/VERIFY only, NOT meta components):** Corrected
  classification (owner 2026-06-23): the nvidia-open kernel driver (pre-meta install; OS-global
  `/lib/modules`/DKMS) and the `system:` build-floor (`build-essential`/`cmake`/`pkg-config`/
  `libssl-dev`/system GCC) are **host prerequisites meta does NOT own or install** — meta only
  detects/verifies them (`gpu.toml` verify hooks). The `/nix` store is **removable, not
  irreducible** (see TASK-0064) — do not declare it a permanent `system:` component. Net: there is
  NO list of "sanctioned system installs" to formalize; only host prerequisites to verify and the
  `/nix` removal to finish.
- [~] **TASK-0066 (H, MEDIUM) — ADDITIVE COMPONENT DONE 2026-06-23 via PR #179 (MERGED, squash `70ddfc0`); destructive migration → TASK-0067:**
  nix-portable isolation (makes nix meta-owned, kills host `/nix`).
  **Shipped (additive):** meta-owned `nix-portable` Epic-H component (10th) — `DavHau/nix-portable`
  `nix-portable-x86_64` (v012) → `.toolchains/nix-portable/bin/nix-portable` + `~/.local/bin` symlink,
  fetched via **authenticated `gh release download`** (5000/hr; redirect+curl `v012` fallback), verify
  handles the self-extracting `#!/usr/bin/env bash`+ELF polyglot (`file | grep -qi executable`),
  self-guarded remove (never touches host `/nix`). Lock 73→74. Verified on-box `✓ [healthy] wired`.
  **DEFERRED to TASK-0067 (SUPERVISED):** the destructive half — remove host `/nix`, re-provision the
  LIVE yazelix terminal off Determinate nix — touches the owner's running shell, needs a human window.
  Replace the root-owned Determinate `/nix` install with **`nix-portable`** (`DavHau/nix-portable`,
  bubblewrap-backed) so nix runs fully isolated in a meta/home-owned store (`~/.nix-portable`),
  host `/nix` gone, binary cache preserved (logical `/nix/store` kept via bwrap namespace), `nixGL`
  for ghostty GPU. **Verified prerequisite on this box (2026-06-23):** use **nix-portable (bwrap)**,
  NOT nix-user-chroot — `apparmor_restrict_unprivileged_userns=1` blocks raw userns (`unshare
  --user` fails) but `bwrap` 0.11.1 works (sanctioned AppArmor profile). Steps: install
  nix-portable → `~/.local/bin`; re-provision yazelix + nu/zellij/mise via nix-portable; migrate
  off the Determinate install; remove the host `/nix`. Unblocks running yazelix nix-isolated TODAY
  (before the TASK-0064 rust-core de-nix lands). Aligns with the meta-owned-isolated nix strategy
  in the ADR §Corrected classification.
- [x] **TASK-0068 (H, MEDIUM) — DONE 2026-06-23 via PR #180 (MERGED, squash `795c2f0`):**
  Authenticated-GitHub-fetch hardening — eliminate the last `api.github.com` rate-limit liability.
  Finding: 6/7 Epic-H components already fetch via the `/releases/latest` web redirect (302, NOT the
  rate-limited JSON API) → immune; only the **`llvm`** component used `api.github.com/.../releases`
  (it must LIST releases to pin latest 21.x since `/releases/latest`→22.x), which 403s when the box's
  unauth 60/hr quota is exhausted (the failure that surfaced in TASK-0066). Fix: `llvm` now lists via
  **authenticated `gh api`** (5000/hr; gh is the meta-owned TASK-0057 component), unauth API fallback
  only if gh absent/unauthed. Resolves same `llvmorg-21.1.8`; lock content_hash regen (count 74,
  unchanged). Gates green; llvm-clang stays `[healthy] wired`. **Future (→ TASK-0069):** a shared
  `gh`→vault-App fetch path (more-isolated token) once the App is usable.
- [!!] **TASK-0067 (H, HARD, SUPERVISED — never auto-run) — destructive `/nix` removal + yazelix migration:**
  The deferred destructive half of TASK-0066. Re-provision yazelix + nu/zellij/mise to run via
  `nix-portable` (`NP_RUNTIME=bwrap nix-portable nix ...`), validated in a throwaway shell first;
  migrate the `~/.bashrc` yazelix auto-enter block (`manifest/nix-yazelix.toml`) + the
  `nix`/`home-manager`/`yazelix` components off the Determinate daemon path; retire the
  `manifest/nix-yazelix.toml` `id="nix"` component and remove the host `/nix`
  (`/nix/nix-installer uninstall`). **SUPERVISED** because yazelix is the owner's LIVE interactive
  shell — autonomous execution can break the running terminal; needs a human migration window.
  Depends on TASK-0066 (nix-portable installed — DONE).
- [ ] **TASK-0069 (H, MEDIUM, owner-gated) — make the vault GitHub App usable as the isolated token source:**
  TASK-0068 uses the `gh` keyring token (account drdave-flexnetos, 5000/hr) — the available auth. The
  more-isolated path the owner built is the vault GitHub App (`secretctl mint-github`, app-id 4044997),
  but it is NOT usable today: (1) `secretctl` is **not on PATH** (not installed to `~/.local/bin`/
  `.toolchains` — file an Epic-H component for it; note `secretd` itself runs from `~/.cargo/bin`, a
  system-depth location worth converging); (2) the App is **enrollment-blocked (404)** — needs the
  ORIGINAL `app.pem` for app-id 4044997 (`secretctl github-app enroll --app-id 4044997 --private-key
  <original-app.pem>`); the vault copy is broker-only/un-revealable by design. **OWNER ACTION REQUIRED**
  for the enrollment + supply of the original pem. Once unblocked: add a shared `gh`→App-installation-
  token fetch helper so Epic-H GitHub fetches use the App (higher quota, scoped, auditable) and route
  through `flexnetos_runner` for CI/job dispatch.
- [x] **TASK-0070 (H, EASY) — DONE 2026-06-23 via PR #185 (MERGED):** dropped mold now that wild is
  the wired linker (TASK-0054). codex-cli `requires` `mold-linker`→`wild-linker`; removed the
  `-fuse-ld=mold` RUSTFLAGS from the codex install/fix hooks (it would OVERRIDE the wild config —
  RUSTFLAGS env > config rustflags — and silently bypass wild); removed the `mold-linker` apt
  component (`dev-tools.toml`); lock 74→73. Verified: codex-cli + wild-linker `[healthy] wired`,
  meta-tree build links via wild. **Owner sudo follow-up:** `sudo apt-get remove -y mold`.

## Key finding (carried)

Most meta-built tools' installed binaries are NEWER than their committed meta sources
(kasetto 3.1.0>3.0.0, rtk 0.42.2>0.42.0) → meta is OUT OF SYNC with what's deployed. The real
work is **sync-meta-source-UP-then-relocate**, not a symlink sweep.
