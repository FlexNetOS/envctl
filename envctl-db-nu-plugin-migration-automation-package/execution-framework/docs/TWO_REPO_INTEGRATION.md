# Two-Repo Integration

- Task: `REQ-041_TWO_REPO_INTEGRATION`
- Goal: Wire envctl database commands and nu_plugin commands through shared protocol and executable task packets
- Generated at: 2026-07-05T05:18:43+00:00
- Verification status: passed

## Verified flow

1. envctl run-ledger fixture creates migration run, operations, approvals, events, evidence, artifact, graph, and validation rows.
2. Shared protocol records are emitted from the fixture and validated against `schemas/shared_protocol.schema.json`.
3. Plugin contracts project those records into `codedb envctl status stream` and `codedb envctl human approvals` table shapes.
4. The task proof, report, and status ledger are written back into the execution framework.

## Contract inputs

- Human approval surface: `generated/REQ-033_PLUGIN_HUMAN_APPROVAL.contract.json`
- Status stream surface: `generated/REQ-034_PLUGIN_STATUS_STREAMS.contract.json`
- Command manifest: `generated/nu_plugin_command_manifest.json`

## Coverage

- Shared protocol record counts: `{"ApprovalDecision": 1, "ApprovalRequest": 1, "ArtifactRecord": 1, "EvidenceRecord": 3, "GraphEdge": 1, "MigrationRecipe": 1, "MigrationRun": 1, "Operation": 2, "ProofRecord": 1, "ReplayRequest": 1, "ReplayResult": 1, "RunEvent": 5, "TargetDescriptor": 1, "ValidationResult": 1}`
- Status rows emitted: `6`
- Approval rows emitted: `1`
