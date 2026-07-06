# Golden Dataset

- Task: `ART-131_GOLDEN_DATASET`
- Contract artifact: `artifact:04-data-migration-golden-dataset-md`
- Canonical path: `migration-artifacts/04-data-migration/golden-dataset.md`
- Generated at: `2026-07-04T23:28:06+00:00`
- Purpose: known-good validation sample for downstream unit validation.
- Expected aggregate result: `pass`.

## Dataset Cases

| case id | domain | expected status | validator | evidence |
|---|---|---|---|---|
| GOLDEN-ART131-001 | target_descriptor | pass | target_descriptor_registered | examples/target-descriptors/flexnetos-vs-lifeos.yaml, generated/envctl_target_registry.json |
| GOLDEN-ART131-002 | artifact_registry | pass | artifact_registry_hash_recorded | Downstream unit validation must have a known-good artifact row with a content hash and producer link. |
| GOLDEN-ART131-003 | shared_protocol | pass | shared_protocol_artifact_record_shape | ArtifactRecord, EvidenceRecord, ValidationResult, ProofRecord |
| GOLDEN-ART131-004 | validation_evidence | pass | validation_evidence_linked | The known-good sample preserves the current warning boundary while still requiring reconciled rows and passing command evidence. |
| GOLDEN-ART131-005 | blocked_path_policy | pass | blocked_path_policy_respected | Golden validation needs a positive sample and explicit proof that sensitive path classes stay out of registry evidence. |

## Validation Contract

- Artifact files must exist at the canonical path and task-local companion paths.
- Registry rows must contain SHA-256 content hashes, producer operations, contract ids, and validation links.
- Evidence must reference target descriptor, repo scan, envctl database reports, artifact registry proof, and shared protocol proof.
- Blocked secret and key paths are represented only as policy patterns, not copied as evidence.
