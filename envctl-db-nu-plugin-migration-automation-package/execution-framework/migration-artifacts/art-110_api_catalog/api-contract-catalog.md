# API Contract Catalog

Task: `ART-110_API_CATALOG`
Generated at: `2026-07-04T23:31:52+00:00`
Target: `flexnetos-vs-lifeos`
Target root: `/home/flexnetos/FlexNetOS`

## Schemas

| Name | Source of Truth | Producer | Consumer | Version |
|---|---|---|---|---|
| `TargetDescriptor` | `envctl_migration_targets` | `envctl` | `nu_plugin` | `1.0.0` |
| `MigrationRecipe` | `envctl_migration_recipes` | `envctl` | `nu_plugin` | `1.0.0` |
| `MigrationRun` | `envctl_migration_runs` | `envctl` | `nu_plugin` | `1.0.0` |
| `RunEvent` | `envctl_migration_run_events` | `envctl` | `nu_plugin` | `1.0.0` |
| `Operation` | `envctl_migration_operations` | `envctl` | `nu_plugin` | `1.0.0` |
| `ApprovalRequest` | `envctl_migration_approvals` | `envctl` | `nu_plugin` | `1.0.0` |
| `ApprovalDecision` | `envctl_migration_approvals` | `envctl` | `nu_plugin` | `1.0.0` |
| `ArtifactRecord` | `envctl_migration_artifacts` | `envctl` | `nu_plugin` | `1.0.0` |
| `EvidenceRecord` | `envctl_migration_evidence` | `envctl` | `nu_plugin` | `1.0.0` |
| `GraphEdge` | `envctl_migration_graph_edges` | `envctl` | `nu_plugin` | `1.0.0` |
| `ValidationResult` | `envctl_migration_validations` | `envctl` | `nu_plugin` | `1.0.0` |
| `ReplayRequest` | `envctl_migration_runs` | `envctl` | `nu_plugin` | `1.0.0` |
| `ReplayResult` | `envctl_migration_run_events` | `envctl` | `nu_plugin` | `1.0.0` |
| `ProofRecord` | `execution_framework_proof_records` | `envctl` | `nu_plugin` | `1.0.0` |

## Consumers

| Consumer | Role | Contract Refs |
|---|---|---|
| `artifact-agent` | artifact generation or validation actor | schemas/shared_protocol.schema.json, generated/contract_manifest.json |
| `codex-cli-background-shell` | automation consumer | schemas/shared_protocol.schema.json, generated/contract_manifest.json |
| `envctl` | producer and durable-state owner | schemas/shared_protocol.schema.json, generated/contract_manifest.json |
| `human-operator` | approval and review actor | schemas/shared_protocol.schema.json, generated/contract_manifest.json |
| `nu_plugin` | interactive renderer and command submitter | schemas/shared_protocol.schema.json, generated/contract_manifest.json |
| `validation-agent` | artifact generation or validation actor | schemas/shared_protocol.schema.json, generated/contract_manifest.json |

## Versions

| Version | Surface Count |
|---|---:|
| `1.0.0` | 15 |
| `unversioned` | 238 |
| `v1` | 81 |
