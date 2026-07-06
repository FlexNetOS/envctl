# Deprecation Map

- Task: `ART-133_DEPRECATION_MAP`
- Contract artifact: `artifact:02-target-state-deprecation-map-md`
- Canonical path: `migration-artifacts/02-target-state/deprecation-map.md`
- Generated at: `2026-07-04T23:28:24+00:00`
- Components: 8
- Retired: 1; replaced: 2; wrapped: 2; preserved: 3

## Map

| component | classification | disposition | replacement | owner | risk |
|---|---|---|---|---|---|
| source/codex-flexnetos-migration-prompt-package/helpers/__pycache__ | retired | Exclude from authoritative migration source and regenerated package outputs. | Source helper scripts and package manifest entries without CPython cache files. | artifact-agent | medium |
| Markdown-only migration artifact status | replaced | Replace with envctl artifact rows, content hashes, validation records, and graph links. | envctl_migration_artifacts plus envctl_migration_evidence, envctl_migration_graph_edges, and envctl_migration_validations. | envctl-db-agent | medium |
| Hardcoded FlexNetOS/lifeos comparison paths in prior prompts | replaced | Replace with target descriptor driven roots and safety policy. | migration-artifacts/_meta/flexnetos-vs-lifeos.target-descriptor.yaml and envctl target registry rows. | target-registry-agent | medium |
| source/codex-flexnetos-migration-prompt-package/helpers/background_scan.sh | wrapped | Keep as an external collector operation during adapter MVP, then replace gradually where native collectors are practical. | envctl operation records that execute/import helper output, followed by native collector implementations. | flexnetos-adapter-agent | medium |
| codex CLI background shell execution | wrapped | Run through execution packets and envctl operation metadata instead of treating raw stdout as the state boundary. | envctl run ledger, operation records, event/evidence hashes, and proof records. | envctl-runner-agent | medium |
| codex-flexnetos-migration-prompt-package artifact contract | preserved | Preserve as the first external package fixture and acceptance contract. | No replacement; import into envctl contracts and recipes. | artifact-agent | low |
| SQLite migration automation schema and in-memory proof fixture | preserved | Preserve for package-local validation while keeping backend portability risk visible. | Compatible envctl migrations for any future durable backend. | envctl-db-agent | medium |
| Shared envctl/nu_plugin protocol schemas | preserved | Preserve as the structured boundary for plugin reads and operator decisions. | No replacement; downstream plugin commands consume this boundary. | shared-protocol-agent | low |

## Evidence

| component id | evidence refs |
|---|---|
| `deprecated-generated-bytecode-cache` | `migration-artifacts/art-105_package_lib_graph/package_lib_graph.json` |
| `prompt-only-artifact-state` | `generated/envctl_migration_db_model.json`, `proof_records/REQ-024_ENVCTL_ARTIFACT_REGISTRY.proof.json` |
| `hardcoded-flexnetos-lifeos-paths` | `migration-artifacts/_meta/flexnetos-vs-lifeos.target-descriptor.yaml`, `generated/flexnetos_target_descriptor_validation_report.json` |
| `external-background-scan-helper` | `prompts/STRATEGY_DECISION.md`, `prompts/UTILIZE_FLEXNETOS_PACKAGE.md`, `prompts/IMPLEMENTATION_PHASES.md` |
| `codex-exec-runtime` | `generated/execution_packets/ART-133_DEPRECATION_MAP.json`, `generated/envctl_run_ledger_report.json`, `proof_records/REQ-022_ENVCTL_RUN_LEDGER.proof.json` |
| `prior-flexnetos-package-contract` | `generated/contract_manifest.json`, `source/codex-flexnetos-migration-prompt-package/PACKAGE_MANIFEST.json` |
| `sqlite-registry-fixture` | `generated/envctl_migration_db_model.json`, `proof_records/REQ-020_ENVCTL_DB_SCHEMA.proof.json` |
| `shared-protocol-schemas` | `generated/shared_protocol_validation_report.json`, `schemas/shared_protocol.schema.json` |

## Validation Links

- Depends on `REQ-024_ENVCTL_ARTIFACT_REGISTRY` for path and hash registration.
- Depends on `REQ-040_SHARED_PROTOCOL_SCHEMAS` for structured protocol compatibility.
- Blocks `VER-300_UNIT_VALIDATION` until the artifact file, registry hash, and validation links are present.
