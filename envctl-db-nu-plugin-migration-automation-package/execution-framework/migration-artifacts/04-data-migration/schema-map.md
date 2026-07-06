# ART-108 Database Schema Map

Generated: `2026-07-04T23:28:33+00:00`
Backend: `sqlite`
Runtime: `python sqlite3 in-memory`

## Summary

- Tables: `16`
- Columns: `155`
- Indexes: `31`
- Foreign keys: `24`
- Constraints: `128`
- Views: `6`
- Triggers: `0`
- Procedures: `0`

## Source Inputs

- `sql/001_migration_automation_schema.sql` (`f0a8ad6ce7fba6023d8bfb765843fef07710387f9e720f7fe01305ce2d52bb0e`)
- `sql/002_views_and_indexes.sql` (`b9c4fee76504746742695119c84bfbb46dedb1e29da480103f2e6b6c5a83106e`)
- `execution-framework/generated/contract_manifest.seed.sql` (`399c1092118d57f78c7647a2d5c2fbd914c5b2ac97d5e41e4deda44722dc8c06`)

## Tables

| Table | Columns | Indexes | Foreign keys | Constraints |
| --- | --- | --- | --- | --- |
| `envctl_migration_agent_sessions` | 7 | 1 | 1 | 4 |
| `envctl_migration_approvals` | 10 | 2 | 2 | 8 |
| `envctl_migration_artifact_contracts` | 7 | 2 | 1 | 8 |
| `envctl_migration_artifacts` | 13 | 3 | 2 | 10 |
| `envctl_migration_checkpoints` | 8 | 1 | 2 | 8 |
| `envctl_migration_evidence` | 9 | 1 | 2 | 8 |
| `envctl_migration_graph_edges` | 9 | 2 | 1 | 7 |
| `envctl_migration_operations` | 16 | 3 | 2 | 10 |
| `envctl_migration_packages` | 6 | 2 | 0 | 7 |
| `envctl_migration_plugin_sessions` | 8 | 1 | 1 | 4 |
| `envctl_migration_recipes` | 7 | 2 | 1 | 9 |
| `envctl_migration_rollbacks` | 8 | 1 | 2 | 8 |
| `envctl_migration_run_events` | 13 | 3 | 2 | 10 |
| `envctl_migration_runs` | 14 | 3 | 3 | 10 |
| `envctl_migration_targets` | 11 | 2 | 0 | 10 |
| `envctl_migration_validations` | 9 | 2 | 2 | 7 |

### `envctl_migration_agent_sessions`

Columns:

| Column | Type | Nullable | Default | PK |
| --- | --- | --- | --- | --- |
| `id` | `TEXT` | yes |  | 1 |
| `run_id` | `TEXT` | yes |  |  |
| `agent_name` | `TEXT` | no |  |  |
| `model_label` | `TEXT` | yes |  |  |
| `authority_level` | `TEXT` | yes |  |  |
| `session_json` | `TEXT` | yes |  |  |
| `created_at_utc` | `TEXT` | no | `strftime('%Y-%m-%dT%H:%M:%fZ','now')` |  |

Indexes:

| Index | Unique | Origin | Columns |
| --- | --- | --- | --- |
| `sqlite_autoindex_envctl_migration_agent_sessions_1` | yes | pk | `id` |

Foreign keys:

| From | References | On update | On delete |
| --- | --- | --- | --- |
| `run_id` | `envctl_migration_runs.id` | NO ACTION | NO ACTION |

Constraints:

| Type | Column | Expression |
| --- | --- | --- |
| primary_key | `id` | `id TEXT PRIMARY KEY` |
| not_null | `agent_name` | `agent_name TEXT NOT NULL` |
| not_null | `created_at_utc` | `created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))` |
| foreign_key |  | `FOREIGN KEY(run_id) REFERENCES envctl_migration_runs(id)` |

### `envctl_migration_approvals`

Columns:

| Column | Type | Nullable | Default | PK |
| --- | --- | --- | --- | --- |
| `id` | `TEXT` | yes |  | 1 |
| `run_id` | `TEXT` | no |  |  |
| `operation_id` | `TEXT` | no |  |  |
| `risk` | `TEXT` | no |  |  |
| `status` | `TEXT` | no |  |  |
| `requested_by` | `TEXT` | yes |  |  |
| `decided_by` | `TEXT` | yes |  |  |
| `reason` | `TEXT` | yes |  |  |
| `requested_at_utc` | `TEXT` | no | `strftime('%Y-%m-%dT%H:%M:%fZ','now')` |  |
| `decided_at_utc` | `TEXT` | yes |  |  |

Indexes:

| Index | Unique | Origin | Columns |
| --- | --- | --- | --- |
| `idx_envctl_migration_approvals_run_status` | no | c | `run_id`, `status` |
| `sqlite_autoindex_envctl_migration_approvals_1` | yes | pk | `id` |

Foreign keys:

| From | References | On update | On delete |
| --- | --- | --- | --- |
| `operation_id` | `envctl_migration_operations.id` | NO ACTION | NO ACTION |
| `run_id` | `envctl_migration_runs.id` | NO ACTION | NO ACTION |

Constraints:

| Type | Column | Expression |
| --- | --- | --- |
| primary_key | `id` | `id TEXT PRIMARY KEY` |
| not_null | `run_id` | `run_id TEXT NOT NULL` |
| not_null | `operation_id` | `operation_id TEXT NOT NULL` |
| not_null | `risk` | `risk TEXT NOT NULL` |
| check | `status` | `status TEXT NOT NULL CHECK (status IN ('open','approved','denied','expired','cancelled'))` |
| not_null | `requested_at_utc` | `requested_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))` |
| foreign_key |  | `FOREIGN KEY(run_id) REFERENCES envctl_migration_runs(id)` |
| foreign_key |  | `FOREIGN KEY(operation_id) REFERENCES envctl_migration_operations(id)` |

### `envctl_migration_artifact_contracts`

Columns:

| Column | Type | Nullable | Default | PK |
| --- | --- | --- | --- | --- |
| `id` | `TEXT` | yes |  | 1 |
| `contract_name` | `TEXT` | no |  |  |
| `contract_version` | `TEXT` | no |  |  |
| `source_package_id` | `TEXT` | yes |  |  |
| `contract_hash` | `TEXT` | no |  |  |
| `contract_json` | `TEXT` | no |  |  |
| `created_at_utc` | `TEXT` | no | `strftime('%Y-%m-%dT%H:%M:%fZ','now')` |  |

Indexes:

| Index | Unique | Origin | Columns |
| --- | --- | --- | --- |
| `sqlite_autoindex_envctl_migration_artifact_contracts_2` | yes | u | `contract_name`, `contract_version` |
| `sqlite_autoindex_envctl_migration_artifact_contracts_1` | yes | pk | `id` |

Foreign keys:

| From | References | On update | On delete |
| --- | --- | --- | --- |
| `source_package_id` | `envctl_migration_packages.id` | NO ACTION | NO ACTION |

Constraints:

| Type | Column | Expression |
| --- | --- | --- |
| primary_key | `id` | `id TEXT PRIMARY KEY` |
| not_null | `contract_name` | `contract_name TEXT NOT NULL` |
| not_null | `contract_version` | `contract_version TEXT NOT NULL` |
| not_null | `contract_hash` | `contract_hash TEXT NOT NULL` |
| not_null | `contract_json` | `contract_json TEXT NOT NULL` |
| not_null | `created_at_utc` | `created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))` |
| unique |  | `UNIQUE(contract_name, contract_version)` |
| foreign_key |  | `FOREIGN KEY(source_package_id) REFERENCES envctl_migration_packages(id)` |

### `envctl_migration_artifacts`

Columns:

| Column | Type | Nullable | Default | PK |
| --- | --- | --- | --- | --- |
| `id` | `TEXT` | yes |  | 1 |
| `run_id` | `TEXT` | no |  |  |
| `artifact_id` | `TEXT` | no |  |  |
| `title` | `TEXT` | no |  |  |
| `artifact_type` | `TEXT` | yes |  |  |
| `status` | `TEXT` | no |  |  |
| `path` | `TEXT` | yes |  |  |
| `content_hash` | `TEXT` | yes |  |  |
| `generated_by_operation_id` | `TEXT` | yes |  |  |
| `evidence_json` | `TEXT` | yes |  |  |
| `links_json` | `TEXT` | yes |  |  |
| `created_at_utc` | `TEXT` | no | `strftime('%Y-%m-%dT%H:%M:%fZ','now')` |  |
| `updated_at_utc` | `TEXT` | no | `strftime('%Y-%m-%dT%H:%M:%fZ','now')` |  |

Indexes:

| Index | Unique | Origin | Columns |
| --- | --- | --- | --- |
| `idx_envctl_migration_artifacts_run_status` | no | c | `run_id`, `status` |
| `sqlite_autoindex_envctl_migration_artifacts_2` | yes | u | `run_id`, `artifact_id` |
| `sqlite_autoindex_envctl_migration_artifacts_1` | yes | pk | `id` |

Foreign keys:

| From | References | On update | On delete |
| --- | --- | --- | --- |
| `generated_by_operation_id` | `envctl_migration_operations.id` | NO ACTION | NO ACTION |
| `run_id` | `envctl_migration_runs.id` | NO ACTION | NO ACTION |

Constraints:

| Type | Column | Expression |
| --- | --- | --- |
| primary_key | `id` | `id TEXT PRIMARY KEY` |
| not_null | `run_id` | `run_id TEXT NOT NULL` |
| not_null | `artifact_id` | `artifact_id TEXT NOT NULL` |
| not_null | `title` | `title TEXT NOT NULL` |
| check | `status` | `status TEXT NOT NULL CHECK (status IN ('complete','partial','unknown','blocked'))` |
| not_null | `created_at_utc` | `created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))` |
| not_null | `updated_at_utc` | `updated_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))` |
| unique |  | `UNIQUE(run_id, artifact_id)` |
| foreign_key |  | `FOREIGN KEY(run_id) REFERENCES envctl_migration_runs(id)` |
| foreign_key |  | `FOREIGN KEY(generated_by_operation_id) REFERENCES envctl_migration_operations(id)` |

### `envctl_migration_checkpoints`

Columns:

| Column | Type | Nullable | Default | PK |
| --- | --- | --- | --- | --- |
| `id` | `TEXT` | yes |  | 1 |
| `run_id` | `TEXT` | no |  |  |
| `operation_id` | `TEXT` | yes |  |  |
| `checkpoint_kind` | `TEXT` | no |  |  |
| `checkpoint_ref` | `TEXT` | no |  |  |
| `checkpoint_hash` | `TEXT` | yes |  |  |
| `metadata_json` | `TEXT` | yes |  |  |
| `created_at_utc` | `TEXT` | no | `strftime('%Y-%m-%dT%H:%M:%fZ','now')` |  |

Indexes:

| Index | Unique | Origin | Columns |
| --- | --- | --- | --- |
| `sqlite_autoindex_envctl_migration_checkpoints_1` | yes | pk | `id` |

Foreign keys:

| From | References | On update | On delete |
| --- | --- | --- | --- |
| `operation_id` | `envctl_migration_operations.id` | NO ACTION | NO ACTION |
| `run_id` | `envctl_migration_runs.id` | NO ACTION | NO ACTION |

Constraints:

| Type | Column | Expression |
| --- | --- | --- |
| primary_key | `id` | `id TEXT PRIMARY KEY` |
| not_null | `run_id` | `run_id TEXT NOT NULL` |
| check |  | `checkpoint_kind TEXT NOT NULL` |
| check |  | `checkpoint_ref TEXT NOT NULL` |
| check |  | `checkpoint_hash TEXT` |
| not_null | `created_at_utc` | `created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))` |
| foreign_key |  | `FOREIGN KEY(run_id) REFERENCES envctl_migration_runs(id)` |
| foreign_key |  | `FOREIGN KEY(operation_id) REFERENCES envctl_migration_operations(id)` |

### `envctl_migration_evidence`

Columns:

| Column | Type | Nullable | Default | PK |
| --- | --- | --- | --- | --- |
| `id` | `TEXT` | yes |  | 1 |
| `run_id` | `TEXT` | no |  |  |
| `operation_id` | `TEXT` | yes |  |  |
| `uri` | `TEXT` | no |  |  |
| `evidence_kind` | `TEXT` | no |  |  |
| `sha256` | `TEXT` | yes |  |  |
| `redacted` | `INTEGER` | no | `0` |  |
| `metadata_json` | `TEXT` | yes |  |  |
| `created_at_utc` | `TEXT` | no | `strftime('%Y-%m-%dT%H:%M:%fZ','now')` |  |

Indexes:

| Index | Unique | Origin | Columns |
| --- | --- | --- | --- |
| `sqlite_autoindex_envctl_migration_evidence_1` | yes | pk | `id` |

Foreign keys:

| From | References | On update | On delete |
| --- | --- | --- | --- |
| `operation_id` | `envctl_migration_operations.id` | NO ACTION | NO ACTION |
| `run_id` | `envctl_migration_runs.id` | NO ACTION | NO ACTION |

Constraints:

| Type | Column | Expression |
| --- | --- | --- |
| primary_key | `id` | `id TEXT PRIMARY KEY` |
| not_null | `run_id` | `run_id TEXT NOT NULL` |
| not_null | `uri` | `uri TEXT NOT NULL` |
| not_null | `evidence_kind` | `evidence_kind TEXT NOT NULL` |
| not_null | `redacted` | `redacted INTEGER NOT NULL DEFAULT 0` |
| not_null | `created_at_utc` | `created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))` |
| foreign_key |  | `FOREIGN KEY(run_id) REFERENCES envctl_migration_runs(id)` |
| foreign_key |  | `FOREIGN KEY(operation_id) REFERENCES envctl_migration_operations(id)` |

### `envctl_migration_graph_edges`

Columns:

| Column | Type | Nullable | Default | PK |
| --- | --- | --- | --- | --- |
| `id` | `TEXT` | yes |  | 1 |
| `run_id` | `TEXT` | no |  |  |
| `from_node` | `TEXT` | no |  |  |
| `to_node` | `TEXT` | no |  |  |
| `edge_type` | `TEXT` | no |  |  |
| `source_artifact_id` | `TEXT` | yes |  |  |
| `confidence` | `TEXT` | yes |  |  |
| `evidence_json` | `TEXT` | yes |  |  |
| `created_at_utc` | `TEXT` | no | `strftime('%Y-%m-%dT%H:%M:%fZ','now')` |  |

Indexes:

| Index | Unique | Origin | Columns |
| --- | --- | --- | --- |
| `idx_envctl_migration_graph_run_nodes` | no | c | `run_id`, `from_node`, `to_node` |
| `sqlite_autoindex_envctl_migration_graph_edges_1` | yes | pk | `id` |

Foreign keys:

| From | References | On update | On delete |
| --- | --- | --- | --- |
| `run_id` | `envctl_migration_runs.id` | NO ACTION | NO ACTION |

Constraints:

| Type | Column | Expression |
| --- | --- | --- |
| primary_key | `id` | `id TEXT PRIMARY KEY` |
| not_null | `run_id` | `run_id TEXT NOT NULL` |
| not_null | `from_node` | `from_node TEXT NOT NULL` |
| not_null | `to_node` | `to_node TEXT NOT NULL` |
| not_null | `edge_type` | `edge_type TEXT NOT NULL` |
| not_null | `created_at_utc` | `created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))` |
| foreign_key |  | `FOREIGN KEY(run_id) REFERENCES envctl_migration_runs(id)` |

### `envctl_migration_operations`

Columns:

| Column | Type | Nullable | Default | PK |
| --- | --- | --- | --- | --- |
| `id` | `TEXT` | yes |  | 1 |
| `run_id` | `TEXT` | no |  |  |
| `parent_operation_id` | `TEXT` | yes |  |  |
| `operation_type` | `TEXT` | no |  |  |
| `phase` | `TEXT` | yes |  |  |
| `status` | `TEXT` | no |  |  |
| `risk` | `TEXT` | no |  |  |
| `idempotency_key` | `TEXT` | no |  |  |
| `command_hash` | `TEXT` | yes |  |  |
| `command_redacted` | `TEXT` | yes |  |  |
| `input_json` | `TEXT` | yes |  |  |
| `output_ref` | `TEXT` | yes |  |  |
| `error_json` | `TEXT` | yes |  |  |
| `started_at_utc` | `TEXT` | yes |  |  |
| `completed_at_utc` | `TEXT` | yes |  |  |
| `created_at_utc` | `TEXT` | no | `strftime('%Y-%m-%dT%H:%M:%fZ','now')` |  |

Indexes:

| Index | Unique | Origin | Columns |
| --- | --- | --- | --- |
| `idx_envctl_migration_operations_run_status` | no | c | `run_id`, `status` |
| `sqlite_autoindex_envctl_migration_operations_2` | yes | u | `run_id`, `idempotency_key` |
| `sqlite_autoindex_envctl_migration_operations_1` | yes | pk | `id` |

Foreign keys:

| From | References | On update | On delete |
| --- | --- | --- | --- |
| `parent_operation_id` | `envctl_migration_operations.id` | NO ACTION | NO ACTION |
| `run_id` | `envctl_migration_runs.id` | NO ACTION | NO ACTION |

Constraints:

| Type | Column | Expression |
| --- | --- | --- |
| primary_key | `id` | `id TEXT PRIMARY KEY` |
| not_null | `run_id` | `run_id TEXT NOT NULL` |
| not_null | `operation_type` | `operation_type TEXT NOT NULL` |
| check | `status` | `status TEXT NOT NULL CHECK (status IN ('queued','ready','awaiting_approval','running','succeeded','failed','blocked','denied','cancelled'))` |
| check | `risk` | `risk TEXT NOT NULL CHECK (risk IN ('R0','R1','R2','R3','R4','R5'))` |
| not_null | `idempotency_key` | `idempotency_key TEXT NOT NULL` |
| not_null | `created_at_utc` | `created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))` |
| unique |  | `UNIQUE(run_id, idempotency_key)` |
| foreign_key |  | `FOREIGN KEY(run_id) REFERENCES envctl_migration_runs(id)` |
| foreign_key |  | `FOREIGN KEY(parent_operation_id) REFERENCES envctl_migration_operations(id)` |

### `envctl_migration_packages`

Columns:

| Column | Type | Nullable | Default | PK |
| --- | --- | --- | --- | --- |
| `id` | `TEXT` | yes |  | 1 |
| `package_name` | `TEXT` | no |  |  |
| `package_path` | `TEXT` | no |  |  |
| `package_hash` | `TEXT` | no |  |  |
| `manifest_json` | `TEXT` | no |  |  |
| `imported_at_utc` | `TEXT` | no | `strftime('%Y-%m-%dT%H:%M:%fZ','now')` |  |

Indexes:

| Index | Unique | Origin | Columns |
| --- | --- | --- | --- |
| `sqlite_autoindex_envctl_migration_packages_2` | yes | u | `package_name`, `package_hash` |
| `sqlite_autoindex_envctl_migration_packages_1` | yes | pk | `id` |

Foreign keys:

No foreign keys recorded.

Constraints:

| Type | Column | Expression |
| --- | --- | --- |
| primary_key | `id` | `id TEXT PRIMARY KEY` |
| not_null | `package_name` | `package_name TEXT NOT NULL` |
| not_null | `package_path` | `package_path TEXT NOT NULL` |
| not_null | `package_hash` | `package_hash TEXT NOT NULL` |
| not_null | `manifest_json` | `manifest_json TEXT NOT NULL` |
| not_null | `imported_at_utc` | `imported_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))` |
| unique |  | `UNIQUE(package_name, package_hash)` |

### `envctl_migration_plugin_sessions`

Columns:

| Column | Type | Nullable | Default | PK |
| --- | --- | --- | --- | --- |
| `id` | `TEXT` | yes |  | 1 |
| `run_id` | `TEXT` | yes |  |  |
| `plugin_name` | `TEXT` | no |  |  |
| `plugin_version` | `TEXT` | yes |  |  |
| `nu_version` | `TEXT` | yes |  |  |
| `human_mode` | `TEXT` | yes |  |  |
| `session_json` | `TEXT` | yes |  |  |
| `created_at_utc` | `TEXT` | no | `strftime('%Y-%m-%dT%H:%M:%fZ','now')` |  |

Indexes:

| Index | Unique | Origin | Columns |
| --- | --- | --- | --- |
| `sqlite_autoindex_envctl_migration_plugin_sessions_1` | yes | pk | `id` |

Foreign keys:

| From | References | On update | On delete |
| --- | --- | --- | --- |
| `run_id` | `envctl_migration_runs.id` | NO ACTION | NO ACTION |

Constraints:

| Type | Column | Expression |
| --- | --- | --- |
| primary_key | `id` | `id TEXT PRIMARY KEY` |
| not_null | `plugin_name` | `plugin_name TEXT NOT NULL` |
| not_null | `created_at_utc` | `created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))` |
| foreign_key |  | `FOREIGN KEY(run_id) REFERENCES envctl_migration_runs(id)` |

### `envctl_migration_recipes`

Columns:

| Column | Type | Nullable | Default | PK |
| --- | --- | --- | --- | --- |
| `id` | `TEXT` | yes |  | 1 |
| `recipe_name` | `TEXT` | no |  |  |
| `recipe_version` | `TEXT` | no |  |  |
| `artifact_contract_id` | `TEXT` | no |  |  |
| `recipe_hash` | `TEXT` | no |  |  |
| `recipe_json` | `TEXT` | no |  |  |
| `created_at_utc` | `TEXT` | no | `strftime('%Y-%m-%dT%H:%M:%fZ','now')` |  |

Indexes:

| Index | Unique | Origin | Columns |
| --- | --- | --- | --- |
| `sqlite_autoindex_envctl_migration_recipes_2` | yes | u | `recipe_name`, `recipe_version` |
| `sqlite_autoindex_envctl_migration_recipes_1` | yes | pk | `id` |

Foreign keys:

| From | References | On update | On delete |
| --- | --- | --- | --- |
| `artifact_contract_id` | `envctl_migration_artifact_contracts.id` | NO ACTION | NO ACTION |

Constraints:

| Type | Column | Expression |
| --- | --- | --- |
| primary_key | `id` | `id TEXT PRIMARY KEY` |
| not_null | `recipe_name` | `recipe_name TEXT NOT NULL` |
| not_null | `recipe_version` | `recipe_version TEXT NOT NULL` |
| not_null | `artifact_contract_id` | `artifact_contract_id TEXT NOT NULL` |
| not_null | `recipe_hash` | `recipe_hash TEXT NOT NULL` |
| not_null | `recipe_json` | `recipe_json TEXT NOT NULL` |
| not_null | `created_at_utc` | `created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))` |
| unique |  | `UNIQUE(recipe_name, recipe_version)` |
| foreign_key |  | `FOREIGN KEY(artifact_contract_id) REFERENCES envctl_migration_artifact_contracts(id)` |

### `envctl_migration_rollbacks`

Columns:

| Column | Type | Nullable | Default | PK |
| --- | --- | --- | --- | --- |
| `id` | `TEXT` | yes |  | 1 |
| `run_id` | `TEXT` | no |  |  |
| `operation_id` | `TEXT` | yes |  |  |
| `rollback_type` | `TEXT` | no |  |  |
| `status` | `TEXT` | no |  |  |
| `plan_json` | `TEXT` | no |  |  |
| `result_json` | `TEXT` | yes |  |  |
| `created_at_utc` | `TEXT` | no | `strftime('%Y-%m-%dT%H:%M:%fZ','now')` |  |

Indexes:

| Index | Unique | Origin | Columns |
| --- | --- | --- | --- |
| `sqlite_autoindex_envctl_migration_rollbacks_1` | yes | pk | `id` |

Foreign keys:

| From | References | On update | On delete |
| --- | --- | --- | --- |
| `operation_id` | `envctl_migration_operations.id` | NO ACTION | NO ACTION |
| `run_id` | `envctl_migration_runs.id` | NO ACTION | NO ACTION |

Constraints:

| Type | Column | Expression |
| --- | --- | --- |
| primary_key | `id` | `id TEXT PRIMARY KEY` |
| not_null | `run_id` | `run_id TEXT NOT NULL` |
| not_null | `rollback_type` | `rollback_type TEXT NOT NULL` |
| check | `status` | `status TEXT NOT NULL CHECK (status IN ('planned','awaiting_approval','running','succeeded','failed','blocked','cancelled'))` |
| not_null | `plan_json` | `plan_json TEXT NOT NULL` |
| not_null | `created_at_utc` | `created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))` |
| foreign_key |  | `FOREIGN KEY(run_id) REFERENCES envctl_migration_runs(id)` |
| foreign_key |  | `FOREIGN KEY(operation_id) REFERENCES envctl_migration_operations(id)` |

### `envctl_migration_run_events`

Columns:

| Column | Type | Nullable | Default | PK |
| --- | --- | --- | --- | --- |
| `id` | `TEXT` | yes |  | 1 |
| `run_id` | `TEXT` | no |  |  |
| `event_seq` | `INTEGER` | no |  |  |
| `event_type` | `TEXT` | no |  |  |
| `phase` | `TEXT` | yes |  |  |
| `actor_type` | `TEXT` | no |  |  |
| `actor_id` | `TEXT` | yes |  |  |
| `operation_id` | `TEXT` | yes |  |  |
| `payload_json` | `TEXT` | no |  |  |
| `evidence_refs_json` | `TEXT` | yes |  |  |
| `previous_event_hash` | `TEXT` | yes |  |  |
| `event_hash` | `TEXT` | yes |  |  |
| `created_at_utc` | `TEXT` | no | `strftime('%Y-%m-%dT%H:%M:%fZ','now')` |  |

Indexes:

| Index | Unique | Origin | Columns |
| --- | --- | --- | --- |
| `idx_envctl_migration_events_run_seq` | no | c | `run_id`, `event_seq` |
| `sqlite_autoindex_envctl_migration_run_events_2` | yes | u | `run_id`, `event_seq` |
| `sqlite_autoindex_envctl_migration_run_events_1` | yes | pk | `id` |

Foreign keys:

| From | References | On update | On delete |
| --- | --- | --- | --- |
| `operation_id` | `envctl_migration_operations.id` | NO ACTION | NO ACTION |
| `run_id` | `envctl_migration_runs.id` | NO ACTION | NO ACTION |

Constraints:

| Type | Column | Expression |
| --- | --- | --- |
| primary_key | `id` | `id TEXT PRIMARY KEY` |
| not_null | `run_id` | `run_id TEXT NOT NULL` |
| not_null | `event_seq` | `event_seq INTEGER NOT NULL` |
| not_null | `event_type` | `event_type TEXT NOT NULL` |
| check | `actor_type` | `actor_type TEXT NOT NULL CHECK (actor_type IN ('system','agent','human','plugin','external'))` |
| not_null | `payload_json` | `payload_json TEXT NOT NULL` |
| not_null | `created_at_utc` | `created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))` |
| unique |  | `UNIQUE(run_id, event_seq)` |
| foreign_key |  | `FOREIGN KEY(run_id) REFERENCES envctl_migration_runs(id)` |
| foreign_key |  | `FOREIGN KEY(operation_id) REFERENCES envctl_migration_operations(id)` |

### `envctl_migration_runs`

Columns:

| Column | Type | Nullable | Default | PK |
| --- | --- | --- | --- | --- |
| `id` | `TEXT` | yes |  | 1 |
| `target_id` | `TEXT` | no |  |  |
| `recipe_id` | `TEXT` | no |  |  |
| `artifact_contract_id` | `TEXT` | no |  |  |
| `status` | `TEXT` | no |  |  |
| `human_mode` | `TEXT` | no |  |  |
| `initiated_by` | `TEXT` | yes |  |  |
| `sandbox_policy` | `TEXT` | yes |  |  |
| `approval_policy` | `TEXT` | yes |  |  |
| `tool_versions_json` | `TEXT` | yes |  |  |
| `reproducibility_hash` | `TEXT` | yes |  |  |
| `started_at_utc` | `TEXT` | yes |  |  |
| `completed_at_utc` | `TEXT` | yes |  |  |
| `created_at_utc` | `TEXT` | no | `strftime('%Y-%m-%dT%H:%M:%fZ','now')` |  |

Indexes:

| Index | Unique | Origin | Columns |
| --- | --- | --- | --- |
| `idx_envctl_migration_runs_target` | no | c | `target_id` |
| `idx_envctl_migration_runs_status` | no | c | `status` |
| `sqlite_autoindex_envctl_migration_runs_1` | yes | pk | `id` |

Foreign keys:

| From | References | On update | On delete |
| --- | --- | --- | --- |
| `artifact_contract_id` | `envctl_migration_artifact_contracts.id` | NO ACTION | NO ACTION |
| `recipe_id` | `envctl_migration_recipes.id` | NO ACTION | NO ACTION |
| `target_id` | `envctl_migration_targets.id` | NO ACTION | NO ACTION |

Constraints:

| Type | Column | Expression |
| --- | --- | --- |
| primary_key | `id` | `id TEXT PRIMARY KEY` |
| not_null | `target_id` | `target_id TEXT NOT NULL` |
| not_null | `recipe_id` | `recipe_id TEXT NOT NULL` |
| not_null | `artifact_contract_id` | `artifact_contract_id TEXT NOT NULL` |
| check | `status` | `status TEXT NOT NULL CHECK (status IN ('created','planning','awaiting_approval','running','paused','validating','completed','failed','blocked','cancelled','denied'))` |
| check | `human_mode` | `human_mode TEXT NOT NULL CHECK (human_mode IN ('observer','approval-gated','operator','agent-only'))` |
| not_null | `created_at_utc` | `created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))` |
| foreign_key |  | `FOREIGN KEY(target_id) REFERENCES envctl_migration_targets(id)` |
| foreign_key |  | `FOREIGN KEY(recipe_id) REFERENCES envctl_migration_recipes(id)` |
| foreign_key |  | `FOREIGN KEY(artifact_contract_id) REFERENCES envctl_migration_artifact_contracts(id)` |

### `envctl_migration_targets`

Columns:

| Column | Type | Nullable | Default | PK |
| --- | --- | --- | --- | --- |
| `id` | `TEXT` | yes |  | 1 |
| `target_id` | `TEXT` | no |  |  |
| `target_type` | `TEXT` | no |  |  |
| `primary_root` | `TEXT` | no |  |  |
| `compare_root` | `TEXT` | yes |  |  |
| `descriptor_json` | `TEXT` | no |  |  |
| `descriptor_hash` | `TEXT` | no |  |  |
| `safety_mode` | `TEXT` | no |  |  |
| `max_auto_risk` | `TEXT` | no |  |  |
| `created_at_utc` | `TEXT` | no | `strftime('%Y-%m-%dT%H:%M:%fZ','now')` |  |
| `updated_at_utc` | `TEXT` | no | `strftime('%Y-%m-%dT%H:%M:%fZ','now')` |  |

Indexes:

| Index | Unique | Origin | Columns |
| --- | --- | --- | --- |
| `sqlite_autoindex_envctl_migration_targets_2` | yes | u | `target_id` |
| `sqlite_autoindex_envctl_migration_targets_1` | yes | pk | `id` |

Foreign keys:

No foreign keys recorded.

Constraints:

| Type | Column | Expression |
| --- | --- | --- |
| primary_key | `id` | `id TEXT PRIMARY KEY` |
| unique | `target_id` | `target_id TEXT NOT NULL UNIQUE` |
| check | `target_type` | `target_type TEXT NOT NULL CHECK (target_type IN ('codebase','data','infrastructure','integration','mixed'))` |
| not_null |  | `primary_root TEXT NOT NULL` |
| not_null | `descriptor_json` | `descriptor_json TEXT NOT NULL` |
| not_null | `descriptor_hash` | `descriptor_hash TEXT NOT NULL` |
| not_null | `safety_mode` | `safety_mode TEXT NOT NULL` |
| not_null | `max_auto_risk` | `max_auto_risk TEXT NOT NULL` |
| not_null | `created_at_utc` | `created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))` |
| not_null | `updated_at_utc` | `updated_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))` |

### `envctl_migration_validations`

Columns:

| Column | Type | Nullable | Default | PK |
| --- | --- | --- | --- | --- |
| `id` | `TEXT` | yes |  | 1 |
| `run_id` | `TEXT` | no |  |  |
| `artifact_id` | `TEXT` | yes |  |  |
| `operation_id` | `TEXT` | yes |  |  |
| `validator` | `TEXT` | no |  |  |
| `status` | `TEXT` | no |  |  |
| `details_json` | `TEXT` | yes |  |  |
| `evidence_json` | `TEXT` | yes |  |  |
| `created_at_utc` | `TEXT` | no | `strftime('%Y-%m-%dT%H:%M:%fZ','now')` |  |

Indexes:

| Index | Unique | Origin | Columns |
| --- | --- | --- | --- |
| `idx_envctl_migration_validations_run_status` | no | c | `run_id`, `status` |
| `sqlite_autoindex_envctl_migration_validations_1` | yes | pk | `id` |

Foreign keys:

| From | References | On update | On delete |
| --- | --- | --- | --- |
| `operation_id` | `envctl_migration_operations.id` | NO ACTION | NO ACTION |
| `run_id` | `envctl_migration_runs.id` | NO ACTION | NO ACTION |

Constraints:

| Type | Column | Expression |
| --- | --- | --- |
| primary_key | `id` | `id TEXT PRIMARY KEY` |
| not_null | `run_id` | `run_id TEXT NOT NULL` |
| not_null | `validator` | `validator TEXT NOT NULL` |
| check | `status` | `status TEXT NOT NULL CHECK (status IN ('pass','fail','warn','blocked','unknown'))` |
| not_null | `created_at_utc` | `created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))` |
| foreign_key |  | `FOREIGN KEY(run_id) REFERENCES envctl_migration_runs(id)` |
| foreign_key |  | `FOREIGN KEY(operation_id) REFERENCES envctl_migration_operations(id)` |

## Views

| View | Columns |
| --- | --- |
| `envctl_migration_artifact_index` | `run_id`, `artifact_id`, `title`, `artifact_type`, `status`, `path`, `content_hash`, `updated_at_utc` |
| `envctl_migration_live_timeline` | `run_id`, `event_seq`, `created_at_utc`, `phase`, `event_type`, `actor_type`, `actor_id`, `operation_id`, `operation_type`, `operation_status`, `payload_json` |
| `envctl_migration_open_approvals` | `approval_id`, `run_id`, `operation_id`, `operation_type`, `risk`, `status`, `requested_by`, `reason`, `requested_at_utc` |
| `envctl_migration_replay_readiness` | `run_id`, `status`, `reproducibility_hash`, `has_reproducibility_hash`, `evidence_missing_hashes`, `artifacts_missing_hashes`, `open_approvals` |
| `envctl_migration_run_latest_status` | `run_id`, `target_id`, `target_type`, `status`, `human_mode`, `started_at_utc`, `completed_at_utc`, `operation_count`, `failed_operation_count`, `open_approval_count`, `artifact_count`, `last_event_at_utc` |
| `envctl_migration_validation_scorecard` | `run_id`, `pass_count`, `fail_count`, `warn_count`, `blocked_count`, `unknown_count` |

### `envctl_migration_artifact_index`

```sql
CREATE VIEW envctl_migration_artifact_index AS
SELECT
  run_id,
  artifact_id,
  title,
  artifact_type,
  status,
  path,
  content_hash,
  updated_at_utc
FROM envctl_migration_artifacts
```

### `envctl_migration_live_timeline`

```sql
CREATE VIEW envctl_migration_live_timeline AS
SELECT
  e.run_id,
  e.event_seq,
  e.created_at_utc,
  e.phase,
  e.event_type,
  e.actor_type,
  e.actor_id,
  e.operation_id,
  o.operation_type,
  o.status AS operation_status,
  e.payload_json
FROM envctl_migration_run_events e
LEFT JOIN envctl_migration_operations o ON o.id = e.operation_id
ORDER BY e.run_id, e.event_seq
```

### `envctl_migration_open_approvals`

```sql
CREATE VIEW envctl_migration_open_approvals AS
SELECT
  a.id AS approval_id,
  a.run_id,
  a.operation_id,
  o.operation_type,
  a.risk,
  a.status,
  a.requested_by,
  a.reason,
  a.requested_at_utc
FROM envctl_migration_approvals a
JOIN envctl_migration_operations o ON o.id = a.operation_id
WHERE a.status = 'open'
```

### `envctl_migration_replay_readiness`

```sql
CREATE VIEW envctl_migration_replay_readiness AS
SELECT
  r.id AS run_id,
  r.status,
  r.reproducibility_hash,
  CASE WHEN r.reproducibility_hash IS NULL THEN 0 ELSE 1 END AS has_reproducibility_hash,
  (SELECT COUNT(*) FROM envctl_migration_evidence ev WHERE ev.run_id = r.id AND ev.sha256 IS NULL) AS evidence_missing_hashes,
  (SELECT COUNT(*) FROM envctl_migration_artifacts ar WHERE ar.run_id = r.id AND ar.content_hash IS NULL) AS artifacts_missing_hashes,
  (SELECT COUNT(*) FROM envctl_migration_approvals a WHERE a.run_id = r.id AND a.status = 'open') AS open_approvals
FROM envctl_migration_runs r
```

### `envctl_migration_run_latest_status`

```sql
CREATE VIEW envctl_migration_run_latest_status AS
SELECT
  r.id AS run_id,
  t.target_id,
  t.target_type,
  r.status,
  r.human_mode,
  r.started_at_utc,
  r.completed_at_utc,
  (SELECT COUNT(*) FROM envctl_migration_operations o WHERE o.run_id = r.id) AS operation_count,
  (SELECT COUNT(*) FROM envctl_migration_operations o WHERE o.run_id = r.id AND o.status = 'failed') AS failed_operation_count,
  (SELECT COUNT(*) FROM envctl_migration_approvals a WHERE a.run_id = r.id AND a.status = 'open') AS open_approval_count,
  (SELECT COUNT(*) FROM envctl_migration_artifacts ar WHERE ar.run_id = r.id) AS artifact_count,
  (SELECT MAX(e.created_at_utc) FROM envctl_migration_run_events e WHERE e.run_id = r.id) AS last_event_at_utc
FROM envctl_migration_runs r
JOIN envctl_migration_targets t ON t.id = r.target_id
```

### `envctl_migration_validation_scorecard`

```sql
CREATE VIEW envctl_migration_validation_scorecard AS
SELECT
  run_id,
  SUM(CASE WHEN status = 'pass' THEN 1 ELSE 0 END) AS pass_count,
  SUM(CASE WHEN status = 'fail' THEN 1 ELSE 0 END) AS fail_count,
  SUM(CASE WHEN status = 'warn' THEN 1 ELSE 0 END) AS warn_count,
  SUM(CASE WHEN status = 'blocked' THEN 1 ELSE 0 END) AS blocked_count,
  SUM(CASE WHEN status = 'unknown' THEN 1 ELSE 0 END) AS unknown_count
FROM envctl_migration_validations
GROUP BY run_id
```

## Triggers

No envctl migration triggers are present.

## Procedures

SQLite has no stored-procedure catalog for this schema; no procedures are present.

## Protocol Context

- REQ-020 schema verification: `passed`
- Artifact registry verification: `passed`
- Shared protocol records: `14`
