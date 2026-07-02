# ADR-0003 catalog-first control plane — implementation slice 1 plan

## Goal

Build the first read-only implementation slice for ADR-0003: an engine-owned catalog scanner plus CLI table output. This slice must not mutate repo files or replace the existing TOML/YAML/JSON sources. It turns existing files into normalized in-memory rows so later PRs can add diff/render/sync/widget behavior.

## Scope

- Add `envctl_engine::catalog` with row contracts for:
  - components
  - component_hooks
  - paths
  - settings
  - env_vars
  - agent_assets
  - registries
  - config_files
  - migration_evidence
  - observed_facts
- Add `Engine::catalog_scan()` as the non-printing shared API.
- Add CLI surfaces:
  - `envctl catalog scan --json`
  - `envctl catalog table <name> [--json]`
- Populate rows from live repo sources where available:
  - manifest TOML + envctl lock
  - agent-env YAML/lock
  - `.codex/config.toml`
  - `.mcp.json`
  - hub `registry.json`
  - `layout.rs` path registry through `MetaLayout`
  - secretd config surfaces when present
  - `.handoff` tasks/ledger/report exports
- Add tests for row contracts, aliases, redaction, and source coverage.

## Non-goals for this PR

- No apply/mutation.
- No DB persistence.
- No authoritative generated-file replacement.
- No bidirectional sync or widget yet.
- No removal of legacy file inputs.

## Verification

- `cargo fmt --all`
- `cargo test -p envctl-engine catalog`
- `cargo build -p envctl`
- runtime checks for catalog scan/table
- relevant gates if touched: shape, agent-env, p7, loop-state, harness-scripts, meta-local-policy
