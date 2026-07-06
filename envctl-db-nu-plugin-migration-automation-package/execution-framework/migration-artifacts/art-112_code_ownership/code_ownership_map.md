# Code Ownership Map

Generated at: `2026-07-04T23:19:29+00:00`
Target root: `/home/flexnetos/FlexNetOS`

## Coverage

| area | count |
|---|---:|
| modules | 3 |
| services | 3 |
| data pipelines | 2 |
| path entries | 26 |
| ownership gaps | 0 |

## Ownership

| id | kind | primary owner | support owner | lane | responsibility |
|---|---|---|---|---|---|
| envctl-engine | module | envctl-db-agent | envctl-runner-agent | lane_b_repo_a | Owns envctl repository state, component catalog, agent environment sync, migration operations, and database-facing command behavior. |
| envctl-artifact-registry | module | envctl-db-agent | validation-agent | lane_b_repo_a | Persists artifact paths, hashes, producers, contract IDs, provenance, graph edges, evidence, and validation links. |
| nu-plugin-control-surface | service | nu-plugin-agent | nu-plugin-visuals-agent | lane_c_repo_b | Renders envctl migration targets, runs, events, operations, artifacts, approvals, validations, graph views, and replay controls. |
| shared-protocol | module | shared-protocol-agent | envctl-db-agent | shared_protocol | Defines records consumed by envctl and nu_plugin: target descriptors, runs, events, operations, artifacts, evidence, graph edges, validations, replay, and proofs. |
| artifact-generation | data_pipeline | artifact-agent | validation-agent | lane_d_filesystem | Synthesizes contract artifacts and registers each artifact with hash, validation evidence, and graph links. |
| validation-evidence | data_pipeline | validation-agent | artifact-agent | lane_e_verification | Owns proof records, logs, validation scorecards, heartbeat state, and replay-ready command evidence. |
| flexnetos-adapter | service | flexnetos-adapter-agent | flexnetos-agent | lane_b_repo_a | Adapts the FlexNetOS filesystem target into envctl migration target descriptors, recipes, imports, artifact records, and replay checks. |
| runtime-secrets-safety | service | security-reproducibility-agent | envctl-db-agent | lane_d_filesystem | Keeps secret-bearing paths out of artifact capture, enforces redaction, and records reproducibility-safe evidence only. |

## Path Evidence

### envctl engine

- `src/envctl/crates/engine/src`: present
- `src/envctl/Cargo.toml`: present
- `src/envctl/agent-env.yaml`: present
- Observed file count: `39`

### envctl artifact registry

- `src/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/scripts/artifact_registry.py`: present
- `src/envctl/envctl-db-nu-plugin-migration-automation-package/sql`: present
- `src/envctl/envctl-db-nu-plugin-migration-automation-package/schemas`: present
- Observed file count: `13`

### nu_plugin control surface

- `src/nu_plugin/crates/codedb`: present
- `src/nu_plugin/crates/codedb_mcp`: present
- `src/nu_plugin/docs`: present
- `src/nu_plugin/tests`: present
- Observed file count: `56`

### shared protocol schemas

- `schemas`: present
- `execution-framework/schemas`: present
- `execution-framework/generated/shared_protocol_manifest.json`: present
- Observed file count: `0`

### migration artifact generation

- `execution-framework/generated/execution_packets/ART-*.json`: not observed
- `execution-framework/migration-artifacts`: present
- `execution-framework/proof_records`: present
- Observed file count: `0`

### validation evidence and proof ledger

- `execution-framework/proof_records`: present
- `execution-framework/logs`: present
- `execution-framework/state`: present
- Observed file count: `0`

### FlexNetOS adapter recipe

- `examples/target-descriptors/flexnetos-vs-lifeos.yaml`: present
- `specs/flexnetos-adapter.md`: present
- `execution-framework/generated/envctl_target_registry.json`: present
- Observed file count: `0`

### runtime secrets and redaction safety

- `src/envctl/crates/secretctl`: present
- `src/envctl/crates/secrets-*`: not observed
- `src/envctl/scripts/classify-secrets.nu`: present
- `src/nu_plugin/docs/SECURITY_AND_SECRET_POLICY.md`: present
- Observed file count: `5`

## Registry Contract

This artifact is registered as both JSON and Markdown records. The registry result stores package-relative paths, SHA-256 hashes, producer operation IDs, contract ID, provenance, evidence references, graph links, and validation links.
