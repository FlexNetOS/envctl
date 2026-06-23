# TASK-0017 — implementation log · STATUS: GREEN

## Changes

- Added a raw TOML composition layer in `Registry::load`:
  - `extends` string/list extraction.
  - relative parent path resolution.
  - cycle guard using canonical paths.
  - depth guard at 8.
  - recursive table merge.
  - `[[component]]` identity merge by component `id`.
- Preserved existing component deserialization and linting after merge, so the public manifest schema
  remains unchanged.
- Added integration tests proving:
  - parent-only components are inherited.
  - child components overlay same-id parent components while preserving inherited hooks.
  - cycle detection fails closed.
  - over-deep chains fail closed.
- Updated docs to describe component-manifest `extends` and to remove `extends` from the future-only
  adoption list.

## Verification

- `cargo fmt --all --check`
- `cargo build -p envctl-engine -p envctl`
- `cargo test -p envctl-engine --test engine manifest_extends -- --nocapture`
- `cargo test -p envctl-engine`
- `cargo clippy -p envctl-engine --all-targets -- -D warnings`
- `cargo run -p envctl -- lock --check`
- `bash ci/gates/p7.sh`
- `bash ci/gates/loop-state.sh`
- `bash ci/gates/no-c.sh`
- `bash ci/gates/shape.sh`
- `bash ci/gates/enable.sh`
- `bash ci/gates/kdf-feature-off.sh`
- `bash ci/gates/harness-scripts.sh`
- `bash ci/gates/agent-env.sh`
