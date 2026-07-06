# Shared Protocol Schemas

Protocol: `envctl_nu_plugin_migration_protocol`
Version: `1.0.0`
Generated at: `2026-07-04T23:09:51+00:00`

## Source Of Truth

envctl migration database owns durable state; nu_plugin renders records and submits auditable commands.

## Compatibility Rule

Minor versions may add optional fields; removals, type narrowing, and enum removal require a new major protocol version.

## Record Contracts

| record | schema ref | source of truth | source schema | nu_plugin shape |
|---|---|---|---|---|
| `TargetDescriptor` | `schemas/shared_protocol.schema.json#/$defs/TargetDescriptor` | `envctl_migration_targets` | `schemas/target_descriptor.schema.json` | Nushell target descriptor record |
| `MigrationRecipe` | `schemas/shared_protocol.schema.json#/$defs/MigrationRecipe` | `envctl_migration_recipes` | `schemas/migration_recipe.schema.json` | Nushell recipe record |
| `MigrationRun` | `schemas/shared_protocol.schema.json#/$defs/MigrationRun` | `envctl_migration_runs` | `generated` | Nushell run status record |
| `RunEvent` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `envctl_migration_run_events` | `schemas/run_event.schema.json` | Nushell table row stream |
| `Operation` | `schemas/shared_protocol.schema.json#/$defs/Operation` | `envctl_migration_operations` | `schemas/operation.schema.json` | Nushell operation table row and detail record |
| `ApprovalRequest` | `schemas/shared_protocol.schema.json#/$defs/ApprovalRequest` | `envctl_migration_approvals` | `schemas/approval_request.schema.json` | Nushell approval table row |
| `ApprovalDecision` | `schemas/shared_protocol.schema.json#/$defs/ApprovalDecision` | `envctl_migration_approvals` | `generated` | Nushell decision record |
| `ArtifactRecord` | `schemas/shared_protocol.schema.json#/$defs/ArtifactRecord` | `envctl_migration_artifacts` | `schemas/artifact_record.schema.json` | Nushell artifact table row and detail record |
| `EvidenceRecord` | `schemas/shared_protocol.schema.json#/$defs/EvidenceRecord` | `envctl_migration_evidence` | `generated` | Nushell evidence table row |
| `GraphEdge` | `schemas/shared_protocol.schema.json#/$defs/GraphEdge` | `envctl_migration_graph_edges` | `generated` | Nushell graph edge table row |
| `ValidationResult` | `schemas/shared_protocol.schema.json#/$defs/ValidationResult` | `envctl_migration_validations` | `schemas/validation_result.schema.json` | Nushell validation table row |
| `ReplayRequest` | `schemas/shared_protocol.schema.json#/$defs/ReplayRequest` | `envctl_migration_runs` | `generated` | Nushell replay command input record |
| `ReplayResult` | `schemas/shared_protocol.schema.json#/$defs/ReplayResult` | `envctl_migration_run_events` | `generated` | Nushell replay result record |
| `ProofRecord` | `schemas/shared_protocol.schema.json#/$defs/ProofRecord` | `execution_framework_proof_records` | `execution-framework/schemas/proof_record.schema.json` | Execution proof file record |

## Verification

Status: `passed`

| check | value |
|---|---|
| `required_record_count` | `14` |
| `manifest_record_count` | `14` |
| `source_schema_count` | `8` |
| `generated_schema_count` | `6` |
| `sample_validation_count` | `14` |

Sample record validation:

- `TargetDescriptor`: `passed`
- `MigrationRecipe`: `passed`
- `MigrationRun`: `passed`
- `RunEvent`: `passed`
- `Operation`: `passed`
- `ApprovalRequest`: `passed`
- `ApprovalDecision`: `passed`
- `ArtifactRecord`: `passed`
- `EvidenceRecord`: `passed`
- `GraphEdge`: `passed`
- `ValidationResult`: `passed`
- `ReplayRequest`: `passed`
- `ReplayResult`: `passed`
- `ProofRecord`: `passed`

No schema or sample-validation errors were found.
