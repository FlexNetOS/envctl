# Implementer Log — ADR-0003 Catalog Phase 1

## Scope

First implementation slice for ADR-0003: establish a read-only catalog/table foundation without changing current file ownership.

## Built

- Added `crates/engine/src/catalog.rs` as an in-memory, read-only catalog importer.
- Added canonical row contracts for:
  - `components`
  - `component_hooks`
  - `paths`
  - `settings`
  - `env_vars`
  - `agent_assets`
  - `registries`
  - `config_files`
  - `migration_evidence`
  - `observed_facts`
- Added explicit manual override fields on settings rows:
  - `manual_override`
  - `override_reason`
  - `override_owner`
  - `override_timestamp`
  - `expires_at`
  - `review_required`
  - `generated_conflict_policy`
- Wired `Engine::catalog_scan()` and exported catalog types from `envctl-engine`.
- Added CLI surfaces:
  - `envctl catalog scan`
  - `envctl catalog scan --json`
  - `envctl catalog table <name>`
  - `envctl catalog table <name> --json`

## Current source coverage

The importer scans current repo surfaces and normalizes them into rows without mutation:

- `manifest/**/*.toml`
- `manifest/envctl.lock`
- `agent-env.yaml`
- `agent-env.lock`
- `.codex/config.toml`
- `.mcp.json`
- hub `registry.json` files
- `crates/engine/src/layout.rs` / `MetaLayout` path registry
- `secretd.toml`
- secrets env schema surfaces in `secretd`, `secrets-engine`, and `secrets-proto`
- `.handoff/tasks/*.json`
- `.handoff/**/*.jsonl`
- `.handoff/loop/*.md`
- `.handoff/decisions/*.md`
- checked-in agent assets under `.agents` / `.Codex`

## Runtime smoke evidence

Current repo scan emits normalized catalog data:

```text
components: 96
component_hooks: 376
paths: 49
settings: 2928
env_vars: 105
agent_assets: 51
registries: 16
config_files: 342
migration_evidence: 0
observed_facts: 694
```

`envctl catalog table env-vars --json` includes 61 secrets/env-schema-derived rows.

## Validation

Passed:

- `cargo fmt --all`
- `cargo test -p envctl-engine catalog`
- `cargo test -p envctl-engine`
- `cargo test -p envctl`
- `cargo clippy --workspace -- -D warnings`
- `bash ci/gates/shape.sh`
- `bash ci/gates/agent-env.sh`
- `bash ci/gates/p7.sh`
- `bash ci/gates/loop-state.sh`
- `bash ci/gates/harness-scripts.sh`
- `bash ci/gates/meta-local-policy.sh`
- runtime `cargo run -q -p envctl --bin envctl -- catalog scan --json`
- runtime `cargo run -q -p envctl --bin envctl -- catalog table components`
- runtime `cargo run -q -p envctl --bin envctl -- catalog table observed-facts`
- runtime `cargo run -q -p envctl --bin envctl -- catalog table env-vars --json`

## Not in this slice

This PR intentionally does not complete ADR-0003. Remaining work includes:

- `catalog diff`
- `catalog render --out <tempdir>`
- import/sync/lock behavior
- deterministic projections
- verifier-gated observed drift reports
- controlled `envctl config edit ...` row mutation/widget flow
- final ADR gap verification pass
