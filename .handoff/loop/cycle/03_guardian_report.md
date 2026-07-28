VERDICT: PASS

# REQ-021 invariant guardian report

## Re-review result

The prior blocking finding is resolved. `TargetDescriptor.metadata` is now
`serde_json::Map<String, Value>` with a default empty map
(`crates/engine/src/migration_db/model.rs:89-90`). This makes omission normalize to `{}` and makes
scalar, array, and null metadata fail deserialization, matching the frozen schema's object type.
Focused coverage verifies omission normalization and scalar refusal
(`crates/engine/src/migration_db/tests.rs:198-232`).

No remaining product blocker was found by source review.

## Contract and invariant review

- PASS: required fields and schema-version minimum are enforced.
- PASS: target and safety enums exactly match the frozen values.
- PASS: target ID and primary root enforce the schema's nonempty constraint.
- PASS: nullable compare root, output-root default, string include/exclude arrays, boolean
  collector map, object metadata, and named integer/string versions match the frozen shapes.
- PASS: JSON and YAML enter one engine-owned parser and normalize to one canonical JSON/hash.
- PASS: registration derives indexed identity, roots, type, safety mode, risk cap, and network /
  destructive policy fields from the descriptor rather than independent CLI values.
- PASS: optional legacy CLI arguments are consistency assertions and fail closed on contradiction.
- PASS: newly added persisted fields have conservative serde defaults for old `Target` rows.
- PASS: replay continues to verify the canonical stored descriptor hash.
- PASS: engine remains synchronous and non-printing.
- PASS: no new dependency, C-linked substrate, manifest, lock, or unrelated product change.

## Test review

The source test set covers:

- schema version zero and empty target ID;
- missing recipe and invalid nested safety mode;
- duplicate natural target ID;
- JSON/YAML normalization and hash equality;
- omitted metadata normalization and scalar metadata refusal;
- CLI validate/add/list/show plus a contradictory legacy assertion.

An explicit serialized pre-REQ-021 `Target` compatibility test would be useful but is not required
to accept the change; the new fields carry serde defaults and existing stored descriptor JSON
remains opaque on row deserialization.

## Verification

- PASS: direct rustfmt `--check` on all touched Rust files.
- PASS: `git diff --check`.
- PASS: `ci/gates/no-c.sh`.
- PASS: `ci/gates/shape.sh`.
- PASS: `cargo test -p envctl-engine migration_db --lib` — 12 passed.
- PASS: `cargo test -p envctl --test migration_target` — 1 passed.
- PASS: `cargo check -p envctl-engine -p envctl` — 0 errors.
- PASS: clippy with `-D warnings`; the unused `TargetType` CLI import found during verification was
  removed.

Verification used a clean committed-HEAD `loop_lib` worktree rather than the dirty main sibling
checkout. This isolates REQ-021 evidence from the unrelated E0515 state previously observed in the
main `loop_lib` checkout; no `loop_lib` product change is part of REQ-021.

## Runtime check

PASS. The real envctl CLI validated and registered the canonical
`examples/target-descriptors/generic-codebase.yaml`, then listed and showed the stored target.
`validate`, `add`, `list`, and `show` all exited 0. Validate and persisted target reported the
identical canonical descriptor hash beginning `e79f5c`, and the stored safety fields matched the
descriptor.

## Scope

The product diff remains confined to the planned migration DB model/API/re-export, CLI target
surface, unit tests, and focused CLI integration test. Cycle plan/log/report changes are expected
harness artifacts. The CLI integration test remains untracked and must be included when the
feature is committed.
