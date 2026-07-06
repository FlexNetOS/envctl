# Business process map

Task: `ART-119_BUSINESS_PROCESS`
Generated at: `2026-07-04T23:28:24+00:00`

## Scope

This artifact maps migration business workflows to the envctl database objects, registry services, proof surfaces, and nu_plugin commands they depend on for the `flexnetos-vs-lifeos` target.

## Summary

- Workflow rows: `8`
- Depending systems: `32`
- High-risk workflows: `2`
- Source of truth: `envctl migration database owns durable state; nu_plugin renders records and submits auditable commands.`

## Workflow Dependency Matrix

| id | workflow | trigger | outcome | process steps | depending systems | downstream artifacts | controls | risk |
|---|---|---|---|---|---|---|---|---|
| BUS-PROC-001 | Target intake and safety scoping | An operator needs to evaluate a FlexNetOS to lifeos migration target. | The migration target is described, bounded, and approval posture is known before any artifact run starts. | select target descriptor<br>register or refresh target row<br>record descriptor hash and safety mode<br>expose target rows to the operator surface | Target Descriptor Registry<br>envctl_migration_targets<br>schemas/target_descriptor.schema.json<br>envctl migration target list | system inventory<br>repository map<br>dependency graph | descriptor hash<br>approval-gated safety mode<br>max auto risk R2 | medium |
| BUS-PROC-002 | Package intake and artifact contract selection | The migration team needs a repeatable artifact package rather than ad hoc prompts. | The package, artifact contract, and recipe become durable commitments for the run. | scan package layout<br>seed contract manifest<br>select recipe<br>bind run to artifact contract | Package Import Registry<br>Artifact Contract Registry<br>Migration Recipe Registry<br>envctl_migration_artifact_contracts | artifact registry<br>run ledger<br>proof ledger | package hash<br>contract hash<br>recipe hash | low |
| BUS-PROC-003 | Run planning and operation sequencing | Operators need a visible order of work before generation begins. | Each artifact task is represented as a planned operation with dependency and blocker context. | load task graph<br>build execution packets<br>identify dependency gates<br>queue artifact generation operations | Task Graph<br>Operation Queue<br>envctl_migration_operations<br>envctl migration run plan | live visuals<br>operation state machine<br>readiness scorecard | idempotency key<br>phase<br>risk class<br>state transition guard | medium |
| BUS-PROC-004 | Approval-gated execution control | A workflow step exceeds the automatic risk threshold or requires operator intervention. | The human decision is recorded without letting plugin commands mutate durable state directly. | surface approval request<br>capture approval decision<br>emit event from envctl<br>resume or block operation | Human Approval Ledger<br>envctl_migration_approvals<br>envctl_migration_open_approvals<br>envctl migration approve | run ledger<br>risk register<br>decision log | approval status<br>human mode<br>event emission rule | high |
| BUS-PROC-005 | Artifact production and registration | A required migration deliverable must be generated and made auditable. | The artifact exists at the contract path and has content hash, producer, evidence, and graph links. | read allowed inputs<br>write task-scoped Markdown and JSON<br>write canonical contract Markdown<br>register artifacts through envctl artifact registry | Artifact Registry<br>Evidence Store<br>Link Graph<br>envctl_migration_artifacts | artifact index<br>validation evidence<br>unit validation | sha256 content hash<br>blocked path policy<br>producer operation id | medium |
| BUS-PROC-006 | Validation evidence and readiness scoring | The team needs proof that generated outputs satisfy completion gates. | Validation rows, evidence refs, and scorecards give a reviewable path to unit validation. | run artifact-specific checks<br>record validation rows<br>link proof records<br>feed readiness scorecard | Validation Ledger<br>envctl_migration_validations<br>envctl_migration_validation_scorecard<br>Proof Record Ledger | readiness scorecard<br>final verification<br>validation reconciliation | validation status<br>evidence hash<br>proof schema | medium |
| BUS-PROC-007 | Operator status, timeline, and plugin presentation | Operators need to see current progress, blockers, and handoffs while work is running. | Nushell displays envctl-owned records as tables and status streams without becoming the state owner. | read run latest status<br>render operation timeline<br>display proof and artifact summaries<br>stream plugin status rows | nu_plugin_envctl_migration<br>envctl_migration_run_latest_status<br>envctl_migration_live_timeline<br>envctl migration status | live visuals<br>operator handoff<br>final verification | state owner rule<br>append-only event sequence<br>plugin read/mutate modes | low |
| BUS-PROC-008 | Replay, rollback, and exception handling | A run must be replayed, audited, rolled back, or marked with an exception before cutover. | The team has checkpoint and rollback context tied to artifacts and evidence. | verify hashes for replay<br>inspect checkpoint and rollback handles<br>record exceptions<br>feed cutover and wave planning | Replay Engine<br>Checkpoint Registry<br>Rollback Registry<br>envctl migration replay | wave plan<br>cutover plan<br>rollback plan<br>exception inventory | replay hash verification<br>checkpoint hash<br>rollback status | high |

## System Dependency Inventory

| system | workflows depending on it |
|---|---|
| Artifact Contract Registry | BUS-PROC-002 |
| Artifact Registry | BUS-PROC-005 |
| Checkpoint Registry | BUS-PROC-008 |
| Evidence Store | BUS-PROC-005 |
| Human Approval Ledger | BUS-PROC-004 |
| Link Graph | BUS-PROC-005 |
| Migration Recipe Registry | BUS-PROC-002 |
| Operation Queue | BUS-PROC-003 |
| Package Import Registry | BUS-PROC-002 |
| Proof Record Ledger | BUS-PROC-006 |
| Replay Engine | BUS-PROC-008 |
| Rollback Registry | BUS-PROC-008 |
| Target Descriptor Registry | BUS-PROC-001 |
| Task Graph | BUS-PROC-003 |
| Validation Ledger | BUS-PROC-006 |
| envctl migration approve | BUS-PROC-004 |
| envctl migration replay | BUS-PROC-008 |
| envctl migration run plan | BUS-PROC-003 |
| envctl migration status | BUS-PROC-007 |
| envctl migration target list | BUS-PROC-001 |
| envctl_migration_approvals | BUS-PROC-004 |
| envctl_migration_artifact_contracts | BUS-PROC-002 |
| envctl_migration_artifacts | BUS-PROC-005 |
| envctl_migration_live_timeline | BUS-PROC-007 |
| envctl_migration_open_approvals | BUS-PROC-004 |
| envctl_migration_operations | BUS-PROC-003 |
| envctl_migration_run_latest_status | BUS-PROC-007 |
| envctl_migration_targets | BUS-PROC-001 |
| envctl_migration_validation_scorecard | BUS-PROC-006 |
| envctl_migration_validations | BUS-PROC-006 |
| nu_plugin_envctl_migration | BUS-PROC-007 |
| schemas/target_descriptor.schema.json | BUS-PROC-001 |

## Evidence Inputs

| input | path | purpose |
|---|---|---|
| target descriptor | `examples/target-descriptors/flexnetos-vs-lifeos.yaml` | target roots, safety posture, and migration purpose |
| repo scan | `execution-framework/generated/package_scan.json` | package folders and automation source inventory |
| envctl database | `execution-framework/generated/envctl_migration_db_model.json` | durable state tables and views that support workflows |
| artifact registry | `execution-framework/generated/envctl_artifact_registry_report.json` | content hash, evidence, graph, validation, and blocked path behavior |
| shared protocol | `execution-framework/generated/shared_protocol_manifest.json` | record contracts shared by envctl and nu_plugin |
| command surface | `execution-framework/generated/nu_plugin_command_manifest.json` | operator commands used by the workflow map |

## Registration Gate

The companion verification report registers the task Markdown, task JSON, and canonical current-state Markdown through the envctl artifact registry smoke path. The proof record records the resulting registry row ids, SHA-256 hashes, validation ids, and graph edges.
