-- envctl migration automation schema, SQLite-compatible baseline.
-- Adapt to envctl's actual migration framework and database backend.

CREATE TABLE IF NOT EXISTS envctl_migration_targets (
  id TEXT PRIMARY KEY,
  target_id TEXT NOT NULL UNIQUE,
  target_type TEXT NOT NULL CHECK (target_type IN ('codebase','data','infrastructure','integration','mixed')),
  primary_root TEXT NOT NULL,
  compare_root TEXT,
  descriptor_json TEXT NOT NULL,
  descriptor_hash TEXT NOT NULL,
  safety_mode TEXT NOT NULL,
  max_auto_risk TEXT NOT NULL,
  created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE TABLE IF NOT EXISTS envctl_migration_packages (
  id TEXT PRIMARY KEY,
  package_name TEXT NOT NULL,
  package_path TEXT NOT NULL,
  package_hash TEXT NOT NULL,
  manifest_json TEXT NOT NULL,
  imported_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE(package_name, package_hash)
);

CREATE TABLE IF NOT EXISTS envctl_migration_artifact_contracts (
  id TEXT PRIMARY KEY,
  contract_name TEXT NOT NULL,
  contract_version TEXT NOT NULL,
  source_package_id TEXT,
  contract_hash TEXT NOT NULL,
  contract_json TEXT NOT NULL,
  created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE(contract_name, contract_version),
  FOREIGN KEY(source_package_id) REFERENCES envctl_migration_packages(id)
);

CREATE TABLE IF NOT EXISTS envctl_migration_recipes (
  id TEXT PRIMARY KEY,
  recipe_name TEXT NOT NULL,
  recipe_version TEXT NOT NULL,
  artifact_contract_id TEXT NOT NULL,
  recipe_hash TEXT NOT NULL,
  recipe_json TEXT NOT NULL,
  created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE(recipe_name, recipe_version),
  FOREIGN KEY(artifact_contract_id) REFERENCES envctl_migration_artifact_contracts(id)
);

CREATE TABLE IF NOT EXISTS envctl_migration_runs (
  id TEXT PRIMARY KEY,
  target_id TEXT NOT NULL,
  recipe_id TEXT NOT NULL,
  artifact_contract_id TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('created','planning','awaiting_approval','running','paused','validating','completed','failed','blocked','cancelled','denied')),
  human_mode TEXT NOT NULL CHECK (human_mode IN ('observer','approval-gated','operator','agent-only')),
  initiated_by TEXT,
  sandbox_policy TEXT,
  approval_policy TEXT,
  tool_versions_json TEXT,
  reproducibility_hash TEXT,
  started_at_utc TEXT,
  completed_at_utc TEXT,
  created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  FOREIGN KEY(target_id) REFERENCES envctl_migration_targets(id),
  FOREIGN KEY(recipe_id) REFERENCES envctl_migration_recipes(id),
  FOREIGN KEY(artifact_contract_id) REFERENCES envctl_migration_artifact_contracts(id)
);

CREATE TABLE IF NOT EXISTS envctl_migration_operations (
  id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  parent_operation_id TEXT,
  operation_type TEXT NOT NULL,
  phase TEXT,
  status TEXT NOT NULL CHECK (status IN ('queued','ready','awaiting_approval','running','succeeded','failed','blocked','denied','cancelled')),
  risk TEXT NOT NULL CHECK (risk IN ('R0','R1','R2','R3','R4','R5')),
  idempotency_key TEXT NOT NULL,
  command_hash TEXT,
  command_redacted TEXT,
  input_json TEXT,
  output_ref TEXT,
  error_json TEXT,
  started_at_utc TEXT,
  completed_at_utc TEXT,
  created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE(run_id, idempotency_key),
  FOREIGN KEY(run_id) REFERENCES envctl_migration_runs(id),
  FOREIGN KEY(parent_operation_id) REFERENCES envctl_migration_operations(id)
);

CREATE TABLE IF NOT EXISTS envctl_migration_run_events (
  id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  event_seq INTEGER NOT NULL,
  event_type TEXT NOT NULL,
  phase TEXT,
  actor_type TEXT NOT NULL CHECK (actor_type IN ('system','agent','human','plugin','external')),
  actor_id TEXT,
  operation_id TEXT,
  payload_json TEXT NOT NULL,
  evidence_refs_json TEXT,
  previous_event_hash TEXT,
  event_hash TEXT,
  created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE(run_id, event_seq),
  FOREIGN KEY(run_id) REFERENCES envctl_migration_runs(id),
  FOREIGN KEY(operation_id) REFERENCES envctl_migration_operations(id)
);

CREATE TABLE IF NOT EXISTS envctl_migration_evidence (
  id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  operation_id TEXT,
  uri TEXT NOT NULL,
  evidence_kind TEXT NOT NULL,
  sha256 TEXT,
  redacted INTEGER NOT NULL DEFAULT 0,
  metadata_json TEXT,
  created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  FOREIGN KEY(run_id) REFERENCES envctl_migration_runs(id),
  FOREIGN KEY(operation_id) REFERENCES envctl_migration_operations(id)
);

CREATE TABLE IF NOT EXISTS envctl_migration_artifacts (
  id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  artifact_id TEXT NOT NULL,
  title TEXT NOT NULL,
  artifact_type TEXT,
  status TEXT NOT NULL CHECK (status IN ('complete','partial','unknown','blocked')),
  path TEXT,
  content_hash TEXT,
  generated_by_operation_id TEXT,
  evidence_json TEXT,
  links_json TEXT,
  created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE(run_id, artifact_id),
  FOREIGN KEY(run_id) REFERENCES envctl_migration_runs(id),
  FOREIGN KEY(generated_by_operation_id) REFERENCES envctl_migration_operations(id)
);

CREATE TABLE IF NOT EXISTS envctl_migration_graph_edges (
  id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  from_node TEXT NOT NULL,
  to_node TEXT NOT NULL,
  edge_type TEXT NOT NULL,
  source_artifact_id TEXT,
  confidence TEXT,
  evidence_json TEXT,
  created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  FOREIGN KEY(run_id) REFERENCES envctl_migration_runs(id)
);

CREATE TABLE IF NOT EXISTS envctl_migration_approvals (
  id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  operation_id TEXT NOT NULL,
  risk TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('open','approved','denied','expired','cancelled')),
  requested_by TEXT,
  decided_by TEXT,
  reason TEXT,
  requested_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  decided_at_utc TEXT,
  FOREIGN KEY(run_id) REFERENCES envctl_migration_runs(id),
  FOREIGN KEY(operation_id) REFERENCES envctl_migration_operations(id)
);

CREATE TABLE IF NOT EXISTS envctl_migration_validations (
  id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  artifact_id TEXT,
  operation_id TEXT,
  validator TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('pass','fail','warn','blocked','unknown')),
  details_json TEXT,
  evidence_json TEXT,
  created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  FOREIGN KEY(run_id) REFERENCES envctl_migration_runs(id),
  FOREIGN KEY(operation_id) REFERENCES envctl_migration_operations(id)
);

CREATE TABLE IF NOT EXISTS envctl_migration_checkpoints (
  id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  operation_id TEXT,
  checkpoint_kind TEXT NOT NULL,
  checkpoint_ref TEXT NOT NULL,
  checkpoint_hash TEXT,
  metadata_json TEXT,
  created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  FOREIGN KEY(run_id) REFERENCES envctl_migration_runs(id),
  FOREIGN KEY(operation_id) REFERENCES envctl_migration_operations(id)
);

CREATE TABLE IF NOT EXISTS envctl_migration_rollbacks (
  id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  operation_id TEXT,
  rollback_type TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('planned','awaiting_approval','running','succeeded','failed','blocked','cancelled')),
  plan_json TEXT NOT NULL,
  result_json TEXT,
  created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  FOREIGN KEY(run_id) REFERENCES envctl_migration_runs(id),
  FOREIGN KEY(operation_id) REFERENCES envctl_migration_operations(id)
);

CREATE TABLE IF NOT EXISTS envctl_migration_agent_sessions (
  id TEXT PRIMARY KEY,
  run_id TEXT,
  agent_name TEXT NOT NULL,
  model_label TEXT,
  authority_level TEXT,
  session_json TEXT,
  created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  FOREIGN KEY(run_id) REFERENCES envctl_migration_runs(id)
);

CREATE TABLE IF NOT EXISTS envctl_migration_plugin_sessions (
  id TEXT PRIMARY KEY,
  run_id TEXT,
  plugin_name TEXT NOT NULL,
  plugin_version TEXT,
  nu_version TEXT,
  human_mode TEXT,
  session_json TEXT,
  created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  FOREIGN KEY(run_id) REFERENCES envctl_migration_runs(id)
);
