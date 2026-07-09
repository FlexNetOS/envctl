# PLAN — agentic-os-blueprint (workspace alignment to the bare-metal Agentic-OS blueprint)

- Date: 2026-07-09 · Target: `agentic-os-blueprint` (fleet-level: envctl · nu_plugin · meta-ruvector · yazelix · handoff · var-runtime)
- Evidence: `findings/verdicts-agentic-os-blueprint.md` (V1–V30; CONFIRMED/QUALIFIED only enter this plan)
- Provenance: 6-auditor blueprint audit + adversarial verify (wf_97a0b5a7-fb9) + **manual runtime re-verification** (live psql / bun drives / binary executions, 2026-07-09)

## 1. Verdict

**The workspace is actively converging on the blueprint (verified alignment 57/100, up from a
first-pass 45) and the correct move is to finish six in-flight convergences rather than build
anything new**: fix the 3-bug `swarm-immune` wrapper (the MinCut immune system already exists and
its native layer is proven exact — V4), calibrate the RuvLTRA tier before its router branch merges
(it runs but routes everything to opus — V7), redeploy the stale envctl binary (V9), wire the
proven MiniLM embedder into the flush (V8), merge the two waiting branches (`codedb_store_pg`,
`bun-rewrite` — V26/V16), and declare the running Postgres as envctl state (V21). ATAS/ESN is the
only blueprint pillar with no code anywhere — route it to Feature Forge as a green-field feature on
the existing `sona::trajectory` substrate (V17), not as a "deviation."
**Confidence: Medium-High** — every roadmap row below is backed by a runtime capture or file:line
from this cycle; what caps it is the missing web-currency/advisory pass (R7 input) and the
untracked provenance of the `codebase` loader.

## 2. Architecture diagrams

### 2a. Current architecture — the proven spine + the var-runtime vector layer

```
Source: nu_plugin/docs/ARCHITECTURE.md (data-flow) ; nu_plugin/docs/ENVCTL_EXPORT_CONTRACT.md:43-55 ;
        envctl crates/engine/src/migration_db/mod.rs:1-15 ; runtime captures V1-V5,V8 (2026-07-09)

  workspace files                 store plane (redb, pure-Rust)          flat-tree exports
  ┌──────────────────┐   ┌──────────────────────────────────┐   ┌─────────────────────────┐
  │ Rust sources      │──▶│ codedb capture [A]                │──▶│ codedb export envctl [A] │
  │ (syn 2 AST rows,  │   │ codedb_store_redb: sha256 blobs,  │   │ checksum-bound rows      │
  │ cargo-metadata)   │   │ validation, source_files          │   │ redb_access=forbidden    │
  └──────────────────┘   └───────────────┬──────────────────┘   └───────────┬─────────────┘
                                          │                                   ▼
                                          │                    ┌─────────────────────────┐
                                          │                    │ envctl = materializer [P]│
                                          │                    │ + migration_db (redb,    │
                                          │                    │ hash-chained, R3 [H])    │
                                          │                    └─────────────────────────┘
  var-runtime vector layer (bun, OUTSIDE the Rust trust boundary)
  ┌──────────────────────────────────────────────────────────────────────────────────┐
  │ agentdb sql.js containers agents/*.rvf.db ──flush [A]──▶ Postgres 17.10 (socket)   │
  │ embed-minilm.mjs (384-d, proven 0.711/0.016) ──UNWIRED──▶ ruvector 0.3.0 ext        │
  │ swarm-immune.mjs ──⚠ 3 wrapper bugs──▶ ruvector_mincut.node (exact, dynamic)       │
  │ gguf-proof: frozen 1.1B console + LoRA hot-swap 0.013ms  [proven live]             │
  └──────────────────────────────────────────────────────────────────────────────────┘
  ⚠ split-brain: semantic_embedding 5157/5157 (fallback 1536-d) vs embedding_minilm 0/5157
```

### 2b. Target state (P7) — after this roadmap lands

```
Source: this plan §3 (rows R1-R10) ; ENVCTL_EXPORT_CONTRACT.md (transport stays) ; ADR-0004 draft

  capture ──▶ codedb redb ──export──▶ envctl materializer [P]
                   │                        │
                   └──BlobStore trait──▶ Postgres+ruvector (envctl-managed component [P])
                        (R5: pg backend merged)   │ episodes + codebase, MiniLM 384-d (R3)
                                                  ▼
  agents ◀──tier──[RuvLTRA FastGRNN, calibrated R4]──── harness router [A]
     │                                                     (session plane stays Fable — Law 8)
     └──coordination graph──▶ swarm-immune (fixed R1) ──▶ isolate/sever [A, fail-closed]
  release lane: cargo gnu (dev) + musl static engine+cli (R9) ──▶ install-in-place binaries
```

### 2c. Control plane (governance → runtime/build bind)

```
Source: home/.claude/rules/laws.md (8 laws) ; ~/.claude/hooks/ listing (V-capture) ;
        .claude/settings.json AGENTDB_* ; ci/gates/*.sh (CLAUDE.md §CI gates) ; runtime capture V7

  laws.md (1-8) ──governs──▶ hooks: guard-bash/guard-write-paths/harness-archive [A]
      │                      statusline reroute-alarm [A] · stop-decision-gate [H]
      │                      ruvector-intel-bridge ──feeds──▶ meta-ruvector/.claude intelligence
      │                            └──▶ router.js [A] (tier-less live; RuvLTRA tier on worktree ⚠)
      ├─Law 8──▶ session model = Fable [H operator] ; subagent tiers = router (ADR-0004 draft)
  settings.json ──configures──▶ AGENTDB_PATH/RUNTIME/FORCE_SQLJS ▶ bun runtime
  rust-toolchain.toml + fenix profile ──▶ cargo build ──gates──▶ ci: no-c, shape, p7, agent-env,
                                                     loop-state, harness-scripts [A, fail-closed]
  .handoff/policy + loop_state ──gates──▶ plan-loop cadence · WRAP-UP-OWED marker [A]
  ⚠ drift: MEMORY says bun-rewrite enforced; hook absent live (V16) · deployed envctl stale (V9)
```

### 2d. Fleet level

```
Source: meta/.meta.yaml (repo DAG) ; verdicts V1-V28 fleet columns

  meta (.meta.yaml DAG) ─────────────────────────────────────────────┐
   ├─ envctl (env manager · migration_db · materializer-by-contract) │ shared substrates
   ├─ nu_plugin (codedb capture/store/export; pgstore branch ⚠)      ├─ weave/repowire (A2A bus)
   ├─ meta-ruvector (LOCAL monorepo, stale vs registry; router-wt ⚠) ├─ handoff (SFVR rvf ledger,
   ├─ yazelix (terminal OS; ccboard TUI pane; runtime tree)          │   witness chains V13)
   └─ rusty-idd-unified (the A/B corpus in Postgres `codebase`)      ├─ var/lib/ruvector (vector
  crates.io pins (NEVER local checkout): ruvector-core 2.2.3,        │   runtime: pg, mincut,
    ruvllm 2.3.0, rvf-runtime 0.3.0, ruvector-postgres 2.0.5 ⚠skew   │   models, agents)
                                                                     └─ var/lib/agentdb (memory)
```

## 3. Sequenced upgrade roadmap

Ordered by value/risk (bounded-blast fixes to already-working substrate first; branch merges next;
new lanes last). Every row cites its verdict; acceptance is 1:1 with a RED test in §13.

1. **UPGRADE: fix `swarm-immune.mjs` wrapper (string→u32 id map + method calls + name-mapped cut set)** | axis: accuracy | rationale: the immune system's native layer is proven exact; 3 call-shape bugs keep the blueprint's hallucination-isolation dark | evidence: V4 captures | blast: `var/lib/ruvector/` only (no Rust, no gates) | effort: XS | risk: low → **APPLY** | acceptance: immune-drive returns `connected()==true`, weighted boundary `[researcher→merge-resolver]` after degrade (currently RED: `NumberExpected`) | reversibility: full (Law-1 archive of the one file) | verdict: V4 CONFIRMED
2. **UPGRADE: rebuild + redeploy envctl so the deployed binary carries `envctl db`** | axis: quality | rationale: `usr/bin/envctl` serves a 2026-07-07 binary; GH#414 code-graph surface unreachable | evidence: V9 | blast: all envctl CLI consumers (additive verb) | effort: XS | risk: low → **REGENERATE** (`cargo build --release -p envctl`) | acceptance: `usr/bin/envctl db --help` exits 0 | reversibility: full (prior binary archived) | verdict: V9 CONFIRMED
3. **UPGRADE: wire MiniLM (384-d) as the flush/backfill embedder; populate `codebase.embedding_minilm`** | axis: accuracy | rationale: proven-discriminating local embedder (0.711/0.016) sits unwired while the fallback emits 1536-d; the minilm column and its hnsw index are 0/5157 | evidence: V8, V2, V3 | blast: var-runtime flush + `ruvector` DB (schema additive) | effort: S | risk: med (dimension doctrine 384 vs 1536 — decide in review) → **PROPOSE** | acceptance: `SELECT count(embedding_minilm) FROM codebase` = 5157 AND manifest `model` ≠ "fallback" | reversibility: column re-nullable; manifest reverts | verdict: V8 CONFIRMED
4. **UPGRADE: calibrate the RuvLTRA FastGRNN tier BEFORE merging `codex-ruvltra-router`** | axis: accuracy | rationale: tier runs live but is a constant (both test prompts → opus @~0.55); merging now would tier-inflate every routed task | evidence: V7 | blast: harness router = every UserPromptSubmit fleet-wide | effort: S | risk: med → **PROPOSE** (merge-gate) | acceptance: 10-prompt fixture routes trivial→haiku ∧ complex→opus, reproducible ×3 runs | reversibility: tier block is fail-closed additive (absent ⇒ pure keyword, observed) | verdict: V7 QUALIFIED(calibration)
5. **UPGRADE: merge `codedb_store_pg` (BlobStore trait + Postgres backend)** | axis: quality | rationale: the blueprint's redb↔postgres hop exists as a tested drop-in backend, unmerged since 4c2fef4 | evidence: V26 | blast: nu_plugin store plane (trait-gated; redb default unchanged) | effort: S | risk: med → **PROPOSE** (PR to develop) | acceptance: master carries `crates/codedb_store_pg` + differential redb↔pg blob parity green | reversibility: backend selectable; redb remains default | verdict: V26 CONFIRMED
6. **UPGRADE: land `feat/bun-rewrite-hook` (or amend MEMORY if declined)** | axis: governance+settings+config | rationale: memory asserts enforcement the box lacks; superset fix is merging 1889fb8 (hook + settings wiring) | evidence: V16 | blast: every agent Bash call (rewrite npm/npx/pnpm→bun) | effort: S | risk: med → **PROPOSE** | acceptance: `~/.claude/hooks/bun-rewrite.sh` present + wired, `npm install` observed rewritten; else MEMORY corrected | reversibility: hook removal restores prior behavior | verdict: V16 CONFIRMED
7. **UPGRADE: `postgres-ruvector` manifest component (declare the running cluster)** | axis: governance+settings+config | rationale: the global-brain store is a hand-started process with no unit/component — violates envctl's own declarative ethos | evidence: V21, V1 | blast: env-manager control plane (additive component) | effort: S | risk: low → **PROPOSE** (component TOML drafted in the audit, detect/install/verify/fix) | acceptance: `envctl auto-detect` lists `postgres-ruvector` present; verify hook green | reversibility: component removal leaves cluster untouched | verdict: V21 CONFIRMED
8. **UPGRADE: reconcile ruvector extension 0.3.0 ↔ client crate 2.0.5 skew** | axis: quality | rationale: server extension and client pin disagree by two majors; a `2.0.0--0.3.0` downgrade script in `ext/` says this bit someone already | evidence: V20 | blast: global-brain store + future envctl `ruvector-pg` feature consumers | effort: M | risk: med → **PROPOSE** | acceptance: `extversion` == a client-supported version, or the pairing pinned in an ADR note | reversibility: extension ALTER path exists both directions | verdict: V20 CONFIRMED
9. **UPGRADE: musl static lane for `envctl-engine`+`envctl` (fenix musl std + `.cargo/config.toml` target block)** | axis: quality | rationale: the blueprint's one REQUIRED fix; envctl is uniquely musl-ready (no-C gate already bans every C lib) | evidence: V10, V11 | blast: build pipeline only (new target dir; gnu lane untouched) | effort: M | risk: med → **PROPOSE** | acceptance: `file target/x86_64-unknown-linux-musl/release/envctl` → "statically linked" ∧ `ci/gates/no-c.sh` green | reversibility: additive target | verdict: V10 QUALIFIED(fleet-CI)
10. **UPGRADE: first envctl ruvector consumer — HNSW index over `codedb export` rows behind the default-OFF `ruvector` feature** | axis: quality | rationale: turns the feature-gated dead-weight pins (V23) into the contract-sanctioned semantic layer (V12) without touching default builds | evidence: V12, V23, V22 | blast: engine (feature-gated module) | effort: M | risk: med → **PROPOSE** | acceptance: feature-gated test answers top-k over exported rows; default `cargo build` bit-identical behavior; no-c green | reversibility: feature stays default-OFF | verdict: V12+V23 CONFIRMED

## 4. Tool-evaluation (R7)

Currency/advisory web pass **did not run this cycle** — calls below are grounded in observed usage
+ on-box versions; CVE column is an explicit gap (see §14).

| tool | observed | call | rationale (evidence) |
|------|----------|------|----------------------|
| ruvector-core | 2.2.3 (crates.io) | **pin** | operator directive "registries not repo" (V22); SIMD real (V23) |
| ruvllm | 2.3.0 (crates.io) | **pin** | frozen-console+hot-swap proven via gguf-proof (V5); cuda opt-in awaits nvcc lane |
| ruvector-postgres (crate) ↔ ruvector ext | 2.0.5 ↔ 0.3.0 | **reconcile** (R8) | two-major skew + downgrade script on disk (V20) |
| rvf-runtime / rvf-types | 0.3.0 / 0.2.1 | **pin** | SFVR ledger production consumer in handoff (V13); no ruvllm bridge upstream (V29) |
| redb | 2.6 (resolves 2.6.3) | **hold** | ≥3.0 wants rust 1.89 vs MSRV (Cargo.toml:99-101 comment); migration_db + codedb depend on it |
| agentdb (npm) | 3.0.0-alpha.17 | **pin exact** | alpha; native better-sqlite3 unbuilt ⇒ `AGENTDB_FORCE_SQLJS=1` contract (manifest, V3) |
| bun | 1.3.13 | **hold** | sole sanctioned JS runtime; enforcement pending R6 (V16) |
| ollama | live pid 3904 | **hold — do NOT remove** | replacement is shimmy+ruvllm; parity NOT yet proven (layout.rs:192-197); gguf-proof advances readiness but is a proof, not a daemon |
| shimmy+ruvllm lane | proof-stage | **advance** | V5 proves the core loop; next: serving surface + parity harness before any ollama removal |
| RuvLTRA GGUFs | 3 models (0.5b×2, 1.1b) pulled 03:59-04:00 | **calibrate** (R4) | live tier non-discriminating (V7) |
| postgres | 17.10 (nix store, pg17-rw) | **hold + manage** | healthy but unmanaged (V21) → R7 component |
| ccboard | vendored crate, TUI pane | **hold (TUI-only)** | axum web mode dormant; TUI wiring is blueprint-consistent (V15/V28) |
| rtk / hf / git-kb / meta CLI / weave | live | **hold** | rtk rewrite observed in-session; weave/repowire = A2A bus (V-fleet); no currency data this cycle |
| MCP baseline (github/context7/exa/memory/playwright/seq-thinking) | configured | **hold** | parity across .mcp.json/.codex per CLAUDE.md; rot-check needs the currency pass |

## 5. Governance, settings & config

- **Law 8 two-plane resolution** (V18): session model stays operator-pinned (Fable, statusline
  reroute-alarm live); subagent/worker **tier** dispatch via RuvLTRA router is the operator's own
  2026-07-09 directive. Encoded as **ADR-0004 (draft)** — calibration-gated (R4) so it cannot merge
  as a constant-opus tier-inflator.
- **Drift found (fail-closed items)**: MEMORY-vs-box on bun-rewrite (R6); deployed-binary-vs-source
  (R2); mission-control.kdl undeployed while yazelix layout carries ccboard (V15) — pick one
  mission-control source of truth; `home/.claude/settings.json` is live-modified in the main
  checkout (uncommitted) — reconcile before the next home/ commit.
- Settings hygiene: `AGENTDB_*` env contract documented in the manifest and settings match (V3);
  hooks chain is guard-heavy and archive-first (Law 1) — no change recommended.
- Routing: APPLY only R1 (var-runtime JS, no gate surface); everything touching hooks, stores,
  router, build lanes is PROPOSE via PR to develop per git-topology.

## 6. Filesystem layout

Qualified (no dedicated auditor artifact this cycle; verified observations only): var-runtime state
correctly lives under `var/lib/{ruvector,agentdb,postgresql}` (FHS-consistent); `usr/bin` deploy-by-
symlink to `target/release` is the staleness vector behind R2 — consider a copy-on-release step;
`agents/*.rvf.db` naming misstates the format (SQLite — V14): rename to `.agentdb.sqlite` or make
them real SFVR containers when the bridge lands; `var/lib/ruvector/pgdata` is a foreign-uid 700 dir
(V21) — explain or archive; `ext/` build artifacts beside `pgrx/` are acceptable build provenance.

## 7. Memory / vector intelligence

Live: agentdb reasoningbank (SQLite via sql.js) + 5 role containers + episodes flush proven at 1 row
(V3); GitKB present; ICM absent by design (mandate archived 2026-07-07). Gap: no cold-start recall
proof ran this cycle; the `codebase` semantic index is fallback-embedded (R3 fixes); MEMORY.md
carried one box-contradicting row (V16) — memory hygiene rule: enforcement claims require the
hook file to exist at write time.

## 8. Auto-research cadence

Gap this cycle: no web/trend/CVE pass (R7 column empty). Known anti-pattern on file: the
`ruvector/runtime` `.claude-flow` daemon re-runs identical no-op findings every 10-20 min (memory,
2026-07-09) — cadence without delta-detection is burn. Recommendation: next plan-loop cycle runs
`plan-trend-researcher` for the §4 inventory with the 90-day recency gate; stale-evidence
invalidation applies to V-rows older than one cycle.

## 9. Rules / policy / agent org

Upgrade-only honored — every roadmap row is additive or a merge of existing work; the cycle's own
lesson (recorded): treating in-flight convergence as "deviation to defend against" inverted Laws 2/8;
the blueprint is the operator's target, worktree-branch work is convergence, and the plan's job is
sequencing it safely. Org: orchestrator + specialist agents at depth-1, A2A over weave/repowire
(V-fleet), human gates at R3 approvals / AskUserQuestion / `[!!]` markers — unchanged.

## 10. Distributed compute fabric

Honest gap: single workstation (dual RTX-5090, CUDA 13.3) — no Lua/mobile/Pi/ESP32 surface exists
in the workspace today; `cuda-oxide` repo + engine `ruvector-cuda` opt-in are the GPU lane seeds.
Blueprint's edge-fabric claims are out of scope until an operator-declared hardware matrix exists.

## 11. Prompt-architecture

The UserPromptSubmit banner IS `router.js` output (V7) — the router is a live prompt-architecture
component, currently tier-less in main; ADR-0004 covers its upgrade path. Skill surface is large
(50+ skills); no overload incident observed this cycle — defer pruning to a dedicated audit.
Hidden coupling worth naming: `ruvector-intel-bridge.sh` funnels all hook events into
meta-ruvector's `.claude` intelligence store — meta-ruvector's dirty working tree (5 modified
intelligence files) is therefore session-state, not code drift.

## 12. Risk policy / backend / interop

HITL stays: R3 approval gate (migration engine), AskUserQuestion blocking (Law 7), `[!!]`
supervised refusal. Backend matrix: Rust (cargo/fenix) for the trust boundary; bun-only JS for the
var-runtime lane (outside the boundary — the SQLite in agentdb is legal there and only there).
Interop: weave (cross-identity bus) + repowire (per-repo jobs) + MCP baseline; no new lanes needed
for this roadmap.

## 13. Test Strategy & Coverage

Coverage today (call-graph reachable): the spine is tested (migration_db state machines, codedb
capture/materialize, runner parity) but **every §3 acceptance lacks an executable guard**. Designed
suite — all additive, all RED now by construction (each encodes an acceptance not yet met):

| # | case (symbol/flow) | assertion | type | RED-now evidence |
|---|--------------------|-----------|------|------------------|
| T1 | `swarm-immune.immuneGraph` drive | named boundary + `connected()===true`; degrade shifts weighted cut | bun integration (var-runtime) | V4 capture (`NumberExpected`) |
| T2 | deployed-binary freshness | `usr/bin/envctl db --help` exit 0 | smoke (bash) | V9 capture (unrecognized subcommand) |
| T3 | embedder wiring | `count(embedding_minilm)==5157` ∧ manifest model ≠ fallback | sql + json golden | V8/V2 (0/5157) |
| T4 | router discrimination | 10-prompt fixture: trivial→haiku ∧ complex→opus, ×3 stable | bun golden fixture | V7 (both→opus) |
| T5 | BlobStore parity | same capture → redb backend ≡ pg backend (blob sha set) | Rust differential | V26 (pg backend unmerged) |
| T6 | musl lane | `file` on musl-target envctl says "statically linked"; no-c green | CI gate + build smoke | V10 (target absent) |

Golden fixtures to capture: the 10-prompt router set; one codedb capture tree for T5; the
immune-drive graph for T1. CI gates touched: `no-c.sh` (T6), a new `harness-scripts`-family check
for T2. Symbols: `codedb_core::store::BlobStore`, `envctl-engine::migration_db`, router.js
`ruvltraRoute`, `swarm-immune::immuneGraph`.

### FF test-build spec

Feature Forge intake (`feature-architect ## Verification plan`): build T1–T6 as specified; T1/T4
live under `var/lib/ruvector/tests/` (bun, run via `bash -lc` from the runtime dir), T2/T6 as gate
scripts in `ci/gates/`-style bash with hermetic guards, T3 as a psql assertion script beside the
flush, T5 as a `#[test]` differential in `nu_plugin` once R5's PR is open. Tests must be RED before
their paired roadmap row lands and GREEN after; no production code in the test commits.

## 14. Gaps

- **ATAS/ESN** — zero implementation anywhere (V17). Not a roadmap row: green-field `feature:`
  routed to Feature Forge — reservoir/ESN over `ruvector-sona::trajectory` (the lock-free trajectory
  buffer) + the intel-bridge Q-learning trajectories. Buildable substrate exists; the blueprint's
  "strange attractor timeline simulation" remains unbuilt and unscored.
- **Tool currency/CVE pass missing** — §4's advisory column is empty; next cycle runs the
  trend-researcher (90-day gate).
- **`codebase` loader untracked** — 5157 embedded rows whose producer is not in tracked source;
  provenance gap until the flush pipeline (R3) re-materializes it reproducibly.
- **`.rvf` → ruvllm adapter bridge absent upstream** (V29) — the blueprint's keystone link needs an
  upstream release or an in-house bridge crate; revisit after R10.
- **COW merge-half absent upstream** (V24) — branch/snapshot exist; merge does not. Watch item.
- **Unbenchmarked**: frozen-console generation *quality* (latency observed, quality not assessed —
  needs an eval harness); mincut at scale (correctness proven on toy graphs only).
- **pgdata foreign-uid dir** unreadable (V21) — needs the operator's account of its origin.
- **Notable REFUTED overclaims** (recorded, never re-admitted): see verdicts ledger footer — incl.
  this cycle's own first-audit errors and the router commit's "haiku PROVEN".

## 15. Confidence

**Medium-High.** High on every runtime-observed row (V1–V16: live captures, reproducible commands);
Medium on fleet-wide generalizations (single-cycle, no web-currency pass, loader provenance open).
Raises to High when: T1–T6 exist and run in CI (closing the acceptance loop), the trend pass fills
§4's advisory column, and R3 re-materializes the `codebase` embeddings reproducibly.
