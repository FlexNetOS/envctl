# Ownership/RACI Matrix

Generated at: `2026-07-04T23:28:55+00:00`
Status: `complete`
Target: `flexnetos-vs-lifeos` (mixed)

## Coverage

| area | count |
|---|---:|
| domains | 8 |
| unique owners | 10 |
| approval owners | 6 |
| build owners | 7 |
| validate owners | 1 |
| cutover owners | 1 |
| support owners | 5 |

## RACI Matrix

| domain | responsible | accountable | consulted | informed | approval | build | validate | cutover | support |
|---|---|---|---|---|---|---|---|---|---|
| Migration governance and scope control | artifact-agent | migration-operator | envctl-db-agent<br>shared-protocol-agent<br>validation-agent | nu-plugin-agent<br>flexnetos-adapter-agent | migration-operator | artifact-agent | validation-agent | migration-operator | validation-agent |
| envctl migration database and artifact registry | envctl-db-agent | envctl-db-agent | artifact-agent<br>shared-protocol-agent<br>validation-agent | nu-plugin-agent<br>migration-operator | envctl-db-agent | envctl-db-agent | validation-agent | migration-operator | envctl-runner-agent |
| Shared protocol schemas and compatibility | shared-protocol-agent | shared-protocol-agent | envctl-db-agent<br>nu-plugin-agent<br>validation-agent | artifact-agent<br>migration-operator | shared-protocol-agent | shared-protocol-agent | validation-agent | migration-operator | envctl-db-agent |
| nu_plugin operator control surface | nu-plugin-agent | nu-plugin-agent | shared-protocol-agent<br>envctl-db-agent<br>validation-agent | artifact-agent<br>migration-operator | migration-operator | nu-plugin-agent | validation-agent | migration-operator | nu-plugin-visuals-agent |
| Artifact generation and package outputs | artifact-agent | artifact-agent | envctl-db-agent<br>validation-agent<br>security-reproducibility-agent | migration-operator<br>nu-plugin-agent | artifact-agent | artifact-agent | validation-agent | migration-operator | artifact-agent |
| Validation evidence, proof records, and gates | validation-agent | validation-agent | artifact-agent<br>envctl-db-agent<br>shared-protocol-agent | migration-operator | validation-agent | validation-agent | validation-agent | migration-operator | validation-agent |
| Cutover, rollback, and post-cutover support | migration-operator | migration-operator | envctl-runner-agent<br>validation-agent<br>flexnetos-adapter-agent | artifact-agent<br>nu-plugin-agent<br>shared-protocol-agent | migration-operator | envctl-runner-agent | validation-agent | migration-operator | envctl-runner-agent |
| Security, redaction, and reproducibility controls | security-reproducibility-agent | security-reproducibility-agent | envctl-db-agent<br>artifact-agent<br>validation-agent | migration-operator<br>nu-plugin-agent | security-reproducibility-agent | security-reproducibility-agent | validation-agent | migration-operator | envctl-db-agent |

## Decision Rights

| domain | decision right | evidence |
|---|---|---|
| Migration governance and scope control | Approve contract completion, scope changes, and R3+ migration actions. | `execution-framework/docs/CONTRACT_MANIFEST.md`<br>`execution-framework/generated/task_graph.csv`<br>`execution-framework/migration-artifacts/09-governance/decision-log.md` |
| envctl migration database and artifact registry | Own database schema, artifact registration, producer operation linkage, and registry fail-closed behavior. | `execution-framework/docs/ENVCTL_ARTIFACT_REGISTRY.md`<br>`execution-framework/generated/envctl_artifact_registry_report.json`<br>`execution-framework/proof_records/REQ-024_ENVCTL_ARTIFACT_REGISTRY.proof.json` |
| Shared protocol schemas and compatibility | Approve protocol record compatibility and schema changes consumed by envctl and nu_plugin. | `execution-framework/docs/SHARED_PROTOCOL_SCHEMAS.md`<br>`execution-framework/generated/shared_protocol_manifest.json`<br>`execution-framework/proof_records/REQ-040_SHARED_PROTOCOL_SCHEMAS.proof.json` |
| nu_plugin operator control surface | Own operator command shape, approval views, replay views, and human-facing migration status display. | `execution-framework/docs/SHARED_PROTOCOL_SCHEMAS.md`<br>`execution-framework/migration-artifacts/art-117_iam_matrix/iam-security-access-matrix.json` |
| Artifact generation and package outputs | Generate required migration artifacts, register hashes, preserve provenance, and link evidence. | `execution-framework/migration-artifacts/art-112_code_ownership/code_ownership_map.json`<br>`execution-framework/generated/contract_manifest.json`<br>`execution-framework/generated/execution_packets/ART-135_RACI.json` |
| Validation evidence, proof records, and gates | Accept or reject validation evidence for unit validation, proof ledgers, and replay-ready gates. | `execution-framework/schemas/proof_record.schema.json`<br>`execution-framework/proof_records/proof_ledger.jsonl`<br>`execution-framework/docs/ENVCTL_VALIDATION_EVIDENCE.md` |
| Cutover, rollback, and post-cutover support | Own final go/no-go, rollback checkpoints, and operational support handoff. | `execution-framework/generated/task_graph.csv`<br>`execution-framework/migration-artifacts/09-governance/risk-register.md`<br>`execution-framework/migration-artifacts/09-governance/migration-readiness-scorecard.md` |
| Security, redaction, and reproducibility controls | Approve blocked-path policy, redaction posture, evidence hashing, and reproducibility-safe artifact capture. | `execution-framework/docs/SECURITY_REDACTION.md`<br>`execution-framework/generated/security_redaction_validation_report.json`<br>`execution-framework/migration-artifacts/art-117_iam_matrix/iam-security-access-matrix.json` |

## Registry Contract

This artifact is registered as task-local JSON and Markdown plus the canonical governance Markdown. Registry records include SHA-256 hashes, producer operation IDs, contract linkage, provenance, validation links, and graph edges.

