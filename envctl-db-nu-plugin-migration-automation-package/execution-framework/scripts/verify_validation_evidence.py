from __future__ import annotations

import json
import sqlite3

from _common import append_proof, make_proof, now, package_root, root
from validation_evidence import ValidationEvidenceStore, fetch_validation
from verify_envctl_db_schema import apply_migrations


TASK_ID = "REQ-025_ENVCTL_VALIDATION_EVIDENCE"
HELPER_ID = "helper-envctl-validation-01"
MODEL_TAG = "gpt-5.3-spark"


def read_json_schema() -> dict:
    return json.loads((package_root() / "schemas" / "validation_result.schema.json").read_text(encoding="utf-8"))


def insert_req025_fixture(conn: sqlite3.Connection) -> dict:
    conn.execute(
        """
        INSERT INTO envctl_migration_targets
          (id, target_id, target_type, primary_root, compare_root, descriptor_json,
           descriptor_hash, safety_mode, max_auto_risk)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            "target-req025",
            "flexnetos-envctl-validation-evidence",
            "mixed",
            "/workspace/flexnetos",
            "/workspace/lifeos",
            '{"schema_version":1,"target":"validation-evidence"}',
            "sha256:target-req025",
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
            "pkg-req025",
            "envctl-db-nu-plugin-migration-automation-package",
            ".",
            "sha256:pkg-req025",
            '{"schema_version":1,"task_id":"REQ-025_ENVCTL_VALIDATION_EVIDENCE"}',
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_artifact_contracts
          (id, contract_name, contract_version, source_package_id, contract_hash, contract_json)
        VALUES (?, ?, ?, ?, ?, ?)
        """,
        (
            "contract-req025",
            "validation-evidence-store",
            "1.0.0",
            "pkg-req025",
            "sha256:contract-req025",
            '{"required":["reconciliation","parity","test_result","proof_record"]}',
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_recipes
          (id, recipe_name, recipe_version, artifact_contract_id, recipe_hash, recipe_json)
        VALUES (?, ?, ?, ?, ?, ?)
        """,
        (
            "recipe-req025",
            "validation-evidence-smoke",
            "1.0.0",
            "contract-req025",
            "sha256:recipe-req025",
            '{"phases":["reconcile","parity","test","proof"]}',
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
            "run-req025",
            "target-req025",
            "recipe-req025",
            "contract-req025",
            "validating",
            "approval-gated",
            "envctl-db-agent",
            "workspace-write",
            "never",
            '{"python":"stdlib","sqlite":"stdlib"}',
            "sha256:run-req025",
            now(),
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
            "op-req025-validate",
            "run-req025",
            "record_validation_evidence",
            "02-envctl-db",
            "succeeded",
            "R1",
            "REQ-025/record-validation-evidence",
            "sha256:command-req025",
            "python3 scripts/verify_validation_evidence.py",
            '{"task_id":"REQ-025_ENVCTL_VALIDATION_EVIDENCE"}',
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_artifacts
          (id, run_id, artifact_id, title, artifact_type, status, path, generated_by_operation_id)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            "artifact-row-req025",
            "run-req025",
            "art-123-validation-reconciliation",
            "ART-123 Validation Reconciliation",
            "validation_reconciliation",
            "partial",
            "execution-framework/generated/envctl_validation_evidence_report.json",
            "op-req025-validate",
        ),
    )
    conn.commit()
    return {
        "run_id": "run-req025",
        "operation_id": "op-req025-validate",
        "artifact_id": "art-123-validation-reconciliation",
    }


def write_sample_evidence() -> dict[str, str]:
    base = root() / "generated" / "validation_evidence"
    base.mkdir(parents=True, exist_ok=True)
    samples = {
        "reconciliation": base / "reconciliation.json",
        "parity": base / "parity.json",
        "test_result": base / "test_results.json",
    }
    samples["reconciliation"].write_text(
        json.dumps(
            {
                "schema_version": "1.0",
                "task_id": TASK_ID,
                "summary": "source and target validation rows reconciled",
                "matched": 12,
                "missing": 0,
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )
    samples["parity"].write_text(
        json.dumps(
            {
                "schema_version": "1.0",
                "task_id": TASK_ID,
                "source": "envctl",
                "target": "nu_plugin",
                "parity": "warn",
                "differences": ["nu_plugin command coverage pending ART-132"],
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )
    samples["test_result"].write_text(
        json.dumps(
            {
                "schema_version": "1.0",
                "task_id": TASK_ID,
                "command": "python3 scripts/verify_validation_evidence.py",
                "status": "pass",
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )
    return {kind: f"execution-framework/generated/validation_evidence/{path.name}" for kind, path in samples.items()}


def record_smoke_validations(conn: sqlite3.Connection, fixture: dict, sample_paths: dict[str, str]) -> list[dict]:
    store = ValidationEvidenceStore(conn, package_root())
    records = [
        {
            "validation_id": "validation-req025-reconciliation",
            "run_id": fixture["run_id"],
            "artifact_id": fixture["artifact_id"],
            "operation_id": fixture["operation_id"],
            "validator": "validation_evidence.py:reconciliation",
            "status": "pass",
            "details": {"matched": 12, "missing": 0, "blocks": ["ART-123_VALIDATION_RECONCILIATION"]},
            "evidence_refs": [
                {"uri": sample_paths["reconciliation"], "evidence_kind": "reconciliation"},
                {"uri": "proof_records/REQ-020_ENVCTL_DB_SCHEMA.proof.json", "evidence_kind": "proof_record"},
            ],
        },
        {
            "validation_id": "validation-req025-parity",
            "run_id": fixture["run_id"],
            "artifact_id": fixture["artifact_id"],
            "operation_id": fixture["operation_id"],
            "validator": "validation_evidence.py:parity",
            "status": "warn",
            "details": {"source": "envctl", "target": "nu_plugin", "pending": ["ART-132_PARITY_DASHBOARD"]},
            "evidence_refs": [
                {"uri": sample_paths["parity"], "evidence_kind": "parity"},
            ],
        },
        {
            "validation_id": "validation-req025-tests",
            "run_id": fixture["run_id"],
            "operation_id": fixture["operation_id"],
            "validator": "validation_evidence.py:test-results",
            "status": "pass",
            "details": {"command": "python3 scripts/verify_validation_evidence.py"},
            "evidence_refs": [
                {"uri": sample_paths["test_result"], "evidence_kind": "test_result"},
            ],
        },
    ]
    return [store.record(record) for record in records]


def expect_rejection(label: str, store: ValidationEvidenceStore, record: dict) -> dict:
    try:
        store.record(record)
    except ValueError as exc:
        return {"case": label, "status": "rejected", "message": str(exc)}
    return {"case": label, "status": "accepted", "message": "validation evidence store accepted an unsafe record"}


def exercise_rejections(conn: sqlite3.Connection, fixture: dict) -> list[dict]:
    store = ValidationEvidenceStore(conn, package_root())
    base_record = {
        "validation_id": "validation-req025-rejection",
        "run_id": fixture["run_id"],
        "operation_id": fixture["operation_id"],
        "validator": "validation_evidence.py:rejection",
        "status": "pass",
    }
    return [
        expect_rejection(
            "invalid-status",
            store,
            {**base_record, "validation_id": "validation-req025-invalid-status", "status": "done"},
        ),
        expect_rejection(
            "blocked-secret-path",
            store,
            {
                **base_record,
                "validation_id": "validation-req025-blocked-secret-path",
                "evidence_refs": [{"uri": "execution-framework/secrets/token.txt", "evidence_kind": "test_result"}],
            },
        ),
        expect_rejection(
            "foreign-operation",
            store,
            {
                **base_record,
                "validation_id": "validation-req025-foreign-operation",
                "operation_id": "op-does-not-exist",
            },
        ),
    ]


def build_report(conn: sqlite3.Connection, fixture: dict, results: list[dict], rejections: list[dict]) -> dict:
    store = ValidationEvidenceStore(conn, package_root())
    schema = read_json_schema()
    scorecard = store.scorecard(fixture["run_id"])
    validation_rows = [
        fetch_validation(conn, "validation-req025-reconciliation"),
        fetch_validation(conn, "validation-req025-parity"),
        fetch_validation(conn, "validation-req025-tests"),
    ]
    evidence_count = conn.execute(
        "SELECT COUNT(*) FROM envctl_migration_evidence WHERE run_id = ?",
        (fixture["run_id"],),
    ).fetchone()[0]
    hashed_evidence_count = conn.execute(
        "SELECT COUNT(*) FROM envctl_migration_evidence WHERE run_id = ? AND sha256 IS NOT NULL",
        (fixture["run_id"],),
    ).fetchone()[0]
    required = set(schema.get("required", []))
    accepted_rejections = [item["case"] for item in rejections if item["status"] != "rejected"]
    errors = []
    if not {"validation_id", "run_id", "validator", "status"}.issubset(required):
        errors.append("validation_result schema required fields drifted")
    if scorecard["counts"].get("pass") != 2 or scorecard["counts"].get("warn") != 1:
        errors.append(f"unexpected scorecard counts: {scorecard['counts']}")
    expected_kinds = {"reconciliation", "parity", "test_result", "proof_record"}
    if not expected_kinds.issubset(set(scorecard["evidence_by_kind"])):
        errors.append(f"missing evidence kinds: {sorted(expected_kinds - set(scorecard['evidence_by_kind']))}")
    if evidence_count != 4:
        errors.append(f"expected 4 evidence rows, got {evidence_count}")
    if hashed_evidence_count != evidence_count:
        errors.append(f"expected all evidence rows to have hashes, got {hashed_evidence_count}/{evidence_count}")
    if not all(row["operation_id"] == fixture["operation_id"] for row in validation_rows):
        errors.append("operation ids were not persisted on all validation rows")
    if accepted_rejections:
        errors.append(f"store accepted unsafe records: {', '.join(accepted_rejections)}")
    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "status": "passed" if not errors else "failed",
        "generated_at": now(),
        "fixture": fixture,
        "schema_source": {
            "path": "schemas/validation_result.schema.json",
            "required": schema.get("required", []),
            "status_values": schema.get("properties", {}).get("status", {}).get("enum", []),
        },
        "results": results,
        "validation_rows": validation_rows,
        "scorecard": scorecard,
        "summary": {
            "validation_count": len(validation_rows),
            "evidence_count": evidence_count,
            "hashed_evidence_count": hashed_evidence_count,
            "rejection_count": len(rejections),
        },
        "coverage": {
            "reconciliation": scorecard["evidence_by_kind"].get("reconciliation", 0) >= 1,
            "parity": scorecard["evidence_by_kind"].get("parity", 0) >= 1,
            "test_results": scorecard["evidence_by_kind"].get("test_result", 0) >= 1,
            "proof_evidence": scorecard["evidence_by_kind"].get("proof_record", 0) >= 1,
            "scorecard_view": scorecard["counts"].get("pass") == 2 and scorecard["counts"].get("warn") == 1,
            "fail_closed_rejections": bool(rejections) and not accepted_rejections,
        },
        "rejection_cases": rejections,
        "errors": errors,
        "evidence": [
            "scripts/validation_evidence.py",
            "scripts/verify_validation_evidence.py",
            "generated/envctl_validation_evidence_report.json",
            "generated/validation_evidence/reconciliation.json",
            "generated/validation_evidence/parity.json",
            "generated/validation_evidence/test_results.json",
            "docs/ENVCTL_VALIDATION_EVIDENCE.md",
            "proof_records/REQ-020_ENVCTL_DB_SCHEMA.proof.json",
        ],
    }


def write_docs(report: dict) -> None:
    coverage = report["coverage"]
    lines = [
        "# envctl validation evidence store",
        "",
        f"Generated at: `{report['generated_at']}`",
        f"Status: `{report['status']}`",
        "",
        "## Persisted evidence",
        "",
        "| evidence class | covered |",
        "|---|---|",
    ]
    for key in ["reconciliation", "parity", "test_results", "proof_evidence", "scorecard_view", "fail_closed_rejections"]:
        lines.append(f"| {key.replace('_', ' ')} | {'yes' if coverage[key] else 'no'} |")
    lines.extend(
        [
            "",
            "## Runtime smoke",
            "",
            f"- Run: `{report['fixture']['run_id']}`",
            f"- Validation rows: `{report['summary']['validation_count']}`",
            f"- Evidence rows: `{report['summary']['evidence_count']}`",
            f"- Hashed evidence rows: `{report['summary']['hashed_evidence_count']}`",
            f"- Scorecard: `{report['scorecard']['counts']}`",
            f"- Evidence kinds: `{report['scorecard']['evidence_by_kind']}`",
            f"- Rejection cases: `{report['summary']['rejection_count']}`",
            "",
            "The smoke records reconciliation, parity, test-result, and proof evidence into the REQ-020 SQLite schema, persists validation rows linked to operations and artifacts, computes SHA-256 hashes for local evidence, reads the validation scorecard view, and confirms unsafe records fail closed.",
        ]
    )
    (root() / "docs" / "ENVCTL_VALIDATION_EVIDENCE.md").write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> None:
    base = package_root()
    conn = sqlite3.connect(":memory:")
    conn.execute("PRAGMA foreign_keys = ON")
    apply_migrations(conn, base)
    fixture = insert_req025_fixture(conn)
    sample_paths = write_sample_evidence()
    results = record_smoke_validations(conn, fixture, sample_paths)
    rejections = exercise_rejections(conn, fixture)
    report = build_report(conn, fixture, results, rejections)

    report_path = root() / "generated" / "envctl_validation_evidence_report.json"
    log_path = root() / "logs" / f"{TASK_ID}.log"
    heartbeat_path = root() / "state" / f"{TASK_ID}.heartbeat.json"
    report_path.write_text(json.dumps(report, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    write_docs(report)
    log_path.write_text(json.dumps(report, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    heartbeat_path.parent.mkdir(parents=True, exist_ok=True)
    heartbeat_path.write_text(
        json.dumps(
            {
                "task_id": TASK_ID,
                "status": "completed" if report["status"] == "passed" else "failed",
                "updated_at": report["generated_at"],
                "proof_uri": f"proof_records/{TASK_ID}.proof.json",
            },
            indent=2,
            sort_keys=False,
        )
        + "\n",
        encoding="utf-8",
    )

    files_changed = [
        "execution-framework/scripts/validation_evidence.py",
        "execution-framework/scripts/verify_validation_evidence.py",
        "execution-framework/generated/envctl_validation_evidence_report.json",
        "execution-framework/generated/validation_evidence/reconciliation.json",
        "execution-framework/generated/validation_evidence/parity.json",
        "execution-framework/generated/validation_evidence/test_results.json",
        "execution-framework/docs/ENVCTL_VALIDATION_EVIDENCE.md",
        "execution-framework/state/REQ-025_ENVCTL_VALIDATION_EVIDENCE.heartbeat.json",
        "execution-framework/logs/REQ-025_ENVCTL_VALIDATION_EVIDENCE.log",
        "execution-framework/proof_records/REQ-025_ENVCTL_VALIDATION_EVIDENCE.proof.json",
        "execution-framework/proof_records/proof_ledger.jsonl",
    ]
    commands_run = [
        "python3 scripts/verify_validation_evidence.py",
        "python3 -m py_compile scripts/validation_evidence.py scripts/verify_validation_evidence.py",
    ]
    proof = make_proof(
        TASK_ID,
        "completed" if report["status"] == "passed" else "failed",
        "codex-cli-local",
        HELPER_ID,
        MODEL_TAG,
        str(base),
        files_changed,
        commands_run,
        report,
        report["evidence"],
        "" if report["status"] == "passed" else "; ".join(report["errors"]),
        "run ART-123_VALIDATION_RECONCILIATION against the validation evidence store"
        if report["status"] == "passed"
        else "fix validation evidence verification errors",
    )
    append_proof(proof)
    print(
        "validation evidence status={status} validations={validations} evidence={evidence} hashed={hashed}".format(
            status=report["status"],
            validations=report["summary"]["validation_count"],
            evidence=report["summary"]["evidence_count"],
            hashed=report["summary"]["hashed_evidence_count"],
        )
    )
    if report["status"] != "passed":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
