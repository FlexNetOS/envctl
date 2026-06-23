# TASK-0016 — implementation log · STATUS: GREEN

## Changes

- Updated `crates/agent-env/src/lock.rs` and `crates/engine/src/agent/lock.rs` so the code comments
  reflect the accepted split: standalone SHA-256 `agent-env.lock` and separate FNV-1a
  `manifest/envctl.lock`.
- Preserved `AGENT_ASSETS_KEY` as a reserved/export label, not an active promise that envctl embeds
  the lock into the component lock.
- Reframed `manifest/agent-env.toml` around `agent-env.yaml` and the built-in `envctl agent` verbs:
  install/fix now run `envctl agent sync --config agent-env.yaml --apply`, and verify now runs
  `envctl agent lock --config agent-env.yaml --check --locked`.
- Tightened `ci/gates/agent-env.sh` to pass `--config agent-env.yaml --check --locked`, matching the
  zero-network drift-gate claim.
- Regenerated `manifest/envctl.lock` with `envctl lock` so the component lock matches the manifest.
- Added `.handoff/decisions/TASK-0016-agent-lock-boundary.md` and refined ADR-0001 to document the
  no-downgrade lock placement.

## Verification

- `cargo fmt --all --check`
- `cargo build -p envctl-engine -p envctl`
- `cargo test -p envctl-agent-env`
- `cargo test -p envctl-engine`
- `cargo clippy -p envctl-engine -p envctl-agent-env --all-targets -- -D warnings`
- `cargo run -p envctl -- lock --check`
- `bash ci/gates/agent-env.sh`
- `bash ci/gates/p7.sh`
- `bash ci/gates/loop-state.sh`
- `bash ci/gates/no-c.sh`
- `bash ci/gates/shape.sh`
- `bash ci/gates/enable.sh`
- `bash ci/gates/kdf-feature-off.sh`
- `bash ci/gates/harness-scripts.sh`
