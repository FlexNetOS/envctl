STATUS: BLOCKED

# REQ-021 implementer log

## Implemented

- Added typed `TargetDescriptor`, `TargetSafety`, and named-version models matching the frozen
  JSON schema, including typed enums, defaults, and explicit `schema_version`/nonempty validation.
- Added the engine-owned `parse_target_descriptor` JSON/YAML parser and made
  `MigrationDb::register_target` derive every indexed/persisted identity and safety field from the
  descriptor.
- Preserved existing redb row readability with serde defaults for newly persisted
  `schema_version`, `allow_network`, and `allow_destructive` fields.
- Updated `migration target add` so the descriptor is authoritative. Legacy target/type/root/safety
  arguments remain optional consistency assertions and fail closed on contradiction.
- Updated `migration target validate` to use the same engine parser and emit the normalized
  descriptor plus its canonical SHA-256.
- Updated migration DB fixtures and added missing-field, invalid nested safety, duplicate ID, and
  JSON/YAML canonical hash parity tests.
- Added focused CLI integration coverage for validate/add/list/show and a contradictory legacy
  assertion.
- Resolved guardian F1 narrowly: metadata is now object-shaped by type, omission normalizes to
  `{}`, and scalar metadata is refused with focused regression coverage.

## Verification

- PASS: direct `rustfmt` over all touched Rust files.
- PASS: `git diff --check`.
- PASS: `ci/gates/shape.sh`.
- PASS: `ci/gates/enable.sh` (the gate also emitted inherited LLVM activation/rollback diagnostics).
- PASS after guardian F1 fix: direct rustfmt check, `git diff --check`, `ci/gates/no-c.sh`, and
  `ci/gates/shape.sh`.
- BLOCKED: `cargo test -p envctl-engine migration_db --lib`.
- BLOCKED: `cargo test -p envctl --test migration_target`.
- BLOCKED: `cargo check -p envctl-engine -p envctl`.
- BLOCKED: `ci/gates/no-c.sh`.

The managed worktree initially lacked its declared sibling `loop_lib` and
`meta_plugin_protocol` path dependencies. Temporary read-only sibling links allowed Cargo to start,
but the untouched, already-dirty `/home/flexnetos/meta/src/loop_lib/src/lib.rs:281` fails first with
Rust `E0515` (“cannot return value referencing temporary value”). Cargo therefore never reached the
envctl crates. The temporary links were removed. The repository-documented Rust 1.88 lane could not
be tried because this profile has neither a rustup frontdoor nor Cargo `+toolchain` support.
The focused metadata regression test and clippy were retried after F1 and remain blocked at the
same untouched `loop_lib/src/lib.rs:281` E0515 before envctl compiles.

## Runtime check

BLOCKED: the real `envctl` binary cannot be built until the sibling `loop_lib` baseline compiles,
so the declared validate/add/list/show smoke could not be executed.
