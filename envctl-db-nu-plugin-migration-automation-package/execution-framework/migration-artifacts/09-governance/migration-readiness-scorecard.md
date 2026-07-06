# Migration readiness scorecard

Generated at: `2026-07-04T23:19:30+00:00`
Overall score: `75.3`
Readiness band: `conditional`

## Domain scores

| Domain | Score | Status | Evidence |
|---|---:|---|---|
| target_descriptor_scope | 100 | ready | `generated/envctl_target_registry.json` |
| envctl_database_control_plane | 100 | ready | `proof_records/REQ-020_ENVCTL_DB_SCHEMA.proof.json`, `proof_records/REQ-024_ENVCTL_ARTIFACT_REGISTRY.proof.json` |
| shared_protocol_contracts | 100 | ready | `generated/shared_protocol_validation_report.json` |
| artifact_registry_and_hashing | 100 | ready | `generated/envctl_artifact_registry_report.json`, `docs/ENVCTL_ARTIFACT_REGISTRY.md` |
| validation_replay_rollback | 25 | blocked | `generated/status_from_proofs.json` |
| plugin_operator_surface | 80 | conditional | `proof_records/REQ-030_PLUGIN_PROTOCOL_MANIFEST.proof.json`, `proof_records/REQ-031_PLUGIN_COMMAND_SURFACE.proof.json`, `proof_records/REQ-032_PLUGIN_LIVE_VISUALS.proof.json`, `proof_records/REQ-034_PLUGIN_STATUS_STREAMS.proof.json` |
| filesystem_security_hardening | 40 | blocked | `generated/status_from_proofs.json`, `scripts/artifact_registry.py` |
| governance_artifact_readiness | 28 | in_progress | `generated/task_graph.csv`, `generated/contract_manifest.json` |
| package_scan_and_contract_lock | 100 | ready | `generated/package_scan.json`, `generated/contract_manifest.json` |

## Blocking gates

- `REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS`
- `REQ-027_ENVCTL_REPLAY_ENGINE`
- `REQ-028_ENVCTL_AGENT_CONTROL_API`
- `REQ-041_TWO_REPO_INTEGRATION`
- `REQ-043_SECURITY_REDACTION`
- `REQ-045_RUN_REPLAY`
- `REQ-033_PLUGIN_HUMAN_APPROVAL`

## Required next actions

- Complete validation evidence and replay/rollback gates before final migration validation.
- Complete filesystem bounds and security redaction gates before unattended artifact replay.
- Complete human approval plugin support before operator-facing runs above approval threshold.
- Generate remaining governance artifacts so readiness decisions have owners, risks, and exceptions attached.

## Contract mapping

- Contract row: `artifact:09-governance-migration-readiness-scorecard-md`
- Canonical path: `migration-artifacts/09-governance/migration-readiness-scorecard.md`
- Task-scoped MD: `migration-artifacts/art-128_readiness_scorecard/readiness-scorecard.md`
- Task-scoped JSON: `migration-artifacts/art-128_readiness_scorecard/readiness-scorecard.json`

## Interpretation

The current migration posture is conditional: the core database, target registry, artifact registry, and shared protocol schemas are ready enough to produce and register artifacts, but final migration execution should wait for validation evidence, replay, rollback, filesystem bounds, redaction, and human approval gates.
