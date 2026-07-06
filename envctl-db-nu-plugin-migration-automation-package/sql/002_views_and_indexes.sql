-- envctl migration automation indexes and query views.

CREATE INDEX IF NOT EXISTS idx_envctl_migration_runs_status ON envctl_migration_runs(status);
CREATE INDEX IF NOT EXISTS idx_envctl_migration_runs_target ON envctl_migration_runs(target_id);
CREATE INDEX IF NOT EXISTS idx_envctl_migration_events_run_seq ON envctl_migration_run_events(run_id, event_seq);
CREATE INDEX IF NOT EXISTS idx_envctl_migration_operations_run_status ON envctl_migration_operations(run_id, status);
CREATE INDEX IF NOT EXISTS idx_envctl_migration_artifacts_run_status ON envctl_migration_artifacts(run_id, status);
CREATE INDEX IF NOT EXISTS idx_envctl_migration_approvals_run_status ON envctl_migration_approvals(run_id, status);
CREATE INDEX IF NOT EXISTS idx_envctl_migration_validations_run_status ON envctl_migration_validations(run_id, status);
CREATE INDEX IF NOT EXISTS idx_envctl_migration_graph_run_nodes ON envctl_migration_graph_edges(run_id, from_node, to_node);

CREATE VIEW IF NOT EXISTS envctl_migration_run_latest_status AS
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
JOIN envctl_migration_targets t ON t.id = r.target_id;

CREATE VIEW IF NOT EXISTS envctl_migration_live_timeline AS
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
ORDER BY e.run_id, e.event_seq;

CREATE VIEW IF NOT EXISTS envctl_migration_artifact_index AS
SELECT
  run_id,
  artifact_id,
  title,
  artifact_type,
  status,
  path,
  content_hash,
  updated_at_utc
FROM envctl_migration_artifacts;

CREATE VIEW IF NOT EXISTS envctl_migration_open_approvals AS
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
WHERE a.status = 'open';

CREATE VIEW IF NOT EXISTS envctl_migration_validation_scorecard AS
SELECT
  run_id,
  SUM(CASE WHEN status = 'pass' THEN 1 ELSE 0 END) AS pass_count,
  SUM(CASE WHEN status = 'fail' THEN 1 ELSE 0 END) AS fail_count,
  SUM(CASE WHEN status = 'warn' THEN 1 ELSE 0 END) AS warn_count,
  SUM(CASE WHEN status = 'blocked' THEN 1 ELSE 0 END) AS blocked_count,
  SUM(CASE WHEN status = 'unknown' THEN 1 ELSE 0 END) AS unknown_count
FROM envctl_migration_validations
GROUP BY run_id;

CREATE VIEW IF NOT EXISTS envctl_migration_replay_readiness AS
SELECT
  r.id AS run_id,
  r.status,
  r.reproducibility_hash,
  CASE WHEN r.reproducibility_hash IS NULL THEN 0 ELSE 1 END AS has_reproducibility_hash,
  (SELECT COUNT(*) FROM envctl_migration_evidence ev WHERE ev.run_id = r.id AND ev.sha256 IS NULL) AS evidence_missing_hashes,
  (SELECT COUNT(*) FROM envctl_migration_artifacts ar WHERE ar.run_id = r.id AND ar.content_hash IS NULL) AS artifacts_missing_hashes,
  (SELECT COUNT(*) FROM envctl_migration_approvals a WHERE a.run_id = r.id AND a.status = 'open') AS open_approvals
FROM envctl_migration_runs r;
