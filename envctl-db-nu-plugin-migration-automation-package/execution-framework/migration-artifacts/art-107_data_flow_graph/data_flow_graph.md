# Data flow graph

Task: `ART-107_DATA_FLOW_GRAPH`
Generated at: `2026-07-04T23:28:55+00:00`

## Scope

This artifact maps how migration data enters, moves through, transforms within, persists in, and exits the envctl migration automation package for the `flexnetos-vs-lifeos` target.

## Summary

- Nodes: `12`
- Edges: `13`
- Flow paths: `3`
- Required stages present: `True`
- Stage counts: `{"data_entry": 3, "exit": 2, "movement": 2, "persistence": 2, "transformation": 3}`

## Graph Nodes

| id | stage | label | data subjects | persistence/redaction note | evidence |
|---|---|---|---|---|---|
| entry:target-descriptor | data_entry | Target descriptor intake | target_id<br>primary_root<br>compare_root<br>include/exclude globs<br>safety policy | blocked paths exclude .env, secrets, private_keys, pem, and key files before graph materialization | `execution-framework/migration-artifacts/_meta/flexnetos-vs-lifeos.target-descriptor.yaml`<br>`execution-framework/generated/envctl_target_registry.json` |
| entry:repo-scan | data_entry | Repository and filesystem scan | file paths<br>manifest metadata<br>service definitions<br>source inventory | secret-like paths are skipped by descriptor policy and artifact registry path policy | `execution-framework/generated/package_scan.json`<br>`execution-framework/migration-artifacts/art-101_directory_tree/directory-tree.json`<br>`execution-framework/migration-artifacts/art-102_repository_map/repository-map.json` |
| entry:operator-command | data_entry | Operator command surface | command arguments<br>approval decisions<br>replay requests<br>proof lookups | commands are represented as redacted command strings in operations and events | `execution-framework/generated/nu_plugin_command_manifest.json`<br>`execution-framework/generated/shared_protocol_manifest.json` |
| move:package-import | movement | Package import and contract bind | package hash<br>contract hash<br>recipe hash<br>artifact requirements | Package files, contract rows, and recipe metadata move into envctl migration package, contract, and recipe records. | `execution-framework/generated/contract_manifest.json`<br>`execution-framework/generated/contract_manifest.seed.sql`<br>`execution-framework/docs/CONTRACT_MANIFEST.md` |
| move:run-operation-events | movement | Run, operation, and event stream | run status<br>operation state<br>risk<br>idempotency key<br>event hash | A migration run owns operation rows and append-only event records; plugins observe the latest status and timeline views. | `execution-framework/generated/envctl_run_ledger_report.json`<br>`execution-framework/generated/operation_state_machine.json`<br>`execution-framework/docs/ENVCTL_RUN_LEDGER.md` |
| transform:redaction-policy | transformation | Security redaction and path policy | path policy<br>redacted command<br>safe evidence URI | Blocked paths and sensitive command details are filtered or represented by hashes before becoming graph evidence. | `execution-framework/generated/security_redaction_validation_report.json`<br>`execution-framework/docs/SECURITY_REDACTION.md`<br>`execution-framework/scripts/artifact_registry.py` |
| transform:protocol-shaping | transformation | Shared protocol shaping | TargetDescriptor<br>Operation<br>ArtifactRecord<br>EvidenceRecord<br>GraphEdge<br>ValidationResult | Envctl DB rows are shaped into shared records consumed by envctl and nu_plugin without moving durable ownership to the plugin. | `execution-framework/generated/shared_protocol_manifest.json`<br>`execution-framework/schemas/shared_protocol.schema.json`<br>`execution-framework/docs/SHARED_PROTOCOL_SCHEMAS.md` |
| transform:artifact-generation | transformation | Artifact generation and hash calculation | artifact body<br>sha256 content hash<br>validation details<br>graph links | Generated Markdown and JSON artifacts are converted into registry records with content hashes, evidence refs, validations, and graph edges. | `execution-framework/generated/envctl_artifact_registry_report.json`<br>`execution-framework/docs/ENVCTL_ARTIFACT_REGISTRY.md`<br>`execution-framework/schemas/proof_record.schema.json` |
| persist:envctl-db | persistence | Envctl migration database | all envctl_migration_* rows and views | Durable migration control-plane state is persisted in targets, packages, contracts, recipes, runs, operations, evidence, artifacts, graph edges, approvals, validations, checkpoints, rollbacks, agent sessions, and plugin sessions. | `execution-framework/generated/envctl_migration_db_model.json`<br>`execution-framework/docs/ENVCTL_DB_SCHEMA.md`<br>`sql/001_migration_automation_schema.sql`<br>`sql/002_views_and_indexes.sql` |
| persist:proof-ledger | persistence | Proof records and proof ledger | files changed<br>commands run<br>verification output<br>checksums | Task completion evidence is written as a proof JSON file and deduplicated proof ledger row. | `execution-framework/proof_records/proof_ledger.jsonl`<br>`execution-framework/schemas/proof_record.schema.json`<br>`execution-framework/proof_templates/PROOF_RECORD_TEMPLATE.json` |
| exit:migration-artifacts | exit | Migration artifact outputs | data_flow_graph.md<br>data_flow_graph.json<br>registry report | Markdown and JSON deliverables leave the generator as migration-artifacts files, with registry hashes proving the emitted content. | `execution-framework/migration-artifacts/art-107_data_flow_graph/data_flow_graph.md`<br>`execution-framework/migration-artifacts/art-107_data_flow_graph/data_flow_graph.json`<br>`execution-framework/generated/art_107_data_flow_graph_registry_report.json` |
| exit:operator-views | exit | Operator and downstream validation views | artifact index rows<br>validation scorecard rows<br>status table rows | Artifact index, validation scorecard, status streams, and VER-300 consume hashes, validation rows, and graph edges. | `execution-framework/generated/live_visuals.json`<br>`execution-framework/generated/final_verification_report.json`<br>`execution-framework/generated/task_graph.normalized.json` |

## Graph Edges

| id | from | to | type | data | via |
|---|---|---|---|---|---|
| edge-data-entry-target-to-db | entry:target-descriptor | persist:envctl-db | enters_as | target descriptor fields<br>descriptor hash<br>safety mode | envctl_migration_targets |
| edge-repo-scan-to-package-import | entry:repo-scan | move:package-import | moves_into | package inventory<br>contract manifest<br>source refs | envctl_migration_packages<br>envctl_migration_artifact_contracts |
| edge-operator-command-to-run | entry:operator-command | move:run-operation-events | submits | run plan requests<br>approval decisions<br>proof queries | envctl endpoints with --emit-event |
| edge-package-import-to-run | move:package-import | move:run-operation-events | binds_recipe_to_run | recipe id<br>artifact contract id<br>run id | envctl_migration_runs<br>envctl_migration_operations |
| edge-run-to-redaction | move:run-operation-events | transform:redaction-policy | normalizes | command_redacted<br>evidence URI<br>blocked path checks | security controls<br>artifact registry path policy |
| edge-db-to-protocol | persist:envctl-db | transform:protocol-shaping | projects | source-of-truth rows<br>record schemas<br>plugin table shapes | shared protocol manifest |
| edge-redaction-to-artifact-generation | transform:redaction-policy | transform:artifact-generation | permits_safe_evidence | safe evidence refs<br>redaction status<br>hashable artifact paths | ArtifactRegistry._validate_path_policy |
| edge-protocol-to-artifact-generation | transform:protocol-shaping | transform:artifact-generation | schemas | ArtifactRecord<br>EvidenceRecord<br>GraphEdge<br>ValidationResult<br>ProofRecord | schemas/shared_protocol.schema.json |
| edge-artifact-generation-to-db | transform:artifact-generation | persist:envctl-db | persists_registry_rows | artifact rows<br>evidence rows<br>graph edges<br>validation rows | envctl_migration_artifacts<br>envctl_migration_evidence<br>envctl_migration_graph_edges<br>envctl_migration_validations |
| edge-artifact-generation-to-proof-ledger | transform:artifact-generation | persist:proof-ledger | records_proof | verification output<br>checksums<br>evidence list | proof_records/ART-107_DATA_FLOW_GRAPH.proof.json<br>proof_records/proof_ledger.jsonl |
| edge-proof-ledger-to-artifact-exit | persist:proof-ledger | exit:migration-artifacts | attests | file checksums<br>commands run<br>registry result | proof record |
| edge-db-to-operator-views | persist:envctl-db | exit:operator-views | renders_as | artifact index<br>validation scorecard<br>timeline<br>latest status | envctl_migration_artifact_index<br>envctl_migration_validation_scorecard |
| edge-artifact-exit-to-validation | exit:migration-artifacts | exit:operator-views | blocks | VER-300_UNIT_VALIDATION input | task graph block edge |

## Flow Paths

| path | entry | movement | transformation | persistence | exit | data classes |
|---|---|---|---|---|---|---|
| flow-target-descriptor-to-status | entry:target-descriptor | move:package-import<br>move:run-operation-events | transform:protocol-shaping | persist:envctl-db | exit:operator-views | target config<br>safety policy<br>status rows |
| flow-repo-scan-to-artifacts | entry:repo-scan | move:package-import | transform:redaction-policy<br>transform:artifact-generation | persist:envctl-db<br>persist:proof-ledger | exit:migration-artifacts<br>exit:operator-views | file inventory<br>artifact body<br>content hash<br>validation evidence |
| flow-operator-command-to-proof | entry:operator-command | move:run-operation-events | transform:protocol-shaping<br>transform:artifact-generation | persist:envctl-db<br>persist:proof-ledger | exit:operator-views | command input<br>operation event<br>artifact/proof lookup |

## Registration Gate

The companion registry report records the Markdown and JSON artifacts in the envctl artifact registry with SHA-256 hashes, evidence rows, validation rows, and graph edges. `VER-300_UNIT_VALIDATION` is the downstream validation consumer.
