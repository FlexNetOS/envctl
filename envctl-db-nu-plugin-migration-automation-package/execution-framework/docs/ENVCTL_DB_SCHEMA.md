# envctl migration automation database schema

Generated at: `2026-07-05T05:51:29+00:00`
Status: `passed`

## Applied migrations

- `sql/001_migration_automation_schema.sql` (`f0a8ad6ce7fba6023d8bfb765843fef07710387f9e720f7fe01305ce2d52bb0e`)
- `sql/002_views_and_indexes.sql` (`b9c4fee76504746742695119c84bfbb46dedb1e29da480103f2e6b6c5a83106e`)
- `execution-framework/generated/contract_manifest.seed.sql` (`399c1092118d57f78c7647a2d5c2fbd914c5b2ac97d5e41e4deda44722dc8c06`)

## Capability coverage

| capability | backing object | covered |
|---|---|---|
| target descriptor registry | `envctl_migration_targets` | yes |
| package import registry | `envctl_migration_packages` | yes |
| artifact contract registry | `envctl_migration_artifact_contracts` | yes |
| migration recipe registry | `envctl_migration_recipes` | yes |
| run ledger | `envctl_migration_runs` | yes |
| operation queue | `envctl_migration_operations` | yes |
| append-only event log | `envctl_migration_run_events` | yes |
| evidence store | `envctl_migration_evidence` | yes |
| artifact registry | `envctl_migration_artifacts` | yes |
| link graph | `envctl_migration_graph_edges` | yes |
| approval gate | `envctl_migration_approvals` | yes |
| validation ledger | `envctl_migration_validations` | yes |
| checkpoint registry | `envctl_migration_checkpoints` | yes |
| rollback handles | `envctl_migration_rollbacks` | yes |
| agent sessions | `envctl_migration_agent_sessions` | yes |
| plugin sessions | `envctl_migration_plugin_sessions` | yes |
| live status views | `envctl_migration_run_latest_status` | yes |
| timeline views | `envctl_migration_live_timeline` | yes |
| replay readiness | `envctl_migration_replay_readiness` | yes |

## Tables

| table | columns | foreign keys | indexes | rows after smoke |
|---|---:|---:|---:|---:|
| `envctl_migration_agent_sessions` | 7 | 1 | 1 | 1 |
| `envctl_migration_approvals` | 10 | 2 | 2 | 1 |
| `envctl_migration_artifact_contracts` | 7 | 1 | 2 | 2 |
| `envctl_migration_artifacts` | 13 | 2 | 3 | 1 |
| `envctl_migration_checkpoints` | 8 | 2 | 1 | 1 |
| `envctl_migration_evidence` | 9 | 2 | 1 | 1 |
| `envctl_migration_graph_edges` | 9 | 1 | 2 | 1 |
| `envctl_migration_operations` | 16 | 2 | 3 | 1 |
| `envctl_migration_packages` | 6 | 0 | 2 | 2 |
| `envctl_migration_plugin_sessions` | 8 | 1 | 1 | 1 |
| `envctl_migration_recipes` | 7 | 1 | 2 | 2 |
| `envctl_migration_rollbacks` | 8 | 2 | 1 | 1 |
| `envctl_migration_run_events` | 13 | 2 | 3 | 1 |
| `envctl_migration_runs` | 14 | 3 | 3 | 1 |
| `envctl_migration_targets` | 11 | 0 | 2 | 1 |
| `envctl_migration_validations` | 9 | 2 | 2 | 1 |

## Views

| view | runtime query | sample rows |
|---|---|---:|
| `envctl_migration_artifact_index` | pass | 1 |
| `envctl_migration_live_timeline` | pass | 1 |
| `envctl_migration_open_approvals` | pass | 0 |
| `envctl_migration_replay_readiness` | pass | 1 |
| `envctl_migration_run_latest_status` | pass | 1 |
| `envctl_migration_validation_scorecard` | pass | 1 |

## Runtime smoke

- Run fixture: `run-req020`
- Operation fixture: `op-req020`
- Events inserted: `1`
- Artifacts inserted: `1`
- Validations inserted: `1`
- Invalid risk rejected: `True`
- Foreign key errors: `0`

The smoke fixture covers target registry, package import, artifact contract, recipe, run, operation, event, evidence, artifact, graph edge, approval, validation, checkpoint, rollback, agent session, and plugin session rows.
