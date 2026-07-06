# test-coverage — prompt-hub

Dimension: `test-coverage`. Target: `prompt-hub` (core lib, the convergence source-of-truth that
must hand a provenance-stamped GOAL ARTIFACT to `rusty-idd`, ADR-0007 / lifeos-meta-front-door).
Read-only analysis + an authored additive RED suite (Feature Forge builds the GREEN side).

All paths below are in the RED worktree
`/home/drdave/Desktop/meta/.worktrees/plan-prompt-hub-red/prompt_hub` unless noted.

## Existing coverage map (reachability, not file presence)

- CLAIM: the real, cargo-built integration suite lives in `prompt-hub/tests/` (15 files: test_hub,
  test_models, test_search, test_security, test_get_rbac, test_accessibility, test_auto_purge,
  test_chaos, test_chaos_auto, test_malware_scan, test_offline, test_qdrant, test_touch, test_voice,
  test_voice_anonymize) | evidence: `prompt-hub/tests/` listing | confidence: high
- CLAIM: the 5 root-level integration files are ORPHANED and never compile/run — the root manifest is
  a *virtual* workspace (`[workspace]` only, no `[package]`), so `cargo test` / `cargo nextest run
  --workspace` build zero test targets from root `tests/` | evidence: `Cargo.toml:1-3` (no `[package]`,
  `grep -c '\[package\]' Cargo.toml` = 0) + `tests/{test_end_to_end,test_hub,test_models,test_search,
  test_security}.rs` | confidence: high
- CLAIM: hub init/config surface IS covered — `PromptHub::new`, `is_initialized`, `config`, `db_path`
  have test callers | evidence: `prompt-hub/tests/test_hub.rs:5-83` reaching `hub.rs:PromptHub::new`
  | confidence: high
- CLAIM: the lib re-export/module-decl surface has a smoke test only (compilation-as-assertion) |
  evidence: `prompt-hub/src/lib.rs:145-162` (`lib_tests`) | confidence: high

## Coverage gaps (ranked; each cites the untested symbol/contract)

- CLAIM: the ADR-0007 convergence contract has ZERO coverage AND zero implementation — prompt_hub
  exposes no goal-artifact emission API at all (no `emit_goal_artifact`, no `provenance`, no
  `schema_version`, no handoff envelope) | evidence: `grep -rni
  'goal_artifact|GoalArtifact|provenance|emit_goal|handoff_envelope|to_goal|export_goal'
  prompt-hub/src/*.rs` returns nothing; contract specified at
  `docs/plans/lifeos-meta-front-door.md:81,123,147` + `docs/adr/0007-plugin-system.md` | confidence: high
- CLAIM: the public planning models carry NO provenance/schema fields, so even the closest emission
  (serialized `Prompt`/`Intent`) cannot satisfy the contract | evidence: `prompt-hub/src/models.rs:387-408`
  (`Prompt` keys: id,name,version,status,system_prompt,…author — no schema_version/provenance),
  `models.rs:557-566` (`Intent`), `models.rs:193-201` (`PromptMeta`) | confidence: high
- CLAIM: `hub.rs` is the central hotspot (181 KB, the `PromptHub` god-object) yet the
  register→search→emit integration path that feeds rusty-idd is untested for any envelope/handoff
  behavior — only init/config is reached | evidence: `prompt-hub/src/hub.rs:913` (`register`), `:981`
  (`get`); test reach limited to `test_hub.rs` (init/config) | confidence: high
- CLAIM: orphaned root `tests/` represents ~40 KB of intended integration coverage (e2e lifecycle,
  search modes, sanitizer-block, lock/unlock, concurrency, pagination) that contributes 0 to the gate
  | evidence: `tests/test_end_to_end.rs:47-282` (never built) | confidence: high

## UPGRADE rows (designed tests — all authored in this cycle)

- UPGRADE: add integration test for prompt_hub→rusty-idd goal-artifact emission contract | axis: accuracy
  | rationale: closes the zero-coverage gap on the ADR-0007 convergence contract (stable schema +
  provenance) — the single highest-value untested capability | evidence:
  `prompt-hub/tests/goal_artifact_contract.rs:1-243` against `models.rs:387-408,557-566` |
  blast: guards the meta front-door handoff (prompt_hub → rusty-idd → plan-loop → feature-forge) | risk: low
- UPGRADE: add hub round-trip emission test (register→search→emit) | axis: accuracy | rationale:
  exercises the real public-API integration path through the `hub.rs` hotspot, not just struct serde |
  evidence: `goal_artifact_contract.rs:184-220` reaching `hub.rs:913` (`register`), `hub.rs` search |
  blast: guards the register→emit path | risk: low
- UPGRADE: add schema-stability (golden/differential) assertion across prompt versions | axis: quality
  | rationale: pins the emitted `schema_version` so rusty-idd's consumer is version-pinned, not coupled
  to prompt content (behavior-preserving discipline) | evidence: `goal_artifact_contract.rs:225-243` |
  blast: guards consumer compatibility across prompt_hub releases | risk: low
- UPGRADE: (handoff to plan-architect, not authored here) un-orphan root `tests/` — either add a root
  `[package]`/test-owner crate or migrate the 5 files into `prompt-hub/tests/` | axis: quality |
  rationale: recover ~40 KB of dead integration coverage | evidence: `Cargo.toml:1-3` +
  `tests/test_end_to_end.rs:47-282` | blast: restores e2e lifecycle coverage to the gate | risk: low

## RED suite — authored, built, RUN (this cycle)

- File: `prompt-hub/tests/goal_artifact_contract.rs` (placed in the member crate's `tests/`, the only
  location cargo builds integration tests — root `tests/` is orphaned per the gap above).
- Command: `cargo test -p prompt-hub --test goal_artifact_contract`
- tests-ran: 7 (real count from the runner: `test result: FAILED. 0 passed; 7 failed; 0 ignored;
  0 measured; 0 filtered out`). Exit-0-with-zero-tests is NOT the case here — 7 tests executed.
- Expected RED failure reason: capability/contract ABSENT (prompt_hub emits no goal artifact; the
  serialized public models carry no `schema_version`/`provenance`). All 7 panic on the contract
  assertion, NONE on a compile error. Verified failure messages cite ADR-0007 and dump the actual
  emitted keys (e.g. schema_version test panic at `goal_artifact_contract.rs:72`).
- CI clippy posture: compiles clean under BOTH the real CI gate `cargo clippy -p prompt-hub
  --all-targets -- -D warnings` (default features, EXIT 0) AND `cargo clippy -p prompt-hub
  --all-targets --all-features -- -D warnings` (EXIT 0). The suite is feature-agnostic (uses only
  always-on public types + serde_json/uuid/chrono/semver/tempfile, all existing deps — no new deps).
- Commit: `6fa3462b1cbdc4032e090f88fabf1b27703c1d28` on branch `plan/prompt-hub-red-tests`
  (off origin/main). ONLY `prompt-hub/tests/goal_artifact_contract.rs` staged; no `prompthub.db` or
  other artifacts (tests use `tempfile::TempDir`). Not pushed (orchestrator ships).

## traceability (plan item ↔ acceptance criterion ↔ test ↔ status)

| Contract / gap (ADR-0007) | Acceptance criterion | Test (`goal_artifact_contract.rs`) | line | status |
|---|---|---|---|---|
| Stable schema rusty-idd can consume | emission carries top-level `schema_version` string | `goal_artifact_declares_stable_schema_version` | 66 | RED |
| Provenance of every claim | emission carries `provenance` object | `goal_artifact_carries_provenance_block` | 82 | RED |
| Source citations ([L#][E#][W#]) | `provenance.sources` is a non-empty array | `goal_artifact_provenance_lists_source_citations` | 98 | RED |
| Producer/consumer binding | `provenance.produced_by="prompt_hub"` + `target="rusty-idd"` | `goal_artifact_identifies_producer_and_targets_rusty_idd` | 120 | RED |
| Goal envelope, not bare record | `artifact_kind="goal_artifact"` + `goal` + `origin_prompt_id` | `goal_artifact_envelope_wraps_the_goal_payload` | 145 | RED |
| Integration path register→emit | hub-registered prompt emits schema+provenance | `registered_prompt_emits_contract_compliant_goal_artifact` | 169 | RED |
| Schema version-pinned (golden) | two prompt versions emit identical `schema_version` | `goal_artifact_schema_is_stable_across_versions` | 225 | RED |

(Line numbers are the `#[test]`/`#[tokio::test]` attribute lines for each case.)

## FF test-build spec (what GREEN looks like — Feature Forge intake)

Test surface (already authored, RED): `prompt-hub/tests/goal_artifact_contract.rs`. Feature Forge
implements the production capability that flips all 7 RED cases to GREEN — additive, no test edits
needed beyond removing the "best-available emission" shim once a real emitter exists.

Concrete cases GREEN must satisfy (one bullet each, symbol/flow + assertion):
- `Prompt` (and/or `Intent`) gains a public goal-artifact emission — e.g.
  `PromptHub::emit_goal_artifact(&self, prompt_id, intent) -> Result<GoalArtifact>` or
  `Prompt::to_goal_artifact(...)` — whose serde form is a top-level OBJECT (envelope), not the bare
  prompt record (`models.rs:387-408`).
- Envelope carries `schema_version: String` (e.g. `"goal-artifact/1"`), stable & identical across
  prompt versions → satisfies `goal_artifact_declares_stable_schema_version` + `…_stable_across_versions`.
- Envelope carries `provenance: { produced_by: "prompt_hub", sources: [..non-empty citations..],
  produced_at, prompt_hub_version }` → satisfies `…_carries_provenance_block`,
  `…_provenance_lists_source_citations`, `…_identifies_producer_and_targets_rusty_idd`.
- Envelope carries `target: "rusty-idd"`, `artifact_kind: "goal_artifact"`, `goal` (intent payload),
  and `origin_prompt_id` → satisfies `…_envelope_wraps_the_goal_payload`.
- The hub round-trip (`register` → `search` → emit) produces the same contract-compliant envelope →
  satisfies `registered_prompt_emits_contract_compliant_goal_artifact`.

Differential/golden fixtures to capture: snapshot one canonical emitted `GoalArtifact` JSON as a
golden under `prompt-hub/tests/fixtures/` and assert byte-stable serialization (the
`…_stable_across_versions` test is the seed of this discipline; promote to an `insta`/golden snapshot).

Coverage target: the ADR-0007 emission API reaches 100% test-caller coverage on its public surface
(emit fn + GoalArtifact (de)serialization + provenance population); the register→emit integration
path gains ≥1 end-to-end test caller.

CI gates the new tests touch: `Default-Features Test Compile` (`ci.yml:58-59`,
`cargo test -p prompt-hub --no-run` + `cargo clippy -p prompt-hub --all-targets -- -D warnings`) and
`Test` (`ci.yml:86`, `cargo nextest run --workspace --all-features`). Both already pass for the RED
suite at compile time; they will run+assert the cases once GREEN.

Secondary FF item (architect-routed): un-orphan root `tests/` so its e2e files re-enter the gate
(add a root test-owner package or migrate into `prompt-hub/tests/`) — see UPGRADE row above.
