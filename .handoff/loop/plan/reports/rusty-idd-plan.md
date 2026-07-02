# rusty-idd — decision-grade PLAN + fleet-convergence report (cycle 1)

Author: plan-architect (R4 + R7). Run: Planning Engineer Loop · fleet-convergence-first-run · 2026-06-26.
Source SHA `5a55284`, branch `plan/lifeos-meta-front-door`. Built **only** from CONFIRMED/QUALIFIED +
feasibility-passed rows in `findings/verdicts.md` (22 CONFIRMED · 1 QUALIFIED · 0 REFUTED). Docs only —
no production code touched. rusty-idd is read-only this run; every promotion below is a DRAFT/PROPOSED
owner action, not an applied change.

Citation discipline: the verifier's two corrections are carried throughout — (a) `merge.rs` is at
`crates/spec/src/model/merge.rs` (21.4 KB), not `crates/spec/src/merge.rs`; (b) there are **5** `spec_*`
CLI command files (`spec_adr`, `spec_archive`, `spec_plan_integration`, `spec_scaffold`, `spec_status`),
not 6. The two QUALIFIED feasibility conditions are honored: U7's adapter boundary keeps weave the
required local route and stays C-free in the trust path; DC-2's weave/A2A transport keeps the filesystem
`.handoff/` contract as the offline fallback behind a feature flag.

Legend (envctl `docs/runbook/DIAGRAMS.md`): `[A]` automated · `[A*]` elevated/sudo · `[P]` preview/dry-run
· `[H]` human-gated · `[!!]` supervised/critical.

---

## Verdict

**rusty-idd is the fleet's intent / why-what control plane — a clean-DAG, engine-first, pure-Rust
single binary whose architecture is sound but whose mass is mis-distributed, and whose one connection to
the fabric is filesystem + JSON-schema contracts, never a live binding.** The single decision-relevant
gap is the **typed convergence/adapter boundary (U7)**: today weave (comms/A2A), icm (memory), grit
(merge), and the `hf` continuity kernel have **zero** library/IPC dependency in product code
(`grep -rinE 'weave|icm|grit|\bhf\b' crates/*/Cargo.toml` → none; the only in-code refs are descriptive
`&str` catalog data at `crates/knowledge/src/lib.rs:3585-3725` and harness-contract text at
`crates/cli/src/commands/harness.rs:208-265` — CONFIRMED C11). The recommendation is **not** to rewire
rusty-idd into the fabric this cycle; it is to (1) finish the contained, high-value hygiene + accuracy
wins, (2) decompose the three god-files that hold the product's risk, and (3) define a C-free adapter
boundary that lets weave/icm/grit/hf bind as libs/IPC as strict upgrades while the filesystem `.handoff/`
contract stays the required fallback.

What makes the recommendation safe to act on: the crate graph is a verified **clean DAG, zero
cross-crate cycles** (`metrics.json:111-114`); `cli` is the unique sink-of-control and nothing depends on
it (C1); and the one seam already shaped to the fabric — `work-order` = `handoff.task.v1` — exists and is
tested intra-crate, it is merely **unconsumed** (24 dead symbols, zero product callers, C7/F5). The path
into the fabric is therefore an *additive wiring* problem, not a redesign.

The one item that must move regardless of the convergence work: the `work-order` card load path is
**fail-open** — the only deserialize (`serde_json::from_str::<WorkOrder>`) silently accepts a card whose
`schema` discriminator, `id` pattern, and `intent_lock` the published contract rejects (CONFIRMED
ts-24/25/26; RED suite already authored and failing for the right reason). That is the highest-risk
coverage gap and the concrete first step of the convergence path (U6 / DC-1).

**Headline confidence: Medium-high** (see `## Confidence`).

---

## rusty-idd convergence report — current state → gap → path

### Current state (CONFIRMED)

- **Shape.** CLI + TUI + library, single binary `rusty-idd` (`crates/cli/src/main.rs::main`); **no**
  internal HTTP/service routes (`metrics.json:151-154`, layering `internal_http_routes: 0`), no daemon,
  no scheduler, no message bus, no embedded/networking runtime (CONFIRMED dc no-C; distributed-compute
  audit §4). It is offline-by-construction in the control path.
- **Gravitational center.** The OpenSpec model: `SpecDoc.contains` is the #1 product symbol at 842
  callers, a one-line query primitive (`crates/spec/src/model/spec.rs:30-32`, C2).
- **Risk concentration.** Three god-files hold the highest blast *and* the largest line-counts at once:
  `runner/src/runner.rs` (blast **803**, 2,146 LOC, ~12 top-level items, C3), `tui/src/app.rs` (blast
  248, 5,708 LOC, C4), `knowledge/src/lib.rs` (blast 105, 7,058 LOC single file, C5).
- **Dead/vendored mass.** ≥278 dead symbols (lower bound, truncated at the 500 cap), 182 (65%) inside
  the vendored `external/codegraph-{core,parser}` trees, which also expose a wider public surface (355)
  than the product's own crates (cli 74, core 71) — C8/C9.
- **Fabric coupling.** Filesystem + JSON-schema only: `.handoff/tasks` read at
  `crates/cli/src/commands/codex.rs:594`; `_workspace/{backlog,loop_state,HANDOFF}.md` declared at
  `crates/merge-tools/src/lib.rs:110`; the `handoff.task.v1` envelope is `crates/work-order` (C12). No
  lib bindings to weave/icm/grit/hf (C11).

### Gap to the fabric (the plannable absence)

| gap | what's missing | evidence |
|---|---|---|
| **G3 — fabric seam un-integrated (headline)** | `work-order` (handoff.task.v1) is consumed by nothing; the path *into* weave/icm/grit/hf is undesigned in product code | C7/C11/C12; graph F5; `work-order` 24 dead (`metrics.json:104`) |
| **G-card — fail-open card load** | no validating consumer; `serde_json::from_str::<WorkOrder>` accepts foreign `schema`, bad `id`, drifted `intent_lock` | ts-24/25/26/27/28; RED suite `crates/work-order/tests/handoff_card_consumer.rs` (3 RED, 1 GREEN) |
| **G1 — god-file risk** | the 3 highest-blast surfaces are also the largest files; high-risk to change, hard to test in isolation | C3/C4/C5; blast_radius `metrics.json:90-92` |
| **G2 — vendored bloat / triple-dup** | 182 dead vendored symbols; handoff vendored 3×; deprecated `serde_yaml` kept in the lock | C8/C14; graph F1/F2 |
| **G4 — control-plane completeness** | 30 dead `spec` symbols; the OpenSpec lifecycle is not fully reachable from the 5 `spec_*` CLI commands | C13 (QUALIFIED, corrected counts); `metrics.json:104` |
| **G5 — layout/governance drift** | `crates/config/` is a non-crate orphan; mixed editions; fail-open Claude harness; decorative agent-guard; toolchain nightly-vs-stable | C6, C15, gov-001, gov-002, gov-003 |

### Path (sequenced; full detail in `## Sequenced upgrade roadmap`)

Contained APPLY wins first (U9 layout guard, U10 spec reachability, U8 serde_yaml) → close the fabric
seam with the fail-closed card consumer and consume `work-order` (U6/DC-1, the RED suite goes GREEN) →
decompose the god-files that de-risk later fabric wiring (U1→U2→U3 / FL-3) → speed/hygiene (U4, U5) →
define the C-free adapter boundary (U7) and its flagged transport (DC-2), with the no-firmware/no-Lua-
runtime guardrail (DC-5) recorded alongside. The analyst's sequencing law is honored: **decompose
runner/tui/knowledge (G1) before wiring fabric adapters (G3) through them.**

---

## ASCII architecture

### Diagram 1 — current state (CONFIRMED edges only)
Source: `graph/rusty-idd.graph.md:25-56`, `reports/codemap-rusty-idd.md:39-47`.

```
                         ┌──────────────────────────────────────┐
                         │  crates/cli   bin: rusty-idd  [A]     │  unique sink-of-control
                         │  src/main.rs::main   blast=2          │  (nothing depends on cli)
                         └───┬───┬───┬───────┬────────┬───────┬──┘
            ┌────────────────┘   │   │       │        │       └────────────────┐
            ▼                    ▼   ▼       ▼        ▼                         ▼
        ┌────────┐         ┌────────┐ ┌───────────┐ ┌────────┐          ┌──────────────┐
        │ core   │         │ spec   │ │ merge-    │ │ runner │          │  knowledge   │
        │ blast=0│         │ blast=0│ │ tools  11 │ │ ★803   │◀──tui    │   ★105       │
        │ facade │         │ facade │ └───────────┘ └────────┘    │     └──┬────────┬──┘
        └────────┘         └────────┘                      ┌──────┘        │        │
                  SpecDoc.contains = 842 callers (#1)    ┌─┴────┐          ▼        ▼
                  the OpenSpec gravitational center      │ tui  │   ┌────────────┐ ┌──────┐
                                                         │ ★248 │   │ codegraph- │ │ core │
                                                         └──────┘   │  parser    │ └──────┘
                                                                    └─────┬──────┘
                                                                          ▼
                                                                  ┌────────────┐
                                                                  │ codegraph- │  355 public
                                                                  │   core     │  120 dead (vendored)
                                                                  └────────────┘
   Unlinked leaves (no internal edges): work-order (handoff.task.v1, 24 dead) · external/repomix-shared
   ★ = blast radius (transitive callers, code impact --depth 3)

   FABRIC COUPLING (CONFIRMED absent as code): weave · icm · grit · hf  ── 0 lib/IPC deps ──┐
   The ONLY attach point is the filesystem + JSON-schema contract:                          │
        .handoff/tasks/*.json  (read @ codex.rs:594)   _workspace/*.md (@ merge-tools:110)  │
        handoff.task.v1 envelope (crates/work-order)  ◀── shaped, UNCONSUMED (the gap) ◀────┘
```

### Diagram 2 — target state (the convergence boundary; all edges PROPOSED/QUALIFIED)
Source: derived from CONFIRMED C11/C12 + feasibility-gated U7 (QUALIFIED) + DC-2 (QUALIFIED);
`findings/architecture-rusty-idd.md:59`, `verdicts.md:39,61`.

```
                       ┌───────────────────────────────────────────┐
                       │  OWNER intent / OpenSpec goal (why+what)    │  [H]
                       └───────────────────────┬───────────────────┘
                                               ▼
                                    ┌───────────────────────┐
                                    │  rusty-idd CONTROL     │  pure-Rust, no-C trust path
                                    │  PLANE (spec engine)   │
                                    └───────────┬───────────┘
                                                │ mints + validates (fail-closed)
                                                ▼
                                    ┌───────────────────────┐
                                    │  work-order  =         │  schema/id/intent_lock enforced
                                    │  handoff.task.v1  [A]  │  ON the deserialize path (U6/DC-1)
                                    └───────────┬───────────┘
                                                ▼
                          ┌───────────────────────────────────────┐
                          │  crates/interop  — TYPED ADAPTER       │  U7 (QUALIFIED, C-free)
                          │  4 fabric ports behind one trait       │
                          └──┬──────────┬──────────┬──────────┬────┘
            filesystem (REQUIRED          weave            icm           grit
            offline fallback) [A]      (A2A/comms)       (memory)       (merge)
                 │ first impl,            │ [P] flag        │ [P]          │ [P]
                 │ always present     ┌───┴───┐         ┌───┴───┐      ┌───┴───┐
                 ▼                    │ DC-2  │         │ icm   │      │ grit  │
        .handoff/tasks/*.json        │ A2A   │ pure-   │ recall│      │ locks │
        (degraded path, never        │ v1.0  │ Rust    │/store │      │       │
         removed — no-downgrade)     └───────┘ tonic   └───────┘      └───────┘
   Invariant gate (U7/DC-2 QUALIFIED): weave stays the required LOCAL route; every adapter is
   C-free in the trust path (no C TLS / native vector lib); filesystem .handoff/ contract = fallback.
```

### Diagram 3 — control plane (hooks → policy/guard → claim → handoff; settings → runtime; toolchain → CI)
Source: `findings/governance-config-rusty-idd.md` (gov-001/002/003/005), `findings/prompt-architecture-rusty-idd.md` (C13/H4),
`.claude/settings.json`, `.codex/hooks.json`, `Justfile:119`.

```
  LIFECYCLE HOOKS                         POLICY / GUARD                 CLAIM            HANDOFF
  ┌───────────────────────┐   invokes    ┌────────────────────┐  gates  ┌──────────┐    ┌─────────────┐
  │ .codex/hooks.json [A]  │────────────▶│ codex workflow-check│───────▶│ .idd/    │──▶ │ .handoff/   │
  │ Session/Pre/Post/Stop/ │   (6 pts)    │ env-check           │        │ LOCK.md  │    │ tasks/*.json│
  │ SubagentStop  (cargo   │             └────────────────────┘  one    │ (1 branch│    │ capsule.json│
  │ run, 180s) [!!]        │              ┌────────────────────┐ writer │  authority)   │ Git>ledger  │
  └───────────────────────┘              │ agent-guard.toml    │  [H]    └──────────┘    │  >cards     │
  ┌───────────────────────┐   invokes    │ mode="warn" (no     │                          └─────────────┘
  │ .claude/settings.json  │────────────▶│ teeth, decorative)  │   GAP gov-001: Claude harness gates ONLY
  │ SessionStart ONLY [A]  │   (1 pt)     │ deny[] unparsed     │   SessionStart → fail-OPEN vs Codex's 6 pts
  └───────────────────────┘              └────────────────────┘   GAP gov-002: deny[] never enforced

  SETTINGS → RUNTIME                                 TOOLCHAIN / CARGO → CI
  ┌────────────────────┐  computes  ┌──────────┐     ┌──────────────────┐  builds/lints  ┌────────────┐
  │ rusty-idd next [A] │──────────▶│ next step │     │ Cargo.toml 1.88  │──────────────▶│ ci.yml [A] │
  │ (engine = oracle)  │           │ (workflow)│     │ resolver=3       │   GAP gov-003: │ clippy -D  │
  └────────────────────┘           └──────────┘     │ NO rust-toolchain│   CI=nightly,  │ on NIGHTLY │
                                                     │  .toml           │   manifest=1.88│ cargo audit│
                                                     └──────────────────┘   → false-green └────────────┘
```

---

## Sequenced upgrade roadmap

Ordered by value/risk using graph centrality + blast-radius: contained-blast APPLY wins first;
high-blast god-file decompositions sequenced before the fabric wiring that would route through them; the
headline boundary (U7) and its transport (DC-2) last, gated on their QUALIFIED conditions. Every row
traces to a CONFIRMED/QUALIFIED + feasible verdict in `verdicts.md`. `P8 acceptance test` = the
falsifiable fail-closed gate that flips the item done.

| # | order | upgrade | axis | target-surface | evidence (file:line / graph) | blast | effort | risk-tier | P8 acceptance test | reversibility (Integrity·Reversibility·Capability-Gain) |
|---|---|---|---|---|---|---|---|---|---|---|
| **U9** | 1 | Resolve `crates/config/` stray dir + CI member-guard | governance | `crates/config/`, `Cargo.toml`, CI | C6 (`ls crates/config`=example.toml only; absent `Cargo.toml:25-37`); graph F3; verdict U9 | 0 | S | **APPLY** | RED test asserts every source-bearing child of `crates/` (except `external/`) is a workspace member — currently fails for `crates/config/` | Integrity preserved · Reversible (`git mv` back) · enforces the layout invariant |
| **U10** | 2 | Wire-or-mark the 30 dead `spec` engine symbols | accuracy | `crates/spec/*`, `crates/cli/src/commands/spec_*.rs` (5 cmds) | C13 QUALIFIED (`metrics.json:104` spec 30 dead; corrected 5 `spec_*` files; `model/merge.rs` 21.4 KB) | 0 (facade `metrics.json:96`) | M | **APPLY** | `code dead` for spec drops (new CLI call-paths) OR each remaining dead public symbol carries a documented intentional-public marker; a test asserts no *undocumented* dead public symbol | Integrity preserved · Reversible · full OpenSpec lifecycle reachable/auditable |
| **U8** | 3 | Migrate vendored `codegraph-core` off deprecated `serde_yaml 0.9` → `serde_norway` | governance | `crates/external/codegraph-core/Cargo.toml:40` | C14 (`cargo tree -i serde_yaml` → sole source codegraph-core; first-party already on serde_norway); verdict U8 | 105 (via knowledge) | S | **PROPOSE** | `cargo tree -i serde_yaml` returns empty (or only documented-accepted) AND build + codegraph tests stay green | Integrity preserved · Reversible (Cargo.toml revert) · no deprecated dep in lock |
| **U6 / DC-1** | 4 | Integrate-or-retire `work-order` **with a fail-closed card consumer** (the RED suite → GREEN) | accuracy / distributed-compute | `crates/work-order` (+`tests/handoff_card_consumer.rs`), `crates/cli/src/commands/codex.rs:594` | C7 (24 dead, no consumers); ts-24/25/26/27/28; verdict U6; FF spec `findings/test-strategy-rusty-idd.md:77-91` | 0 today (integration adds edges) | M | **PROPOSE** | the 3 RED tests pass via fail-closed deserialize (schema `const` + id `pattern` + intent_lock recompute) AND `baseline_well_formed_card_loads` stays GREEN AND `code dead` for work-order → ~0 OR crate removed and repo builds | Integrity preserved (additive-only behavior) · Reversible (command behind a flag) · real handoff-envelope consumption — the fabric seam |
| **U1** | 5 | Decompose `runner/src/runner.rs` into sub-modules behind unchanged public API | quality | `crates/runner/src/runner.rs` (→ `runner/` modules) | C3 (blast 803, 2,146 LOC, 12 items; `metrics.json:90`); hotspot `BatchImplState.new` 221 (`metrics.json:21`); verdict U1 | 803 (realized ≈0 if API held) | L | **PROPOSE** | `runner` public symbol set byte-identical before/after (cargo-public-api / `code symbols` diff = ∅) AND existing runner/tui tests green | Integrity preserved (no behavior change) · fully Reversible (mechanical move) · testable units + lower change-risk on the #1 surface |
| **U2** | 6 | Split `tui/src/app.rs` into screen/state/input modules under `tui/src/app/` | quality | `crates/tui/src/app.rs` | C4 (5,708 LOC, blast 248; `App.new` 355, `handle_config_input` 126; `metrics.json:13-16,32-35,84-87`); verdict U2 | 248 (realized ≈0) | L | **PROPOSE** | `tui` public API diff = ∅ AND a new unit test exercises an extracted input-handler module in isolation | Integrity preserved · Reversible · per-screen testability, faster incremental builds |
| **U3** | 7 | Split `knowledge/src/lib.rs`; repo catalog → external data file | quality | `crates/knowledge/src/lib.rs` (catalog `:3585-3725`) | C5 (7,058 LOC single file, blast 105; `metrics.json:91`); verdict U3 | 105 | M | **PROPOSE** | catalog facts load from an external data file (round-trip test: parsed == prior hard-coded set) AND `knowledge` public API diff = ∅ | Integrity preserved · Reversible · fleet topology editable without recompiling the engine |
| **FL-3** | (5-7 coordinated) | Gate `no src/*.rs > 1500 LOC` in first-party crates | filesystem-layout | first-party `crates/*/src/**` | FL-3 CONFIRMED feasible (`verdicts.md:63`); same behavior-preserving class as U1-U3 | n/a (gate) | S | **PROPOSE** | golden: public API + tests unchanged pre/post; gate: no first-party `src/*.rs` > 1500 LOC — RED today on knowledge/tui/runner | Integrity preserved · Reversible · drift fails CI (coordinate with U1-U3 to avoid duplicate work) |
| **U4** | 8 | Feature-gate the 182 dead vendored codegraph symbols (off by default) | speed | `crates/external/codegraph-{core,parser}` (Cargo features + `#[cfg]`) | C8/C9 (182 of 278 dead; 355 public); knowledge imports `default-features=false` (`lib.rs:7-12`); verdict U4 QUALIFIED | 105 (sole consumer) | M | **PROPOSE** | `cargo build -p rusty_idd_knowledge` green with the slim feature set AND `code dead` in `external/codegraph-*` drops by ≥100 AND a before/after build-time + dead-count measurement is recorded (magnitude must be measured, not asserted) | Integrity preserved (gated, not deleted) · Reversible (feature flag) · smaller build/audit surface |
| **U5** | 9 | De-duplicate vendored upstreams (handoff vendored 3×) — one canonical copy | governance | `third_party/upstream/*`, `imports/*`, `.gitignore` | C8; graph F1 (handoff 3×); verdict U5 (vendored trees not members `Cargo.toml:25-37`) | 0 on product | M | **PROPOSE** | exactly one tracked path per vendored upstream remains; `code doctor` symbol_count drops by the duplicate delta AND product `cargo build` unaffected | Integrity preserved · Reversible (git history) · truthful index + smaller repo (owner-walled: deletes tracked trees) |
| **U7** | 10 | **Define the typed convergence/adapter boundary** (filesystem first impl; weave/icm/grit/hf as adapters) | governance | new `crates/interop` (or `core` trait module); catalog `knowledge/src/lib.rs:3585-3725` → typed registry | C11/C12 (0 lib deps; filesystem-only); trends D1 (A2A v1.0); verdict U7 **QUALIFIED** (`verdicts.md:39`) | ~0 (new boundary, opt-in) | L | **PROPOSE** | a trait defines the 4 fabric ports with ≥ the filesystem adapter implementing it AND a contract test asserts the `handoff.task.v1` round-trip through the boundary. **Condition:** weave stays the required local route; every adapter C-free in the trust path | Integrity preserved · Reversibility: hard to fully back out once depended on (owner-walled) · the path INTO the one fabric (the headline gain) |
| **DC-2** | 11 | Bind issued work-orders to weave/A2A transport carrying `correlation_id` (behind a feature flag) | distributed-compute | `crates/work-order` (`lib.rs:67-69`) + interop weave adapter | DC-2 **QUALIFIED** (`verdicts.md:61`); weave pure-Rust; A2A v1.0 LF-governed (trends D1) | low (first live network/IPC dep) | M | **PROPOSE** | an emitted order appears as a weave job keyed by `correlation_id`; a stub remote executor ACKs it. **Condition:** gated behind a transport feature flag; filesystem `.handoff/` contract remains the offline/degraded fallback | Integrity preserved · Medium reversibility (feature flag) · live delivery to a remote executor without losing the offline path |
| **DC-5** | record now | Guardrail ADR: no `mlua`/`esp-hal`/`no_std` in rusty-idd; firmware + Lua/Luau runtime belong to fleet-executor repos | distributed-compute (guardrail) | ADR-candidate only | DC-5 CONFIRMED feasible (`verdicts.md:62`); protects the no-C boundary (mlua links C Lua) | 0 | S | **PROPOSE** | the guardrail recorded as an ADR-candidate; no embedded/Lua-runtime crate enters rusty-idd's `Cargo.toml` (CI grep gate) | Integrity preserved · trivially Reversible · protects the no-C/no-downgrade invariant |

**Axis tally (gated rows):** quality 3 (U1,U2,U3) · accuracy 2 (U10, U6) · speed 1 (U4) · governance 3
(U9,U8,U5) + U7 (governance, headline) · distributed-compute 2 (DC-1/DC-2) + DC-5 (guardrail) ·
filesystem-layout 1 (FL-3). APPLY: U9, U10. All others PROPOSE (owner-walled or structural). See
`../risk-policy.md` for the APPLY/PROPOSE/SUPERVISED classification and the trust-boundary/secrets/
destructive/provider-model rows.

---

## Per-axis roll-up

### Governance

CONFIRMED claims (spot-gated): **gov-001** fail-OPEN harness drift — `.claude/settings.json` gates only
SessionStart while `.codex/hooks.json` gates 6 lifecycle points, so a Claude agent runs ungated; **gov-002**
agent-guard is decorative — its sole consumer (`crates/core/src/validation.rs:48`) checks existence only,
`mode="warn"`, no PreToolUse hook parses `deny`; **gov-003** toolchain drift — CI defaults to nightly
(`scripts/ci/envctl-rust-env.sh:121`), manifest advertises stable 1.88 (`Cargo.toml:22`), no
`rust-toolchain.toml`, so local clippy ≠ the nightly `-D warnings` CI gate (false-green locally, red in
CI). Gated upgrade in the roadmap: **U9** (config orphan + member-guard, APPLY) and **U8** (serde_yaml,
PROPOSE). Analyst-proposed, not-yet-verifier-gated governance candidates (carried to the next cycle's
gate, listed honestly so they don't masquerade as confirmed plan rows): gov-001 (gate the Claude harness
identically to Codex via render-owned hooks), gov-002 (give the deny[] real PreToolUse teeth, `warn`→`block`
— **strengthens**, never weakens), gov-003 (`rust-toolchain.toml` of record), gov-005 (prebuilt-binary
hooks vs per-call `cargo run` — token/latency burn), gov-006 (single dependency bot: Renovate xor
Dependabot), gov-007 (cap `*.idd-bak-*` rotation — APPLY-class hygiene), gov-008 (gate the actually-enforced
`.codex/*` control plane in `validation.rs`).

### Filesystem layout

CONFIRMED basis: **C6** `crates/config/` is not a workspace member (orphan); **C15** mixed editions (core
2021 vs tui/runner 2024) forcing `resolver="3"`. Read-only audit verdicts (V1-V9): of 4,716 tracked files,
**3,744 (79%)** live under `imports/` (1,055) + `third_party/` (2,689); `target/` correctly externalized
(30 GB, ignored — PASS). Gated roadmap row: **FL-3** (`no src/*.rs > 1500 LOC` gate, coordinated with
U1-U3); **U9** resolves V1; **U5** resolves the V6 triple-vendored handoff. Analyst-proposed not-yet-gated:
FL-1 (`example.toml` → `examples/`), FL-2 (rename `work-order` → `rusty-idd-work-order` for prefix
consistency), FL-4/FL-5 (vendored upstreams behind submodule/`cargo vendor` lock; single codegraph owner),
FL-6 (`git rm --cached` the 3 tracked `*.idd-bak-*`; the `.gitignore:19` rule does not hold for files
committed before it existed).

### Memory/vector

CONFIRMED basis: **mem (.kb)** rusty-idd has NO `.kb/` of its own — it is recalled only if the fleet
daemon indexed the path (no `git_kb`/`.kb` refs in `crates/`). The memory audit (high-confidence
inventory): rusty-idd runs a **fourth, parallel memory organ** — the `.idd/knowledge` code-graph index
(`crates/knowledge/src/lib.rs:147-158`), a 47 MB git-tracked `index.json` — that duplicates git-kb's
code-intelligence shape but has **no embeddings, no semantic recall, no ICM decision/error memory**. ICM
is absent from product code (named only as a harness-checker contract `crates/cli/src/commands/harness.rs:208-265`
and a catalog anchor). The vendored codegraph vector store (`graph_vector.rs`, `store_embeddings`) is
**dead in product** (knowledge imports `default-features=false`, only parsing types). The
`knowledge-vector|surrealdb|cloud` Cargo features are inert (0 `cfg` sites). Analyst-proposed not-yet-gated
candidates: make `.idd/knowledge` a projection of git-kb rather than a rival; stop committing the 47 MB
blob + enforce the `workspace_fingerprint` freshness gate; wire real RAG behind `knowledge-vector` (NO-C
embedder review required) or delegate to git-kb; make ICM recall/store a real seam; unify the three planes
behind one provenance-tagged recall facade. These intersect U7 (the recall facade is a candidate fabric
port) but are NOT in the gated roadmap this cycle.

### Auto-research

CONFIRMED basis (autoresearch audit, high-confidence): continuous/daemon auto-research is **N/A** (CLI, no
resident process), but pull-mode is PRESENT and fail-closed — CI runs `cargo audit --deny warnings`
(`.github/workflows/ci.yml:60-61`, `promote-verify.yml:86-87`) pulling RustSec every run; `dependabot.yml`
opens weekly currency PRs; the IDD manifest-refresh diff gate (`ci.yml:56-59`) invalidates a stale code
index. The one real gap: the residual deprecated **`serde_yaml`** is **invisible** to the repo's own
`cargo audit` gate (RustSec tracks it as *issue* #2132, not a published advisory) and is caught only by the
loop's trend research — this is exactly why **U8** (migrate off it) is sequenced as an early PROPOSE.
Analyst-proposed not-yet-gated: add a `cargo deny` bans rule so CI (not just the loop) fails while
`serde_yaml 0.9.34+deprecated` is in the lock; optional `--watch` push-mode (needs an ADR — resident
process in a CLI); a scheduled cadence for the loop's web research.

### Rules/policy

CONFIRMED basis (rules-policy audit, HIGH confidence): the owner's **Upgrade Only / No Downgrades** law is
codified verbatim (`AGENTS.md:42`), with strict-parity-before-removal (`:11`), single integration authority
(`.idd/LOCK.md:7`), and evidence-gated PRs (`AGENTS.md:48-56`). The agent org chart is **read-only-by-
default, single-writer**: 3 of 4 Codex subagents are `read-only`, only `rusty-idd-implementer` is
`workspace-write` (`max_threads=4`, `max_depth=1`). Confirmed weaknesses: agent-guard `mode="warn"` has no
teeth (matches gov-002); rusty-idd does **not** participate in weave/A2A (continuity is filesystem cards
only — every `weave` hit in `crates/knowledge/src/lib.rs` is catalog/classifier/test data, not a comms
call); the roster lacks commander/continuity/evolution/escalation roles; the conflict-escalation pointer
`AGENTS.md:15` targets a **non-existent** `AI_MERGE/05_conflict_risk_register.md`; the AI_MERGE agent queue
is human-serialized with unassigned owners. These motivate the roadmap's U7 (typed boundary) + DC-2
(observable weave membership) without weakening any guard. Analyst-proposed not-yet-gated: promote the
Claude guard `warn`→`deny` (strengthens), add a dual-model **background** lane preserving single-writer +
explicit-auth, emit a weave heartbeat (degrade-safe), repair the escalation artifact, bound `.idd-bak-*`.

### Distributed compute

CONFIRMED basis: **dc no-C** — the only third-party native surface is blake3 + serde/serde_json/schemars;
no FFI/C, no `mlua`/`rusqlite`/`openssl-sys`/`-sys`/`cc`/`bindgen` in `crates/*` (excl. external). rusty-idd
runs on the workstation host only; the hardware matrix (mobile / AI glasses / Pi / ESP32 / device fabric)
exists **only as `OperatingLayerDefinition` knowledge data** at `crates/knowledge/src/lib.rs:3500-3747` —
it *knows about* distributed compute but executes none of it (the intent-plane role). Lua/Luau has **no
executable presence** (zero deps; appears only as catalog string data). The one real convergence seam is
the `handoff.task.v1` work-order, shaped but unconsumed. Gated roadmap rows: **DC-1** (consume work-order,
folded into U6), **DC-2** (weave/A2A transport, QUALIFIED, flagged), **DC-5** (guardrail: no firmware /
Lua-runtime in this binary — protects the no-C boundary). The work-order's `allows_network` /`path_scope`
fields (`lib.rs:62-65`) are the egress/residency policy a future executor binding must honor (analyst
DC-3, not-yet-gated).

### Test Strategy

(Lifted from `findings/test-strategy-rusty-idd.md` — verified findings; the testing component of the plan.)

**Current coverage (by call-graph reachability, not file presence):** work-order carries **13 in-crate
unit tests** covering the producer seam + intra-crate round-trip (`crates/work-order/src/lib.rs:371-604`);
merge-tools has 4 in-crate tests; cli/core integration tests exist (`crates/{cli,core}/tests/`:
`codex_cli.rs`, `harness_cli.rs`, `adr_check_cli.rs`, `smoke.rs`). All CONFIRMED high-confidence.

**Ranked coverage gaps (untested convergence seam):**
1. **No fail-closed card consumer** — the only deserialize path silently accepts a card whose `schema`
   discriminator (`task.schema.json:88`), `id` pattern (`:55`), and `intent_lock` the published contract
   rejects (probe: `serde_json::from_str::<WorkOrder>` returns `is_ok=true` for a foreign/tampered card).
   This is the **highest-risk gap** — every `.handoff/tasks/*.json` a downstream `hf` consumer reads.
2. **Tampered card loads silently** — a drifted card deserializes Ok while `intent_unchanged()==false`; the
   "provable contract" promise (`crates/work-order/src/lib.rs:1-7`) is unenforced on load.
3. **Producer never wired to consumer** — `work_orders_from_bundle` (`lib.rs:342`) is never connected to
   the `.handoff/tasks` consumer (`codex.rs:594`); `contains_task_card` (`codex.rs:886`) accepts ANY
   `*.json` (fail-open). Schema *drift* is NOT the gap (live schema == committed schema modulo EOF newline);
   *enforcement* is.

**Designed + AUTHORED suite (additive RED tests, real run captured):** in
`crates/work-order/tests/handoff_card_consumer.rs` — `consumer_rejects_foreign_schema_discriminator` (RED),
`consumer_rejects_id_violating_published_pattern` (RED), `consumer_rejects_card_with_drifted_intent_lock`
(RED), `baseline_well_formed_card_loads` (GREEN, over-fix fence). Real run:
`cargo test -p work-order --test handoff_card_consumer` → `FAILED. 1 passed; 3 failed` — RED for the right
reason (the validating consumer is unbuilt, not a compile error). Worktree commit `2f8a42f` (not pushed).
The cli-side fail-open (`codex.rs:886`) is designed-but-not-authored (needs cli private fns) and handed to
Feature Forge. This suite is promoted as the Feature-Forge test-build item (see `## Promotion` + the
ROADMAP test-build row); the carried **`## FF test-build spec`** is reproduced there.

### Prompt-architecture

CONFIRMED basis (prompt-architecture audit, HIGH confidence): rusty-idd's prompt architecture is a
deliberate **engine-owned single-source-of-truth control plane** — the workflow lives in the Rust engine
(`rusty-idd next` / spec artifact-DAG) and per-vendor instruction surfaces are ~10-line **generated,
drift-gated** thin adapters (ADR-0010/0015; `render --check` in `just ci`). Architecturally strong. Residual
risk at the edges the render gate does not cover: root `CLAUDE.md`/`GEMINI.md` are hand-maintained ~99%
duplicate prose outside the render set (silent drift path, D2); the model lanes (Codex `gpt-5.5*` vs Claude
`opus`) live in scattered TOML + test fixtures with **no governing decision record** (M6); every lifecycle
hook is a compile-gated `cargo run` (H4 — a non-building tree breaks SessionStart/PreToolUse gating). Three
ADR candidates surfaced (C1 bring root bridges under the SoT boundary; C2 model-lane policy of record; C3
hook execution contract). These are **not** promoted as ADRs this cycle — they are real but routine
governance reconciliations, not the genuine architecture decision; only the convergence/adapter boundary
(U7) clears that bar (see `## Promotion`).

---

## Tool-evaluation

What the **graph** shows rusty-idd imports/links, cross-referenced with the **researcher's** 90-day
currency + advisories (`research/rusty-idd.trends.md`, all dated, recency window 2026-03-28 → 2026-06-26).
Recommend **upgrade / hold** per tool with the cited reason.

| tool / crate | graph shows (links/imports) | pinned (Cargo.lock 2026-06-26) | currency + advisory (researcher, dated) | recommend | reason |
|---|---|---|---|---|---|
| clap | `crates/cli` CLI surface | 4.6.1 | latest on crates.io, MSRV 1.85; no RustSec advisory (accessed 2026-06-26, HIGH) | **HOLD** | current; optional pin `4.6` to lock the MSRV floor |
| ratatui | `crates/tui` app/ui | 0.30.2 | latest 0.30.x line; 0.30 = modular multi-crate + no_std (ratatui.rs, 2026-06-26, HIGH) | **HOLD** | current; modular split is an *opportunity* for the U2 TUI decomposition (depend on sub-crates) |
| crossterm | `crates/tui` backend | 0.29.0 | no advisory surfaced (MED, absence-of-evidence) | **HOLD** | rely on CI `cargo audit` for ongoing coverage |
| serde / serde_json | all crates (work-order, spec, knowledge…) | 1.0.228 / 1.0.150 | no RustSec advisory (HIGH) | **HOLD** | current |
| schemars | `crates/work-order` (card schema gen) | per lock | no advisory surfaced | **HOLD** | shapes the JSON Schema only (note ts-24: it does NOT validate serde deserialize — see U6) |
| blake3 | `crates/work-order` intent_lock | per lock | pure-Rust/intrinsics, no C linkage (CONFIRMED dc no-C) | **HOLD** | keeps the no-C trust boundary; do not swap for a C hasher |
| tokio | `crates/knowledge` (repomix rt-multi-thread) | 1.52.3 | 2026 advisories (RUSTSEC-2026-0057/0060) are tokio **0.1**-era; 1.x unaffected (rustsec.org, 2026, HIGH) | **HOLD** | unaffected; confirm `cargo audit`/`cargo deny` stay in CI |
| anyhow | first-party error model | 1.0.102 | no advisory (MED) | **HOLD** | current |
| toml | config parse | 0.9.6 | no advisory (MED) | **HOLD** | current |
| comrak | markdown render | 0.52 | no advisory (MED) | **HOLD** | current |
| minijinja | template render | 2 | no advisory (MED) | **HOLD** | current |
| thiserror | spec / transitive | 2 (spec) / 1.0.69 (transitive) | no advisory | **HOLD** | current; transitive 1.x is benign |
| serde_norway | `crates/spec`, `crates/runner` (YAML, "NOT serde_yml") | 0.9 | maintained dtolnay-fork successor, but maintainers **not** committed long-term (rust-lang forum, 2026, MED) | **HOLD (WATCH)** | sound current choice; track maintenance — `serde-yaml-ng` is the fallback if it stalls |
| **serde_yaml** | **transitive only**, sole source `external/codegraph-core` → parser → knowledge → cli | **0.9.34+deprecated** | **DEPRECATED/UNMAINTAINED**; upstream archived; RustSec issue #2132 (2024, status current) | **UPGRADE** | migrate vendored codegraph-core → `serde_norway` (**U8**); first-party already migrated; clears the deprecated crate from `Cargo.lock` |
| A2A (Agent2Agent) | NOT linked — external interop standard for weave | n/a | **v1.0 stable**, Linux-Foundation-governed, signed cards + gRPC, 150+ orgs (LF press + DEV 2026-03-14, HIGH) | **ADOPT-AS-TARGET** | the cross-vendor interop boundary U7/DC-2 converge toward; keep weave the required local route |
| OpenSpec / spec-kit (SDD field) | rusty-idd embeds OpenSpec workflow skills | n/a | OpenSpec 1.x + intent-driven template (2026-05-10); spec-kit v0.11.0 (2026-05-27), 30+ agents (HIGH) | **HOLD (WATCH)** | rusty-idd's OpenSpec-binding is on-trend; track the custom-schema/profiles + cross-artifact-analysis patterns as a best-practice baseline |

**Edition note (C15):** mixed editions (core 2021 vs tui/runner 2024) force `resolver="3"`; pinned deps are
all current with no live advisory — no action beyond U8.

---

## Risk policy

Full classification (APPLY / PROPOSE / **SUPERVISED**) and the trust-boundary / secrets / destructive /
provider-model rows are in the companion **`../risk-policy.md`**. Summary: only **U9** and **U10** are
APPLY (contained, blast 0, reversible, no trust-boundary crossing). The two headline boundary upgrades —
**U7** (adapter boundary) and **DC-2** (weave/A2A transport) — are **SUPERVISED**: they cross the
no-C-in-trust-path invariant and introduce the first live network/IPC dependency, so each is owner-gated
and must satisfy its QUALIFIED condition (weave required + C-free; filesystem fallback retained behind a
flag). No upgrade in this plan weakens a guard, downgrades a working surface, or relaxes a rule
(Upgrade-Only honored).

---

## Promotion (docs only — DRAFTS; rusty-idd is read-only this run)

- **ROADMAP rows (canonical copy here; promotion INTO rusty-idd/docs is a PROPOSED owner action):**
  `reports/ROADMAP-rusty-idd.md` — the `docs/ROADMAP.md` upgrade rows + the Feature-Forge **test-build**
  row shaped to `feature-architect`'s `## Verification plan` intake (the generate+run handoff carrying the
  `## FF test-build spec`).
- **DRAFT ADR (one genuine architecture decision):**
  `reports/ADR-DRAFT-rusty-idd-convergence-boundary.md` — *Typed convergence/adapter boundary for
  weave/icm/grit/hf (U7): keep the filesystem contract, add live binding, no-downgrade.* The routine
  upgrades (U8/U9/U10/U1-U3/U4/U5) get ROADMAP rows, **not** ADRs. The prompt-architecture ADR candidates
  (C1/C2/C3) and DC-5 guardrail are recorded as ADR-*candidates* in the ROADMAP, not emitted as ADRs this
  cycle.

---

## Confidence

**Overall: Medium-high.** The structural spine is HIGH-confidence: the clean-DAG/zero-cycles verdict, the
god-file blast/LOC numbers, the dead-code lower bound, the zero-lib-deps-on-fabric headline (C11), and the
fail-open card-load behavior (ts-24/25/26, with a real RED run) are all re-verified against source at SHA
`5a55284`. The tool-currency calls are HIGH (primary sources, in-window). What keeps it from HIGH overall:

**Named gaps / what stayed inconclusive / what wasn't examined (honest, not "fully planned"):**
- **Truncation:** dead-code (≥278) and public-API (≥500) are truncated at the git-kb 500-row cap — lower
  bounds. Before any DONE that depends on an *exhaustive* count, re-run explicit per-crate `code symbols`
  + `code dead` (the U4/U10 magnitudes especially).
- **Symbol-level cycles not computed:** `git-kb code` exposes no full edge dump and `code flows` returned
  `[]`; the clean-DAG verdict is authoritative at the **crate** level only.
- **Cross-repo fabric edges UNCONFIRMED:** the north-star map marks rusty-idd↔hf/weave, weave-as-load-
  bearing-transport, grit cross-repo adoption, and hf↔icm as UNCONFIRMED — they need cross-repo
  `kb_callers` verification before U7/DC-2 detail design.
- **Axis upgrades not yet verifier-gated:** only the architecture set (U1-U10) and DC-2/DC-5/FL-3 passed
  the feasibility gate this cycle. The governance (gov-001..008 upgrades), memory (U1-U5), autoresearch
  (U1-U4), rules-policy (UP-1..5), and filesystem (FL-1/2/4/5/6) **upgrade** rows are grounded in CONFIRMED
  *claims* but their feasibility-gate is pending the next verifier pass — they are reported as candidates,
  not promoted as gated plan rows.
- **U4 speed magnitude (QUALIFIED):** the build-time/binary-size win is plausible but unquantified; the
  acceptance test requires a measured before/after, not an assertion.
- **U7/DC-2 (QUALIFIED) walls:** feasible *only* under the C-free + weave-required + filesystem-fallback
  conditions; the live transport is a genuine architecture decision (the DRAFT ADR).

**What would raise confidence to HIGH:** (1) an exhaustive per-crate symbol/dead re-run lifting the
truncation; (2) cross-repo `kb_callers` confirming the fabric edges; (3) a verifier pass feasibility-gating
the per-axis upgrade candidates; (4) the U4 build-time measurement. Completeness sweep
(`reports/codemap-rusty-idd.md:95-115`): all 11 crates + the convergence axis re-derived non-zero from the
graph — the picture is whole enough to conclude this cycle's plan, with the truncation caveat recorded.
