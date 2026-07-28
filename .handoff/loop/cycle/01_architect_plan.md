VERDICT: GO

# REQ-021 target registry

The canonical target descriptor is the authority. The current implementation only checks that
the descriptor is a JSON object and accepts duplicated CLI fields that may contradict the hashed
document.

## Required changes

- Add typed `TargetDescriptor`, `TargetSafety`, and named-version models matching the frozen schema.
- Centralize parsing/validation in the engine and reuse it for registration and CLI preview.
- Make registration derive persisted target fields and safety policy from the descriptor.
- Accept canonical JSON, YAML, and YML descriptor files.
- Preserve list/show, canonical descriptor hashes, run creation, replay, and approval behavior.
- Update invalid legacy fixtures and cover malformed descriptors, nested policy, duplicates,
  JSON/YAML hash parity, and CLI validate/add/list/show.

## Impact

MEDIUM. Direct consumers are CLI dispatch and migration DB tests. Transitive consumers include
run creation/events, approval gating, replay verification, views, and run export. GitNexus had no
matching indexed symbols and its CLI fallback could not load because `make` was unavailable, so
call paths were confirmed from source.

## Target repos

One repo: envctl. Modules are linearly dependent: model -> API -> CLI/tests. Sequential path.

## Runtime surface

`envctl migration target validate/add/list/show`, plus safe run-create/replay regression checks.

## Verification

- `cargo test -p envctl-engine migration_db --lib`
- focused envctl CLI integration tests
- `cargo check -p envctl-engine -p envctl`
- `cargo clippy -p envctl-engine -p envctl -- -D warnings`
- no-C and shape gates
- runtime JSON/YAML validate and add/list/show smoke
