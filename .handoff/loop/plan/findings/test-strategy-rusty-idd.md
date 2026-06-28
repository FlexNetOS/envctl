# test-coverage — rusty-idd

Dimension: `test-coverage`. Target: **rusty-idd**. Read-only on product code; tests are the one
permitted additive mutation. Authored + run in the isolated worktree
`/home/drdave/Desktop/meta/.worktrees/plan-rusty-idd-red/rusty-idd` (branch
`plan/rusty-idd-red-tests`, off `origin/develop`). Every row cites real `file:line`.

## Verdict (1-liner)

rusty-idd's product crates carry real unit/integration tests (work-order alone has 13 in-crate
tests), but the **fleet-convergence seam is untested where it matters**: the `handoff.task.v1`
card contract is *produced* and *round-tripped within* work-order, yet **no fail-closed
validating consumer exists** — the only deserialize path (`serde_json::from_str::<WorkOrder>`)
silently accepts cards the published schema rejects. The new RED suite encodes that gap.

## Existing coverage (mapped from the graph + the crates' own `#[cfg(test)]`)

- CLAIM: work-order has 13 in-crate unit tests covering the producer seam + intra-crate round-trip | evidence: `crates/work-order/src/lib.rs:371-604` (`seam_bundle_to_workorders`, `roundtrips_through_json` @ `lib.rs:453-460`, `task_schema_is_nonempty_and_requires_contract_fields` @ `lib.rs:531-555`, `intent_lock_detects_drift` @ `lib.rs:445-451`) | confidence: high
- CLAIM: merge-tools has 4 in-crate tests (package data + `verify_workspace`) | evidence: `crates/merge-tools/src/lib.rs:324-388` | confidence: high
- CLAIM: cli/core integration tests exist | evidence: codemap `crates/{cli,core}/tests/` (`codex_cli.rs`, `harness_cli.rs`, `adr_check_cli.rs`, `smoke.rs`) | confidence: high

## Coverage gaps (untested convergence seam — reachability, not file presence)

- CLAIM: work-order's `roundtrips_through_json` (`lib.rs:453-460`) proves only intra-crate serde round-trip; **no test drives the cross-boundary contract a downstream consumer enforces** (schema `pattern`/`const`, intent_lock provability) | evidence: no test reaches a validating load path — none exists (`to_json` @ `lib.rs:222`, no `from_card`/`load`/`validate` counterpart) | confidence: high
- CLAIM: the only card deserialize path silently accepts a foreign discriminator AND a non-canonical id | evidence: probe run in worktree — `serde_json::from_str::<WorkOrder>` of a card with `schema:"openai.task.v1"` + `id:"task-lowercase"` returns `is_ok=true`; the published contract forbids both (`imports/handoff/schemas/task.schema.json:88` discriminator pattern, `:55` id pattern) | confidence: high
- CLAIM: a tampered card (objective edited post-mint, stale intent_lock) loads silently — the "provable contract" promise (`crates/work-order/src/lib.rs:1-7`) is unenforced on load | evidence: probe — drifted card deserializes Ok while `intent_unchanged()==false`; nothing on the load path runs the check | confidence: high
- CLAIM: work-order is an unconsumed S1 spike — 24 dead symbols, zero product callers; the producer (`work_orders_from_bundle` @ `lib.rs:342`) is never wired to the `.handoff/tasks` consumer (`crates/cli/src/commands/codex.rs:592`, `contains_task_card` @ `codex.rs:886`) | evidence: codemap §Dead code (work-order 24), graph F5 | confidence: high
- CLAIM: the `.handoff/tasks` consumer in codex.rs does no card validation — `contains_task_card` accepts ANY `*.json` file (fail-open) | evidence: `crates/cli/src/commands/codex.rs:886-888` delegates to `contains_file_with_extension(dir,"json")` — no parse, no schema check | confidence: high (designed-only; see suite extension)
- CLAIM: the live generated schema and the committed downstream schema are in sync (modulo trailing newline) — so schema *drift* is NOT the gap; the gap is *enforcement* | evidence: in-worktree diff of `task_schema_json()` vs `imports/handoff/schemas/task.schema.json` differs only by EOF newline | confidence: high

## Designed + AUTHORED suite (additive RED tests)

All authored in `crates/work-order/tests/handoff_card_consumer.rs` (integration tests, real
public API, no new dependency — uses work-order's existing `serde_json`). Run scoped to the
crate. Built, ran, and FAILED on assertion (RED for the right reason: the validating consumer
is unbuilt — not a compile error). One baseline test is GREEN by design and must stay GREEN.

- UPGRADE: add fail-closed integration test `consumer_rejects_foreign_schema_discriminator` for the card-load discriminator check | axis: accuracy | rationale: closes the "no validating consumer" gap on the schema `const` (`task.schema.json:88`) | evidence: `crates/work-order/tests/handoff_card_consumer.rs` + probe (is_ok=true today) | blast: every `.handoff/tasks/*.json` a downstream `hf` consumer reads | risk: low
- UPGRADE: add fail-closed integration test `consumer_rejects_id_violating_published_pattern` | axis: accuracy | rationale: enforces published id `pattern` (`task.schema.json:55`) on load | evidence: `crates/work-order/tests/handoff_card_consumer.rs` | blast: id-keyed card discovery / claim | risk: low
- UPGRADE: add provability test `consumer_rejects_card_with_drifted_intent_lock` | axis: accuracy | rationale: makes the blake3 intent_lock "provable contract" promise (`lib.rs:1-7,71`) enforced at load, not just computable | evidence: `crates/work-order/tests/handoff_card_consumer.rs` + probe (drift card loads, intent_unchanged=false) | blast: drift-sentinel guarantee across the seam | risk: low
- UPGRADE: add baseline guard `baseline_well_formed_card_loads` (GREEN) | axis: quality | rationale: prevents an over-broad fail-closed fix that rejects valid cards | evidence: `crates/work-order/tests/handoff_card_consumer.rs` | blast: regression fence on the GREEN path | risk: low

### Designed-but-not-authored (next slice, requires touching cli private fns — out of this crate scope)

- UPGRADE: add cli integration test that `.handoff/tasks/<id>.json` evidence requires a *valid* card (not any `.json`) | axis: accuracy | rationale: closes the fail-open `contains_task_card` (`codex.rs:886`) | evidence: `crates/cli/src/commands/codex.rs:886-888` | blast: the autonomous-workflow evidence gate | risk: low

## Real run (captured in the worktree)

Command: `cargo test -p work-order --test handoff_card_consumer`

tests-ran: 4

Result: `test result: FAILED. 1 passed; 3 failed; 0 ignored` — the 3 RED tests panicked on the
designed assertions (deserialize accepted the malformed/tampered card); the baseline passed.

- `consumer_rejects_foreign_schema_discriminator` — RED — panic @ `handoff_card_consumer.rs:69` ("a card whose `schema` != handoff.task.v1 must be rejected on load … accepted a foreign discriminator")
- `consumer_rejects_id_violating_published_pattern` — RED — panic @ `handoff_card_consumer.rs:84` ("`id` violates the published ^[A-Z]*TASK- pattern … accepted it")
- `consumer_rejects_card_with_drifted_intent_lock` — RED — panic @ `handoff_card_consumer.rs:109` ("content no longer matches its recorded intent_lock … accepted the tampered card")
- `baseline_well_formed_card_loads` — GREEN — a well-formed card still deserializes (over-fix fence)

Expected RED failure reason (all three): the **only card-load path is non-validating** —
`serde_json::from_str::<WorkOrder>` ignores the published schema's `pattern`/`const` constraints
and the intent_lock provability promise. No fail-closed consumer exists. An exit-0-with-zero-tests
is impossible here: the run reported 4 tests (`parse_tests_ran` = 4 > 0); 3 are RED.

Commit (worktree, branch `plan/rusty-idd-red-tests`, NOT pushed): `2f8a42f`.

## traceability (plan item ↔ acceptance criterion ↔ test ↔ RED/GREEN)

| plan item | acceptance criterion (falsifiable) | test (file:name) | source under test | state |
|---|---|---|---|---|
| Convergence seam: work-order card is consumed via a fail-closed contract | A card whose `schema` ≠ `handoff.task.v1` is rejected on load | `handoff_card_consumer.rs::consumer_rejects_foreign_schema_discriminator` | `lib.rs:42`+`task.schema.json:88` | RED |
| Same | A card whose `id` violates `^[A-Z]*TASK-[A-Z0-9][A-Z0-9-]*$` is rejected on load | `handoff_card_consumer.rs::consumer_rejects_id_violating_published_pattern` | `lib.rs:47`+`task.schema.json:55` | RED |
| Same — provable contract | A card whose content no longer matches its recorded `intent_lock` is rejected on load | `handoff_card_consumer.rs::consumer_rejects_card_with_drifted_intent_lock` | `lib.rs:71,194-200` | RED |
| Regression fence | A well-formed `handoff.task.v1` card still loads | `handoff_card_consumer.rs::baseline_well_formed_card_loads` | `lib.rs:342,222` | GREEN |

## FF test-build spec (Feature-Forge GREEN handoff)

What Feature Forge must implement to turn each RED test GREEN — engine-first, additive-only on
behavior, no-downgrade (a well-formed card must still load → keep `baseline_well_formed_card_loads`
GREEN).

- **Test surface (already authored, do not rewrite):** `crates/work-order/tests/handoff_card_consumer.rs`. The tests exercise the default deserialize path (`serde_json::from_str::<WorkOrder>`), so the validation must live ON that path, not in a separate opt-in function (a parallel "safe" loader would leave the default path fail-open and the tests RED).
- **Capability to build (one mechanism flips all three RED tests):** make `WorkOrder` deserialization **fail-closed**. Recommended: a `#[serde(try_from = "WorkOrderRaw")]` (or custom `Deserialize`) where `WorkOrderRaw` is the current field set and `TryFrom<WorkOrderRaw> for WorkOrder` enforces, in order:
  1. `schema == "handoff.task.v1"` (the schema `const`/regex @ `lib.rs:41`, `task.schema.json:88`) — else `Err`.
  2. `id` matches `^[A-Z]*TASK-[A-Z0-9][A-Z0-9-]*$` (`lib.rs:47`, `task.schema.json:55`) — else `Err`.
  3. recomputed base intent_lock (objective/path_scope/acceptance via `compute_intent_lock` @ `lib.rs:156`) matches the recorded `intent_lock` — else `Err` (the provability gate; reuse `intent_unchanged()` logic @ `lib.rs:194`).
- **Where the producer must round-trip to (close the unconsumed-spike gap, F5):** add a writer that emits each `WorkOrder` to `.handoff/tasks/<id>.json` and wire it so `crates/cli/src/commands/codex.rs:592` consumes validated cards (replacing the fail-open `contains_task_card` @ `codex.rs:886-888` with a load-and-validate). This is the producer↔consumer seam the spike never connected.
- **Differential/golden fixtures to capture:** golden `task.schema.json` parity is already enforceable — assert `task_schema_json()` (`lib.rs:233`) equals the committed `imports/handoff/schemas/task.schema.json` (currently identical except EOF newline); add this as a golden so the published contract can never drift from the Rust shape.
- **Coverage target:** the validating load path (criteria A/B/C) reaches `must == 1` test caller each; the cli `.handoff/tasks` consumer (`codex.rs:886`) gains its first validating test.
- **CI gate(s) touched:** `cargo test --workspace --locked` (the work-order tests + any new cli test), `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings` (the merge-tools `verify` phase gates @ `crates/merge-tools/src/lib.rs:81-87`).

## Method notes / limits

- Coverage = reachability: claims of "no test reaches X" are from reading the crates' own
  `#[cfg(test)]` + tests/ dirs and the codemap dead-code count (work-order 24, zero product
  callers), not from a coverage tool. The empirical probes (run in-worktree, then deleted)
  confirmed the current accept-the-bad-card behavior before the RED tests were committed.
- The cli-side fail-open finding (`codex.rs:886`) is designed-only this slice (it needs the cli
  crate to build + touches private fns); it is handed to FF above, not yet authored as RED.
