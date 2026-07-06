# Migration Wave Plan

- Task: `ART-120_WAVE_PLAN`
- Generated at: `2026-07-04T23:28:32+00:00`
- Target: `flexnetos-vs-lifeos`
- Primary root: `/home/flexnetos/FlexNetOS`
- Compare root present: `False`
- Safety mode: `approval-gated`

## Sequence

| wave | move group | entry gate | exit gate | rationale | complete | pending |
|---|---|---|---|---|---:|---:|
| W0 Control-plane contract lock | envctl database, artifact registry, shared schemas, target descriptor, security and filesystem guardrails | Execution framework package is present and task graph packets have been generated. | REQ-020 through REQ-025, REQ-040, REQ-042 through REQ-044, and REQ-200 have completed proof records. | Cutover artifacts need durable schema, registry hash capture, redaction policy, and a target descriptor before any migration batch can be trusted. | 11 | 0 |
| W1 Inventory and ownership baseline | system inventory, repository map, directory tree, dependency maps, toolchain map, ownership map, debug entrypoint map | W0 proof records exist and registry rows can store generated hashes. | Current-state and code-analysis artifacts have canonical markdown plus task-local JSON where applicable. | The migration must know what exists, who owns it, and how modules call each other before runtime or data movement is sequenced. | 8 | 0 |
| W2 Configuration, infrastructure, and access baseline | environment matrix, configuration inventory, infrastructure topology, IAM/security matrix, observability map | W1 dependency and ownership artifacts are complete. | Configuration and access surfaces have redacted, registry-linked evidence. | Config, IAM, infrastructure, and telemetry define the safety envelope for any service or data move. | 4 | 1 |
| W3 Runtime, interface, and data contract mapping | runtime dependencies, data flow, schema map, lineage, API catalog, event/message map | W2 safety and observability baselines are available. | Runtime, API, event, and data artifacts are registered and reconciliation candidates are known. | Runtime dependencies and contracts should move before business cutover planning, because they expose ordering constraints and parity checks. | 0 | 6 |
| W4 Governance and business readiness | business process map, wave plan, risk register, decision log, readiness scorecard, business capability map, RACI | W1 through W3 describe system, runtime, and data surfaces well enough to plan moves. | Governance artifacts expose risks, decisions, owners, and readiness gaps for validation. | Governance should freeze the sequence and ownership after the technical dependency map is visible, not before. | 5 | 3 |
| W5 Validation, parity, and release evidence | validation reconciliation, test coverage, golden dataset, parity dashboard, shadow traffic, unit validation | Governance plan, risk, and readiness artifacts are registered. | VER-300 and follow-on verification lanes can consume registered artifact hashes and evidence links. | Validation runs last among planning waves so it checks the planned move sequence against actual artifacts and parity evidence. | 2 | 4 |
| W6 Cutover, rollback, and decommission controls | rollback checkpoints, replay engine, cutover checklist, rollback plan, deprecation map, exception inventory, technical debt ledger, release handoff | W5 validation has passed or has explicit human-approved exceptions. | Release packaging and handoff can proceed with documented rollback and decommission evidence. | Irreversible or release-facing steps wait for validation, rollback checkpoints, and exception handling to be explicit. | 1 | 10 |

## Wave Task Detail

### W0 - Control-plane contract lock

- Rollback anchor: `history/pre_execution_framework_manifest.json`

| task | title | status | proof |
|---|---|---|---|
| REQ-020_ENVCTL_DB_SCHEMA | Implement envctl database migration tables | completed | proof_records/REQ-020_ENVCTL_DB_SCHEMA.proof.json |
| REQ-021_ENVCTL_TARGET_REGISTRY | Implement target registry | completed | proof_records/REQ-021_ENVCTL_TARGET_REGISTRY.proof.json |
| REQ-022_ENVCTL_RUN_LEDGER | Implement run ledger | completed | proof_records/REQ-022_ENVCTL_RUN_LEDGER.proof.json |
| REQ-023_ENVCTL_OPERATION_STATE | Implement operation state machine | completed | proof_records/REQ-023_ENVCTL_OPERATION_STATE.proof.json |
| REQ-024_ENVCTL_ARTIFACT_REGISTRY | Implement artifact registry | completed | proof_records/REQ-024_ENVCTL_ARTIFACT_REGISTRY.proof.json |
| REQ-025_ENVCTL_VALIDATION_EVIDENCE | Implement validation evidence store | completed | proof_records/REQ-025_ENVCTL_VALIDATION_EVIDENCE.proof.json |
| REQ-040_SHARED_PROTOCOL_SCHEMAS | Implement shared protocol schemas | completed | proof_records/REQ-040_SHARED_PROTOCOL_SCHEMAS.proof.json |
| REQ-042_FILESYSTEM_BOUNDS | Implement filesystem work boundaries | completed | proof_records/REQ-042_FILESYSTEM_BOUNDS.proof.json |
| REQ-043_SECURITY_REDACTION | Implement redaction and secret safety | completed | proof_records/REQ-043_SECURITY_REDACTION.proof.json |
| REQ-044_INSTALL_BOOTSTRAP | Implement install/bootstrap wiring | completed | proof_records/REQ-044_INSTALL_BOOTSTRAP.proof.json |
| REQ-200_FLEXNETOS_TARGET_DESCRIPTOR | Create FlexNetOS target descriptor | completed | proof_records/REQ-200_FLEXNETOS_TARGET_DESCRIPTOR.proof.json |

### W1 - Inventory and ownership baseline

- Rollback anchor: `Remove W1-generated artifact files and proof rows only.`

| task | title | status | proof |
|---|---|---|---|
| ART-100_SYSTEM_INVENTORY | Build System inventory | completed | proof_records/ART-100_SYSTEM_INVENTORY.proof.json |
| ART-101_DIRECTORY_TREE | Build Directory and subdirectory hierarchy tree | completed | proof_records/ART-101_DIRECTORY_TREE.proof.json |
| ART-102_REPOSITORY_MAP | Build Repository map | completed | proof_records/ART-102_REPOSITORY_MAP.proof.json |
| ART-103_SERVICE_DEP_GRAPH | Build Application/service dependency graph | passed | proof_records/ART-103_SERVICE_DEP_GRAPH.proof.json |
| ART-104_TOOLCHAIN_TREE | Build Toolchain dependency tree | completed | proof_records/ART-104_TOOLCHAIN_TREE.proof.json |
| ART-105_PACKAGE_LIB_GRAPH | Build Package/library dependency graph | completed | proof_records/ART-105_PACKAGE_LIB_GRAPH.proof.json |
| ART-112_CODE_OWNERSHIP | Build Code ownership map | completed | proof_records/ART-112_CODE_OWNERSHIP.proof.json |
| ART-113_DEBUG_CODE_MAP | Build Code map for debugging | completed | proof_records/ART-113_DEBUG_CODE_MAP.proof.json |

### W2 - Configuration, infrastructure, and access baseline

- Rollback anchor: `Revert generated W2 artifacts and keep blocked secret paths excluded.`

| task | title | status | proof |
|---|---|---|---|
| ART-114_ENV_CONFIG_MATRIX | Build Environment matrix | completed | proof_records/ART-114_ENV_CONFIG_MATRIX.proof.json |
| ART-115_CONFIG_INVENTORY | Build Configuration inventory | completed | proof_records/ART-115_CONFIG_INVENTORY.proof.json |
| ART-116_INFRA_TOPOLOGY | Build Infrastructure topology map | passed | proof_records/ART-116_INFRA_TOPOLOGY.proof.json |
| ART-117_IAM_MATRIX | Build IAM/security access matrix | completed | proof_records/ART-117_IAM_MATRIX.proof.json |
| ART-118_OBSERVABILITY | Build Observability map | pending | proof_records/ART-118_OBSERVABILITY.proof.json |

### W3 - Runtime, interface, and data contract mapping

- Rollback anchor: `Use task proofs to remove only W3 artifacts and replayable operation records.`

| task | title | status | proof |
|---|---|---|---|
| ART-106_RUNTIME_DEP_MAP | Build Runtime dependency map | pending | proof_records/ART-106_RUNTIME_DEP_MAP.proof.json |
| ART-107_DATA_FLOW_GRAPH | Build Data flow graph | pending | proof_records/ART-107_DATA_FLOW_GRAPH.proof.json |
| ART-108_DB_SCHEMA_MAP | Build Database schema map | pending | proof_records/ART-108_DB_SCHEMA_MAP.proof.json |
| ART-109_DATA_LINEAGE | Build Data lineage map | pending | proof_records/ART-109_DATA_LINEAGE.proof.json |
| ART-110_API_CATALOG | Build API contract catalog | pending | proof_records/ART-110_API_CATALOG.proof.json |
| ART-111_EVENT_MAP | Build Event/message contract map | pending | proof_records/ART-111_EVENT_MAP.proof.json |

### W4 - Governance and business readiness

- Rollback anchor: `Remove W4 generated records together with their proof and ledger entries.`

| task | title | status | proof |
|---|---|---|---|
| ART-119_BUSINESS_PROCESS | Build Business process map | pending | proof_records/ART-119_BUSINESS_PROCESS.proof.json |
| ART-120_WAVE_PLAN | Build Migration wave plan | pending | proof_records/ART-120_WAVE_PLAN.proof.json |
| ART-125_RISK_REGISTER | Build Risk register | completed | proof_records/ART-125_RISK_REGISTER.proof.json |
| ART-126_DECISION_LOG | Build Decision log / ADRs | completed | proof_records/ART-126_DECISION_LOG.proof.json |
| ART-127_BLAST_RADIUS | Build Blast-radius map | completed | proof_records/ART-127_BLAST_RADIUS.proof.json |
| ART-128_READINESS_SCORECARD | Build Migration readiness scorecard | completed | proof_records/ART-128_READINESS_SCORECARD.proof.json |
| ART-129_BUSINESS_CAPABILITY | Build Business capability map | completed | proof_records/ART-129_BUSINESS_CAPABILITY.proof.json |
| ART-135_RACI | Build Ownership/RACI matrix | pending | proof_records/ART-135_RACI.proof.json |

### W5 - Validation, parity, and release evidence

- Rollback anchor: `Preserve raw failure logs, then remove only generated validation artifacts for this package.`

| task | title | status | proof |
|---|---|---|---|
| ART-123_VALIDATION_RECONCILIATION | Build Validation and reconciliation reports | pending | proof_records/ART-123_VALIDATION_RECONCILIATION.proof.json |
| ART-124_TEST_COVERAGE | Build Test coverage matrix | pending | proof_records/ART-124_TEST_COVERAGE.proof.json |
| ART-130_SHADOW_TRAFFIC | Build Shadow traffic comparison report | completed | proof_records/ART-130_SHADOW_TRAFFIC.proof.json |
| ART-131_GOLDEN_DATASET | Build Golden dataset | completed | proof_records/ART-131_GOLDEN_DATASET.proof.json |
| ART-132_PARITY_DASHBOARD | Build Parity dashboard | pending | proof_records/ART-132_PARITY_DASHBOARD.proof.json |
| VER-300_UNIT_VALIDATION | Run unit/integration validation | pending | proof_records/VER-300_UNIT_VALIDATION.proof.json |

### W6 - Cutover, rollback, and decommission controls

- Rollback anchor: `Follow task-specific rollback plans and require human approval for gated control-plane operations.`

| task | title | status | proof |
|---|---|---|---|
| REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS | Implement checkpoints and rollback handles | pending | proof_records/REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS.proof.json |
| REQ-027_ENVCTL_REPLAY_ENGINE | Implement replay/reproduce engine | pending | proof_records/REQ-027_ENVCTL_REPLAY_ENGINE.proof.json |
| REQ-041_TWO_REPO_INTEGRATION | Implement two-repo integration | pending | proof_records/REQ-041_TWO_REPO_INTEGRATION.proof.json |
| REQ-045_RUN_REPLAY | Implement run/replay templates | pending | proof_records/REQ-045_RUN_REPLAY.proof.json |
| ART-121_CUTOVER | Build Cutover checklist | pending | proof_records/ART-121_CUTOVER.proof.json |
| ART-122_ROLLBACK | Build Rollback plan | pending | proof_records/ART-122_ROLLBACK.proof.json |
| ART-133_DEPRECATION_MAP | Build Deprecation map | completed | proof_records/ART-133_DEPRECATION_MAP.proof.json |
| ART-134_EXCEPTION_INVENTORY | Build Exception inventory | pending | proof_records/ART-134_EXCEPTION_INVENTORY.proof.json |
| ART-136_TECH_DEBT_LEDGER | Build Technical debt ledger | pending | proof_records/ART-136_TECH_DEBT_LEDGER.proof.json |
| REL-400_PACKAGE_ARCHIVE | Create upgraded package archive | pending | proof_records/REL-400_PACKAGE_ARCHIVE.proof.json |
| REL-401_HANDOFF | Prepare final handoff | pending | proof_records/REL-401_HANDOFF.proof.json |

## Validation Links

- Depends on `REQ-024_ENVCTL_ARTIFACT_REGISTRY` for content hashes and registry rows.
- Depends on `REQ-040_SHARED_PROTOCOL_SCHEMAS` for shared proof and artifact record compatibility.
- Blocks `VER-300_UNIT_VALIDATION` until this wave plan is registered with validation evidence.

## Source Inputs

- `migration-artifacts/_meta/flexnetos-vs-lifeos.target-descriptor.yaml`
- `generated/flexnetos_target_descriptor_validation_report.json`
- `generated/package_scan.json`
- `generated/envctl_migration_db_model.json`
- `generated/task_graph.csv`
- `generated/envctl_artifact_registry_report.json`
- `generated/shared_protocol_validation_report.json`
- `generated/status_from_proofs.json`
