# weave — decision-grade plan (cycle 4, A2A transport plane)

- Target: **weave** — the fleet's agent-to-agent (A2A) TRANSPORT plane (SQLite-mailbox broker +
  terminal-pane injector). DISTINCT from handoff's witnessed-receipts plane.
- Source (READ-ONLY): `/home/drdave/Desktop/meta/.worktrees/plan-weave-red/weave` @ `4fe2419`,
  branch `plan/weave-red-tests`.
- Built from: `findings/verdicts.md` (16 CONFIRMED / 4 QUALIFIED / 0 REFUTED / 9 FEASIBLE +
  1 feasibility-QUALIFIED), the 8 axis findings, `graph/weave.{graph.md,metrics.json}`,
  `reports/codemap-weave.md`, `research/weave.trends.md`.
- Date: 2026-06-26. Docs-only synthesis; no production code touched. ADR/ROADMAP rows are DRAFTS
  (owner-wall: never written into weave's own tree).
- Legend (DIAGRAMS.md): `[A]` automated · `[A*]` elevated/sudo · `[P]` preview/dry-run ·
  `[H]` human-gated · `[!!]` supervised/critical.

---

## Verdict

**weave is a cleanly-layered, dependency-hygiene-positive A2A transport plane that is
architecturally adapter-ready — and the single highest-value convergence move is an additive,
default-off A2A v1.0 interop adapter attached at the `Store`/`Intent` seam, with the already-committed
RED suite (`weave-core/tests/a2a_interop.rs`, tests-ran 3) as its acceptance contract.**

The structure is sound and the gate confirmed it: a strictly-downward 4-crate spine
(`weave-core <- weave-inject <- weave-mcp <- weave`) with **zero internal deps in `weave-core`** and
**zero repo-escaping path deps** (ARCH-01, codemap §Dependency hygiene); one real broker abstraction
(`Store` trait, `weave-core/src/store.rs:73`, **~90 methods** — not 29; ARCH-03 QUALIFIED); one
load-bearing wire schema (`Intent`, `weave-core/src/model.rs:216`, blast 1238; ARCH-04); and an
owner-only, idempotent, guarded deliver verb (`SqliteStore.send`, `store.rs:3153`; ARCH-05). There is
**no A2A v1.0 / gRPC / AgentCard / JSON-RPC-A2A adapter anywhere** (ARCH-09 CONFIRMED, grep-empty);
convergence is schema-mapping work over a stable seam, and the RED suite already encodes the contract.

The debt is real but contained: a **9,631-line `main.rs` god-file** (ARCH-07 CONFIRMED, exact), an
**unenforced dual-backend parity invariant** (ARCH-11 CONFIRMED — and the gate found a real divergence:
`LibsqlStore.send` guards `guard_writable()` before `check_ident`, `SqliteStore.send` does **not**),
a **hand-mirrored CLI↔MCP verb surface** (71 CLI verbs / **72 MCP dispatch arms / 76 catalog
entries** — not 78; ARCH-06 QUALIFIED), and governance drift (documented "6" CI gates vs **7**
enforced incl. `audit`; Python in CI vs the no-Python Rust-native invariant — both CONFIRMED).

Tool currency is healthy: one trivial bump owed (`rusqlite 0.40.0 → 0.40.1`); the entire advisory
budget (5 RUSTSEC ids) is scoped to the optional `libsql` remote-TLS feature, upstream-blocked, and
CI-gated — the default `sqlite` build is advisory-clean.

**Overall confidence: HIGH** (see `## Confidence`). The plan is built only from CONFIRMED/QUALIFIED +
feasibility-passed rows; refuted overclaims and the one feasibility-qualified item are reported under
gaps, not smuggled into the roadmap.

---

## ASCII architecture

### A. Current structure (compiler-enforced layering, from `graph/weave.graph.md` §1)

`Source: graph/weave.graph.md:11-36`

```
                 ┌───────────────────────────────────────────────┐
   bin / CLI     │  weave        (src/main.rs, 9631 lines)        │  blast→427
   71 verbs      │  71 Cmd:: arms · two sequential matches        │
   [H] serve     │  :4494 pre-store · :4660 post-store dispatch    │
                 └───────┬───────────────┬───────────────┬────────┘
                         │ depends-on     │               │
                 ┌───────▼───────┐        │               │
   MCP plane     │  weave-mcp    │        │               │  blast(mcp.rs)→124
   72 arms /     │  call_tool    │        │               │
   76 catalog    │  flat router  │        │               │
                 │  obscura·http │        │               │
                 └───┬───────┬───┘        │               │
                     │       │ depends-on │               │
              ┌──────▼──┐    │     ┌───────▼───────────────▼─────┐
   injector   │ weave-  │    └────▶│  weave-core   (the spine)   │ blast(model.rs)→1238
   7 muxes    │ inject  │─────────▶│  store(SqliteStore broker)· │ blast(store)→462
   [A] local  │         │ depends  │  store_libsql(LibsqlStore)· │ blast(store_libsql)→488
              └─────────┘   -on    │  model(Intent)·config·sign· │ blast(config)→345
   blast→205                       │  webpolicy·memory·archive   │
                                   └─────────────────────────────┘
   Manifest DAG:  weave-core (0 internal deps) ◀ inject ◀ mcp ◀ weave
   No upward Cargo dep exists → layering CLEAN at compile time. 0 escaping path deps.
```

### B. Target structure (additive A2A adapter + extracted dispatch + parity harness)

`Source: findings/architecture-weave.md UPGRADE rows U-ARCH-1..4; verdicts.md U-ARCH-2`

```
   ┌──────────────────────────────────────────────────────────────────┐
   │  weave (bin) — main.rs as THIN clap dispatcher (target <2000 ln)  │
   │     └─▶ weave/src/dispatch/*.rs  per-verb handlers [!!] extracted  │
   └───────┬──────────────────────────────────────────────────────────┘
           │ + cross-guard test: 71 CLI ⇄ 72 MCP arms ⇄ 76 catalog [A]
   ┌───────▼───────────────────────────────────────────────────────────┐
   │  weave-mcp — flat router unchanged                                  │
   │     + (optional) a2a.rs surface (default-off `a2a` feature) [P]     │
   └───────┬────────────────────────────────────────────────────────────┘
   ┌───────▼────────────────────────────────────────────────────────────┐
   │  weave-core — the spine (Store/Intent seam = adapter attach point)   │
   │   ┌──────────────────────────────────────────────────────────────┐  │
   │   │  NEW a2a.rs (or model.rs fns): to_a2a / from_a2a  [P] add-only │  │
   │   │   Intent ⇄ A2A Message · {jsonrpc,method:"message/send",...}   │  │
   │   │   AgentCard signing rides existing `sign` (ed25519, default-off)│  │
   │   └──────────────────────────────────────────────────────────────┘  │
   │   SQLite-mailbox transport UNCHANGED (strict upgrade — never removed) │
   │   + tests/store_conformance.rs: run over BOTH dyn Store impls  [A]    │
   └──────────────────────────────────────────────────────────────────────┘
   RED contract (committed, FAILING): weave-core/tests/a2a_interop.rs (3 cases)
```

### C. Control plane (transport + authorization, from rules-policy + governance findings)

`Source: findings/rules-policy-org-weave.md §2,§4; findings/governance-config-weave.md`

```
   OWNER (revenaugh.david) ── risk/governance boundary [H]
        │ approves STRUCTURAL + security-config
        ▼
   weave-orchestrator (skill, model=opus) ── commander
        ├─ weave-planner   [A] map→layers
        ├─ weave-implementer[A] write Rust, mirror BOTH backends
        ├─ weave-verifier  [A] full gate on sqlite+libsql, RED→route-back
        └─ weave-guardian  [!!] invariant+drift gate; APPROVE/BLOCK wins
   background loop lane (ralph-weave.sh):
        Phase A worker = claude/opus [A]
        Phase B GUARDIAN = MiniMax minimax-m3:cloud [!!] cross-vendor — ADR-uncovered
   live-tool authorization (default-OFF, opt-in):
        weave hook pretooluse [!!] deny-by-default → routes ask over weave's OWN mailbox
```

---

## Sequenced upgrade roadmap

Ordered by **value/risk using graph centrality + blast-radius**: contained-blast / high-value moves
first; the highest-blast change (`main.rs` extraction, blast 427 on the dispatch god-file) is
sequenced **last**, behind the safety-net items (parity harness + verb-parity test) that make a large
move safe. Every row traces to a CONFIRMED/QUALIFIED verdict and a FEASIBLE upgrade.

Columns: **axis · target-surface · evidence · blast · effort · risk-tier · P8-test · reversibility.**

### R1 — A2A v1.0 interop adapter (the headline convergence)  [PROPOSE-additive]
- **axis:** accuracy
- **target-surface:** new `weave-core/src/a2a.rs` (or `to_a2a`/`from_a2a` on `model.rs`) + new
  `weave-mcp` A2A surface, default-off `a2a` feature; AgentCard signing via existing default-off
  `sign` (ed25519-dalek).
- **evidence:** ARCH-09 CONFIRMED (no A2A surface exists, grep-empty); U-ARCH-2 FEASIBLE — rides the
  already-present `serde_json` (no new dep) and pure-Rust `ed25519-dalek` (no C in trust boundary);
  research §A1/§A2/§E1 (A2A v1.0 = JSON-RPC 2.0/SSE/gRPC + signed AgentCards; `sign` is the local
  analogue); RED suite `weave-core/tests/a2a_interop.rs` tests-ran 3 all-RED-on-assertion.
- **blast:** 1238 (`model.rs` Intent schema) — **contained** because additive: new fns/module, never
  mutate `Intent`'s existing serde; native Tier-2 goldens (`integration.rs:3541/3646`) stay GREEN.
- **effort:** L
- **risk-tier:** PROPOSE (new public protocol surface at the highest-blast schema; default-off keeps
  blast contained). See `risk-policy.md`.
- **P8-test:** the committed RED suite is the acceptance contract — `to_a2a` Message mapping
  (`kind`/`role`/`messageId`/`parts`), `from_a2a` inbound parse into `Intent`, JSON-RPC
  `message/send` envelope; + designed round-trip property test and `--features sign` AgentCard-shape
  test (Feature Forge GREEN target). Native Tier-2 + sign suites must stay GREEN (no regression).
- **reversibility:** integrity-preserving (default-off, additive) · reversible (feature-gate off) ·
  capability-gain = industry A2A interop without abandoning the mailbox.
- **graph-grounded rationale:** the `Store`/`Intent` seam is the single stable broker abstraction the
  whole CLI dispatches through (`open_store -> Box<dyn Store>`); the adapter attaches there and rides
  it. High centrality, but additive design makes the effective blast contained — value/risk wins first.

### R2 — Dual-backend conformance harness  [APPLY]
- **axis:** quality
- **target-surface:** new `weave-core/tests/store_conformance.rs` (parametrized `fn
  run_store_conformance(s: &dyn Store)` / `macro_rules!`), run once over `SqliteStore`, once over
  `LibsqlStore`. Touches no production code.
- **evidence:** ARCH-11 CONFIRMED — "both backends identical" is comment-asserted only (`store.rs:80`);
  no shared harness exists (`grep store_conformance/both_backends` empty); 160 vs 95 `#[test]` (libsql
  40% less exercised). The gate found a **real divergence**: `LibsqlStore.send` (`store_libsql.rs:1499`)
  calls `self.guard_writable()?` before the `check_ident` block; `SqliteStore.send` (`store.rs:3153`)
  has **no** `guard_writable` — a verified parity asymmetry (the ready first divergence target).
- **blast:** 462 (`store.rs`) + 488 (`store_libsql.rs`) — tests-only, zero production blast.
- **effort:** M
- **risk-tier:** APPLY (additive test module; touches no production code).
- **P8-test:** the conformance suite IS the test; **acceptance re-scoped** from the analyst's stale
  "all 29 methods" to the trait's real surface (**~90 required methods**, ARCH-03) so it does not
  silently under-cover. First locked divergence: `send`'s `guard_writable` ordering.
- **reversibility:** integrity-preserving (tests only) · fully reversible (delete the file) ·
  capability-gain = enforced dual-backend parity + a regression fence for every future `Store`/A2A change.
- **graph-grounded rationale:** lowest-risk (APPLY, no prod code), de-risks both the two highest-blast
  store files AND R1/R6 — sequenced second as the safety net that makes the larger moves safe.

### R3 — Single-source the CLI↔MCP verb mirror  [PROPOSE]
- **axis:** governance+settings+config
- **target-surface:** `weave/src/main.rs` (`Cmd` enum) + `weave-mcp/src/mcp.rs:434` (`call_tool`
  router) — additive cross-guard test (low-risk path) now; declarative-registry derive is the heavier
  deferred option.
- **evidence:** ARCH-06 QUALIFIED + gap #3 — two hand-maintained flat matches with no cross-guard; a
  verb added to one plane and not the other is a silent capability gap. U-ARCH-4 FEASIBLE.
- **blast:** 124 (`mcp.rs`) + 427 (`main.rs`).
- **effort:** M
- **risk-tier:** PROPOSE (touches the control-plane verb surface of two crates).
- **P8-test:** a test enumerates the **measured** surfaces — **71 CLI verbs / 72 MCP arms / 76 catalog
  entries** (NOT the stale "71↔78") — and fails RED on any orphan in either direction (modulo an
  explicit allowlist).
- **reversibility:** integrity-preserving · reversible · capability-gain = drift-proof CLI/MCP parity.
- **graph-grounded rationale:** medium blast, drift-proofs the control surface before the big `main.rs`
  extraction (R6) reshuffles it — sequenced as a prerequisite-guard for R6.

### R4 — Documented-gate truthfulness (6→7) + Python-out-of-CI  [PROPOSE]
- **axis:** governance+settings+config
- **target-surface:** `CLAUDE.md:43` + `.handoff/policy.toml:36-37` (doc edit); `scripts/target_smoke.py`,
  `scripts/supply_chain_audit.py` + `.github/workflows/ci.yml:69,181` (Rust `xtask` port).
- **evidence:** GOV-003 CONFIRMED (docs say 6, `sync-master.yml:46` enforces 7 incl. `audit`);
  GOV-004 CONFIRMED (`ci.yml:69,181` run `python3 scripts/*.py` vs the no-Python invariant
  `CLAUDE.md:22,47-59`). U-GOV-001 + U-GOV-002 FEASIBLE.
- **blast:** doc-only (gate alignment) + CI/build-tooling (Python removal).
- **effort:** S (doc) + M (Python→Rust port).
- **risk-tier:** PROPOSE (protected files + CI plane).
- **P8-test:** a diff of the three required-check lists (`policy.toml`, `CLAUDE.md`,
  `sync-master.yml:46`) is empty (all 7 names); `find . -name '*.py' -not -path '*/target/*'` returns
  empty AND CI's `target-smoke`+`audit` jobs pass with Rust-native steps producing the **same**
  `target-smoke.json` schema and `cargo deny` posture (a port, never a relaxation — never weaken).
- **reversibility:** trivial (doc) / medium (git-revert restores the .py + CI lines).
- **graph-grounded rationale:** low structural blast, high truthfulness value; restores the headline
  Rust-native invariant. Independent of R1/R6 → can land in parallel.

### R5 — Memory-organ separation-of-concerns + ICM blindness  [PROPOSE]
- **axis:** memory-vector-intelligence
- **target-surface:** ADR classifying `weave-core/src/memory.rs` as a bounded send-time augmentation
  cache (NOT a fleet recall organ) + CLAUDE.md/ARCHITECTURE.md doc pointer to ICM+handoff for durable
  recall. No code deletion required.
- **evidence:** MEM-2 CONFIRMED — `memory.rs` is a real FS-backed scoped persistent-memory organ
  (`~/.config/weave/memory`, 5 MCP tools, auto-injected via `build_context_prefix` `memory.rs:332`)
  with **zero `icm` references** — a 6th memory surface that cannot see ICM. U-MEM-1 FEASIBLE;
  U-MEM-2 feasibility-QUALIFIED → prefer option (b) doc-contract (weave memory = explicit local cache;
  durable recall stays ICM+handoff). Option (a) wiring `build_context_prefix` to ICM is
  feasibility-constrained (cross-binary coupling weave-core→ICM the transport plane should not own) —
  pursue only if itself ADR'd.
- **blast:** instruction/docs only (no code/build).
- **effort:** S (ADR + docs).
- **risk-tier:** PROPOSE (governance decision; ADR + doc-gate).
- **P8-test:** a `.handoff` doc-gate fails closed if `weave-core/src/memory.rs` exists but no ADR
  classifies it; provenance/opt-out RED test asserts `--no-memory` fully suppresses the
  `<weave-memory>` block and every injected line carries a `scope::key` label (`memory.rs:402`);
  no-vector regression-fence test (asserts weave introduces no embedding/vector/RAG dep — passes today).
- **reversibility:** high (decision + docs; no code deletion to start).
- **graph-grounded rationale:** docs-blast only; removes a silent memory-fragmentation risk and a
  separation-of-concerns ambiguity in the spine. Independent of the structural moves.

### R6 — `main.rs` 9631-line dispatch extraction  [SUPERVISED]
- **axis:** quality
- **target-surface:** `weave/src/main.rs:4660+` → new `weave/src/dispatch/*.rs` per-verb handlers,
  extending the existing `dispatch_memory`/`dispatch_lease`/`dispatch_job` pattern (`:7090/7305/7410`).
- **evidence:** ARCH-07 CONFIRMED (9631 lines exact, 71 `Cmd::` arms, two sequential matches), gap #2;
  filesystem-layout WV-FSL-1 (#2 largest source file; 10 sibling modules already prove the extraction
  pattern). U-ARCH-3 FEASIBLE (pure behavior-identical move).
- **blast:** 427 (`main.rs`) — the **highest-blast bin** file; large blast on the dispatch.
- **effort:** L
- **risk-tier:** **SUPERVISED** (large structural refactor on the highest-blast binary entrypoint; see
  `risk-policy.md`). One verb-group per PR to bound blast radius.
- **P8-test:** `wc -l weave/src/main.rs` below an agreed cap (e.g. <2000) — a `main_rs_line_cap` CI
  gate, ratcheted down per PR; each extracted verb dispatches through a named handler fn with ≥1 direct
  unit test; all existing `weave/tests/{integration,prop,security}.rs` pass unchanged; `cargo
  build`/`clippy` clean.
- **reversibility:** integrity-preserving (pure move, behavior-identical) · reversible (revert the
  module split) · capability-gain = per-verb testability + lower edit-blast.
- **graph-grounded rationale:** highest-blast change → sequenced LAST, behind R2 (parity harness) and
  R3 (verb-parity test) so the move is fenced by tests before it touches the god-file.

---

## Tool-evaluation

Cross-reference of what the **graph shows weave imports/links** (codemap §Build/run, research verified
pins) against the **researcher's** 90-day currency + advisories (`research/weave.trends.md` §D, crates.io
API accessed 2026-06-26; one RUSTSEC re-verified). Recommendation per tool with the cited date.

| tool / crate (pin) | links via | latest stable | currency | advisory | recommendation (cited) |
|---|---|---|---|---|---|
| `rusqlite 0.40.0` (bundled) | default `sqlite` backend | 0.40.1 (2026-06-06) | 1 patch behind | none on default build | **UPGRADE** to 0.40.1 — trivial, in-window (`weave.trends.md:144,173`) |
| `tokio 1.52.3` | `libsql` async client | 1.52.3 (2026-05-08) | exact match | — | **HOLD** — current (`weave.trends.md:142`) |
| `libsql 0.9.30` (opt) | `libsql` backend | 0.9.30 stable; 0.10.0-pre.4 | current stable | owns the 5-id advisory cluster | **HOLD** stable; 0.10 is pre-release — do not chase. Track for rustls-0.23 TLS upgrade (the single unblock that clears all 4 rustls-webpki ids + rustls-pemfile) (`weave.trends.md:143,167-178`) |
| `serde 1.0.228` / `serde_json 1.0.150` | core + **A2A adapter carrier** | 1.0.228 / 1.0.x | current (unsuperseded) | none | **HOLD** — the A2A adapter rides this; no new dep needed (`weave.trends.md:146-147`; U-ARCH-2) |
| `ed25519-dalek 2` + `sha2 0.10` (opt `sign`) | `sign` feature — **AgentCard signing** | 2.2.0 (2025-07-09) / 0.10.x | current (unsuperseded) | none | **HOLD** — reuse as the A2A signed-AgentCard primitive; pure-Rust, no C (`weave.trends.md:148-149`; research §A2) |
| `reqwest 0.12` (opt `llm`/`surfaces`) | optional HTTP client | 0.13.4 (2026-05-25) | 1 minor behind | (rustls cluster only under libsql) | **HOLD/EVALUATE** — optional-features only; 0.13 is a new minor; schedule, not urgent (`weave.trends.md:150,176`) |
| `clap 4.6.1` | CLI | 4.6.1 (2026-04-15) | exact | — | **HOLD** — current (`weave.trends.md:145`) |
| `anyhow 1.0.102` / `criterion 0.7.0` (dev) | error model / bench | 1.0.x / 0.7.x | current line | — | **HOLD** (`weave.trends.md:151-152`) |
| RUSTSEC-2026-0104/0098/0099/0049 (rustls-webpki), RUSTSEC-2025-0134 (rustls-pemfile) | only under `--features libsql` remote-TLS | fix ≥0.103.x | n/a | **scoped, documented, CI-gated** (`deny.toml` WL-044b) | **HOLD (upstream-blocked)** — default `sqlite` build compiles none of these; re-audit each cycle; cleared when libsql adopts rustls 0.23 (`weave.trends.md:154-183`) |
| **A2A v1.0 (LF standard)** | NOT a dep — the interop *target* | v1.0 (GA 2026-04-09) | — | — | **ADOPT via additive adapter** (R1) — JSON-RPC 2.0/SSE; gRPC optional and, if ever added, must use pure-Rust `tonic`/`prost` not a C protobuf (`weave.trends.md:26-44`; verdicts U-ARCH-2) |

Currency verdict: **healthy.** One trivial bump owed (`rusqlite 0.40.1`); the advisory budget is
confined to the opt-in `libsql` surface and upstream-blocked; the A2A adapter introduces **no new
dependency** (rides serde_json + ed25519). No tool forces a downgrade or an urgent action.

---

## Governance, settings & config

`Source: findings/governance-config-weave.md`

- **PreToolUse gate has real teeth AND is opt-in** (GOV-001 CONFIRMED): deny-by-default for dangerous
  tools (`main.rs:8896-8919`), enforces its own short timeout and emits explicit `deny` rather than
  relying on Claude's 600s fail-OPEN timeout; installed only by `weave setup --pretooluse`, inert
  without an approver. The strongest control in the repo, dormant by default → U-GOV-009 documents +
  optionally arms it (security TIGHTENING only; never weaken deny-by-default).
- **Drift to fix** (→ R4): documented "6" CI gates vs **7** enforced incl. `audit` (GOV-003);
  Python in CI vs no-Python invariant (GOV-004). Plus low-sev doc drift: Harness pointer omits
  `weave-loop`/`session-relay`/`continuity-steward` (GOV-002); stale `policy.toml:6` "PRs target
  master" comment (GOV-005); `ecc-tools.json` wrong owner `drdave-flexnetos/weave` (GOV-007);
  dangling root `AGENTS.md` reference in `.codex/AGENTS.md:3` (GOV-008); no `rust-toolchain.toml` pin
  (GOV-011). All FEASIBLE doc/config edits (U-GOV-001..008).
- **Positives:** strong protected-file + destructive-command guards (`rules.toml:37-62`); the advisory
  ignore-list is scoped, documented, time-bounded, CI-gated (GOV-012); token burn actively guarded.
- **Owner walls (cannot conclude from worktree):** GitHub branch-protection required-check set; `hf`
  verb fail-closed-vs-no-op behavior for block-mode hooks (GOV-009). Flagged, not silently passed.

## Filesystem layout

`Source: findings/filesystem-layout-weave.md`

- **Positive:** clean Cargo layout, committed `Cargo.lock`, **no repo-escaping path deps** (sibling
  `../weave-*` only), correct `.handoff` ignore rules, **no system-level writes** (verified clean).
- **DRIFT:** `main.rs` god-file (→ R6); 2 Python scripts (→ R4); oversized root docs (minor, WV-FSL-6).
- **OWNER-WALL (PROPOSE):** user-global runtime writes — `~/.local/share/weave/messages.db`,
  `~/.config/weave/config.toml`, `~/.config/weave/memory/**` — XDG-correct as standard practice but
  unmanaged w.r.t. the meta-residency invariant with **no exemption ADR** (WV-FSL-3). Memory is
  mis-rooted under `$XDG_CONFIG_HOME` (config) when it is regenerable state → should move to
  `$XDG_DATA_HOME` with read-fallback (WV-FSL-4). Both default to PROPOSE pending an owner decision.

## Memory/vector intelligence

`Source: findings/memory-vector-intelligence-weave.md`

- The SQLite store is a **transport/event log, not recall memory** (MEM-1 CONFIRMED) — mailbox/queue/
  receipt schema; the only query surface is FTS5 over message bodies (searching traffic, not facts).
- weave **has NO vector/embeddings/RAG** (MEM-3 CONFIRMED) — and that is **correct** for a transport
  plane. No vector upgrade proposed (U3 genuine N/A); keep FTS5 lexical for message search. A
  regression-fence test guards against any future embedding/vector dep.
- weave **does ship a real memory organ** (`memory.rs`) blind to ICM — the separation-of-concerns +
  ICM-blindness item is R5.

## Auto-research

`Source: findings/autoresearch-weave.md`

- weave has **no repo-native code-intelligence index** (no `.kb/`, no git-kb config, no CI re-index;
  C1 CONFIRMED) and **no web auto-research bot** (no dependabot/renovate; W1 CONFIRMED). The
  plan-loop's code graph is built **harness-side** by the cartographer per cycle (C2).
- It **does** enforce a strong, fail-closed, event-driven **advisory currency gate** (`deny.toml` +
  `supply_chain_audit.py` + CI `audit`) with a genuine **stale-advisory self-invalidation** mechanism
  (`check_libsql_tree_tracks_tls` forces removal of the rustls ignores the instant upstream unblocks;
  S2 CONFIRMED). The daemon heartbeat TTL is the runtime analogue of staleness invalidation (S4).
- Optional upgrades (low-priority, FEASIBLE): repo-native git-kb freshness CI step (U1); a
  renovate/dependabot config to close the new-release recency blind spot (U2, e.g. surface the owed
  `rusqlite 0.40.1`). Both additive, no trust-boundary code.

## Rules/policy & org

`Source: findings/rules-policy-org-weave.md`

- **Genuine No-Downgrades regime** (CLAIM-P1): enforced at three layers — source invariant
  (`CLAUDE.md:47-59`), merge ancestor-guard (`develop`→`master`, ff-only), and a gate-strengthen-only
  retro law. The A2A convergence (R1) is explicitly framed as a **strict-upgrade adapter** — keep the
  SQLite mailbox, ADD A2A, never replace.
- **Real file-defined agent org chart** (CLAIM-P2): orchestrator (opus) → planner/implementer/
  verifier/guardian, guardian kept separate for invariant+drift; guardian BLOCK wins. **Dual-model /
  cross-vendor background lane** (CLAIM-P3): MiniMax `minimax-m3:cloud` is the autonomous-loop
  guardian — ADR-uncovered → `ADR-DRAFT-weave-cross-vendor-model-lane.md`.
- **The lease primitive** is the mesh mutual-exclusion the parallel plan-loop reuses
  (`require_disjoint_write_scopes`, `rules.toml:18`; `reserve_lease` `store.rs:750`); making the
  loop's lease consumption an explicit verified call path is U-5 (low-med, additive).
- weave is itself the **A2A substrate** — its A2A map is its own product surface, not a dependency.

## Distributed compute

`Source: findings/distributed-compute-weave.md`

- weave is the cross-machine **distribution substrate** but its reach is **host-class only**: Tier-2 =
  signed cross-store delivery + HTTP push (`#[cfg(feature="surfaces")]`, default-off). Constrained
  nodes (Pi Zero / ESP32 / mobile / wearables) **cannot host** weave (std-only, no `no_std`/embedded,
  no Lua/Luau, no WASM) — they can only be **external HTTP clients** POSTing a `weave_push` JSON-RPC
  `Intent` into a host-resident `weave serve`.
- Upgrades (FEASIBLE, additive/doc): specify a constrained-node minimal-client contract (DC-W1); reuse
  `sign` (ed25519) as the cross-vendor trust primitive aligned with A2A signed AgentCards (DC-W2 —
  converges with R1); record a no-ADR/ADR decision on a Rust `no_std`/Lua relay-node policy plane
  (DC-W3); validate on Raspberry Pi aarch64 + document the "min node = 64-bit Linux host" floor
  (DC-W4); note the cross-machine liveness gap (TTL-only, fails open; DC-W5); keep `sqlite` default,
  bump `rusqlite 0.40.1` (DC-W6, = the R4/tool-eval bump).

## Test Strategy & Coverage

`Source: findings/test-strategy-weave.md`

### Current coverage (by call-graph reachability)
- weave is **heavily tested overall** (809 inferred entrypoints dominated by `#[test]` symbols;
  `weave/tests/{integration,security,prop}.rs` + dense `#[cfg(test)]` modules per file).
- The **native `Intent` wire path IS covered**: `integration.rs:3541` (tier2 dedup keyed on intent
  id), `:3646` (misaddressed intent not committed), `security.rs:1388` (signed-intent failing
  verification always rejected). **The gap is interop, not the native path.**

### Ranked coverage gaps (each citing the symbol)
1. **A2A interop seam has ZERO test caller AND ZERO implementation** — `to_a2a`/`from_a2a`/
   `message/send`/`AgentCard` exist nowhere; `Intent` (`model.rs:216`) serializes flat
   `[id,ts,to,to_host,from,subject,body,sig,...]` with no `kind/role/messageId/parts`. **Highest-risk
   gap** — `model.rs` blast 1238 and its A2A evolution is unguarded.
2. **Dual-backend parity unguarded across ~90 `Store` methods** — no `store_conformance`; the verified
   `LibsqlStore.send`/`SqliteStore.send` `guard_writable` asymmetry is the proof the gap is real (→ R2).
3. **CLI↔MCP verb mirror has no cross-guard** — a verb orphaned on one plane is undetected (→ R3).

### Designed suite (closes the gaps + covers the roadmap upgrades)
- **Committed RED (additive, FAILING-on-assertion):** `weave-core/tests/a2a_interop.rs` — 3 cases
  (`intent_serializes_to_a2a_message_object`, `a2a_message_deserializes_into_intent`,
  `intent_frames_as_a2a_jsonrpc_request`); `cargo test -p weave-core --test a2a_interop` → 0 passed /
  3 failed, **tests-ran = 3** (not a fail-open exit-0); commit `b7f466f` on `plan/weave-red-tests`
  (unpushed). These are the GREEN target for R1.
- **Designed for Feature Forge to author alongside the adapter:** round-trip property test
  `from_a2a(to_a2a(i)) == i` over core fields (proptest is a dev-dep); `--features sign` AgentCard
  signature-shape test over `sign.rs`; the `store_conformance.rs` parity suite (R2, re-scoped to ~90
  methods); the CLI↔MCP cross-guard test (R3, 71/72/76); the governance/layout fail-closed gates
  (gate-count diff, `db_path_is_xdg`, `no_new_crate`, `scripts_rust_native`, `residency_adr_present`).
- **Golden fixtures:** one published-A2A-v1.0 `message/send` JSON-RPC request fixture (validate against
  the v1.0 schema) to diff the adapter's output; the native Tier-2 goldens must remain unchanged.
- **Coverage target:** `to_a2a`/`from_a2a` each reached by ≥1 test; A2A-1/2/3 GREEN; native Tier-2 +
  sign suites still GREEN (no regression).

### FF test-build spec (carried from plan-test-strategist — promoted in step 7)
Intake shape = `feature-architect ## Verification plan`. The RED suite `weave-core/tests/a2a_interop.rs`
is committed and FAILING; Feature Forge implements the A2A adapter (R1) until GREEN, **additively** —
never remove the SQLite-mailbox transport. On adapter landing, migrate the 3 cases to drive
`Intent::to_a2a()`/`Intent::from_a2a()` directly. CI gates touched: `cargo test -p weave-core` (new
`a2a_interop` binary), `cargo fmt --check` + `cargo clippy` (preflight subset), and the `sign`-feature
lane if the AgentCard case is added. This is promoted as a **Feature-Forge test-build ROADMAP row**
(see `reports/ROADMAP-weave.md`).

## Prompt-architecture

`Source: findings/prompt-architecture-weave.md`

- **Largest tool grant in the fleet is token-safe by design** (PA-TOOLS QUALIFIED): **72 dispatch arms
  / 76 catalog entries** (NOT 78) collapse to ONE standing `weave` meta-tool via progressive
  disclosure, byte-budget-gated (`MAX_STANDING_TOOLS_BYTES=8192`) and test-locked; meta-tool `call`
  re-applies the destructive-op gate (PA-METACALL CONFIRMED) so it is not a bypass.
- **Genuine bounded recursion (not an artifact):** `call_tool ↔ tool_meta` is real mutual recursion —
  `tool_meta` `mode=call` dispatches back through `call_tool`; termination is guaranteed by the
  `if want == "weave" { return Err }` self-target guard (ARCH-12 sub-claim REFUTED as artifact → it is
  real, safe, guarded recursion). The `open ↔ open_conn` SCC is a true resolver artifact; the inject
  5-cluster is INCONCLUSIVE (untraced this pass).
- **ADR-uncovered:** the dual-model / cross-vendor lane (MiniMax guardian) → second ADR draft.
- **Ungoverned instruction drift:** ecc-generated `weave-instincts.yaml` tells agents "camelCase /
  relative imports" — false for a snake_case Rust repo; the same drift class CLAUDE.md fixed once for
  the skill but never for the instinct/identity sidecars. Bring under the drift guard or delete
  (FEASIBLE, instruction-only).

## Risk policy

Per-upgrade APPLY/PROPOSE/SUPERVISED classification with trust-boundary, secrets, destructive,
provider, and model dimensions is in **`risk-policy.md`** (this directory). Summary: R2 is APPLY
(tests-only); R1 is PROPOSE-additive (new protocol surface, default-off, no new dep, no C); R3/R4/R5
are PROPOSE; **R6 (`main.rs` extraction) is SUPERVISED** (large blast on the highest-blast dispatch
file). No upgrade is auto-applied that touches the trust boundary, secrets, or a destructive op
without supervision.

## Confidence

**Overall: HIGH.**

- HIGH on structure (layering, dep hygiene, the `Store`/`Intent` seam, `send` path), the A2A-absence
  convergence finding, the tool-currency/advisory table, and the test gap — all directly verified
  against source by the gate (16 CONFIRMED), with the RED suite as live proof of the interop gap.
- The 4 QUALIFIED corrections are honored as plan facts: `Store` ~90 methods (not 29 — re-scopes R2),
  72/76 tools (not 78 — fixes R3's test), 71 CLI verbs, and `call_tool↔tool_meta` as real guarded
  recursion. The one feasibility-QUALIFIED item (U-MEM-2) is planned conservatively (R5 prefers the
  doc-contract; ICM wiring only if itself ADR'd).
- **What stayed INCONCLUSIVE / not examined (named gaps):** the inject 5-cluster SCC
  (`spawn/kill/run_bounded*`) was not traced; GitHub branch-protection required-checks and the `hf`
  block-mode-hook fail-closed behavior are owner-walls outside the worktree; whether the Python
  scripts encode logic that must be preserved on the Rust port is a Feature-Forge porting-scope
  question. None block the plan; a deeper pass should trace the inject SCC and resolve the two owner
  walls via `gh api .../protection` and the shared `hf` binary.
- **What would raise confidence to VERY HIGH:** (a) Feature Forge takes the committed RED suite GREEN
  for R1; (b) the conformance harness (R2) lands and locks the `guard_writable` divergence; (c) the
  two owner walls are resolved.

### Reported (excluded from the roadmap, per build-from-confirmed-evidence)
- **Refuted overclaims:** the analyst "29 Store methods" (real ~90), "78 MCP tools" (real 72/76), and
  the "`call_tool↔tool_meta` resolver artifact" framing (it is real bounded recursion) — corrected
  above, not used as plan facts.
- **Feasibility-qualified:** U-MEM-2 option (a) (wire `build_context_prefix` to ICM) is constrained by
  a transport-plane-inappropriate cross-binary coupling → defaulted to option (b).
- **No fully-infeasible upgrade** this cycle (9 FEASIBLE, 1 feasibility-QUALIFIED, 0 INFEASIBLE).
