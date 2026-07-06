from __future__ import annotations

import argparse
import json
import sqlite3
from pathlib import Path
from typing import Any

from _common import append_proof, make_proof, now, package_root, root, sha256_file
from envctl_run_ledger import OperationRecord, RunLedger, apply_migrations, canonical_json, sha256_json
from replay_engine import ReplayEngine, ReplayRequest, build_replay_command_summary
from rollback_checkpoints import RollbackCheckpointStore


TASK_ID = "REQ-027_ENVCTL_REPLAY_ENGINE"
HELPER_ID = "helper-envctl-replay-01"
MODEL_TAG = "gpt-5.3-spark"


def rel_hash(relpath: str) -> str:
    return "sha256:" + sha256_file(package_root() / relpath)


def read_recipe(relpath: str) -> dict[str, Any]:
    path = package_root() / relpath
    phases: list[dict[str, Any]] = []
    current_phase: dict[str, Any] | None = None
    current_op: dict[str, Any] | None = None
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("recipe_id:"):
            recipe_id = line.split(":", 1)[1].strip()
        elif line.startswith("version:"):
            version = line.split(":", 1)[1].strip()
        elif line.startswith("- phase_id:"):
            current_phase = {
                "phase_id": line.split(":", 1)[1].strip(),
                "approval_gate": False,
                "operations": [],
            }
            phases.append(current_phase)
            current_op = None
        elif line.startswith("approval_gate:") and current_phase is not None:
            current_phase["approval_gate"] = line.split(":", 1)[1].strip().lower() == "true"
        elif line.startswith("- operation_id:") and current_phase is not None:
            current_op = {"operation_id": line.split(":", 1)[1].strip()}
            current_phase["operations"].append(current_op)
        elif line.startswith("operation_type:") and current_op is not None:
            current_op["operation_type"] = line.split(":", 1)[1].strip()
        elif line.startswith("risk:") and current_op is not None:
            current_op["risk"] = line.split(":", 1)[1].strip()
        elif ":" in line and current_op is not None:
            key, value = line.split(":", 1)
            value = value.strip()
            if value.startswith("[") and value.endswith("]"):
                current_op[key.strip()] = [item.strip() for item in value[1:-1].split(",") if item.strip()]
    return {
        "schema_version": "1",
        "recipe_id": locals().get("recipe_id", Path(relpath).stem),
        "version": locals().get("version", "1"),
        "phases": phases,
    }


def insert_recipe_catalog(conn: sqlite3.Connection) -> dict[str, str]:
    recipe = read_recipe("examples/recipes/full-migration-artifact-run.yaml")
    target_descriptor = json.loads((root() / "generated" / "envctl_target_registry.json").read_text(encoding="utf-8"))[
        "registry_rows"
    ][0]
    target_descriptor_json = {
        "schema_version": "1.0",
        "target_id": target_descriptor["target_id"],
        "target_type": target_descriptor["target_type"],
        "primary_root": target_descriptor["primary_root"],
        "compare_root": target_descriptor["compare_root"],
    }
    target_id = "target-req027-flexnetos-vs-lifeos"
    package_id = "pkg-req027-replay-fixture"
    contract_id = "contract-req027-replay-fixture"
    recipe_id = "recipe-req027-full-migration-artifact-run"
    conn.execute(
        """
        INSERT INTO envctl_migration_targets
          (id, target_id, target_type, primary_root, compare_root, descriptor_json,
           descriptor_hash, safety_mode, max_auto_risk)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            target_id,
            target_descriptor["target_id"],
            target_descriptor["target_type"],
            target_descriptor["primary_root"],
            target_descriptor["compare_root"],
            canonical_json(target_descriptor_json),
            target_descriptor["descriptor_hash"],
            target_descriptor["safety_mode"],
            target_descriptor["max_auto_risk"],
        ),
    )
    manifest = json.loads((root() / "generated" / "execution_manifest.json").read_text(encoding="utf-8"))
    conn.execute(
        """
        INSERT INTO envctl_migration_packages
          (id, package_name, package_path, package_hash, manifest_json)
        VALUES (?, ?, ?, ?, ?)
        """,
        (
            package_id,
            "envctl-db-nu-plugin-migration-automation-package",
            "execution-framework/generated/execution_manifest.json",
            rel_hash("execution-framework/generated/execution_manifest.json"),
            canonical_json(manifest),
        ),
    )
    contract_json = {
        "schema_version": "1.0",
        "required_inputs": [
            "target descriptor",
            "artifact contract",
            "recipe",
            "package manifest",
            "command/operation inputs",
            "evidence hashes",
            "artifact hashes",
            "tool versions",
            "approval decisions",
        ],
    }
    conn.execute(
        """
        INSERT INTO envctl_migration_artifact_contracts
          (id, contract_name, contract_version, source_package_id, contract_hash, contract_json)
        VALUES (?, ?, ?, ?, ?, ?)
        """,
        (
            contract_id,
            "req027-replay-contract",
            "1.0.0",
            package_id,
            sha256_json(contract_json),
            canonical_json(contract_json),
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_recipes
          (id, recipe_name, recipe_version, artifact_contract_id, recipe_hash, recipe_json)
        VALUES (?, ?, ?, ?, ?, ?)
        """,
        (
            recipe_id,
            recipe["recipe_id"],
            str(recipe["version"]),
            contract_id,
            rel_hash("examples/recipes/full-migration-artifact-run.yaml"),
            canonical_json(recipe),
        ),
    )
    conn.commit()
    return {
        "target_id": target_id,
        "package_id": package_id,
        "contract_id": contract_id,
        "recipe_id": recipe_id,
    }


def insert_replay_fixture(conn: sqlite3.Connection) -> dict[str, Any]:
    catalog = insert_recipe_catalog(conn)
    ledger = RunLedger(conn)
    run = ledger.create_run(
        run_id="run-req027-replay",
        target_id=catalog["target_id"],
        recipe_id=catalog["recipe_id"],
        artifact_contract_id=catalog["contract_id"],
        human_mode="approval-gated",
        initiated_by="envctl-replay-agent",
        sandbox_policy="workspace-write",
        approval_policy="never",
        tool_versions={"python": "stdlib", "sqlite": sqlite3.sqlite_version, "envctl_replay": "1.0"},
    )
    ledger.append_event(
        run_id=run["run_id"],
        event_type="run_created",
        phase="02-envctl-db",
        actor_type="agent",
        actor_id=HELPER_ID,
        payload={"task_id": TASK_ID, "status": "created"},
    )
    operations = [
        OperationRecord(
            operation_id="op-req027-replay-hash",
            run_id=run["run_id"],
            operation_type="replay.hash",
            phase="replay-ready",
            status="succeeded",
            risk="R1",
            idempotency_key="REQ-027/replay-hash",
            command_redacted="envctl replay dry-run --run-id run-req027-replay",
            input={"task_id": TASK_ID, "deterministic": True},
            output_ref="execution-framework/generated/envctl_replay_report.json",
        ),
        OperationRecord(
            operation_id="op-req027-manual-cutover",
            run_id=run["run_id"],
            operation_type="manual_operator",
            phase="cutover-planning",
            status="ready",
            risk="R4",
            idempotency_key="REQ-027/manual-cutover",
            command_redacted="envctl replay apply --requires-approval",
            input={
                "task_id": TASK_ID,
                "non_deterministic": True,
                "replay_note": "cutover operation needs an operator-approved window",
            },
        ),
    ]
    for op in operations:
        ledger.record_operation(op)
    ledger.link_evidence(
        run_id=run["run_id"],
        operation_id="op-req027-replay-hash",
        uri="execution-framework/generated/execution_manifest.json",
        evidence_kind="package_manifest",
        sha256=rel_hash("execution-framework/generated/execution_manifest.json"),
        metadata={"required_replay_input": True},
    )
    ledger.link_evidence(
        run_id=run["run_id"],
        operation_id="op-req027-replay-hash",
        uri="examples/recipes/full-migration-artifact-run.yaml",
        evidence_kind="recipe",
        sha256=rel_hash("examples/recipes/full-migration-artifact-run.yaml"),
        metadata={"required_replay_input": True},
    )
    ledger.link_evidence(
        run_id=run["run_id"],
        operation_id="op-req027-replay-hash",
        uri="execution-framework/proof_records/REQ-022_ENVCTL_RUN_LEDGER.proof.json",
        evidence_kind="proof_record",
        sha256=rel_hash("execution-framework/proof_records/REQ-022_ENVCTL_RUN_LEDGER.proof.json"),
        metadata={"depends_on": "REQ-022_ENVCTL_RUN_LEDGER"},
    )
    ledger.link_evidence(
        run_id=run["run_id"],
        operation_id="op-req027-replay-hash",
        uri="execution-framework/proof_records/REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS.proof.json",
        evidence_kind="proof_record",
        sha256=rel_hash("execution-framework/proof_records/REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS.proof.json"),
        metadata={"depends_on": "REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS"},
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_artifacts
          (id, run_id, artifact_id, title, artifact_type, status, path,
           content_hash, generated_by_operation_id, evidence_json, links_json)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            "artifact-req027-replay-doc",
            run["run_id"],
            "req027-replay-spec",
            "Replay and reproducibility spec",
            "spec",
            "complete",
            "specs/replay-and-reproducibility.md",
            rel_hash("specs/replay-and-reproducibility.md"),
            "op-req027-replay-hash",
            canonical_json({"evidence_refs": ["specs/replay-and-reproducibility.md"]}),
            canonical_json([]),
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_approvals
          (id, run_id, operation_id, risk, status, requested_by, reason)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        """,
        (
            "approval-req027-manual-cutover",
            run["run_id"],
            "op-req027-manual-cutover",
            "R4",
            "open",
            HELPER_ID,
            "apply replay for R4 manual cutover operation",
        ),
    )
    conn.commit()
    store = RollbackCheckpointStore(conn, package_root())
    checkpoint = store.record_checkpoint(
        run_id=run["run_id"],
        operation_id="op-req027-replay-hash",
        checkpoint_kind="artifact_boundary",
        checkpoint_ref="execution-framework/generated/execution_manifest.json",
        metadata={"task_id": TASK_ID, "replay_safe": True},
        actor_id=HELPER_ID,
    )
    ledger.append_event(
        run_id=run["run_id"],
        event_type="replay_fixture_ready",
        phase="02-envctl-db",
        actor_type="system",
        actor_id="replay-engine",
        operation_id="op-req027-replay-hash",
        evidence_refs=[
            "execution-framework/generated/execution_manifest.json",
            "execution-framework/proof_records/REQ-022_ENVCTL_RUN_LEDGER.proof.json",
            "execution-framework/proof_records/REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS.proof.json",
        ],
        payload={"checkpoint_id": checkpoint.checkpoint_id, "replay_input_hash_available": True},
    )
    return {"run": run, "catalog": catalog, "checkpoint": checkpoint.__dict__}


def exercise_replay(conn: sqlite3.Connection) -> dict[str, Any]:
    engine = ReplayEngine(conn, package_root())
    dry_request = ReplayRequest(
        replay_id="replay-req027-dry-run",
        run_id="run-req027-replay",
        mode="dry_run",
        requested_by=HELPER_ID,
        operation_ids=["op-req027-replay-hash"],
        reason="verify deterministic replay inputs before integration",
    )
    apply_request = ReplayRequest(
        replay_id="replay-req027-apply-blocked",
        run_id="run-req027-replay",
        mode="apply",
        requested_by=HELPER_ID,
        operation_ids=["op-req027-manual-cutover"],
        reason="prove approvals block R4 apply replay",
    )
    dry_result = engine.replay(dry_request)
    apply_result = engine.replay(apply_request)
    conn.execute(
        """
        UPDATE envctl_migration_evidence
        SET sha256 = ?
        WHERE uri = ?
        """,
        ("sha256:0000000000000000000000000000000000000000000000000000000000000000", "examples/recipes/full-migration-artifact-run.yaml"),
    )
    conn.commit()
    mismatch_result = engine.replay(
        ReplayRequest(
            replay_id="replay-req027-mismatch",
            run_id="run-req027-replay",
            mode="dry_run",
            requested_by=HELPER_ID,
            operation_ids=["op-req027-replay-hash"],
            reason="prove hash mismatch detection",
        )
    )
    command_summary = {
        "dry_run": build_replay_command_summary(dry_request),
        "apply_blocked": build_replay_command_summary(apply_request),
    }
    checks = {
        "dry_run_passes": dry_result["status"] == "pass",
        "event_chain_valid": dry_result["event_chain"]["chain_valid"],
        "reproducibility_hash_matches": dry_result["hash_status"]["reproducibility_hash_matches"],
        "evidence_hashes_match": dry_result["hash_status"]["evidence_matches"] >= 4
        and not dry_result["hash_status"]["evidence_mismatches"],
        "artifact_hashes_match": dry_result["hash_status"]["artifact_matches"] == 1,
        "apply_replay_blocked_by_approval": apply_result["status"] == "blocked"
        and len(apply_result["required_approvals"]) == 1,
        "non_deterministic_operation_detected": len(apply_result["non_deterministic_operations"]) == 1,
        "mismatch_detected": mismatch_result["status"] == "fail"
        and len(mismatch_result["hash_status"]["evidence_mismatches"]) == 1,
        "safe_next_action_present": bool(dry_result["safe_next_action"]),
    }
    errors = [name for name, ok in checks.items() if not ok]
    return {
        "dry_run": dry_result,
        "apply_blocked": apply_result,
        "mismatch": mismatch_result,
        "command_summary": command_summary,
        "checks": checks,
        "errors": errors,
    }


def write_docs(report: dict[str, Any]) -> None:
    lines = [
        "# envctl replay engine",
        "",
        f"Generated at: `{report['generated_at']}`",
        f"Status: `{report['status']}`",
        "",
        "## Replay surface",
        "",
        "- Reconstructs target descriptor, artifact contract, recipe, package manifest, operation inputs, proof/evidence hashes, artifact hashes, tool versions, approvals, checkpoints, and hash-chained run state.",
        "- Produces dry-run and apply-mode replay results using `ReplayRequest` / `ReplayResult` compatible fields from the shared protocol schema.",
        "- Fails closed on hash mismatches, blocked paths, open approvals, and non-deterministic operations.",
        "",
        "## Runtime smoke",
        "",
        f"- Dry-run status: `{report['replay']['dry_run']['status']}`",
        f"- Apply status: `{report['replay']['apply_blocked']['status']}`",
        f"- Mismatch status: `{report['replay']['mismatch']['status']}`",
        f"- Event chain valid: `{report['replay']['dry_run']['event_chain']['chain_valid']}`",
        f"- Evidence matches: `{report['replay']['dry_run']['hash_status']['evidence_matches']}`",
        f"- Artifact matches: `{report['replay']['dry_run']['hash_status']['artifact_matches']}`",
        f"- Required approvals in apply replay: `{len(report['replay']['apply_blocked']['required_approvals'])}`",
        f"- Non-deterministic operations in apply replay: `{len(report['replay']['apply_blocked']['non_deterministic_operations'])}`",
        "",
        "## Commands",
        "",
        "```bash",
        report["replay"]["command_summary"]["dry_run"][0],
        report["replay"]["command_summary"]["apply_blocked"][0],
        "python3 scripts/verify_replay_engine.py",
        "```",
    ]
    (root() / "docs" / "ENVCTL_REPLAY_ENGINE.md").write_text("\n".join(lines) + "\n", encoding="utf-8")


def write_outputs(report: dict[str, Any]) -> None:
    report_path = root() / "generated" / "envctl_replay_report.json"
    log_path = root() / "logs" / f"{TASK_ID}.log"
    heartbeat_path = root() / "state" / f"{TASK_ID}.heartbeat.json"
    report_path.write_text(json.dumps(report, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    write_docs(report)
    log_path.write_text(json.dumps(report, indent=2, sort_keys=False) + "\n", encoding="utf-8")
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


def build_report() -> dict[str, Any]:
    conn = sqlite3.connect(":memory:")
    conn.execute("PRAGMA foreign_keys = ON")
    applied = apply_migrations(conn, package_root())
    fixture = insert_replay_fixture(conn)
    replay = exercise_replay(conn)
    status = "passed" if not replay["errors"] else "failed"
    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "status": status,
        "generated_at": now(),
        "database_backend": "sqlite",
        "runtime": "python sqlite3 in-memory",
        "applied_migrations": applied,
        "fixture": fixture,
        "replay": replay,
        "coverage": {
            "target_descriptor": replay["checks"]["dry_run_passes"],
            "recipe": replay["checks"]["evidence_hashes_match"],
            "packet_manifest": replay["checks"]["evidence_hashes_match"],
            "proof_hashes": replay["checks"]["evidence_hashes_match"],
            "artifact_hashes": replay["checks"]["artifact_hashes_match"],
            "state_hash_chain": replay["checks"]["event_chain_valid"],
            "approval_gate": replay["checks"]["apply_replay_blocked_by_approval"],
            "non_determinism_detection": replay["checks"]["non_deterministic_operation_detected"],
            "mismatch_detection": replay["checks"]["mismatch_detected"],
        },
        "errors": replay["errors"],
        "evidence": [
            "scripts/replay_engine.py",
            "scripts/verify_replay_engine.py",
            "generated/envctl_replay_report.json",
            "docs/ENVCTL_REPLAY_ENGINE.md",
            "specs/replay-and-reproducibility.md",
            "examples/recipes/full-migration-artifact-run.yaml",
            "generated/execution_manifest.json",
            "proof_records/REQ-022_ENVCTL_RUN_LEDGER.proof.json",
            "proof_records/REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS.proof.json",
        ],
    }


def main() -> None:
    parser = argparse.ArgumentParser(description="Verify the envctl replay/reproduce engine.")
    parser.add_argument("--json", action="store_true", help="print the full report JSON")
    args = parser.parse_args()

    report = build_report()
    write_outputs(report)
    files_changed = [
        "execution-framework/scripts/replay_engine.py",
        "execution-framework/scripts/verify_replay_engine.py",
        "execution-framework/generated/envctl_replay_report.json",
        "execution-framework/docs/ENVCTL_REPLAY_ENGINE.md",
        "execution-framework/state/REQ-027_ENVCTL_REPLAY_ENGINE.heartbeat.json",
        "execution-framework/logs/REQ-027_ENVCTL_REPLAY_ENGINE.log",
        "execution-framework/proof_records/REQ-027_ENVCTL_REPLAY_ENGINE.proof.json",
        "execution-framework/proof_records/proof_ledger.jsonl",
    ]
    commands_run = [
        "python3 scripts/verify_replay_engine.py",
        "python3 -m py_compile scripts/replay_engine.py scripts/verify_replay_engine.py",
    ]
    proof = make_proof(
        TASK_ID,
        "completed" if report["status"] == "passed" else "failed",
        "codex-cli-local",
        HELPER_ID,
        MODEL_TAG,
        str(package_root()),
        files_changed,
        commands_run,
        {
            "status": report["status"],
            "checks": report["replay"]["checks"],
            "dry_run_status": report["replay"]["dry_run"]["status"],
            "apply_status": report["replay"]["apply_blocked"]["status"],
            "mismatch_status": report["replay"]["mismatch"]["status"],
            "errors": report["errors"],
        },
        report["evidence"],
        "" if report["status"] == "passed" else "; ".join(report["errors"]),
        "REQ-041 two-repo integration can consume replay report and command summary"
        if report["status"] == "passed"
        else "fix replay verification errors",
    )
    append_proof(proof)
    if args.json:
        print(json.dumps(report, indent=2, sort_keys=False))
    else:
        print(
            "envctl replay status={status} dry_run={dry} apply={apply} mismatch={mismatch} approvals={approvals}".format(
                status=report["status"],
                dry=report["replay"]["dry_run"]["status"],
                apply=report["replay"]["apply_blocked"]["status"],
                mismatch=report["replay"]["mismatch"]["status"],
                approvals=len(report["replay"]["apply_blocked"]["required_approvals"]),
            )
        )
    if report["status"] != "passed":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
