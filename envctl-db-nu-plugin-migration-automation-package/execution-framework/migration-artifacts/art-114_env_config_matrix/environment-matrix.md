# Environment Matrix

Task: `ART-114_ENV_CONFIG_MATRIX`
Generated at: `2026-07-04T23:21:06+00:00`
Status: `complete`

## Target

- Target: `flexnetos-vs-lifeos` (`mixed`)
- Primary root: `/home/flexnetos/FlexNetOS`
- Compare root: `/home/flexnetos/lifeos`
- Descriptor hash: `sha256:b3f653f9e9cda7991821687f041cb540ce3e4342bde03ef333aa0e72a6b42384`

## Dev / Stage / Prod Differences

| Dimension | Dev | Stage | Prod |
|---|---|---|---|
| Root and source of truth | FlexNetOS workspace primary_root with local execution-framework writes | Same primary_root plus envctl SQLite model for replayable validation state | Production targets are referenced only; mutation is blocked in this phase |
| Configuration inputs | target descriptor, generated task packet, repo/package scan, contract manifest | same inputs plus proof records from registry and shared schema tasks | approved cutover configuration only; not generated or applied by ART-114 |
| Secrets and credentials | blocked path patterns are excluded from reads and artifact content | evidence is linked by hash and metadata; raw secrets remain out of scope | secret material must stay external to artifacts and requires separate controls |
| Risk and approval | R2 maximum automatic risk, approval-gated descriptor | R2 validation lane; higher-risk cutover work belongs to later tasks | production-impacting or destructive work requires human approval |
| Outputs | matrix markdown and JSON under migration-artifacts plus proof/log files | registry rows, validation rows, graph links, and proof ledger evidence | no output is written into production by this task |

## Environment Detail

### dev

- Purpose: Local agent execution, discovery, artifact generation, and deterministic smoke validation.
- Root: `/home/flexnetos/FlexNetOS`
- Network policy: disabled by descriptor unless a later approved task explicitly enables it
- Write policy: workspace-write for execution-framework and generated migration-artifacts only
- Risk ceiling: R2
- Approval mode: approval-gated
- Data policy: No blocked secret paths are read or emitted; credential material is excluded by packet policy.
- Validation: artifact file existence, JSON parse, registry content_hash equality, proof ledger append

### stage

- Purpose: Replay, reconciliation, and validation lane before any production-impacting change.
- Root: `/home/flexnetos/FlexNetOS`
- Network policy: kept disabled in descriptor for this package; external staging integrations require explicit approval
- Write policy: artifact/proof writes plus registry rows in the envctl database model
- Risk ceiling: R2 by descriptor, R3 only when a later approval-gated cutover task declares it
- Approval mode: approval-gated
- Data policy: Use redacted evidence links and hashes, not raw secrets or private keys.
- Validation: shared protocol schema evidence, registry validation links, hash-backed proof record, later VER-300 unit validation

### prod

- Purpose: Production reference and cutover target; this artifact generation task performs no production mutation.
- Root: `external production targets, not written by this task`
- Network policy: no production network writes from ART-114
- Write policy: blocked for production systems in this phase
- Risk ceiling: R5 operations require human approval and are outside this artifact task
- Approval mode: human approval required for destructive or production-impacting operations
- Data policy: Secrets, private keys, .env files, pem/key material, and private_keys paths are blocked from artifact output.
- Validation: production differences are recorded as policy constraints, cutover/rollback tasks must add separate approval evidence, artifact registry stores only file hashes and metadata

## Drift Controls

- Capture dev/stage/prod differences as artifact data before VER-300 validation.
- Register content hashes in envctl_migration_artifacts.
- Link registry evidence to target descriptor, packet, and prerequisite proof records.
- Keep production mutations out of artifact generation.

## Evidence

- `execution-framework/generated/envctl_target_registry.json`
- `execution-framework/generated/package_scan.json`
- `execution-framework/generated/contract_manifest.json`
- `execution-framework/proof_records/REQ-024_ENVCTL_ARTIFACT_REGISTRY.proof.json`
- `execution-framework/proof_records/REQ-040_SHARED_PROTOCOL_SCHEMAS.proof.json`
