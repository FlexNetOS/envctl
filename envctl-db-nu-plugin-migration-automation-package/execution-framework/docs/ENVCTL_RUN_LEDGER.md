# envctl migration run ledger

Generated at: `2026-07-04T23:09:30+00:00`
Status: `passed`

## Ledger surfaces

- Run lineage: `envctl_migration_runs` stores target, recipe, artifact contract, status, tool versions, and reproducibility hash.
- Operation state: `envctl_migration_operations` stores idempotent operation rows with status, risk, command hash, input, output, and error fields.
- Event stream: `envctl_migration_run_events` is append-only per run and hash-chained by `(run_id, event_seq)`.
- Proof links: `envctl_migration_evidence` links proof records and generated reports back to a run and operation.

## Runtime smoke

- Run: `run-req022`
- Status: `completed`
- Operations: `1`
- Events: `5`
- Hash chain valid: `True`
- Duplicate event sequence rejected: `True`
- Invalid operation status rejected: `True`

## Timeline

| seq | event | actor | operation | operation status |
|---:|---|---|---|---|
| 1 | `run_created` | `agent` | `` | `` |
| 2 | `operation_started` | `agent` | `op-req022-run-ledger` | `succeeded` |
| 3 | `proof_linked` | `system` | `op-req022-run-ledger` | `succeeded` |
| 4 | `operation_succeeded` | `agent` | `op-req022-run-ledger` | `succeeded` |
| 5 | `run_completed` | `system` | `` | `` |

## Evidence

| kind | uri | sha256 |
|---|---|---|
| `proof_record` | `proof_records/REQ-020_ENVCTL_DB_SCHEMA.proof.json` | `sha256:b3a2108255360c6c2e1316e66c3da76562973ed45581c07132468531f589a8ea` |
| `run_ledger_report` | `generated/envctl_run_ledger_report.json` | `` |
