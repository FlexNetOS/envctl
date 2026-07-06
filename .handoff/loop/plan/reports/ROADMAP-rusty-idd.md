# DRAFT ROADMAP rows — rusty-idd (promote to rusty-idd `docs/ROADMAP.md`)

These are DRAFT `docs/ROADMAP.md` rows. The canonical plan is `reports/rusty-idd-plan.md`. Promotion INTO
`rusty-idd/docs/ROADMAP.md` is a **PROPOSED owner action** — rusty-idd is read-only this run, so these are
authored here, not written into rusty-idd's tree (owner-wall). Every row traces to a CONFIRMED/QUALIFIED +
feasible verdict in `findings/verdicts.md`. Order = the sequenced roadmap (value/risk by centrality +
blast-radius).

## Upgrade rows

| seq | id | upgrade | axis | tier | blast | target-surface | acceptance gate | trace |
|---|---|---|---|---|---|---|---|---|
| 1 | U9 | Resolve `crates/config/` orphan + CI workspace-member guard | governance | APPLY | 0 | `crates/config/`, `Cargo.toml`, CI | RED member-guard test (fails on `crates/config/`) | C6, verdict U9 |
| 2 | U10 | Wire-or-mark the 30 dead `spec` symbols (5 `spec_*` CLI cmds) | accuracy | APPLY | 0 | `crates/spec/*`, `crates/cli/src/commands/spec_*.rs` | no undocumented dead public symbol in spec | C13 (QUALIFIED), verdict U10 |
| 3 | U8 | Migrate vendored codegraph-core off deprecated `serde_yaml 0.9` → `serde_norway` | governance | PROPOSE | 105 | `crates/external/codegraph-core/Cargo.toml:40` | `cargo tree -i serde_yaml` empty | C14, verdict U8 |
| 4 | U6/DC-1 | Consume `work-order` with a fail-closed card consumer (RED suite → GREEN) | accuracy / dist-compute | PROPOSE | 0 | `crates/work-order`, `crates/cli/src/commands/codex.rs:594` | 3 RED tests GREEN; baseline GREEN; work-order dead → ~0 | C7, ts-24/25/26, verdict U6 |
| 5 | U1 | Decompose `runner/src/runner.rs` behind unchanged public API | quality | PROPOSE | 803 | `crates/runner/src/runner.rs` | runner public-API diff = ∅; tests green | C3, verdict U1 |
| 6 | U2 | Split `tui/src/app.rs` into screen/state/input modules | quality | PROPOSE | 248 | `crates/tui/src/app.rs` | tui public-API diff = ∅; extracted-module unit test | C4, verdict U2 |
| 7 | U3 | Split `knowledge/src/lib.rs`; repo catalog → external data file | quality | PROPOSE | 105 | `crates/knowledge/src/lib.rs` | catalog round-trip == prior set; public-API diff = ∅ | C5, verdict U3 |
| 7b | FL-3 | Gate `no first-party src/*.rs > 1500 LOC` (coordinate with U1-U3) | filesystem-layout | PROPOSE | gate | first-party `crates/*/src/**` | gate RED today on knowledge/tui/runner | FL-3 (verdicts.md:63) |
| 8 | U4 | Feature-gate the 182 dead vendored codegraph symbols (off by default) | speed | PROPOSE | 105 | `crates/external/codegraph-{core,parser}` | slim build green; `code dead` drops ≥100 + measured before/after | C8/C9, verdict U4 (QUALIFIED) |
| 9 | U5 | De-duplicate vendored upstreams (handoff vendored 3×) | governance | PROPOSE (destructive, owner-walled) | 0 | `third_party/upstream/*`, `imports/*`, `.gitignore` | one tracked path per upstream; product build unaffected | C8, F1, verdict U5 |
| 10 | U7 | Typed convergence/adapter boundary (filesystem first impl; weave/icm/grit/hf adapters) | governance | **SUPERVISED** | ~0 | new `crates/interop` (or `core` trait module) | trait + filesystem adapter + handoff.task.v1 round-trip test; weave required + C-free | C11/C12, verdict U7 (QUALIFIED) |
| 11 | DC-2 | Bind work-orders to weave/A2A transport (behind a feature flag) | distributed-compute | **SUPERVISED** | low | `crates/work-order` + interop weave adapter | weave job keyed by correlation_id; stub executor ACKs; filesystem fallback retained | DC-2 (QUALIFIED, verdicts.md:61) |
| — | DC-5 | Guardrail ADR-candidate: no `mlua`/`esp-hal`/`no_std` in rusty-idd | dist-compute (guardrail) | PROPOSE | 0 | ADR-candidate + CI grep gate | no embedded/Lua-runtime crate enters Cargo.toml | DC-5 (verdicts.md:62) |

ADR-candidates recorded (not emitted this cycle): prompt-architecture C1 (root CLAUDE.md/GEMINI.md under
the render SoT boundary), C2 (model-lane policy of record), C3 (hook execution contract: prebuilt binary
vs per-call `cargo run`). The one genuine architecture decision (U7) is emitted as a DRAFT ADR —
`reports/ADR-DRAFT-rusty-idd-convergence-boundary.md`.

## Feature-Forge test-build row (the generate + run handoff)

This row is shaped to Feature Forge's `feature-architect` `## Verification plan` intake. The RED suite is
already AUTHORED and RED-run by the planning-engineer (test-strategist); Feature Forge builds the
production code that flips it GREEN. Do NOT rewrite the tests.

| field | value |
|---|---|
| backlog id | FF-rusty-idd-001 |
| title | Fail-closed `handoff.task.v1` card consumer + producer↔consumer wiring |
| kind | test-build (RED → GREEN); engine-first, additive-only, no-downgrade |
| RED suite (authored, do not rewrite) | `crates/work-order/tests/handoff_card_consumer.rs` (worktree commit `2f8a42f`, branch `plan/rusty-idd-red-tests`, not pushed) |
| RED tests | `consumer_rejects_foreign_schema_discriminator`, `consumer_rejects_id_violating_published_pattern`, `consumer_rejects_card_with_drifted_intent_lock` |
| GREEN fence (must stay GREEN) | `baseline_well_formed_card_loads` |
| trace | ts-24/25/26/27/28 (CONFIRMED); roadmap row U6/DC-1 |

### `## FF test-build spec` (carried from `findings/test-strategy-rusty-idd.md`)

- **Test surface (already authored, do not rewrite):** `crates/work-order/tests/handoff_card_consumer.rs`.
  The tests exercise the default deserialize path (`serde_json::from_str::<WorkOrder>`), so the validation
  must live ON that path, not in a separate opt-in function (a parallel "safe" loader would leave the
  default path fail-open and the tests RED).
- **Capability to build (one mechanism flips all three RED tests):** make `WorkOrder` deserialization
  fail-closed. Recommended: `#[serde(try_from = "WorkOrderRaw")]` (or custom `Deserialize`) where
  `WorkOrderRaw` is the current field set and `TryFrom<WorkOrderRaw> for WorkOrder` enforces, in order:
  1. `schema == "handoff.task.v1"` (the schema `const`/regex @ `lib.rs:41`, `task.schema.json:88`) — else `Err`.
  2. `id` matches `^[A-Z]*TASK-[A-Z0-9][A-Z0-9-]*$` (`lib.rs:47`, `task.schema.json:55`) — else `Err`.
  3. recomputed base intent_lock (objective/path_scope/acceptance via `compute_intent_lock` @ `lib.rs:156`)
     matches the recorded `intent_lock` — else `Err` (reuse `intent_unchanged()` @ `lib.rs:194`).
- **Where the producer must round-trip to (close the unconsumed-spike gap, F5):** add a writer that emits
  each `WorkOrder` to `.handoff/tasks/<id>.json` and wire it so `crates/cli/src/commands/codex.rs:594`
  consumes validated cards (replacing the fail-open `contains_task_card` @ `codex.rs:886-888` with a
  load-and-validate).
- **Differential/golden fixtures:** assert `task_schema_json()` (`lib.rs:233`) equals the committed
  `imports/handoff/schemas/task.schema.json` (identical except EOF newline) — add as a golden so the
  published contract cannot drift from the Rust shape.
- **Coverage target:** the validating load path (criteria A/B/C) reaches `must == 1` test caller each; the
  cli `.handoff/tasks` consumer (`codex.rs:886`) gains its first validating test.
- **CI gate(s) touched:** `cargo test --workspace --locked`, `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets --all-features -- -D warnings` (the merge-tools `verify` phase
  gates @ `crates/merge-tools/src/lib.rs:81-87`).
