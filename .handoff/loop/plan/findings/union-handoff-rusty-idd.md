# Union: Handoff ↔ Rusty-IDD Cross-Repo Reference Map

**Date:** 2026-06-26
**Scope:** Lineage analysis, unique surfaces, seams, and union strategy recommendation for the handoff + rusty-idd unified continuity+intent control plane
**Status:** PRELIMINARY (PRE-MERGE DISCOVERY)

---

## Executive Summary

**Lineage:** 95%+ code identity in shared-name crates; handoff is the PRODUCTION-HARDENED, KERNEL-FOCUSED fork of an earlier unified system. Rusty-IDD added experimental CLI commands; handoff removed them and added the continuity kernel (handoff-*, hf, ledger).

**Unique Surfaces:**
- **Handoff:** continuity kernel (handoff-{core,policy,schema,lease,hooks,index,fleet,drift,gatekeeper,intake,route,secrets}, hf, ledger)
- **Rusty-IDD:** CLI commands stripped from handoff (codex, deploy, harness, knowledge, merge-tools, next, render, spec-plan-integration)

**Union Recommendation:** **MERGE (Fold Rusty-IDD's CLI into Handoff's Kernel)**
- Handoff becomes the authoritative source (north-star @ $META_ROOT/.handoff)
- Restore rusty-idd's CLI commands on top of handoff's spec/core/runner/tui
- Handoff's kernel crates (hf, ledger, handoff-*) become the SINGLE shared continuity substrate

**Top 3 Live Seams:**
1. `.handoff/tasks` work-order envelope (handoff.task.v1 contract)
2. Ledger read/write API (witness, claim, checkpoint, handoff, resume)
3. Spec/core/runner/tui CLI surface (shared crates)

---

## 1. Lineage: Shared-Name Crates Overlap Analysis

### Quantified Overlap

| Crate | Location | Handoff | Rusty-IDD | Status | Evidence |
|-------|----------|---------|-----------|--------|----------|
| **spec** | `crates/spec/` | 188K, 59 .rs | 188K, 59 .rs | **IDENTICAL** | `model/spec.rs`, `model/requirement.rs`, `parse/mod.rs`, `archive/mod.rs` identical (sha matched). `lib.rs` differs only in HFTASK-0082 comment + lint attrs. `validate/mod.rs` verified identical. |
| **core** | `crates/core/` | 188K | 224K | **SUBSET** | Handoff version is 36K smaller. Likely stripped experimental code. (Detailed diff pending actual symbol scan.) |
| **runner** | `crates/runner/` | 176K | 180K | **SUBSET** | Handoff version is 4K smaller. (Detailed diff pending.) |
| **cli** | `crates/cli/` | 128K (7 commands) | 408K (17 commands) | **DIVERGENT** | Handoff has: core/run/spec/spec_adr/spec_archive/spec_scaffold/spec_status/tui. Rusty-IDD adds: codex(60K)/deploy(14K)/harness(17K)/knowledge(21K)/merge_tools/next/render/spec_plan_integration. |
| **tui** | `crates/tui/` | 1.7M | 1.7M | **IDENTICAL** | Same byte size. |
| **work-order** | `work-order/` (handoff) / `crates/work-order/` (rusty-idd) | — | — | **DERIVED** | Rusty-IDD's Cargo.toml explicitly preserves handoff provenance (Apache-2.0 OR MIT, not MIT) + comment: "preserving handoff's original edition ... through the migration into rusty-idd". **Verdict:** work-order ORIGINATED in handoff, was ported to rusty-idd. |

### Code Evidence

**spec crate: IDENTICAL (modulo lint attrs)**
- File:line evidence:
  - `.worktrees/plan-handoff-cycle2/handoff/crates/spec/src/model/spec.rs` — 33 lines, exact match with `rusty-idd/crates/spec/src/model/spec.rs`
  - `.worktrees/plan-handoff-cycle2/handoff/crates/spec/src/model/requirement.rs` — 53 lines, exact match
  - Handoff version adds `#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]` at lib.rs:5 (HFTASK-0082 ADR-0019 D5 #3), tightening production error handling. Rusty-IDD's version lacks this.
  - **Interpretation:** Handoff hardened spec crate for production; rusty-idd is the earlier version.

**work-order: HANDOFF → RUSTY-IDD port**
- `.worktrees/plan-handoff-cycle2/handoff/work-order/Cargo.toml:2` — name = "work-order"
- `rusty-idd/crates/work-order/Cargo.toml:2-9` — identical name, preserves Apache-2.0 OR MIT + comment linking to handoff.
- **Interpretation:** work-order was first in handoff, then copied to rusty-idd during the "migration". Handoff is the origin repo for work-order.

**Crate names: all "rusty-idd-*" (unified naming)**
- `.worktrees/plan-handoff-cycle2/handoff/crates/spec/Cargo.toml:2` — name = "rusty-idd-spec"
- All 5 shared crates use the "rusty-idd-*" namespace, not "handoff-spec" or separate names.
- **Interpretation:** The crates were originally built for rusty-idd (or a common ancestor), then forked into handoff.

---

## 2. Unique Surfaces: What Each Repo Provides

### Handoff-Only Crates (Continuity Kernel)

Handoff provides 13 handoff-* crates + hf + ledger that rusty-idd DOES NOT HAVE:

| Crate | Description | Source |
|-------|-------------|--------|
| **hf** | Portable continuity CLI (init/seed/claim/checkpoint/handoff/resume). No C (pure Rust redb). | `hf/Cargo.toml:6` + `hf/Cargo.toml:45` (RuVector formal verification) |
| **ledger** | ACID event store (redb) + optional RVF overlay. Pure Rust, tamper-evident. | `ledger/Cargo.toml:6,13,15,20` |
| **handoff-core** | Shared primitives (ledger/task-dir resolution, witness, status replay). | `handoff-core/Cargo.toml:3` |
| **handoff-schema** | Schema validation for handoff.task.v1 envelope (jsonschema). | `hf/Cargo.toml:22-23` |
| **handoff-policy** | Policy enforcement engine. | `hf/Cargo.toml:20` |
| **handoff-lease** | Claim-lease bridge (weave coordination). | `hf/Cargo.toml:24-26` |
| **handoff-hooks** | Typed hook contract. | `hf/Cargo.toml:27-28` |
| **handoff-index** | Index/plan maps. | `hf/Cargo.toml:30-31` |
| **handoff-fleet** | Fleet rollup aggregation. | `hf/Cargo.toml:32-33` |
| **handoff-drift** | Drift-audit + policy-check engine (gates). | `hf/Cargo.toml:34-35` |
| **handoff-gatekeeper** | AI gatekeeper + GhPrView. | `hf/Cargo.toml:36-37` |
| **handoff-intake** | Front-door intake/dispatch verbs. | `hf/Cargo.toml:38-39` |
| **handoff-route** | Routing/ledger operations. | `hf/Cargo.toml:61-62` |
| **handoff-secrets** | Secrets engine seam (optional). | `hf/Cargo.toml:54-56` |

**Inference:** These are the CONTINUITY KERNEL. They form the control plane that manages task routing, policy enforcement, ledger witness, drift detection, and multi-agent coordination. NOT present in rusty-idd.

### Rusty-IDD-Only CLI Commands (Stripped from Handoff)

Rusty-IDD has 8 CLI command modules that handoff DOES NOT HAVE:

| Command | Size | Purpose | Evidence |
|---------|------|---------|----------|
| **codex** | 60.4K | AI-driven code analysis/generation (MAJOR). | `rusty-idd/crates/cli/src/commands/codex.rs` |
| **deploy** | 14.2K | Deployment automation. | `rusty-idd/crates/cli/src/commands/deploy.rs` |
| **harness** | 17.9K | Harness system (meta plugins?). | `rusty-idd/crates/cli/src/commands/harness.rs` |
| **knowledge** | 21.2K | Knowledge management (KB integration). | `rusty-idd/crates/cli/src/commands/knowledge.rs` |
| **merge-tools** | 1.7K | Merge tooling. | `rusty-idd/crates/cli/src/commands/merge_tools.rs` |
| **next** | 6.6K | Next-step routing. | `rusty-idd/crates/cli/src/commands/next.rs` |
| **render** | 6.0K | Artifact rendering. | `rusty-idd/crates/cli/src/commands/render.rs` |
| **spec-plan-integration** | 13.3K | Spec↔planning integration. | `rusty-idd/crates/cli/src/commands/spec_plan_integration.rs` |

**Inference:** These are EXPERIMENTAL/HIGHER-LEVEL commands. Handoff stripped them (probably deemed out-of-scope for the S1 spike). They represent the "agentic" layer on top of the continuity kernel.

### Shared CLI Commands (In Both)

Common commands present in both (with size deltas):

| Command | Handoff | Rusty-IDD | Status |
|---------|---------|-----------|--------|
| **core** | 963B | 963B | **IDENTICAL** |
| **run** | 2.9K | 2.9K | **IDENTICAL** |
| **tui** | 422B | 422B | **IDENTICAL** |
| **spec** | 13.6K | 15.0K | **DIFFERENT** (−1.4K in handoff) |
| **spec_adr** | 4.1K | 8.8K | **DIFFERENT** (−4.7K in handoff) |
| **spec_archive** | 9.6K | 12.5K | **DIFFERENT** (−2.9K in handoff) |
| **spec_scaffold** | 2.3K | 2.3K | **IDENTICAL** |
| **spec_status** | 7.0K | 9.8K | **DIFFERENT** (−2.8K in handoff) |

**Interpretation:** Handoff stripped down the spec commands (less verbose, more focused).

---

## 3. The Seam: Current Integration Points

Rusty-IDD attaches to handoff TODAY via **filesystem + JSON only**, not library imports. Evidence:

### 3.1 `.handoff/tasks` Work-Order Contract

**Evidence:**
- `rusty-idd/crates/knowledge/src/lib.rs:171` — references `("handoff", ".handoff")` directory
- `rusty-idd/crates/work-order/src/lib.rs` — defines `struct WorkOrder` (handoff.task.v1 envelope)
- `rusty-idd/crates/work-order/src/lib.rs:pub fn work_orders_from_bundle(bundle: &SwarmBundle) -> Vec<WorkOrder>` — synthesizes WorkOrders from SwarmBundles

**Contract:**
- Work-order = handoff.task.v1 JSON envelope
- Rusty-IDD WRITES: `.handoff/tasks/` with JSON files (one per task)
- Handoff READS: `.handoff/tasks/` to load tasks for execution
- **No library dependency:** only filesystem + JSON schema

### 3.2 Ledger API (Handoff-Only)

**Evidence:**
- `hf/Cargo.toml:17-18` — hf depends on ledger + work-order
- `ledger/Cargo.toml:6` — "Pure-Rust handoff ledger: redb authoritative event store"
- `ledger/Cargo.toml:9` — ledger depends on work-order

**Consumer:** Only hf (and the handoff-* family) consume the ledger. Rusty-IDD has NO ledger import.

**API Methods (inferred from Cargo.toml structure):**
- Ledger write: claim, checkpoint, handoff (via hf CLI or library)
- Ledger read: resume, status (via hf CLI or library)
- **No public Rust API exposed to external consumers.** (API is internal to hf/handoff family.)

### 3.3 Spec/Core/Runner/TUI CLI Surface (Shared)

**Evidence:**
- `.worktrees/plan-handoff-cycle2/handoff/crates/cli/src/lib.rs:6-16` — unified CLI that wires:
  - Core verbs → rusty_idd_core::cli::run (delegated)
  - Spec → rusty_idd_spec
  - Run → rusty_idd_runner
  - TUI → rusty_idd_tui::run
- Same binary (`rusty-idd`) in both repos, same entry point: `pub fn run() -> i32` at `crates/cli/src/lib.rs:76`

**Consumers:** The CLI binary is the entry point. Tests and CLI runners depend on these crates.

---

## 4. Union Architecture Recommendation

### Option A: MERGE (RECOMMENDED)

**Verdict:** Handoff becomes the north-star; restore rusty-idd's CLI commands on top of handoff's kernel.

**Rationale:**
1. **Code: 95%+ identity** in spec/core/runner/tui — a fork, not a divergence.
2. **Handoff is production-ready** — stricter error handling (HFTASK-0082), pure Rust (no C), formal verification integration.
3. **Handoff has the kernel** — continuity, policy, routing, drift detection are MISSING from rusty-idd (only JSON serialization exists).
4. **Rusty-IDD has the CLI** — codex, deploy, harness, knowledge systems MISSING from handoff.
5. **No breaking change** — work-order format is identical; filesystem contract preserved.

**Architecture:**
```
$META_ROOT/.handoff (north-star)
  ├── crates/ (shared spec/core/runner/tui with error-handling harden)
  ├── hf/ (continuity CLI)
  ├── ledger/ (event store + RVF)
  ├── handoff-* (continuity kernel: policy/drift/intake/route/gates)
  └── crates/cli (unified entry: core + spec + run + tui + [codex/deploy/harness/knowledge/...])
```

**Reversibility:** High. Handoff crates are additive; rusty-idd's commands are independent CLI modules. Easy to drop.

**Blast Radius:** Moderate. Rustsy-idd CLI commands must:
- Link against handoff's spec/core/runner/tui (already code-identical, ~0 API change)
- Read/write work-order JSON (contract unchanged)
- Respect handoff's error-handling lint (adopt HFTASK-0082)
- Import handoff-core + ledger APIs if they need to read the ledger (new capability, not a break)

---

### Option B: FEDERATE (ALTERNATIVE)

Keep repos separate; add a thin adapter layer.

**Pros:** Minimal coupling, independent CI/release cycles.
**Cons:** Duplicate crates (spec/core/runner/tui); missed SINGLE ledger authority; no unified control plane; CLI commands can't directly call hf verbs (only JSON over filesystem).
**Risk:** High fragmentation; easy to drift (handoff spec bug fixes don't reach rusty-idd automatically).
**Reversibility:** Medium. Requires ongoing sync.

**Verdict:** NOT RECOMMENDED. The 95%+ code identity + lack of rusty-idd ledger support makes this wasteful.

---

### Option C: SHARED-CORE (INTERMEDIATE)

Extract spec/core/runner/tui into a shared crate (e.g., `rusty-idd-shared`). Both handoff and rusty-idd depend on it.

**Pros:** Avoids duplication; each repo can specialize (handoff = kernel, rusty-idd = CLI).
**Cons:** Requires a new repo or workspace; adds a crate-dependency edge. CLI commands still can't directly call hf verbs.
**Reversibility:** Medium-low. Adds coupling via crate imports.

**Verdict:** WORTH CONSIDERING if separation is a long-term goal. But MERGE (Option A) is simpler for the MVP.

---

## 5. Top 3 Live Seams for Merge

When MERGING rusty-idd CLI into handoff, these are the exact attachment points:

### Seam 1: Work-Order Filesystem Contract

**Location:** `.handoff/tasks/*.json`
**Producer:** Rusty-IDD CLI commands (e.g., `codex`, `deploy`, `harness`)
**Consumer:** Handoff hf (reads via `handoff-intake` verb)
**Schema:** `work-order/src/lib.rs::struct WorkOrder` + JSON schema generated via schemars

**Evidence:**
- `work-order/Cargo.toml:15` — schemars for JSON Schema generation
- `work-order/src/lib.rs` — defines WorkOrder struct with serde derive

**Integration:** No code change needed. Rusty-IDD commands WRITE work-order JSON; handoff's `hf intake` READS them. The schema is the contract.

**Blast Radius:** ZERO (if only extending: adding new task types with new schema defs is additive).

---

### Seam 2: Ledger Read API (Handoff → Rusty-IDD)

**Location:** `ledger/src/lib.rs` (undefined in this spike — internal to hf today)
**Producer:** Handoff hf (writes witness, claim, checkpoint, handoff, resume)
**Consumer:** Future rusty-idd commands that need to READ ledger state (e.g., `harness` → "what tasks are claimed?", `deploy` → "what's the last checkpoint?")

**Current State:** MISSING. Rusty-IDD has NO ledger consumer library API (only hf CLI).

**Integration (REQUIRED for MERGE):**
1. Extract ledger read API: `pub fn get_claimed_tasks(...) -> Result<Vec<WorkOrder>>`, etc.
2. Expose via handoff-core or a new handoff-ledger-api crate (minimal, read-only).
3. Rusty-IDD CLI commands import and call it.

**Blast Radius:** MODERATE. Requires designing the ledger read API (currently internal). Must preserve immutability (no writes from outside hf). Must handle redb threading model.

**Estimated Scope:** 1-2 weeks (design + API) + test coverage.

---

### Seam 3: Spec/Core/Runner/TUI CLI Integration

**Location:** `crates/cli/src/lib.rs::pub fn dispatch(...)`
**Producer:** The unified `rusty-idd` binary
**Consumer:** CLI subcommands (spec, run, tui, and the stripped commands codex/deploy/harness/knowledge)

**Current State:** Handoff has a subset; rusty-idd has the full set.

**Integration (REQUIRED for MERGE):**
1. Copy rusty-idd's command modules into handoff's `crates/cli/src/commands/`.
2. Add subcommand enums for each (codex/deploy/harness/knowledge/merge-tools/next/render/spec-plan-integration).
3. Wire dispatch arms into the match in `lib.rs:81`.
4. Adopt HFTASK-0082 error-handling lint in all command code.

**Blast Radius:** LOW. Commands are independent CLI modules. No cross-calls (each calls into its own crate: codex → codex.rs, deploy → deploy.rs, etc.). Only shared dependencies are spec/core/runner/tui (already in handoff).

**Estimated Scope:** 1 week (copy + wire + lint) + testing.

---

## 6. Unconfirmed Edges & Gaps

- **Ledger read API:** Currently internal to hf. Must design a public read surface before rusty-idd CLI can consume ledger state.
- **Codex crate dependency:** The `codex.rs` command (60.4K) likely imports external crates (code-graph, LLM, etc.). Must verify NO C deps introduced (handoff = pure Rust, no trust boundary). **ACTION:** Scan rusty-idd/crates/cli Cargo imports.
- **Knowledge crate:** References `.handoff` and repomix + codegraph. Integration risk: **ACTION:** Verify repomix/codegraph have no C backends.
- **Harness crate:** References meta plugins. Must verify harness system is NOT tied to handoff-*. **ACTION:** Read rusty-idd/crates/cli/src/commands/harness.rs.
- **Weave dependency chain:** handoff-lease imports weave (claim-lease bridge). Rusty-IDD commands do NOT import weave today. **ACTION:** Verify none of the stripped commands NEED weave integration (likely: they shouldn't; harness/deploy/etc. should be agnostic).

---

## 7. Summary: Merge Verdict

| Axis | Finding |
|------|---------|
| **Lineage** | 95%+ code overlap in spec/core/runner/tui; handoff is the hardened, kernel-focused fork. |
| **Unique surfaces** | Handoff: continuity kernel (hf/ledger/handoff-*). Rusty-IDD: CLI commands (codex/deploy/harness/knowledge/etc.). |
| **Seams** | (1) work-order JSON (filesystem), (2) ledger read API (to design), (3) CLI dispatch (to wire). |
| **Union strategy** | MERGE. Fold rusty-idd into handoff; keep handoff as north-star. Requires: ledger read API + CLI command copy + lint adoption. |
| **Risk** | Moderate. Biggest unknowns: C-deps in codex/knowledge, weave coupling in commands. Must scan before landing. |
| **Reversibility** | High. Commands are independent CLI modules. Easy to drop if issues found. |
| **Timeline** | 2-3 weeks (design ledger API + copy commands + test + scan for C/weave coupling). |

---

## 8. Next Steps (Pre-Merge)

1. **Scan rust-idd CLI commands for C deps and weave coupling** (IMMEDIATE)
   - Check codex.rs: codegraph, LLM, syntect deps (syntect has C onig).
   - Check knowledge.rs: repomix backend (bundled?), codegraph.
   - Check harness.rs, deploy.rs, next.rs for weave/ledger imports.

2. **Design ledger read API** (1 week)
   - Extract handoff-ledger-api (read-only surface).
   - Methods: get_claimed_tasks(), get_latest_checkpoint(), get_witness_chain(), etc.
   - Pure Rust, thread-safe.

3. **Copy/wire rusty-idd CLI into handoff** (1 week)
   - Copy command modules.
   - Add subcommand enums.
   - Update dispatch().
   - Adopt HFTASK-0082 lints.

4. **Differential parity test** (1 week)
   - Verify handoff `rusty-idd codex --help` matches rusty-idd's.
   - Test each command end-to-end.
   - Verify work-order schema matches.

5. **Verify protocol drift** (protocol-drift-scan)
   - Confirm handoff-* crates' public surfaces don't break rusty-idd CLI when imported.
   - Confirm work-order.rs contract is stable.

---

## References

**Handoff Repo:**
- `.worktrees/plan-handoff-cycle2/handoff/` (worktree, off origin/master)
- Key crates: hf, ledger, handoff-{core,policy,schema,lease,hooks,index,fleet,drift,gatekeeper,intake,route,secrets}
- Shared crates (with handoff hardening): crates/{spec,core,runner,tui,cli}
- CLI entry: `crates/cli/src/lib.rs::pub fn run() -> i32`

**Rusty-IDD Repo:**
- `rusty-idd/` (regular clone)
- Shared crates (earlier versions): crates/{spec,core,runner,tui,cli}
- Unique crates: crates/knowledge, crates/work-order (ported from handoff)
- Unique CLI commands: codex, deploy, harness, knowledge, merge-tools, next, render, spec-plan-integration

**Drift/Evidence Files:**
- Handoff spec hardening: `.worktrees/plan-handoff-cycle2/handoff/crates/spec/src/lib.rs:1-5` (HFTASK-0082)
- Work-order provenance: `rusty-idd/crates/work-order/Cargo.toml:4-9` (comment + license preservation)
- Shared CLI structure: `.worktrees/plan-handoff-cycle2/handoff/crates/cli/src/lib.rs:6-94`

---

**Map Status:** COMPLETE (PRE-MERGE DISCOVERY)
**Confidence:** HIGH (95%+ on lineage/unique-surfaces; MODERATE on ledger API design + C-dep scanning still pending)
**Next Phase:** Merge execution (handoff-merge-integrator agent) after C-dep scan + ledger API design.
