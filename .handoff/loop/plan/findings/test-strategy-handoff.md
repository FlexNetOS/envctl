# Test Strategy & Coverage — handoff (cycle 2, union with rusty-idd)

- **Dimension:** `test-coverage` (always-on) for target **handoff**, focused on the UNION seam (handoff ↔ rusty-idd).
- **Worktree (authored + verified):** `/home/drdave/Desktop/meta/.worktrees/plan-handoff-cycle2/handoff` @ branch `plan/handoff-union-cycle2`.
- **Coverage = reachability**, not file presence (call graph: `graph/handoff.{symbols,callgraph,metrics}.json`).
- **Read-only on product code.** The one additive mutation this cycle is the RED test file `work-order/tests/union_failclosed.rs` (99 LOC, committed `d74ad4b`).
- All rows cite `path:line` / symbol. No fabricated coverage numbers; the build wall is recorded fail-closed.

---

## 1. Existing-coverage map (by call-graph reachability)

The kernel's contract-bearing logic that ships tests today:

- CLAIM: `work-order` intake/mint seam is well-covered (PRODUCER side) | evidence: `work-order/src/lib.rs:429-672` — 15 `#[test]`s reach `work_orders_from_bundle` (`:371`), `work_orders_from_bundle_with` (`:320`), `synthesized_task_id` (`:396`), `WorkOrder::compute_intent_lock` (`:175`), `intent_unchanged` (`:213`), `intent_components` (`:225`), `full_intent_lock` (`:201`), `task_schema_json` (`:257`) | confidence: high
- CLAIM: `handoff-schema::validate_card` fail-closed is covered for the JSON-SCHEMA path | evidence: `handoff-schema/src/lib.rs:156-191` — 4 `#[test]`s (`valid_card_validates_ok`, `card_missing_intent_lock_is_rejected_naming_the_field`, `card_with_wrong_schema_const_is_rejected`, `card_with_bad_id_is_rejected`) reach `validate_card` (`:43`) | confidence: high
- CLAIM: `validate_card` is the kernel's highest-blast fail-closed gate, and it IS tested | evidence: `graph/handoff.graph.md:63` — `validate_card` blast-radius 40 transitive callers; tests at `handoff-schema/src/lib.rs:166-191` | confidence: high

## 2. Coverage gaps (ranked by graph risk signals)

- CLAIM: there is **no fail-closed work-order LOADER** binding deserialize + schema-validation + intent-lock integrity; the only load surface (`serde_json::from_str::<WorkOrder>`) is FAIL-OPEN | evidence: `work-order/src/lib.rs:56-92` (`WorkOrder` derives `Deserialize`; `#[schemars(regex(...))]` at `:60,:66` is schema-only and NOT enforced by serde) + the only round-trip test `work-order/src/lib.rs:545-551` calls bare `serde_json::from_str` with no validation | confidence: high
- CLAIM: the union consumer (rusty-idd intent plane) inherits this fail-open via a MIRRORED copy, reproducing cycle-1's silent-accept defect | evidence: `union-handoff-rusty-idd.md:132` + `rusty-idd/crates/work-order/src/lib.rs:35` ("mirrors `…/handoff/schemas/task.schema.json`"); cycle-1 verdicts ts-25/ts-26 "silent-accept defect" (`dimensions.md:8`) | confidence: high
- CLAIM: `validate_card` (JSON-schema) CANNOT catch intent_lock-vs-content drift — it cannot recompute blake3; only `WorkOrder::intent_unchanged` can, and nothing on a load path chains the two | evidence: `handoff-schema/src/lib.rs:43-62` (pure JSON-schema, no hashing) vs `work-order/src/lib.rs:213-219` (`intent_unchanged` recomputes blake3); no symbol binds them | confidence: high
- CLAIM: the **ledger read API is MISSING** — no public surface for an intent-plane consumer to read claimed/checkpoint state; it is internal to `hf` | evidence: `union-handoff-rusty-idd.md:248-263` (Seam 2) + `codemap-handoff.md:91` (Seam 2) + `graph/handoff.graph.md:61` (`Ledger.open` blast 120, all callers in-kernel) | confidence: high
- CLAIM: the ledger crate **cannot be tested standalone in this worktree** — its `../../RuVector/*` path deps are absent, so the whole Cargo workspace fails to resolve | evidence: `cargo build -p work-order` → `failed to read .../plan-handoff-cycle2/RuVector/crates/rvf/rvf-crypto/Cargo.toml (os error 2)`; `ledger/Cargo.toml` rvf-crypto path dep; `codemap-handoff.md:95` standalone-ization blocker | confidence: high

## 3. Designed suite (UPGRADE rows — the tests that close the gaps)

- UPGRADE: add **fail-closed-refusal** tests for the work-order LOAD boundary (foreign-schema / malformed-id / drifted-intent_lock) | axis: accuracy | rationale: closes the no-fail-closed-loader gap (§2) — the union's mirror of cycle-1's silent-accept | evidence: `work-order/src/lib.rs:56-92,213` | blast: every downstream consumer that loads `handoff.task.v1` cards (rusty-idd intent plane via mirror) | risk: low
  - **AUTHORED + COMMITTED this cycle** as `work-order/tests/union_failclosed.rs` (`d74ad4b`). RED verified (see §4).
- UPGRADE: add a **fail-closed-refusal** integration test for `handoff-intake` work-order intake (reject foreign-schema / bad-id card at the front door) | axis: accuracy | rationale: the intake verb is the front-door union attach point (`codemap-handoff.md:93`) and has 0 reachable fail-closed test for a drifted card | evidence: `handoff-intake` (17 sym) `→ handoff-core, work-order` | blast: `hf intake` dispatch path | risk: low
  - **BLOCKED** this cycle: `handoff-intake → handoff-core → ledger → RuVector` (missing path dep) — cannot build standalone. Owner wall recorded (§5). Feature Forge runs it post-RuVector-strategy.
- UPGRADE: add a **public ledger read-API contract** test (`get_claimed`/`latest_checkpoint`/`witness_chain` read surface) once that API exists | axis: accuracy | rationale: encodes Seam 2 — the intent plane needs witnessed reads, not file IO | evidence: `union-handoff-rusty-idd.md:248-263` | blast: `Ledger.open` (blast 120, `graph/handoff.graph.md:61`) | risk: low
  - **BLOCKED + DESIGN-ONLY**: the API is unbuilt AND `ledger` cannot build standalone (RuVector). Cannot author a COMPILING RED today (a call to a non-existent `Ledger::query_claimed` would fail-to-COMPILE, which is forbidden). Promote after the ledger read-API is designed + a RuVector strategy lands.
- UPGRADE: add a **differential/golden** parity test for the `handoff.task.v1` schema across handoff ↔ rusty-idd mirror | axis: quality | rationale: prove the mirror does not drift from `work_order::task_schema_json` (`work-order/src/lib.rs:257`) | evidence: `codemap-handoff.md:90` (mirror seam) | blast: schema contract for both repos | risk: low (golden capture of `task_schema_json()`)

## 4. RED authoring + count verification (P8)

**Authored (additive-only):** `work-order/tests/union_failclosed.rs` — 4 integration tests against the EXISTING public surface (`WorkOrder`, `SwarmBundle`, `work_orders_from_bundle`, `intent_unchanged`, `to_json`) + `serde_json` (visible to integration tests via the crate's normal `[dependencies]` — empirically confirmed: NO Cargo.toml edit required).

**Build wall (fail-closed, recorded):** the worktree workspace cannot resolve (`ledger → ../../RuVector/crates/rvf/rvf-crypto` absent), so `cargo test -p work-order` fails at manifest-load. `work-order` is RuVector-free, so RED was verified by building it in a **standalone mirror** (scratchpad copy, identical `src/` + the authored `tests/`, self-contained Cargo.toml, edition 2024, rustc 1.96.0).

- **command (standalone mirror):** `cargo test --test union_failclosed`
- **command (worktree, post-RuVector):** `cargo test -p work-order --test union_failclosed`
- **expected RED failure reason:** the fail-closed loader is unbuilt — `serde_json` accepts a foreign-schema card / a malformed-id card, and loads a card whose `intent_lock` does not match its content; the three asserts fail loudly.
- **tests-ran: 4** (REAL — `running 4 tests` / `1 passed; 3 failed; 0 ignored; 0 measured`). Not an exit-0-zero-tests fail-open.
- **observed result:** `test result: FAILED. 1 passed; 3 failed` — exactly the RED state (3 RED + 1 passing fixture sanity).

### traceability — plan-item ↔ acceptance criterion ↔ test path/name ↔ RED|GREEN

| Plan item / UPGRADE | Acceptance criterion (falsifiable) | Test (path::name) | State |
|---|---|---|---|
| Fail-closed work-order loader | A card with `schema != handoff.task.v1` is REJECTED at load | `work-order/tests/union_failclosed.rs::workorder_load_rejects_foreign_schema_card` | RED (serde accepts → assert fails) |
| Fail-closed work-order loader | A card whose `id` violates `^[A-Z]*TASK-[A-Z0-9][A-Z0-9-]*$` is REJECTED at load | `work-order/tests/union_failclosed.rs::workorder_load_rejects_malformed_id_card` | RED (serde accepts → assert fails) |
| Fail-closed work-order loader | A card whose stored `intent_lock` ≠ its content is REJECTED/re-verified at load | `work-order/tests/union_failclosed.rs::workorder_load_rejects_card_with_drifted_intent_lock` | RED (`intent_unchanged()` false → assert fails) |
| Fixture integrity (sanity) | A genuine minted card loads clean + `intent_unchanged()` | `work-order/tests/union_failclosed.rs::fixture_is_a_clean_valid_card` | GREEN (guards the RED is about the loader, not a bad fixture) |
| handoff-intake fail-closed refusal | `hf intake` rejects a foreign-schema/bad-id card | (designed) `handoff-intake/tests/intake_failclosed.rs` | BLOCKED (RuVector — owner wall) |
| Ledger read-API contract | `Ledger` exposes a read-only `get_claimed`/`latest_checkpoint` surface | (design-only) `ledger/tests/read_api.rs` | BLOCKED (API unbuilt + RuVector) |

GREEN definition for the loader items: the test FLIPS to passing once `work_order` exposes a fail-closed loader (e.g. `WorkOrder::from_card_json(&str) -> Result<WorkOrder, LoadError>`) that chains `serde_json` + `handoff_schema::validate_card` + `intent_unchanged`, and the tests assert `.is_err()` on the bad cards / reject the drifted one.

## 5. Owner wall (environment cannot run — recorded, not faked)

- **Wall:** RuVector path deps absent → workspace won't resolve → `cargo test -p <crate>` impossible in this worktree for any crate transitively touching `ledger`/`hf`.
- **Exact command to reproduce:** `cd /home/drdave/Desktop/meta/.worktrees/plan-handoff-cycle2/handoff && cargo build -p work-order` → `failed to read .../plan-handoff-cycle2/RuVector/crates/rvf/rvf-crypto/Cargo.toml`.
- **Standalone-buildable crates (RuVector-free, leaf/near-leaf):** `work-order` (verified RED here), `handoff-schema` (→ work-order), `handoff-policy`, `handoff-lease`, `handoff-secrets`, `handoff-test-support`.
- **Blocked-until-RuVector crates:** `ledger`, `handoff-core`, `handoff-intake`, `handoff-index`, `handoff-fleet`, `handoff-drift`, `handoff-route`, `handoff-gatekeeper`, `hf`. The ledger read-API + handoff-intake refusal tests are gated on a RuVector strategy (vendor / path-dep / publish) — `codemap-handoff.md:95`.

---

## FF test-build spec (GREEN handoff for Feature Forge)

**Verification plan intake for the union work — what to build + run.**

- **Test surface (files/modules to add tests in):**
  - `work-order/tests/union_failclosed.rs` — **already authored + committed (`d74ad4b`)**; Feature Forge flips it GREEN by implementing the loader (below).
  - `handoff-intake/tests/intake_failclosed.rs` — NEW (after RuVector strategy): front-door refusal of foreign-schema/bad-id cards.
  - `ledger/tests/read_api.rs` — NEW (after the read-API is designed): contract test for the public read surface.
- **Production change required to flip the authored RED → GREEN (Feature Forge implements; test is additive-only):**
  - Add `WorkOrder::from_card_json(s: &str) -> Result<WorkOrder, LoadError>` (and/or `try_from_value(Value)`) in `work-order/src/lib.rs` that: (1) `serde_json` deserializes, (2) calls `handoff_schema::validate_card` on the raw `Value` (re-exported or via a thin dep so work-order stays leaf-friendly — or move the validator call to the consumer wrapper), (3) calls `intent_unchanged()` and rejects on mismatch. Then update the 3 RED tests to call the new loader and assert `.is_err()` / rejection.
- **Concrete cases (one bullet each — symbol/flow + assertion):**
  - `work_order::WorkOrder` load — foreign `schema` const → loader returns `Err` (assert `.is_err()`).
  - `work_order::WorkOrder` load — `id` violating `^[A-Z]*TASK-[A-Z0-9][A-Z0-9-]*$` → loader returns `Err`.
  - `work_order::WorkOrder` load — `intent_lock` ≠ recomputed (`intent_unchanged()` false) → loader returns `Err`.
  - `handoff_intake` intake verb — same three bad cards refused at the front door (fail-closed, no card written to `.handoff/tasks/`).
  - `ledger::Ledger` read API — `get_claimed()` returns only claimed work-orders; `latest_checkpoint()` returns the last witnessed checkpoint; reads never mutate the witness chain.
- **Differential/golden fixtures to capture:**
  - Golden of `work_order::task_schema_json()` (`work-order/src/lib.rs:257`) — assert the rusty-idd mirror (`rusty-idd/crates/work-order`) reproduces it byte-for-byte (drift gate for the mirror seam).
- **Coverage target:** every contract-bearing fail-closed branch of the card-LOAD path has a reachable refusal test (foreign-schema, bad-id, drifted-lock at minimum); the ledger read API has a read-only contract test.
- **CI gate(s) touched:** `cargo test` / `cargo nextest` (workspace test gate) once RuVector resolves in CI (`hf/Cargo.toml:46` notes CI clones `FlexNetOS/meta-ruvector` as sibling `RuVector/`); the `Format`/clippy preflight gates are unaffected (tests are additive). RuVector availability is the prerequisite for running anything below the `work-order`/`handoff-schema` leaf layer.

---

**Status:** test-coverage analysed; RED suite authored + committed + verified RED (`tests-ran: 4`). Awaiting plan-verifier (is-it-really-untested + feasibility) before plan-architect lifts the suite + FF spec.
