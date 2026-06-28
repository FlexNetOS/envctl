# handoff — decision-grade PLAN (cycle 2, planned as the union with rusty-idd)

Author: plan-architect · Date: 2026-06-26 · Target: **handoff** (the `hf` continuity kernel + the
witnessed `.handoff` ledger), planned explicitly as the **handoff + rusty-idd UNION** (owner D1).
Built ONLY from CONFIRMED/QUALIFIED + feasibility-passed rows in `findings/verdicts.md` (handoff
cycle 2: **57 CONFIRMED / 3 QUALIFIED / 0 REFUTED / 39 feasible / 0 infeasible**). Docs only — no
production code touched. Worktree under audit: `…/.worktrees/plan-handoff-cycle2/handoff` @ `f6abf96`
(verifier ran empirical experiments @ `d74ad4b`). Companion files: `reports/union-plan-handoff-rusty-idd.md`,
`reports/north-star-DRAFT.md`, `reports/ADR-DRAFT-handoff-rusty-idd-union.md`, `reports/ROADMAP-handoff.md`,
`risk-policy.md` (`## handoff (cycle 2)`).

---

## Verdict

**handoff is the production-hardened continuity kernel and the fleet's policy + operational-memory
substrate — and it is the correct north-star home for the handoff+rusty-idd union — BUT it is NOT
standalone today: the #1 blocker is the RuVector `../../RuVector/*` path-dep, which makes the entire
workspace unresolvable at `$META_ROOT + handoff`.**

Why this verdict, from confirmed evidence:

- **It already implements the hard part.** A strict-DAG pure-Rust kernel (`hf` is the universal sink,
  zero Cargo cycles — A-C1) with **real-teeth** fail-closed gates that `process::exit(1)` under
  `fail_mode="block"` (rp-teeth: `handoff-drift/src/lib.rs:676,792`, `hf/src/cognitum.rs:135`,
  `handoff-gatekeeper/src/lib.rs:304-386`), a redb-authoritative event store with a tamper-evident
  witness chain, and a committed-JSONL cold-start that re-verifies fail-closed
  (`ledger/src/export.rs`). The trend field independently names this design (ESAA event-sourcing →
  deterministic projection; durable-execution checkpoint/resume — trends §B1/§B2). handoff IS that,
  in Rust, with no server.
- **The blocker is empirical, not theoretical.** EXP-1 CONFIRMED that `cargo build -p ledger` AND
  `cargo build -p ledger --no-default-features --features redb-store` both fail at workspace
  **manifest-load** (`failed to read …/RuVector/crates/rvf/rvf-crypto/Cargo.toml`). The path dep
  poisons the whole workspace — even the leaf `work-order` cannot build in-tree. The union cannot
  build at its own north-star location until this is resolved.
- **The union is a MERGE, not a rebuild.** crates/{cli,core,runner,spec,tui} are a stale partial
  fork of rusty-idd's superset (A-C9), sharing identical `rusty-idd-*` package names — a Cargo name
  collision the union must dedup. handoff has the kernel rusty-idd lacks; rusty-idd has the 8 CLI
  commands handoff stripped. Fold rusty-idd's CLI **under** handoff's gates (UP-1); handoff stays the
  north-star.

**Single most decision-relevant item:** resolve the RuVector path-dep (A-U1 — vendor / publish /
git-pin). It is the gate on which the union build, the ledger read-API design, every below-leaf test,
and standalone portability all depend. Everything else sequences behind it.

**Three KEY CORRECTIONS the evidence forces (carried into every section below):**
1. The witness chain is **SHAKE-256 hash-linked (SHA3-256 action hash), UNSIGNED** — NOT
   "blake3+ed25519-signed" (EXP-3: `rvf-crypto/src/witness.rs:4`; `ledger/src/v1.rs:20` imports no
   `sign`). blake3 is used only for `work-order::compute_intent_lock`. Any seed/trends/doc text saying
   "blake3+ed25519 witness chain" is wrong (mem-U3).
2. `ledger`'s `rvf-crypto` does **not** carry `default-features=false`; `ed25519-dalek` is compiled
   into the default build though the witness path never signs (A-C7 QUALIFIED). The C-free conclusion
   holds (sha3 + ed25519-dalek are pure-Rust); the wording was inaccurate.
3. The RVF "semantic recall" vector plane is **dead**: `query_by_intent` has 0 production callers,
   there is no `hf recall` verb, and the embeddings are SHA3-256 pseudo-embeddings ("semantic" is a
   misnomer) — written on every append, read by nothing (mem-1/2/6).

---

## ASCII architecture

Conventions: envctl `docs/runbook/DIAGRAMS.md`. Automation legend —
`[A]` automated · `[A*]` elevated/sudo · `[P]` preview/dry-run · `[H]` human-gated · `[!!]` supervised/critical.

### Diagram 1 — CURRENT (two co-resident programs + the RuVector residency break)
Source: `graph/handoff.graph.md §1,§5`, `reports/codemap-handoff.md §2-5`, verdicts A-C1/A-C5/A-C8/EXP-1.

```
   handoff worktree  (21 Cargo members — strict DAG, hf = universal sink; A-C1)
   ┌──────────────────────── KERNEL (16 crates) ─────────────────────────────┐
   │  hf (604 sym, hub bin) ──depends-on──▶ ALL 15 below  + RuVector path deps │
   │   ├ ledger (164) ─ work-order        handoff-drift (33) ─ core,ledger,wo  │
   │   ├ work-order (74) [handoff.task.v1 SoT, schemars]                       │
   │   ├ handoff-core (27)   handoff-schema (10) ─ work-order                  │
   │   ├ handoff-fleet (43)  handoff-route (13)   handoff-gatekeeper (26)      │
   │   ├ handoff-intake (17) handoff-index (22)   handoff-hooks (24)           │
   │   └ handoff-policy (55) handoff-lease (28)   handoff-secrets (12)         │
   └───────────────────────────────────┬──────────────────────────────────────┘
        call edges between groups: RIDD─▶KERNEL = 1 ; KERNEL─▶RIDD = 41        │
        (41 are same-name resolver collisions, NOT real deps; A-C8)            │
   ┌──────────── RIDD-TOOLKIT (5 crates, rusty-idd-* lineage) ────────┐        │
   │  crates/cli=rusty-idd-cli (97) ─ core,runner,spec                 │        │
   │  crates/core(153) crates/runner(240) crates/spec(205)            │        │
   │  crates/tui(270) ─ vendor/syntect (846 sym; root vendor/, fs-4)  │        │
   └──────────────────────────────────────────────────────────────────┘        │
                                                                                 ▼
   ╔══════════════════════════════════════════════════════════════════╗  [!!] BLOCKER
   ║  ../../RuVector/*  PATH DEPS  (ABSENT in worktree — EXP-1)         ║  manifest-load
   ║  ledger → rvf-crypto (redb-store) + rvf-runtime/index/types (v2)  ║  FAILS for the
   ║  hf → ruvector-verified, ruvector-domain-expansion,              ║  WHOLE workspace
   ║       cognitum-gate-tilezero(opt, default-on)                     ║  (even leaf wo)
   ╚══════════════════════════════════════════════════════════════════╝
   Witness chain = SHAKE-256 hash-link + SHA3-256 action, UNSIGNED (EXP-3).
   RVF vector overlay (v2, DEFAULT): SHA3 pseudo-embeddings, 0 readers (mem-1/2/6).
```

### Diagram 2 — TARGET (the union: one canonical workspace, RuVector resolved, fork deduped)
Source: A-U1/A-U3/A-U4, `findings/filesystem-layout-handoff.md §4`, `findings/union-handoff-rusty-idd.md §4`.

```
   handoff/  =  $META_ROOT + handoff  (THE portable kernel ROOT / north-star)   [H] owner-walled merge
   ┌──────────────────────────────────────────────────────────────────────────┐
   │ CONTINUITY KERNEL (16, unchanged)   hf · ledger · work-order · handoff-*   │
   │   ledger feature graph: default=[redb-store]  ·  v2 (RVF overlay) OPT-IN   │ [P] A-U2
   │   RuVector: vendored under vendor/ruvector/ + [patch]  (mirrors syntect)   │ [!!] A-U1
   ├──────────────────────────────────────────────────────────────────────────┤
   │ CANONICAL SHARED SET (dedup winner = rusty-idd superset; A-U4)            │ [H] A-U4
   │   crates/{cli,core,runner,spec,tui}  + RESTORED cmds: codex/deploy/harness │
   │      /knowledge/merge-tools/next/render/spec-plan-integration              │
   │   + folded extras: config, knowledge, merge-tools, external/codegraph-*    │
   │   HFTASK-0082 lint hardening re-applied on the winner                      │
   ├──────────────────────────────────────────────────────────────────────────┤
   │ ONE compiler-enforced contract: rusty-idd deps handoff work-order +        │ [P] A-U3
   │   handoff-schema::validate_card  (kills the mirrored-copy drift, A-C13/G2) │
   └──────────────────────────────────────────────────────────────────────────┘
   gain: cargo build --workspace green with NO sibling RuVector  (acceptance: A-U1 E1 gate)
```

### Diagram 3 — CONTROL PLANE (the union pipeline; owner D1/D3 convergence)
Source: `findings/resolved-decisions.md` D1/D3, `findings/prompt-architecture-handoff.md §6`, `findings/rules-policy-org-handoff.md §3`, trends §C1/§D1.

```
   user intent
       │ vibe / why+what
       ▼
   ┌────────────────────────┐   D3: transforms intent → MODEL-READY language (the interpreter)
   │ harness_hub  FRONT DOOR │  [H] non-deterministic; sits UPSTREAM of handoff's classifier
   │ (intent → model lang)   │
   └───────────┬────────────┘
               │ interpreted intent
               ▼
   ┌──────────────────────────────────────────────────────────────────────┐
   │ handoff = WITNESSED CONTRACT + DETERMINISTIC classifier + LEDGER       │
   │   prompt_hub/intake: vibe → WorkOrder  (NON-LLM, byte-identical card)  │ [A] pa §6
   │   work-order (handoff.task.v1 SoT) ─▶ validate_card [!!] fail-closed   │ [A] A-C12
   │   ledger (redb + SHAKE-256 witness) ─▶ committed JSONL (cold-start)    │ [A] mem
   │   policy gates: check-edit/check-handoff/drift/gatekeeper  exit(1)     │ [!!] rp-teeth
   └───────┬───────────────────────────────────────────────┬──────────────┘
           │ why/what (OpenSpec intent)                     │ claims + leases
           ▼                                                ▼
   ┌──────────────────┐                              ┌─────────────┐  DISTINCT PLANE
   │ rusty-idd        │  ⇄  curated decision/spec    │  weave A2A  │  transport only;
   │ (intent: why/what)│     events on the SAME log   │ (transport) │  degrades to
   └──────────────────┘                              └──────┬──────┘  ledger-only [A] rp-A2A
                                                            ▼
                                          models / distributed compute ─▶ output ─▶ user
```

---

## Sequenced upgrade

Ordered by **value/risk = graph centrality + blast-radius**: high-centrality, contained-blast wins
first; high-blast / invariant-crossing changes sequence behind their prerequisites. Each row carries
**axis · target-surface · evidence · blast · effort · risk-tier · P8-test · reversibility**.
Full roadmap promoted in `reports/ROADMAP-handoff.md`; the union-specific 5-step sequence is in
`reports/union-plan-handoff-rusty-idd.md`. Every row traces to a CONFIRMED/QUALIFIED + feasible verdict.

Centrality/blast anchors (graph): `Ledger.open` blast 120/74 · `ledger_path` 54/27 ·
`validate_card` 40/5 · `McpServer.new` in-109 · `compute_intent_lock` in-18
(`graph/handoff.graph.md §2,§3`).

| seq | id | upgrade | axis | target-surface | evidence | blast | effort | tier | P8-test | reversibility |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | A-U1 | Move RuVector off `../../` path deps (vendor / publish / git-pin) | governance | `hf/Cargo.toml:48,52,59`, `ledger/Cargo.toml:16-20,31-35`, `.github/workflows/ci.yml` | EXP-1, A-C5/A-C6, gov-003 | entire KERNEL (Ledger.open 120) | L | **SUPERVISED** | `cargo build --workspace` green with NO sibling `RuVector/` (E1 gate; RED today) | Integrity (witness unchanged) · Reversible (dep-source swap / vendor dir removable) · Capability-Gain: provable standalone kernel |
| 2 | A-U4 | Dedup/converge stale `crates/{cli,core,runner,spec,tui}` → rusty-idd superset; re-apply HFTASK-0082 lint | quality | `crates/*` (handoff) vs rusty-idd; pkg names `rusty-idd-*` | A-C9, fs-2/V3, codemap §4 | 5 crates + 3rd binary; ~0 KERNEL (1 real edge) | L | **SUPERVISED** | one `rusty-idd-*` pkg per name (`cargo metadata`); spec/tui differential-golden parity passes (E3) | Integrity (per-crate behavior preserved before delete) · Reversible (git history) · Gain: single toolkit + restored CLI |
| 3 | A-U3 | Replace rusty-idd mirrored `work-order` with a real dep on handoff `work-order`+`validate_card` | accuracy | rusty-idd `crates/work-order` → handoff `work-order`/`handoff-schema` (seam 1) | A-C13/G2, codemap §5.1 | rusty-idd wo consumers + schema gen | M | PROPOSE | rusty-idd cards pass handoff `validate_card`; two `task.schema.json` byte-identical or duplicate deleted (RED today) | Integrity strengthened (one gate) · Reversible (re-vendor) · Gain: union contract coherence |
| 4 | ts-U1 | Fail-closed work-order LOADER tests (AUTHORED + RED-verified) → FF flips GREEN | accuracy | `work-order/tests/union_failclosed.rs` (`d74ad4b`); `work-order/src/lib.rs:56-92,213` | ts-2/ts-RED (1 pass/3 fail empirical) | every `handoff.task.v1` loader (rusty-idd mirror) | S | APPLY | the 3 RED tests flip GREEN when `WorkOrder::from_card_json` chains serde+validate_card+intent_unchanged | Integrity (additive) · Reversible · Gain: closes the silent-accept fail-open |
| 5 | gov-U1 | Bridge `hooks.toml` 5 block gates → Claude `PreToolUse(Edit\|Write\|Bash)` → `hf hook run` | governance | `.claude/settings.json` (+`hooks.toml` note) | gov-001 (HEADLINE fail-OPEN), AGENTS.md L7 | every Claude session edit | M | PROPOSE | an out-of-scope edit by a Claude session is DENIED by the hook (live attempt), not voluntary | Integrity (never weakens) · Reversible (revert settings hunk) · Gain: closes kernel's own banned fail-open |
| 6 | A-U2 | Split ledger feature graph: `default=[redb-store]`, `v2` (RVF overlay) opt-in | quality | `ledger/Cargo.toml:29-37` | A-U2 (QUALIFIED, coupled to A-U1), mem-U2 | ledger + v2-overlay readers | M | PROPOSE | default tree excludes rvf-runtime/index/types (`cargo tree` assert); `query_by_intent` only under `--features v2` (gated on A-U1) | Integrity (store/witness identical) · Reversible (re-add v2) · Gain: minimal/embedded build |
| 7 | mem-U3 | Correct witness provenance to SHAKE-256 in docs/seed/trends (optionally wire ed25519 behind a feature) | accuracy | `rvf-crypto/src/witness.rs:4`, `ledger/src/v1.rs:20,848`; trends §A3/§B2 | EXP-3 (KEY CORRECTION), mem-3 | doc surface (signing additive) | S | APPLY (doc) / PROPOSE (signing) | claim text == implementation; a witness-claim RED test asserts SHAKE-256 (and, if signed, a tampered sig fails `verify_witness_chain`) | Integrity (doc-only / additive) · Reversible · Gain: no false-crypto drift |
| 8 | UP-1 | Fold rusty-idd CLI UNDER handoff's policy gates (replace the toothless guard) | rules-policy | rusty-idd cmd modules → `hf policy check-edit`/gatekeeper | rp-teeth, UP-1 | rusty-idd CLI (LOW; independent modules) | M | PROPOSE | a rusty-idd command's out-of-scope/protected write is REFUSED (exit 1) identical to native `hf` | Integrity (adds a gate) · Reversible (remove wiring) · Gain: one enforced control plane |
| 9 | A-U5 | Compile-time test: exactly one `Ledger` per feature set + ADR note on the cfg-gated v1/v2 | quality | `ledger/tests/`, `ledger/src/lib.rs:36-40` | EXP-2, A-U5 (gated on A-U1) | ledger (test-scoped) | S | APPLY (gated) | builds under `redb-store` and `v2`, each asserts a single resolvable `ledger::Ledger` (runnable after A-U1) | Integrity (additive) · Reversible · Gain: guards a fake-SCC regression |
| 10 | mem-U1 | Resolve the dead vector plane: wire `query_by_intent`→`hf recall` with REAL embeddings, OR delete v2-default + delegate recall | memory-vector | `ledger/src/v2.rs:42-56,344-346` | mem-1/2/6 (QUALIFIED) | ledger overlay | M | **SUPERVISED** | `hf recall` returns semantic hits OR an ADR records delegation + no append writes an unread vector; **condition: any native embedder is C-free** | Integrity (feature-gated) · Reversible · Gain: recall that is read, or no write-amp |

Lower-blast governance/hygiene rows (APPLY-class, sequence anytime; full set in `reports/ROADMAP-handoff.md`):
gov-U4 (`command -v rusty-idd` guard on SessionStart), gov-U5 (`rust-toolchain.toml` pin 1.96.0),
gov-U6 (commit `.mcp.json` for `hf-mcp`), gov-U7 (tighten `Bash(git -C * push:*)`), gov-U9 (doc-sync
the 8 guard patterns), pa-U2 (pin `hf`↔`hf-mcp`), pa-U3 (trim 1541-skill catalog), ar-U1 (git-kb
index-staleness gate), ar-U2 (symmetric `cargo audit` per-PR), ar-U4 (one fleet currency bot),
fs-U3 (untrack `.idea/`), fs-U4 (route generated skills-catalog off root), fs-U6 (mark `schemas/*`
provenance), ts-U4 (golden `task_schema_json` parity across the mirror). SUPERVISED-class deferred:
DC-3 (native weave binding — first live network dep), ar-U5 (delete `legacy-sqlite` after fleet
migration). PROPOSE-class structural: UP-3 (add `evolution-steward`), UP-4/pa-U4 (witnessed dual-model
lane), mem-U5 (decision/why memory), mem-U6 (unified recall facade), DC-2 (leaf-node proxy), DC-4
(`allows_network`/`path_scope` egress enforcement), DC-5 (no-embedded/no-Lua guardrail), pa-U1
(reconcile dual front door), fs-U5 (home root orphans), UP-2 (enforce declared network/dep policies),
ar-U3 (scheduled research cadence), A-U6 (manifest-cross-checked graph-integrity gate — planning-only).

---

## Tool-evaluation

(R7) The tools/crates the **graph** shows handoff imports/links, cross-referenced with the
**researcher's** 90-day currency + advisories (`research/handoff.trends.md`, dated 2026-06-26;
window 2026-03-28→2026-06-26). Verdict per tool: **HOLD** (current, no action) or **UPGRADE/RESOLVE**.

| tool / crate | linked by (graph evidence) | pinned | currency / advisory (date) | verdict |
|---|---|---|---|---|
| **redb 4.1.0** | `ledger` authoritative store (`ledger/Cargo.toml`) | 4.1.0 | LATEST line, released 2026-04-19; no RustSec advisory; crash-safe ACID, stable file format (trends §A1, in-window) | **HOLD** — keep `cargo audit`/`deny` in CI (4.1 had AI-found-bug churn) |
| **blake3 1.8.5** | `work-order::compute_intent_lock` (NOT the witness chain — EXP-3) | 1.8.5 | latest; no advisory (trends §A2) | **HOLD** |
| **sha3 (SHAKE-256 / SHA3-256)** | the ACTUAL witness chain (`rvf-crypto/src/witness.rs:4`, `ledger/v1.rs:22`) | via rvf-crypto | pure-Rust; no advisory | **HOLD** — and fix docs that call it "blake3+ed25519" (mem-U3) |
| **ed25519-dalek 2.2.0 + curve25519-dalek 4.1.3** | `ledger` default build (A-C7) — present but the witness path NEVER signs | 2.2.0 / 4.1.3 | past ALL signing advisories (RUSTSEC-2022-0093 fixed 2.0; RUSTSEC-2024-0344 fixed 4.1.3) (trends §A3) | **HOLD** — pin-floor `curve25519-dalek >= 4.1.3` explicitly; either wire signing (mem-U3b) or note it's an unused-by-witness residual |
| **rusqlite 0.31.0 / libsqlite3-sys 0.28.0** | `ledger` `legacy-sqlite` feature ONLY — the C-SQLite→redb importer; never in default no-C build (dc-4) | 0.31.0 / 0.28.0 | the ONLY C dep; past all listed advisories; currency lag (trends §A4) | **HOLD (gated residual)** → **REMOVE** once fleet legacy ledgers migrated (ar-U5, SUPERVISED) |
| **toml 0.8.23** | shared config stack | 0.8.23 | minor lag; 0.9.x is out (rusty-idd already on 0.9.6); no advisory (trends §A5) | **OPTIONAL UPGRADE** to 0.9 for fleet consistency — not a gate, verify-before-bump |
| **syntect 5.3.0 (vendored, root `vendor/`)** | `crates/tui` transitively via tui-markdown (fs-4 correction) | vendored fork | fork remediates RUSTSEC-2024-0320 (yaml-rust) + RUSTSEC-2025-0141 (bincode) by upgrade, not suppression (gov supply-chain) | **HOLD** — record-positive |
| **clap 4.6.1 · serde 1.0.228 · serde_json 1.0.150 · ratatui 0.30.1 · crossterm 0.29.0 · tokio 1.52.3 · anyhow 1.0.102** | shared CLI/TUI/async stack | current | all current or one patch behind; 2026 tokio advisories are tokio-0.1 legacy, N/A (trends §A6) | **HOLD** (ratatui 0.30.1 vs rusty-idd 0.30.2 = trivial align) |
| **RuVector: rvf-crypto / rvf-runtime / rvf-index / rvf-types / ruvector-verified / ruvector-domain-expansion / cognitum-gate-tilezero** | `hf` + `ledger` via `../../RuVector/*` path deps (A-C5, EXP-1) | path dep | NOT a published/vendored dep — a CI-clone side-effect; standalone blocker | **RESOLVE (A-U1, SUPERVISED)** — vendor/publish/git-pin; the #1 action of the whole plan |

Net: the **first-party crypto/store stack is current and advisory-clear**; the only real tool action
is **RuVector** (a residency/portability problem, not a CVE) plus the optional toml 0.9 alignment and
the eventual `legacy-sqlite` removal. No invoked LLM/provider/vendor HTTP client exists in the kernel
(dc-3) — there is no provider-currency surface to evaluate.

---

## Governance

Per-axis roll-up — `findings/governance-config-handoff.md`. handoff IS the fleet's governance
substrate; the defects are bridge/portability gaps, not missing teeth.

- **HEADLINE fail-OPEN seam (gov-001, CONFIRMED empirical):** `.handoff/hooks/hooks.toml` declares 5
  `fail_mode="block"` hard gates (`:24,30,42,62,108`) but `.claude/settings.json` wires only
  `SessionStart`/`SessionEnd` — **no `PreToolUse`/`PostToolUse`**. A Claude edit can go out-of-scope
  without tripping the block gate: the kernel's own L7 fail-OPEN class. → **gov-U1 (PROPOSE, seq 5).**
- **agent-guard decorative in-repo (gov-002):** 8 destructive patterns authored, but enforcement lives
  in an uncommitted envctl user-global layer; a fresh clone is inert. → **gov-U2 = UP-5 (PROPOSE).**
- **Portability/config drift:** RuVector path deps (gov-003 = A-U1); no `rust-toolchain.toml` while CI
  pins 1.96.0 (gov-005 → gov-U5); `hf-mcp` binary with no `.mcp.json` (gov-006 → gov-U6, MCP rot);
  unconditional `exec rusty-idd next` SessionStart hook (gov-004 → gov-U4); `Bash(git -C * push:*)`
  any-repo push grant (gov-007 → gov-U7); rule lists 4 of 8 guard patterns (gov-009 → gov-U9, APPLY).
- **Record-positive:** `.cargo/audit.toml` `ignore=[]` (nothing suppressed); vendored syntect removes
  two advisories by upgrade; token budget capped (`policy.toml` wrap_strategy/context_budget_pct/cycle_flush).
- **Owner walls (not upgrades):** `master`-vs-`main` is a recorded owner alias (gov-008); the
  user-global agent-guard layer is unverifiable from a read-only worktree.

## Filesystem layout

Per-axis roll-up — `findings/filesystem-layout-handoff.md`. The kernel is clean at the SYSTEM
boundary (local-first, no global installs) but FAILS the **portable-root** mandate on two counts.

- **META→sibling upward path-dep [V1/V2]:** `hf`+`ledger` reach `../../RuVector/*` — the residency
  contradiction; `$META_ROOT+handoff` is not a portable root today (= A-U1).
- **Duplicate-lineage [V3]:** `crates/{cli,core,runner,spec,tui}` collide on `rusty-idd-*` pkg names
  with rusty-idd → dedup mandatory on union (= A-U4). Per-crate winner table: tui/spec trivial,
  runner moderate, core/cli substantial — rusty-idd superset wins, re-apply HFTASK-0082 lint.
- **USER→repo leak [V7]:** `.idea/` (13 JetBrains files) committed → un-track (fs-U3, APPLY).
- **Root clutter:** generated `.agent/skills-catalog.md` 313K blob (fs-U4), `intent-driven-template/`
  + Node `spike/ruvocal-mcp-bridge/` orphans at a pure-Rust root (fs-U5), `schemas/*.schema.json`
  unmarked-generated (fs-U6).
- **Corrected inbound facts (not silently accepted):** `_workspace*` are gitignored ephemeral, NOT
  committed [V11]; vendored syntect is at root `vendor/`, NOT `crates/tui/vendor/` [§0].
- **OK/correct:** `.handoff/` text-vs-binary split, `.kb/store/**` tracked + cache ignored, root
  `vendor/syntect` `[patch]`. FF enforcement gates E1-E8 specified to make each verdict fail in CI.

## Memory/vector

Per-axis roll-up — `findings/memory-vector-intelligence-handoff.md`. handoff is a **real, durable
operational-memory organ** but its vector plane is dead and its decision-memory is absent.

- **Strong (keep):** committed `.handoff/ledger.events.jsonl` is the continuity truth; cold-start
  `rebuild_from_jsonl` fails closed on witness-count mismatch (best-in-fleet); handoff IS a
  first-class git-kb member (committed `.kb/`) and drives git-kb for doc-sync (mem-5).
- **Dead weight (fix):** RVF overlay (`v2`, DEFAULT) writes a 384-dim SHA3 pseudo-embedding on EVERY
  append; `query_by_intent` has 0 callers, no `hf recall` verb — write-amplification with zero read
  (mem-1/2/6). → **mem-U1 (SUPERVISED, seq 10)** wire-with-real-embeddings-or-delete; **A-U2/mem-U2**
  stop paying for it by default.
- **Provenance correction:** witness chain = SHAKE-256 hash-link + SHA3-256 action, UNSIGNED — not
  blake3+ed25519 (mem-3, = EXP-3). → **mem-U3 (APPLY doc / PROPOSE signing).**
- **Missing:** ICM has 0 product refs (mem-4) — decision/"why" memory lives in commit prose. →
  **mem-U5 (PROPOSE)** ICM or ledger-curated decision events (ESAA pattern, trends §B1/§D1).
- **Five disjoint stores, no unified recall** (handoff ledger / RVF / ICM / git-kb / rusty-idd
  `.idd/knowledge`) → **mem-U6 (PROPOSE facade + ADR).**

## Auto-research

Per-axis roll-up — `findings/autoresearch-handoff.md`. Constant *daemon* auto-research is N/A (no
resident process), but pull/event-driven code + web research with **fail-closed** drift invalidation
is PRESENT and verifiable.

- **Code:** handoff drives git-kb via the `hf kb` seam (ADR-0003) + its own committed `.kb/`
  (HFTASK-0072); `handoff-index` re-derives nav maps + the task DAG on demand (C1-C3). One-shot/pull,
  not continuous (C4).
- **Web:** `rustsec/audit-check@v2` runs on the promotion gate with `ignore=[]` (C6/C7); Renovate
  opens currency PRs (C8). **Asymmetric (C9):** the advisory gate fires on promotion, NOT per-PR. →
  **ar-U2 (APPLY)** symmetric per-PR audit; **ar-U4 (APPLY)** one fleet currency bot (Renovate vs
  rusty-idd's Dependabot).
- **Invalidation:** `handoff-drift`/`hf drift` is the fail-closed engine (5-surface intent_lock incl.
  `northstar`; `exit(1)`→PreHandoff block, C12/C13/C15). A real stale `.git/gitkb/code.db` incident
  was caught + discarded this cycle (C16) — but by human cartography, not a check. → **ar-U1 (APPLY)**
  git-kb index-staleness gate; **ar-U3 (PROPOSE)** scheduled research cadence.

## Rules/policy

Per-axis roll-up — `findings/rules-policy-org-handoff.md`. HEADLINE: handoff's gates have **real
teeth** (`exit(1)` under `fail_mode="block"`) vs rusty-idd's advisory `mode="warn"` agent-guard
(rp-teeth, CONFIRMED empirical) — for the union the teeth live in handoff; rusty-idd folds UNDER them.

- **Enforced:** fail-closed L7, deny-without-claim, protected-files (two denylists), checkpoint+test+
  drift before handoff, ADR-on-architecture-change, gatekeeper end-state approval — all with `exit(1)`.
- **Declared-but-unenforced:** `default_network_mode`/`default_dependency_mode` are policy DATA with
  no kernel enforcement path (rp-declared-unenforced) → **UP-2 (QUALIFIED, default-warn→block).**
- **Org chart:** 9-agent org, no `evolution-steward`, uniform-opus with no per-role `model:` lane
  (rp-org-chart) → **UP-3 (PROPOSE add steward)**, **UP-4 (QUALIFIED witnessed dual-model lane —
  No-Downgrades guard blocks a silent gate-tier downgrade).**
- **A2A discipline (rp-A2A):** weave=transport plane, handoff=witnessed-receipts plane stay DISTINCT;
  offline degrades to ledger-only (`Reserve::Unsupported`→`ProceedDegraded`). Keep them un-fused.
- **Intent ≠ gate:** "Upgrade Only / parity-before-removal" is north-star intent with no machine gate
  (rp-upgrade-only-is-intent) — recorded honestly; the union's strict-upgrade-only is a frame, not yet
  enforced.
- **UP-1 (PROPOSE, seq 8)** fold rusty-idd CLI under the gates; **UP-5=gov-U2** self-enforce agent-guard.

## Distributed compute

Per-axis roll-up — `findings/distributed-compute-handoff.md`. handoff is a pure-Rust, no-daemon,
no-network kernel — the *runtime-execution* half (phones/glasses/Pi/ESP32) is genuine N/A — but it is
the **witnessed control/continuity plane** that distributes work across nodes.

- **What it implements (the hard part):** witnessed, fail-closed, offline-capable coordination across
  Git-reachable `std` hosts — `handoff-fleet` (git-as-sync rollup, no daemons), `handoff-route`
  (two-ledger residency, fail-closed), `handoff-lease` (weave mesh lease, degrades to ledger-only),
  `hf-mcp` (the T11 MCP control seam). Transport = Git + `weave`/`gh` subprocess, zero in-process
  network surface (dc-1).
- **Lua/Luau: GENUINE N/A — zero presence** (dc-2, 0 grep hits). → **DC-5 (PROPOSE guardrail ADR):**
  no embedded/Lua/in-kernel-network stack in handoff; firmware + Lua belong to executor repos
  (protects the no-C boundary — mlua links C Lua).
- **Missing executor leg:** no model of non-`std` leaf nodes → **DC-2 (PROPOSE)** leaf-node proxy
  contract over the existing MCP/work-order seam; **DC-3 (QUALIFIED, SUPERVISED)** native weave mesh
  binding (first live network dep — feature-gated, byte-identical offline fallback); **DC-4
  (PROPOSE)** enforce `allows_network`/`path_scope` cross-node egress at the gatekeeper/route seam.
- **The real compute coupling = RuVector path-dep (DC-1 = A-U1)** — the standalone blocker.

## Test Strategy

The testing component — lifted from `findings/test-strategy-handoff.md` (coverage = call-graph
reachability, not file presence). Read-only: it PLANS tests; planning-engineer authored + RED-ran the
additive suite; **Feature Forge builds production code and GREEN-runs it.**

**Current coverage (reachable):**
- `work-order` producer/mint seam well-covered — 15 `#[test]`s reach `work_orders_from_bundle`,
  `compute_intent_lock`, `intent_unchanged`, `task_schema_json` (`work-order/src/lib.rs:429-672`).
- `handoff-schema::validate_card` JSON-schema path covered — 4 tests
  (`handoff-schema/src/lib.rs:156-191`); `validate_card` is the highest-blast fail-closed gate
  (blast 40, `graph/handoff.graph.md:63`) and IS tested.

**Coverage gaps (ranked by graph risk):**
1. **No fail-closed work-order LOADER** — the only load surface is `serde_json::from_str` (FAIL-OPEN);
   `#[schemars(regex)]` is schema-doc only, NOT serde-enforced (`work-order/src/lib.rs:56-92`). The
   union consumer (rusty-idd) inherits this via the mirror (ts-2/ts-3).
2. **`validate_card` cannot catch intent_lock-vs-content drift** — it is pure JSON-schema and cannot
   recompute blake3; nothing on a load path chains it with `intent_unchanged` (ts-4).
3. **Ledger read API MISSING** — no public surface for an intent-plane consumer; internal to `hf`
   (`Ledger.open` blast 120, all callers in-kernel) (ts-5).
4. **`ledger` cannot be tested standalone** — the RuVector wall fails the whole workspace at
   manifest-load (ts-6, EXP-1).

**Designed suite (closes the gaps + covers the roadmap's upgrades):**
- AUTHORED + COMMITTED + RED-verified: `work-order/tests/union_failclosed.rs` (`d74ad4b`) — 4 tests,
  empirically `1 passed; 3 failed` (foreign-schema / malformed-id / drifted-intent_lock all FAIL-OPEN;
  fixture GREEN). This is **ts-U1**, a true RED (`tests-ran: 4`), not an exit-0 fail-open.
- BLOCKED-on-RuVector (designed, FF runs post-A-U1): `handoff-intake/tests/intake_failclosed.rs`
  (ts-U2, front-door refusal); `ledger/tests/read_api.rs` (ts-U3, design-only until the read API is
  built — a call to a non-existent `Ledger::query_claimed` would fail to COMPILE, which is forbidden).
- CHEAP drift gate: golden of `task_schema_json()` asserted byte-identical against the rusty-idd
  mirror (ts-U4).

**FF test-build spec (the GREEN handoff — carried below; promoted as a Feature-Forge backlog item):**

### FF test-build spec

Verification-plan intake for Feature Forge (`feature-architect` `## Verification plan`). The RED suite
is AUTHORED + RED-run by the planning-engineer; **Feature Forge builds the production code that flips
it GREEN — do NOT rewrite the tests.**

- **backlog id:** FF-handoff-001
- **title:** Fail-closed `handoff.task.v1` work-order LOADER (+ intake refusal + ledger read-API once unblocked)
- **kind:** test-build (RED → GREEN); engine-first, additive-only, no-downgrade
- **RED suite (authored, do not rewrite):** `work-order/tests/union_failclosed.rs` (commit `d74ad4b`)
- **RED tests:** `workorder_load_rejects_foreign_schema_card`, `workorder_load_rejects_malformed_id_card`,
  `workorder_load_rejects_card_with_drifted_intent_lock`
- **GREEN fence (must stay GREEN):** `fixture_is_a_clean_valid_card`
- **Production change to flip RED→GREEN:** add `WorkOrder::from_card_json(s: &str) -> Result<WorkOrder, LoadError>`
  (and/or `try_from_value(Value)`) in `work-order/src/lib.rs` that (1) `serde_json` deserializes,
  (2) calls `handoff_schema::validate_card` on the raw `Value`, (3) calls `intent_unchanged()` and
  rejects on mismatch; then point the 3 RED tests at the new loader and assert `.is_err()`.
- **Blocked-until-A-U1 cases:** `handoff-intake` refuses foreign-schema/bad-id cards at the front door
  (no card written to `.handoff/tasks/`); `ledger` exposes read-only `get_claimed()`/`latest_checkpoint()`
  that never mutate the witness chain.
- **Differential/golden:** golden of `work_order::task_schema_json()` == the rusty-idd mirror byte-for-byte.
- **Coverage target:** every fail-closed branch of the card-LOAD path has a reachable refusal test;
  the ledger read API has a read-only contract test.
- **CI gate(s) touched:** `cargo test`/`cargo nextest` (workspace test gate) once RuVector resolves in
  CI (`hf/Cargo.toml:46`); Format/clippy preflight unaffected (tests additive). **RuVector availability
  (A-U1) is the prerequisite for running anything below the `work-order`/`handoff-schema` leaf layer.**

## Prompt-architecture

Per-axis roll-up — `findings/prompt-architecture-handoff.md`. Where handoff sits relative to owner D3's
Front-Door interpreter (harness_hub).

- **handoff is the binding/landing point, NOT the interpreter (pa §6):** `prompt_hub` IS the confirmed
  front-door seam (`hf prompt-hub "<vibe>"` → `WorkOrder`), but the intent→spec transform is
  **DETERMINISTIC keyword classification**, explicitly chosen to be non-LLM so it can't rubber-stamp
  drift (pa-determinism-intake). harness_hub's intent→model-language interpretation sits **UPSTREAM**
  of, or replaces, that classifier; handoff supplies the verifiable landing contract (`work-order`
  schemars SoT + witnessed ledger).
- **Dual front door (pa-dual-front-door / pa-fork-drift):** TWO SessionStart hooks (`hf` loop-entry +
  `rusty-idd next`), and the in-repo `rusty-idd-cli` fork lacks the `next`/`render` verbs its own hook
  needs (resolves an external superset binary on PATH). → **pa-U1 (PROPOSE)** reconcile to ONE
  canonical Front Door.
- **`hf-mcp` (pa-hf-mcp):** ~35 tools, each a shell-out to `hf`; mutating tools (ship/done/claim)
  ungated at the MCP layer (governance delegated to `hf`'s own gates). → **pa-U2 (PROPOSE)** pin/version-
  stamp `hf`↔`hf-mcp` instead of PATH+warn.
- **Single opus lane (pa-single-opus-lane):** model routing lives only in one skill → **pa-U4 (APPLY)**
  make the lane explicit policy (= UP-4 documentation half).
- **ADR-candidates:** canonical Front Door & interpreter boundary; `hf-mcp` as the union's T11 control
  seam; the deterministic-classifier-vs-LLM-interpreter split (the constraint harness_hub must respect:
  interpret upstream, never author acceptance criteria the gate trusts). These feed
  `reports/ADR-DRAFT-handoff-rusty-idd-union.md` and the north-star DRAFT.

## Risk policy

Execution risk-tiers, trust-boundary / secrets / destructive / provider / model rows, and the
APPLY/PROPOSE/SUPERVISED classification of the union steps are in the companion `risk-policy.md`
(`## handoff (cycle 2)` section, appended this cycle, cycle-1 content preserved). Summary:

- **SUPERVISED (owner-walled, large blast / invariant-crossing):** **A-U1 (RuVector resolution)** and
  **A-U4 (the MERGE / fork dedup)** — large blast (Ledger.open 120; 5 crates + name collision),
  owner-walled; **DC-3** (first live network dep); **mem-U1** (native embedder must be C-free);
  **ar-U5** (delete last C dep only after fleet migration).
- **PROPOSE:** the governance/policy/structural rows (gov-U1/U2/U4/U5/U6/U7, A-U2, A-U3, UP-1/2/3/4,
  mem-U3-signing/U5/U6, pa-U1/U2, DC-2/U4/U5, fs-U5, ar-U3).
- **APPLY:** contained/gate-only/doc rows (ts-U1, ts-U4, A-U5(gated), A-U6, gov-U9, mem-U3-doc, pa-U3,
  pa-U4, ar-U1/U2/U4, fs-U3/U4/U6).
- **Invariant (CONFIRMED dc):** NO C in the trust path — only native surface is the pure-Rust
  sha3/blake3/ed25519-dalek + redb; `rusqlite` is feature-gated migration-only. Every upgrade respects
  it; the filesystem `.handoff/` contract is preserved as the offline fallback in every transport row.

---

## Confidence

**Overall: HIGH on the kernel facts and the blocker; MEDIUM-HIGH on the union execution.**

- **HIGH** — the architecture (strict DAG, two-program split, witness algorithm), the RuVector blocker
  (EXP-1 empirical: manifest-load fails even for `redb-store`), the real-teeth gates (rp-teeth
  empirical), the dead RVF plane (0 callers grep), the fail-OPEN hooks seam (gov-001 empirical), the
  RED suite (`1 passed / 3 failed`), and the three KEY CORRECTIONS (EXP-3 witness=SHAKE-256-unsigned;
  A-C7 rvf-crypto default-features; pseudo-embeddings). All cite path:line / graph row / empirical run.
- **MEDIUM-HIGH** — the union *execution* (A-U4 MERGE): lineage/superset/collision facts are CONFIRMED
  but the exact divergence % is tool-derived (A-C9 QUALIFIED), and the per-crate core/cli reconcile is
  substantial (~33%/~40%). The C-dep scan of rusty-idd's `codex`/`knowledge` (syntect-onig/codegraph)
  passed in cycle-1 but **must be re-checked at land** (union-2 condition c).

**Named gaps / what stayed INCONCLUSIVE / not examined:**
- **Ledger read API (Seam 2) is unbuilt** — ts-U3 is design-only; cannot author a compiling RED until
  the API is designed AND A-U1 lands. The union's witnessed-read seam is the largest open design item.
- **Below-leaf tests are blocked** — `ledger`/`handoff-core`/`handoff-intake`/`hf` cannot build or be
  tested standalone in the worktree (RuVector wall); ts-U2 and the standalone-build gate (E1) are
  gated on A-U1.
- **MCP enforcement unverified** — whether `hf-mcp` inherits the same fail-closed gates as the `hf`
  CLI was not exercised (read-only) — flagged for the verifier (gov gap).
- **User-global agent-guard layer unreadable** from the worktree — the in-repo portability defect
  stands regardless, but the actual enforcement on a non-envctl host is INCONCLUSIVE (gov-002).
- **dead-code (1258 candidates)** is heavily false-positive (clap string-dispatch) — not a removal
  list; per-symbol triage required (A-C15) — A-U6 plans the manifest-cross-check gate.

**What would raise confidence:** (a) land A-U1 and run the standalone-build gate (E1) + the below-leaf
tests — flips the standalone claim and ts-U2/ts-U3 from blocked to runnable; (b) the at-land C-dep
re-scan of rusty-idd `codex`/`knowledge`; (c) design the ledger read API so ts-U3 can be authored as a
compiling RED. No false "fully planned": the kernel is well-understood, the union is feasible under the
stated conditions, and the build is provably blocked until A-U1.
