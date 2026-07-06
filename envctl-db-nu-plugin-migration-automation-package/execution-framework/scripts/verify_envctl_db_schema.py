from __future__ import annotations

import json
import sqlite3
from pathlib import Path

from _common import append_proof, make_proof, now, package_root, root, sha256_file


TASK_ID = "REQ-020_ENVCTL_DB_SCHEMA"

MIGRATION_FILES = [
    "sql/001_migration_automation_schema.sql",
    "sql/002_views_and_indexes.sql",
    "execution-framework/generated/contract_manifest.seed.sql",
]

REQUIRED_TABLES = {
    "envctl_migration_targets": {
        "id",
        "target_id",
        "target_type",
        "primary_root",
        "descriptor_json",
        "descriptor_hash",
        "safety_mode",
        "max_auto_risk",
    },
    "envctl_migration_packages": {
        "id",
        "package_name",
        "package_path",
        "package_hash",
        "manifest_json",
    },
    "envctl_migration_artifact_contracts": {
        "id",
        "contract_name",
        "contract_version",
        "source_package_id",
        "contract_hash",
        "contract_json",
    },
    "envctl_migration_recipes": {
        "id",
        "recipe_name",
        "recipe_version",
        "artifact_contract_id",
        "recipe_hash",
        "recipe_json",
    },
    "envctl_migration_runs": {
        "id",
        "target_id",
        "recipe_id",
        "artifact_contract_id",
        "status",
        "human_mode",
        "reproducibility_hash",
    },
    "envctl_migration_operations": {
        "id",
        "run_id",
        "parent_operation_id",
        "operation_type",
        "status",
        "risk",
        "idempotency_key",
        "command_hash",
        "command_redacted",
    },
    "envctl_migration_run_events": {
        "id",
        "run_id",
        "event_seq",
        "event_type",
        "actor_type",
        "operation_id",
        "payload_json",
        "previous_event_hash",
        "event_hash",
    },
    "envctl_migration_evidence": {
        "id",
        "run_id",
        "operation_id",
        "uri",
        "evidence_kind",
        "sha256",
        "redacted",
    },
    "envctl_migration_artifacts": {
        "id",
        "run_id",
        "artifact_id",
        "title",
        "status",
        "path",
        "content_hash",
        "generated_by_operation_id",
    },
    "envctl_migration_graph_edges": {
        "id",
        "run_id",
        "from_node",
        "to_node",
        "edge_type",
        "source_artifact_id",
    },
    "envctl_migration_approvals": {
        "id",
        "run_id",
        "operation_id",
        "risk",
        "status",
        "requested_by",
        "decided_by",
    },
    "envctl_migration_validations": {
        "id",
        "run_id",
        "artifact_id",
        "operation_id",
        "validator",
        "status",
    },
    "envctl_migration_checkpoints": {
        "id",
        "run_id",
        "operation_id",
        "checkpoint_kind",
        "checkpoint_ref",
        "checkpoint_hash",
    },
    "envctl_migration_rollbacks": {
        "id",
        "run_id",
        "operation_id",
        "rollback_type",
        "status",
        "plan_json",
    },
    "envctl_migration_agent_sessions": {
        "id",
        "run_id",
        "agent_name",
        "model_label",
        "authority_level",
        "session_json",
    },
    "envctl_migration_plugin_sessions": {
        "id",
        "run_id",
        "plugin_name",
        "plugin_version",
        "nu_version",
        "human_mode",
        "session_json",
    },
}

REQUIRED_VIEWS = {
    "envctl_migration_run_latest_status",
    "envctl_migration_live_timeline",
    "envctl_migration_artifact_index",
    "envctl_migration_open_approvals",
    "envctl_migration_validation_scorecard",
    "envctl_migration_replay_readiness",
}

CAPABILITY_MAP = {
    "target descriptor registry": "envctl_migration_targets",
    "package import registry": "envctl_migration_packages",
    "artifact contract registry": "envctl_migration_artifact_contracts",
    "migration recipe registry": "envctl_migration_recipes",
    "run ledger": "envctl_migration_runs",
    "operation queue": "envctl_migration_operations",
    "append-only event log": "envctl_migration_run_events",
    "evidence store": "envctl_migration_evidence",
    "artifact registry": "envctl_migration_artifacts",
    "link graph": "envctl_migration_graph_edges",
    "approval gate": "envctl_migration_approvals",
    "validation ledger": "envctl_migration_validations",
    "checkpoint registry": "envctl_migration_checkpoints",
    "rollback handles": "envctl_migration_rollbacks",
    "agent sessions": "envctl_migration_agent_sessions",
    "plugin sessions": "envctl_migration_plugin_sessions",
    "live status views": "envctl_migration_run_latest_status",
    "timeline views": "envctl_migration_live_timeline",
    "replay readiness": "envctl_migration_replay_readiness",
}


def read_sql(base: Path, relpath: str) -> str:
    path = base / relpath
    if not path.exists():
        raise FileNotFoundError(path)
    return path.read_text(encoding="utf-8")


def apply_migrations(conn: sqlite3.Connection, base: Path) -> list[dict]:
    applied = []
    for relpath in MIGRATION_FILES:
        sql = read_sql(base, relpath)
        conn.executescript(sql)
        applied.append(
            {
                "path": relpath,
                "sha256": sha256_file(base / relpath),
                "bytes": len(sql.encode("utf-8")),
            }
        )
    return applied


def table_columns(conn: sqlite3.Connection, table: str) -> list[dict]:
    return [
        {
            "cid": row[0],
            "name": row[1],
            "type": row[2],
            "not_null": bool(row[3]),
            "default": row[4],
            "primary_key": bool(row[5]),
        }
        for row in conn.execute(f"PRAGMA table_info({table})")
    ]


def foreign_keys(conn: sqlite3.Connection, table: str) -> list[dict]:
    return [
        {
            "id": row[0],
            "seq": row[1],
            "table": row[2],
            "from": row[3],
            "to": row[4],
            "on_update": row[5],
            "on_delete": row[6],
            "match": row[7],
        }
        for row in conn.execute(f"PRAGMA foreign_key_list({table})")
    ]


def indexes(conn: sqlite3.Connection, table: str) -> list[dict]:
    result = []
    for row in conn.execute(f"PRAGMA index_list({table})"):
        index_name = row[1]
        result.append(
            {
                "name": index_name,
                "unique": bool(row[2]),
                "origin": row[3],
                "partial": bool(row[4]),
                "columns": [col[2] for col in conn.execute(f"PRAGMA index_info({index_name})")],
            }
        )
    return result


def sqlite_objects(conn: sqlite3.Connection, kind: str) -> list[str]:
    return [
        row[0]
        for row in conn.execute(
            """
            SELECT name
            FROM sqlite_master
            WHERE type = ? AND name LIKE 'envctl_migration_%'
            ORDER BY name
            """,
            (kind,),
        )
    ]


def insert_lifecycle_fixture(conn: sqlite3.Connection) -> dict:
    conn.execute(
        """
        INSERT INTO envctl_migration_targets
          (id, target_id, target_type, primary_root, compare_root, descriptor_json,
           descriptor_hash, safety_mode, max_auto_risk)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            "target-req020",
            "flexnetos-vs-lifeos",
            "mixed",
            "/workspace/flexnetos",
            "/workspace/lifeos",
            '{"schema_version":1,"target":"flexnetos-vs-lifeos"}',
            "sha256:target-req020",
            "approval-gated",
            "R3",
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_packages
          (id, package_name, package_path, package_hash, manifest_json)
        VALUES (?, ?, ?, ?, ?)
        """,
        (
            "pkg-req020",
            "req020-fixture",
            "source/req020-fixture",
            "sha256:pkg-req020",
            '{"schema_version":1}',
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_artifact_contracts
          (id, contract_name, contract_version, source_package_id, contract_hash, contract_json)
        VALUES (?, ?, ?, ?, ?, ?)
        """,
        (
            "contract-req020",
            "req020-contract",
            "1.0.0",
            "pkg-req020",
            "sha256:contract-req020",
            '{"rows":[]}',
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_recipes
          (id, recipe_name, recipe_version, artifact_contract_id, recipe_hash, recipe_json)
        VALUES (?, ?, ?, ?, ?, ?)
        """,
        (
            "recipe-req020",
            "req020-recipe",
            "1.0.0",
            "contract-req020",
            "sha256:recipe-req020",
            '{"phases":[]}',
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_runs
          (id, target_id, recipe_id, artifact_contract_id, status, human_mode,
           initiated_by, sandbox_policy, approval_policy, tool_versions_json,
           reproducibility_hash, started_at_utc)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            "run-req020",
            "target-req020",
            "recipe-req020",
            "contract-req020",
            "running",
            "approval-gated",
            "envctl-db-agent",
            "workspace-write",
            "never",
            '{"python":"3.14","sqlite":"3.46"}',
            "sha256:run-req020",
            "2026-07-04T23:00:00Z",
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_operations
          (id, run_id, operation_type, phase, status, risk, idempotency_key,
           command_hash, command_redacted, input_json)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            "op-req020",
            "run-req020",
            "apply_schema",
            "02-envctl-db",
            "succeeded",
            "R1",
            "REQ-020/apply-schema",
            "sha256:command-req020",
            "python3 scripts/verify_envctl_db_schema.py",
            '{"task_id":"REQ-020_ENVCTL_DB_SCHEMA"}',
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_run_events
          (id, run_id, event_seq, event_type, phase, actor_type, actor_id,
           operation_id, payload_json, evidence_refs_json, previous_event_hash, event_hash)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            "event-req020-1",
            "run-req020",
            1,
            "operation_succeeded",
            "02-envctl-db",
            "agent",
            "envctl-db-agent",
            "op-req020",
            '{"status":"succeeded"}',
            '["generated/envctl_migration_db_model.json"]',
            None,
            "sha256:event-req020-1",
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_evidence
          (id, run_id, operation_id, uri, evidence_kind, sha256, redacted, metadata_json)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            "evidence-req020",
            "run-req020",
            "op-req020",
            "generated/envctl_migration_db_model.json",
            "schema_introspection",
            "sha256:evidence-req020",
            0,
            '{"runtime":"sqlite-memory"}',
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_artifacts
          (id, run_id, artifact_id, title, artifact_type, status, path,
           content_hash, generated_by_operation_id, evidence_json, links_json)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            "artifact-req020",
            "run-req020",
            "envctl-db-schema-model",
            "envctl DB schema model",
            "database_model",
            "complete",
            "generated/envctl_migration_db_model.json",
            "sha256:artifact-req020",
            "op-req020",
            '["evidence-req020"]',
            '["docs/ENVCTL_DB_SCHEMA.md"]',
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_graph_edges
          (id, run_id, from_node, to_node, edge_type, source_artifact_id, confidence, evidence_json)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            "edge-req020",
            "run-req020",
            "target-req020",
            "artifact-req020",
            "produces",
            "envctl-db-schema-model",
            "high",
            '["evidence-req020"]',
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_approvals
          (id, run_id, operation_id, risk, status, requested_by, decided_by, reason)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            "approval-req020",
            "run-req020",
            "op-req020",
            "R1",
            "approved",
            "envctl-db-agent",
            "policy",
            "schema-only smoke fixture",
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_validations
          (id, run_id, artifact_id, operation_id, validator, status, details_json, evidence_json)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            "validation-req020",
            "run-req020",
            "envctl-db-schema-model",
            "op-req020",
            "verify_envctl_db_schema.py",
            "pass",
            '{"required_tables":16}',
            '["generated/envctl_migration_db_model.json"]',
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_checkpoints
          (id, run_id, operation_id, checkpoint_kind, checkpoint_ref, checkpoint_hash, metadata_json)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        """,
        (
            "checkpoint-req020",
            "run-req020",
            "op-req020",
            "schema_before_apply",
            "history/pre_execution_framework_manifest.json",
            "sha256:checkpoint-req020",
            '{"rollback_scope":"generated-only"}',
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_rollbacks
          (id, run_id, operation_id, rollback_type, status, plan_json, result_json)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        """,
        (
            "rollback-req020",
            "run-req020",
            "op-req020",
            "remove_generated_artifacts",
            "planned",
            '{"remove":["generated/envctl_migration_db_model.json"]}',
            None,
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_agent_sessions
          (id, run_id, agent_name, model_label, authority_level, session_json)
        VALUES (?, ?, ?, ?, ?, ?)
        """,
        (
            "agent-session-req020",
            "run-req020",
            "envctl-db-agent",
            "gpt-5.3-spark",
            "repo",
            '{"task_id":"REQ-020_ENVCTL_DB_SCHEMA"}',
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_plugin_sessions
          (id, run_id, plugin_name, plugin_version, nu_version, human_mode, session_json)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        """,
        (
            "plugin-session-req020",
            "run-req020",
            "nu_plugin_envctl_migration",
            "0.1.0",
            "0.105",
            "approval-gated",
            '{"surface":"status-stream"}',
        ),
    )
    conn.commit()
    return {
        "run_id": "run-req020",
        "operation_id": "op-req020",
        "event_count": conn.execute(
            "SELECT COUNT(*) FROM envctl_migration_run_events WHERE run_id = ?",
            ("run-req020",),
        ).fetchone()[0],
        "artifact_count": conn.execute(
            "SELECT COUNT(*) FROM envctl_migration_artifacts WHERE run_id = ?",
            ("run-req020",),
        ).fetchone()[0],
        "validation_count": conn.execute(
            "SELECT COUNT(*) FROM envctl_migration_validations WHERE run_id = ?",
            ("run-req020",),
        ).fetchone()[0],
    }


def assert_check_constraint(conn: sqlite3.Connection) -> dict:
    try:
        conn.execute(
            """
            INSERT INTO envctl_migration_operations
              (id, run_id, operation_type, status, risk, idempotency_key)
            VALUES (?, ?, ?, ?, ?, ?)
            """,
            (
                "op-invalid-risk",
                "run-req020",
                "invalid",
                "queued",
                "R9",
                "REQ-020/invalid-risk",
            ),
        )
    except sqlite3.IntegrityError as exc:
        return {"invalid_operation_risk_rejected": True, "error": str(exc)}
    raise AssertionError("operation risk CHECK constraint accepted invalid value R9")


def query_views(conn: sqlite3.Connection) -> dict:
    result = {}
    for view in sorted(REQUIRED_VIEWS):
        rows = conn.execute(f"SELECT * FROM {view} LIMIT 5").fetchall()
        result[view] = {"query_ok": True, "sample_row_count": len(rows)}
    return result


def build_model(conn: sqlite3.Connection, applied: list[dict], smoke: dict, constraint_check: dict) -> dict:
    tables = sqlite_objects(conn, "table")
    views = sqlite_objects(conn, "view")
    table_model = {}
    for table in tables:
        table_model[table] = {
            "columns": table_columns(conn, table),
            "foreign_keys": foreign_keys(conn, table),
            "indexes": indexes(conn, table),
            "row_count": conn.execute(f"SELECT COUNT(*) FROM {table}").fetchone()[0],
        }

    missing_tables = sorted(set(REQUIRED_TABLES) - set(tables))
    missing_columns = {
        table: sorted(columns - {col["name"] for col in table_model.get(table, {}).get("columns", [])})
        for table, columns in REQUIRED_TABLES.items()
    }
    missing_columns = {table: cols for table, cols in missing_columns.items() if cols}
    missing_views = sorted(REQUIRED_VIEWS - set(views))
    capability_coverage = {
        capability: {
            "object": obj,
            "covered": obj in tables or obj in views,
        }
        for capability, obj in CAPABILITY_MAP.items()
    }
    foreign_key_errors = [
        {
            "table": row[0],
            "rowid": row[1],
            "parent": row[2],
            "fkid": row[3],
        }
        for row in conn.execute("PRAGMA foreign_key_check")
    ]
    errors = []
    if missing_tables:
        errors.append(f"missing required tables: {', '.join(missing_tables)}")
    if missing_columns:
        errors.append(f"missing required columns: {json.dumps(missing_columns, sort_keys=True)}")
    if missing_views:
        errors.append(f"missing required views: {', '.join(missing_views)}")
    uncovered = [name for name, item in capability_coverage.items() if not item["covered"]]
    if uncovered:
        errors.append(f"uncovered capabilities: {', '.join(uncovered)}")
    if foreign_key_errors:
        errors.append(f"foreign key errors: {foreign_key_errors}")

    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "generated_at": now(),
        "status": "passed" if not errors else "failed",
        "database_backend": "sqlite",
        "runtime": "python sqlite3 in-memory",
        "applied_migrations": applied,
        "summary": {
            "required_table_count": len(REQUIRED_TABLES),
            "actual_table_count": len(tables),
            "required_view_count": len(REQUIRED_VIEWS),
            "actual_view_count": len(views),
            "capability_count": len(CAPABILITY_MAP),
            "index_count": sum(len(item["indexes"]) for item in table_model.values()),
            "foreign_key_count": sum(len(item["foreign_keys"]) for item in table_model.values()),
        },
        "errors": errors,
        "required_tables": sorted(REQUIRED_TABLES),
        "required_views": sorted(REQUIRED_VIEWS),
        "capability_coverage": capability_coverage,
        "tables": table_model,
        "views": {view: {"sql": conn.execute("SELECT sql FROM sqlite_master WHERE type = 'view' AND name = ?", (view,)).fetchone()[0]} for view in views},
        "view_runtime_queries": query_views(conn),
        "lifecycle_smoke": smoke,
        "constraint_check": constraint_check,
        "foreign_key_check": foreign_key_errors,
    }


def write_docs(model: dict) -> None:
    lines = [
        "# envctl migration automation database schema",
        "",
        f"Generated at: `{model['generated_at']}`",
        f"Status: `{model['status']}`",
        "",
        "## Applied migrations",
        "",
    ]
    for item in model["applied_migrations"]:
        lines.append(f"- `{item['path']}` (`{item['sha256']}`)")
    lines.extend(
        [
            "",
            "## Capability coverage",
            "",
            "| capability | backing object | covered |",
            "|---|---|---|",
        ]
    )
    for capability, item in model["capability_coverage"].items():
        covered = "yes" if item["covered"] else "no"
        lines.append(f"| {capability} | `{item['object']}` | {covered} |")
    lines.extend(
        [
            "",
            "## Tables",
            "",
            "| table | columns | foreign keys | indexes | rows after smoke |",
            "|---|---:|---:|---:|---:|",
        ]
    )
    for table in model["required_tables"]:
        item = model["tables"][table]
        lines.append(
            f"| `{table}` | {len(item['columns'])} | {len(item['foreign_keys'])} | "
            f"{len(item['indexes'])} | {item['row_count']} |"
        )
    lines.extend(
        [
            "",
            "## Views",
            "",
            "| view | runtime query | sample rows |",
            "|---|---|---:|",
        ]
    )
    for view in model["required_views"]:
        result = model["view_runtime_queries"][view]
        lines.append(f"| `{view}` | {'pass' if result['query_ok'] else 'fail'} | {result['sample_row_count']} |")
    lines.extend(
        [
            "",
            "## Runtime smoke",
            "",
            f"- Run fixture: `{model['lifecycle_smoke']['run_id']}`",
            f"- Operation fixture: `{model['lifecycle_smoke']['operation_id']}`",
            f"- Events inserted: `{model['lifecycle_smoke']['event_count']}`",
            f"- Artifacts inserted: `{model['lifecycle_smoke']['artifact_count']}`",
            f"- Validations inserted: `{model['lifecycle_smoke']['validation_count']}`",
            f"- Invalid risk rejected: `{model['constraint_check']['invalid_operation_risk_rejected']}`",
            f"- Foreign key errors: `{len(model['foreign_key_check'])}`",
            "",
            "The smoke fixture covers target registry, package import, artifact contract, recipe, run, operation, event, evidence, artifact, graph edge, approval, validation, checkpoint, rollback, agent session, and plugin session rows.",
        ]
    )
    (root() / "docs" / "ENVCTL_DB_SCHEMA.md").write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> None:
    base = package_root()
    conn = sqlite3.connect(":memory:")
    conn.execute("PRAGMA foreign_keys = ON")
    applied = apply_migrations(conn, base)
    smoke = insert_lifecycle_fixture(conn)
    constraint_check = assert_check_constraint(conn)
    model = build_model(conn, applied, smoke, constraint_check)

    model_path = root() / "generated" / "envctl_migration_db_model.json"
    report_path = root() / "generated" / "envctl_migration_db_validation_report.json"
    log_path = root() / "logs" / f"{TASK_ID}.log"
    model_path.write_text(json.dumps(model, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    report = {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "status": model["status"],
        "generated_at": model["generated_at"],
        "summary": model["summary"],
        "errors": model["errors"],
        "evidence": [
            "generated/envctl_migration_db_model.json",
            "generated/envctl_migration_db_validation_report.json",
            "docs/ENVCTL_DB_SCHEMA.md",
        ],
    }
    report_path.write_text(json.dumps(report, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    write_docs(model)
    log_path.write_text(json.dumps(report, indent=2, sort_keys=False) + "\n", encoding="utf-8")

    files_changed = [
        "execution-framework/scripts/verify_envctl_db_schema.py",
        "execution-framework/generated/envctl_migration_db_model.json",
        "execution-framework/generated/envctl_migration_db_validation_report.json",
        "execution-framework/docs/ENVCTL_DB_SCHEMA.md",
        "execution-framework/logs/REQ-020_ENVCTL_DB_SCHEMA.log",
        "execution-framework/proof_records/REQ-020_ENVCTL_DB_SCHEMA.proof.json",
        "execution-framework/proof_records/proof_ledger.jsonl",
    ]
    commands_run = [
        "python3 scripts/verify_envctl_db_schema.py",
    ]
    proof = make_proof(
        TASK_ID,
        "completed" if model["status"] == "passed" else "failed",
        "codex-cli-local",
        "helper-envctl-db-01",
        "gpt-5.3-spark",
        str(base),
        files_changed,
        commands_run,
        report,
        report["evidence"],
        "" if model["status"] == "passed" else "; ".join(model["errors"]),
        "run REQ-021 through REQ-028 envctl database API tasks" if model["status"] == "passed" else "fix schema validation errors",
    )
    append_proof(proof)
    print(
        "envctl db schema status={status} tables={tables} views={views} indexes={indexes} fks={fks}".format(
            status=model["status"],
            tables=model["summary"]["actual_table_count"],
            views=model["summary"]["actual_view_count"],
            indexes=model["summary"]["index_count"],
            fks=model["summary"]["foreign_key_count"],
        )
    )
    if model["status"] != "passed":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
