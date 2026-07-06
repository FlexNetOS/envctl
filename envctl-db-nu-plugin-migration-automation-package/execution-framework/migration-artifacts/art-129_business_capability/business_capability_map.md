# Business capability map

Task: `ART-129_BUSINESS_CAPABILITY`
Generated at: `2026-07-04T23:19:56+00:00`

## Scope

This artifact maps envctl migration automation systems to the business functions they support for the `flexnetos-vs-lifeos` target. The map is grounded in the target descriptor, package scan, envctl database model, artifact registry, and shared protocol manifests.

## Summary

- Capability rows: `9`
- Envctl DB objects mapped: `23`
- nu_plugin commands mapped: `13`
- Source of truth: `envctl migration database owns durable state; nu_plugin renders records and submits auditable commands.`

## Capability Matrix

| id | business function | business outcome | technical systems | envctl DB objects | nu_plugin surface | controls | confidence |
|---|---|---|---|---|---|---|---|
| BUS-CAP-001 | Migration target intake and scoping | Operators can define the FlexNetOS versus lifeos migration target, preserve safety limits, and select the comparison roots for evidence-based planning. | Target Descriptor Registry<br>schemas/target_descriptor.schema.json<br>envctl migration target list<br>envctl migration target add | envctl_migration_targets | envctl migration target list<br>envctl migration targets | descriptor hash<br>safety mode<br>max auto risk | high |
| BUS-CAP-002 | Package import and contract governance | Prompt packages, artifact contracts, and recipes become versioned business commitments rather than loose files. | Package Import Registry<br>Artifact Contract Registry<br>Migration Recipe Registry<br>contract manifest seed SQL | envctl_migration_packages<br>envctl_migration_artifact_contracts<br>envctl_migration_recipes | envctl migration packages<br>envctl migration package inspect<br>envctl migration package import | contract hash<br>recipe hash<br>source package id | high |
| BUS-CAP-003 | Execution planning and run control | Migration work can be planned, started, paused, resumed, and inspected as auditable operations. | Run Manager<br>Operation Queue<br>Operation State Machine<br>envctl run ledger | envctl_migration_runs<br>envctl_migration_operations<br>envctl_migration_run_events<br>envctl_migration_run_latest_status | envctl migration run plan<br>envctl migration run start<br>envctl migration pause<br>envctl migration resume<br>envctl migration status | operation idempotency key<br>risk class<br>state transition model | high |
| BUS-CAP-004 | Human approval and intervention control | Risky migration changes are gated by explicit approval records and visible operator state. | Approval Gate<br>Human Approval Ledger<br>Live Visuals<br>Agent Control API | envctl_migration_approvals<br>envctl_migration_open_approvals | envctl migration approve<br>envctl migration status | approval-required-from-risk R3<br>approval status<br>human mode | medium |
| BUS-CAP-005 | Evidence, artifact, and lineage governance | Every produced migration deliverable can be tied to content hashes, producers, evidence records, and graph edges. | Artifact Registry<br>Evidence Store<br>Link Graph<br>Proof Record Ledger | envctl_migration_artifacts<br>envctl_migration_evidence<br>envctl_migration_graph_edges<br>envctl_migration_artifact_index | envctl migration proof<br>envctl migration status | SHA-256 content hash<br>producer operation id<br>contract id<br>blocked path rejection | high |
| BUS-CAP-006 | Validation and readiness decision support | Migration readiness can be scored from validation rows, replay checks, and generated proof evidence before downstream verification starts. | Validation Ledger<br>Validation Scorecard View<br>Replay Engine<br>Readiness Reports | envctl_migration_validations<br>envctl_migration_validation_scorecard<br>envctl_migration_replay_readiness | envctl migration replay<br>envctl migration proof<br>envctl migration status | validation status<br>evidence refs<br>replay readiness view | medium |
| BUS-CAP-007 | Rollback and checkpoint assurance | Operators can identify rollback handles and checkpoints before applying migration changes. | Rollback Registry<br>Checkpoint Registry<br>pre-execution framework manifest | envctl_migration_checkpoints<br>envctl_migration_rollbacks | envctl migration status<br>envctl migration replay | rollback plan JSON<br>checkpoint hash<br>rollback status | medium |
| BUS-CAP-008 | Operator presentation through Nushell | Operators get table-shaped, command-oriented access while envctl remains the durable source of truth. | nu_plugin_envctl_migration<br>Shared Protocol Schemas<br>envctl JSON command boundary | envctl_migration_plugin_sessions<br>envctl_migration_agent_sessions<br>execution_framework_proof_records | envctl migration target list<br>envctl migration run plan<br>envctl migration status<br>envctl migration proof | envctl owns durable state<br>mutations emit events<br>shared record schemas | high |
| BUS-CAP-009 | Operational observability and live status | Status, timeline, and stream records let teams monitor artifact production and blockers during the migration. | Append-only Event Log<br>Live Timeline View<br>Plugin Status Streams<br>Live Visuals Renderer | envctl_migration_run_events<br>envctl_migration_live_timeline<br>envctl_migration_run_latest_status | envctl migration status | event sequence<br>actor type<br>previous event hash<br>stream status contract | medium |

## Evidence Inputs

| input | path | purpose |
|---|---|---|
| target descriptor | `examples/target-descriptors/flexnetos-vs-lifeos.yaml` | target roots, safety posture, and migration purpose |
| repo scan | `execution-framework/generated/package_scan.json` | package folders and migration automation source inventory |
| envctl database | `execution-framework/generated/envctl_migration_db_model.json` | durable state tables, views, and capability coverage |
| artifact registry | `execution-framework/generated/envctl_artifact_registry_report.json` | content hash, evidence, graph, validation, and fail-closed path behavior |
| shared protocol | `execution-framework/generated/shared_protocol_manifest.json` | envctl and nu_plugin record contracts |

## Registration Gate

The companion verification report registers this Markdown artifact and its JSON source through the envctl artifact registry smoke path. The proof record records the resulting registry row ids, SHA-256 hashes, validation ids, and graph edges.
