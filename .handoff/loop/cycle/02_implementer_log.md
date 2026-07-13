# Feature Forge implementer log

## Lane A — agent lock zero-network audit

Status: **GREEN**

Implemented:

- Replaced the `prev.clone()` locked-check self-diff with `audit_lock_zero_network`.
- Local skill, command, and MCP sources are selected and SHA-256 rehashed in place; missing
  local sources fail closed.
- Remote sources are never materialized during `--check --locked`; configured selectors must
  have matching lock identities, non-empty hashes, and exact revision labels or the audit errors.
- Remote root configs and remote `extends` are rejected before an HTTP request is constructed;
  this applies to locked checks, locked sync, and locked remove follow-up sync. Local config
  inheritance continues to work, and normal non-locked modes retain remote loading.
- Lock comparison now covers lock version, additions/removals, and every field of skill and
  non-skill asset entries (hash, revision, source identity, destination, scope where represented,
  and kind/name).
- Preserved `Engine::agent_lock`, `Event::AgentLockChecked`, JSON shape, exit-1-on-drift, no-write
  behavior, and the SHA-256 agent lock / FNV-1a component lock separation.
- Added a hermetic forced-drift counterexample to `ci/gates/agent-env.sh` while keeping the real
  gate on `agent lock --check --locked`.

TDD proof:

- Red first: the new full-field lock comparison test failed because destination/hash/assets were
  ignored; locked Engine tests then exposed the self-diff behavior.
- Green focused tests:
  - `CARGO_TARGET_DIR=/tmp/envctl-lane-a-target cargo test -p envctl-agent-env lock_check_compares_every_skill_and_asset_field`
  - `CARGO_TARGET_DIR=/tmp/envctl-lane-a-target cargo test -p envctl-agent-env zero_network_loader_rejects_remote_root_and_remote_extends`
  - `CARGO_TARGET_DIR=/tmp/envctl-lane-a-target cargo test -p envctl-engine --test agent_command_parity c09_lock_check_locked`
  - `CARGO_TARGET_DIR=/tmp/envctl-lane-a-target cargo test -p envctl-engine --test agent_command_parity`
  - `CARGO_TARGET_DIR=/tmp/envctl-lane-a-target cargo test -p envctl-engine --test agent_sync`
  - `CARGO_TARGET_DIR=/tmp/envctl-lane-a-target cargo test -p envctl --test agent`
  - `CARGO_TARGET_DIR=/tmp/envctl-lane-a-target cargo test -p envctl-agent-env -- --skip fetch_config_text_remote` (250 unit + 83 integration passed; 1 intentionally ignored)
  - `cargo fmt --all -- --check`
  - `CARGO_TARGET_DIR=/tmp/envctl-lane-a-target cargo clippy -p envctl-agent-env -p envctl-engine -p envctl --tests -- -D warnings`
  - `CARGO_TARGET_DIR=/tmp/envctl-lane-a-target bash ci/gates/agent-env.sh`
  - `/tmp/envctl-lane-a-target/debug/envctl agent lock --config agent-env.yaml --check --locked --json --color never` returned exit 0 with `saved: false` and empty drift.

Environment note: an unfiltered `cargo test -p envctl-agent-env` reached 249 passing tests but
four pre-existing `extend::tests::fetch_config_text_remote_*` fixtures could not bind localhost
under this agent's socket-restricted sandbox (`PermissionDenied`). The same suite with only those
four socket fixtures skipped is green; this is not introduced by Lane A.
