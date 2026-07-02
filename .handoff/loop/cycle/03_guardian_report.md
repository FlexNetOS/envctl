# Guardian Report — ADR-0003 Catalog Phase 1

## Verdict

PASS-WITH-NOTES

This slice safely establishes the ADR-0003 read-only catalog/table foundation and CLI inspection surface. It does not claim ADR completion.

## Invariants checked

- Existing TOML/YAML/JSON/Rust/handoff inputs remain accepted; no source format was removed or replaced.
- Catalog scan and table commands are read-only inspection paths.
- No generated file writes, lock updates, or sync mutations were added.
- Engine owns the catalog logic; CLI only invokes the shared engine surface and formats output.
- Row contracts include first-class ownership/source/provenance fields and explicit manual override metadata.
- `config_files` rows represent scanned source/control-plane files.
- `observed_facts` rows capture read-only verifier-style observations from current repo files.

## Validation evidence

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

Runtime smoke passed:

- `cargo run -q -p envctl --bin envctl -- catalog scan --json`
- `cargo run -q -p envctl --bin envctl -- catalog table components`
- `cargo run -q -p envctl --bin envctl -- catalog table observed-facts`
- `cargo run -q -p envctl --bin envctl -- catalog table env-vars --json`

Observed normalized row counts from the current repo:

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

## Notes / remaining ADR risk

- `migration_evidence` has a row contract but no current evidence rows because this slice does not perform migrations or adoption actions.
- ADR-0003 still requires diff/render/import/sync/lock/config-edit/widget behavior and final code-research-verify coverage.
- The next safe slice should add read-only `catalog diff` and/or `catalog render --out <tempdir>` without applying mutations.
