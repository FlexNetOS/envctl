# Decision Log / ADRs

Task: `ART-126_DECISION_LOG`
Generated at: `2026-07-04T23:20:02+00:00`
Status: `complete`
Owner lane: `lane_d_filesystem`
Owner agent: `artifact-agent`

## Source Summary

| source | status | key facts |
|---|---|---|
| Target descriptor registry | `passed` | `5` descriptors across `5` target types |
| Repo scan | `available` | `15` top-level entries, `6` specs, `23` prompts |
| envctl database | `passed` | `16` tables, `6` views, `19` capabilities |
| Artifact registry | `passed` | hashes/provenance/validation links/fail-closed rejection coverage recorded |
| Shared protocol | `passed` | `envctl_nu_plugin_migration_protocol` `1.0.0` with `14` records |
| Contract manifest | `required` | `09-governance-decision-log-md` at `migration-artifacts/09-governance/decision-log.md` |

## Decisions

### ADR-ART126-001 - Use envctl database as durable migration source of truth

- Status: `accepted`
- Owner: `envctl-db-agent`
- Date: `2026-07-04T23:20:02+00:00`
- Decision: Durable migration state is owned by envctl database tables and views; nu_plugin records render and command against that state.
- Rationale: The shared protocol manifest names envctl migration database as the source of truth and validates 14 record contracts for nu_plugin-facing shapes.
- Evidence: `execution-framework/docs/SHARED_PROTOCOL_SCHEMAS.md`, `execution-framework/generated/shared_protocol_manifest.json`, `execution-framework/generated/envctl_migration_db_model.json`
- Consequences:
  - Migration commands must persist auditable state before downstream plugin display is considered complete.
  - Schema changes must preserve the shared protocol compatibility rule or move to a new major version.

### ADR-ART126-002 - Register migration artifacts with hashes, provenance, and validation links

- Status: `accepted`
- Owner: `envctl-db-agent`
- Date: `2026-07-04T23:20:02+00:00`
- Decision: Generated artifacts are registered through the envctl artifact registry with package-relative paths, SHA-256 hashes, producer operations, provenance, graph edges, and validation rows.
- Rationale: REQ-024 passed with persisted paths, hashes, producers, contract ids, provenance, validation links, and fail-closed path rejection coverage.
- Evidence: `execution-framework/docs/ENVCTL_ARTIFACT_REGISTRY.md`, `execution-framework/generated/envctl_artifact_registry_report.json`, `execution-framework/scripts/artifact_registry.py`
- Consequences:
  - Blocked paths such as secrets, private keys, .env files, and key material remain outside artifact registration.
  - Completion gates can compare registry hashes with on-disk artifacts instead of relying on file presence alone.

### ADR-ART126-003 - Keep target descriptors approval-gated and typed

- Status: `accepted`
- Owner: `envctl-target-registry`
- Date: `2026-07-04T23:20:02+00:00`
- Decision: Migration targets are represented as registered descriptors with explicit target type, root path, safety mode, and maximum automatic risk.
- Rationale: The target registry passed with 5 registered descriptors covering 5 target types; the primary target is flexnetos-vs-lifeos at /home/flexnetos/FlexNetOS with safety mode approval-gated.
- Evidence: `execution-framework/generated/envctl_target_registry.json`, `execution-framework/docs/ENVCTL_TARGET_REGISTRY.md`
- Consequences:
  - Artifact generation may read descriptor facts but should not infer target roots from ambient shell state.
  - Higher-risk migration operations must remain approval-gated according to descriptor policy.

### ADR-ART126-004 - Treat the contract manifest as the canonical artifact path map

- Status: `accepted`
- Owner: `artifact-agent`
- Date: `2026-07-04T23:20:02+00:00`
- Decision: Task-local artifacts are generated under migration-artifacts/art-126_decision_log, while the contract-visible governance markdown is mirrored to migration-artifacts/09-governance/decision-log.md.
- Rationale: The execution packet requires art-126 task artifacts, and the contract manifest requires the governance decision log at migration-artifacts/09-governance/decision-log.md with ART-126_DECISION_LOG as producer.
- Evidence: `execution-framework/generated/execution_packets/ART-126_DECISION_LOG.json`, `execution-framework/generated/contract_manifest.json`, `execution-framework/docs/CONTRACT_MANIFEST.md`
- Consequences:
  - Downstream task proofs can find task-local JSON and markdown evidence.
  - Contract completeness checks can find the required governance decision-log path.

### ADR-ART126-005 - Use proof records and heartbeat files as execution completion evidence

- Status: `accepted`
- Owner: `artifact-agent`
- Date: `2026-07-04T23:20:02+00:00`
- Decision: A task is complete only when generated artifacts exist, registry registration reports matching hashes, validation evidence is linked, heartbeat state is updated, and proof_records contains the task proof.
- Rationale: The packet completion gate requires artifact existence, recorded hash, and linked validation evidence; the existing execution framework stores proofs and heartbeats per task.
- Evidence: `execution-framework/generated/execution_packets/ART-126_DECISION_LOG.json`, `execution-framework/proof_templates/PROOF_RECORD_TEMPLATE.json`, `execution-framework/scripts/_common.py`
- Consequences:
  - Validation failures should be reflected in proof status instead of hidden behind generated files.
  - Rollback remains scoped to files added by this task and the proof ledger entry.

### ADR-ART126-006 - Keep filesystem artifact work inside allowed target and execution paths

- Status: `accepted`
- Owner: `lane_d_filesystem`
- Date: `2026-07-04T23:20:02+00:00`
- Decision: Decision-log generation writes only migration-artifacts, execution-framework logs, state, scripts, and proof records allowed by the task packet.
- Rationale: The packet scope permits migration-artifacts and execution-framework paths while blocking environment files, secrets, private keys, PEM files, and key files.
- Evidence: `execution-framework/generated/execution_packets/ART-126_DECISION_LOG.json`, `execution-framework/scripts/artifact_registry.py`
- Consequences:
  - Secret-like paths are not scanned or registered as evidence.
  - The rollback plan can remove only ART-126-added files without touching unrelated package state.

### ADR-ART126-007 - Use reproducible SQLite smoke registration for artifact proof

- Status: `accepted`
- Owner: `envctl-db-agent`
- Date: `2026-07-04T23:20:02+00:00`
- Decision: Artifact proof is generated by applying the package migrations to an in-memory SQLite database, inserting a task run and operation, and registering the generated files through the same ArtifactRegistry path used by REQ-024.
- Rationale: Current envctl database validation already proves the schema with 16 tables, 6 views, and seeded contract rows; using the same registry code gives deterministic proof without requiring a host database mutation.
- Evidence: `execution-framework/generated/envctl_migration_db_model.json`, `execution-framework/scripts/verify_envctl_db_schema.py`, `execution-framework/scripts/artifact_registry.py`
- Consequences:
  - This artifact task can be validated in isolation by VER-300 unit validation.
  - A later persistent envctl integration can replay the same artifact records using the recorded paths and hashes.

## Completion Notes

- This file is mirrored to the contract path `migration-artifacts/09-governance/decision-log.md`.
- The task-local JSON index is `migration-artifacts/art-126_decision_log/decision-log.json`.
- Registry proof records the SHA-256 hash for each generated artifact path.
