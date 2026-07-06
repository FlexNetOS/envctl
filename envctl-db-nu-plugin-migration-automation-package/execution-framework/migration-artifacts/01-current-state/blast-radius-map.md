# Blast-radius map

Generated at: `2026-07-04T23:19:11+00:00`
Task: `ART-127_BLAST_RADIUS`
Contract row: `artifact:01-current-state-blast-radius-map-md`
Canonical path: `migration-artifacts/01-current-state/blast-radius-map.md`

## Coverage

| kind | count |
|---|---:|
| services | 1 |
| tables | 2 |
| queues | 2 |
| apis | 2 |
| credentials | 2 |
| failure modes total | 9 |

## Failure modes

| surface | kind | risk | what breaks | containment |
|---|---|---|---|---|
| `envctl migration database` | service | critical | target descriptor lookup; artifact hash registration; run ledger status; validation scorecard; replay readiness | Stop artifact completion claims, preserve logs, restore schema/seed from package sources, then rerun registry verification. |
| `envctl_migration_artifacts` | table | critical | migration-artifacts hash provenance; contract row satisfaction; artifact index view; downstream verification gates | Keep generated files on disk but mark the task failed until the registry row and content hash are present. |
| `envctl_migration_evidence` | table | high | auditability; proof-to-artifact trace; rollback confidence; human review of generated outputs | Regenerate evidence rows from existing files and hashes before marking completion. |
| `envctl_migration_operations` | queue | high | idempotency tracking; producer operation foreign keys; status stream rendering; replay command selection | Do not enqueue follow-on validations until the producer operation is restored or replayed. |
| `envctl_migration_run_events` | queue | high | live timeline; goal-loop status; audit ordering; replay determinism | Freeze replay claims and rebuild from operation/proof records with hash-chain validation. |
| `scripts/artifact_registry.py` | api | critical | artifact registration; blocked-path enforcement; content-hash verification; graph and validation links | Run fail-closed rejection cases before trusting any newly generated artifact rows. |
| `shared protocol and nu_plugin command surface` | api | high | operator command rendering; status streams; approval decisions; artifact and validation display | Treat envctl database rows as source of truth and regenerate plugin-facing schemas/manifests. |
| `Codex CLI provider credential` | credential | medium | parallel artifact generation; codex exec packet workflow; proof log capture | Do not expose or inspect secret values; verify only command availability and preserve stdout logs. |
| `workspace filesystem write authority` | credential | high | artifact materialization; heartbeat updates; proof ledger append; rollback manifest usefulness | Fail closed with no completion proof until the write scope matches the packet allowed paths. |

## Critical paths

### artifact generation to registry to validation

- Depends on: `credential-filesystem-write-scope, api-artifact-registry-python, service-envctl-migration-db, table-envctl-migration-artifacts, table-envctl-migration-evidence`
- Break effect: Generated files may exist without durable hash/evidence registration, so the artifact cannot gate validation.

### run replay and status stream

- Depends on: `queue-envctl-migration-operations, queue-envctl-migration-run-events, api-nu-plugin-protocol`
- Break effect: Operators lose reliable ordering, replayability, and UI/API status for migration operations.

## Detection commands

- `SELECT artifact_id, path, content_hash FROM envctl_migration_artifact_index`
- `codex exec < generated/execution_packets/ART-127_BLAST_RADIUS.json`
- `proof record evidence list misses generated artifact paths`
- `python3 scripts/verify_artifact_registry.py`
- `python3 scripts/verify_envctl_db_schema.py`
- `python3 scripts/verify_envctl_run_ledger.py`
- `python3 scripts/verify_operation_state_machine.py`
- `python3 scripts/verify_plugin_command_surface.py`
- `python3 scripts/verify_shared_protocol_schemas.py`
- `registry evidence_ids are empty`
- `test writes under migration-artifacts/, execution-framework/state/, execution-framework/proof_records/`
