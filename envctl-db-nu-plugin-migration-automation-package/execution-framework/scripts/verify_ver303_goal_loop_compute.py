#!/usr/bin/env python3
"""Validate that the goal loop computes a complete runnable path."""

from __future__ import annotations

import re
from copy import deepcopy
from pathlib import Path
from typing import Any

from _common import append_proof, make_proof, now, read_json, read_task_graph, root, sha256_file, write_json
from goal_loop import compute


TASK_ID = "VER-303_GOAL_LOOP_COMPUTE"
HELPER_ID = "helper-goal-validate-01"
MODEL_TAG = "gpt-5.3-spark"
ACTOR = "goal-validation-agent"

PACKET_PATH = "generated/execution_packets/VER-303_GOAL_LOOP_COMPUTE.json"
REPORT_PATH = "generated/ver303_goal_loop_compute_report.json"
HEARTBEAT_PATH = "state/VER-303_GOAL_LOOP_COMPUTE.heartbeat.json"
LOG_PATH = "logs/VER-303_GOAL_LOOP_COMPUTE.log"
PROOF_PATH = "proof_records/VER-303_GOAL_LOOP_COMPUTE.proof.json"
STATUS_REPORT_PATH = "generated/status_report.json"
STATUS_FROM_PROOFS_PATH = "generated/status_from_proofs.json"
GOAL_STATE_PATH = "state/goal_loop_state.json"
LEDGER_PATH = "proof_records/proof_ledger.jsonl"

DEPENDENCY_PROOFS = {
    "VER-302_PACKET_SCHEMA_VALIDATION": "proof_records/VER-302_PACKET_SCHEMA_VALIDATION.proof.json",
    "REQ-045_RUN_REPLAY": "proof_records/REQ-045_RUN_REPLAY.proof.json",
}

TERMINAL_COMPLETE = {"completed", "complete", "pass", "passed"}
SECRET_PATTERNS = [
    re.compile(r"-----BEGIN [A-Z ]*PRIVATE KEY-----"),
    re.compile(r"\bsk-[A-Za-z0-9_-]{20,}\b"),
    re.compile(r"(?i)(password|secret|token|api[_-]?key)\s*[:=]\s*['\"]?[A-Za-z0-9_./+=-]{12,}"),
]


def is_complete_status(status: str | None) -> bool:
    return str(status).lower() in TERMINAL_COMPLETE


def read_ledger_records() -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    ledger_path = root() / LEDGER_PATH
    for line in ledger_path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line:
            continue
        records.append(__import__("json").loads(line))
    return records


def synthetic_complete_proof(task_id: str) -> dict[str, Any]:
    return {
        "task_id": task_id,
        "status": "completed",
    }


def project_run_path(rows: list[dict[str, Any]], proofs: list[dict[str, Any]]) -> dict[str, Any]:
    synthetic_proofs = [deepcopy(proof) for proof in proofs]
    path: list[dict[str, Any]] = []

    for _ in range(len(rows) + 1):
        state = compute(rows, synthetic_proofs)
        if state["approval_blocker_count"] > 0:
            return {
                "stop_reason": "approval_blocker",
                "path": path,
                "final_state": state,
            }
        dispatch = state.get("dispatch_packets", [])
        if not dispatch:
            return {
                "stop_reason": "complete" if state["pending_count"] == 0 else "stalled",
                "path": path,
                "final_state": state,
            }

        next_task = dispatch[0]
        task_id = next_task["task_id"]
        path.append(
            {
                "task_id": task_id,
                "parallel_group": next_task.get("parallel_group"),
                "max_parallel": next_task.get("max_parallel"),
            }
        )
        synthetic_proofs = [proof for proof in synthetic_proofs if proof.get("task_id") != task_id]
        synthetic_proofs.append(synthetic_complete_proof(task_id))

    return {
        "stop_reason": "loop_guard_triggered",
        "path": path,
        "final_state": compute(rows, synthetic_proofs),
    }


def secret_findings(paths: list[Path]) -> list[str]:
    findings: list[str] = []
    for path in paths:
        text = path.read_text(encoding="utf-8")
        for pattern in SECRET_PATTERNS:
            if pattern.search(text):
                findings.append(str(path.relative_to(root())))
                break
    return findings


def write_status_from_ledger(rows: list[dict[str, Any]], proofs: list[dict[str, Any]]) -> None:
    proof_map = {proof.get("task_id"): proof for proof in proofs if proof.get("task_id")}
    tasks = []
    for row in rows:
        proof = proof_map.get(row["task_id"])
        tasks.append(
            {
                "task_id": row["task_id"],
                "phase": row["phase"],
                "owner_lane": row["owner_lane"],
                "status": proof.get("status", "pending") if proof else "pending",
                "proof_uri": row["proof_uri"],
            }
        )
    write_json(
        STATUS_FROM_PROOFS_PATH,
        {
            "schema_version": "1.0",
            "generated_at": now(),
            "tasks": tasks,
        },
    )


def main() -> None:
    base = root()
    generated_at = now()
    rows = read_task_graph("generated/task_graph.csv")
    packet = read_json(PACKET_PATH)
    status_report = read_json(STATUS_REPORT_PATH)
    proofs = read_ledger_records()

    errors: list[str] = []
    warnings: list[str] = []

    dependency_results: dict[str, Any] = {}
    for task_id, proof_path in DEPENDENCY_PROOFS.items():
        proof = read_json(proof_path)
        ok = is_complete_status(proof.get("status"))
        dependency_results[task_id] = {
            "proof_path": proof_path,
            "status": proof.get("status"),
            "ok": ok,
        }
        if not ok:
            errors.append(f"dependency proof not completed: {task_id}")

    authoritative_state = compute(rows, proofs)
    current_dispatch = [item["task_id"] for item in authoritative_state.get("dispatch_packets", [])]
    current_runnable = [item["task_id"] for item in authoritative_state.get("runnable_tasks", [])]
    current_blocked = {item["task_id"]: item for item in authoritative_state.get("blocked_tasks", [])}

    projected = project_run_path(rows, proofs)
    projected_path = [item["task_id"] for item in projected["path"]]
    projected_final = projected["final_state"]

    checks = {
        "dependencies_completed": all(item["ok"] for item in dependency_results.values()),
        "current_dispatch_targets_ver303": current_dispatch == [TASK_ID],
        "current_runnable_contains_only_ver303": current_runnable == [TASK_ID],
        "current_has_no_approval_blockers": authoritative_state.get("approval_blocker_count") == 0,
        "current_status_marks_ver303_pending": authoritative_state.get("statuses", {}).get(TASK_ID) == "pending",
        "current_status_surface_matches_compute": status_report.get("dispatch_packets", []) == authoritative_state.get("dispatch_packets", []),
        "projected_path_begins_with_ver303": projected_path[:1] == [TASK_ID],
        "projected_advances_to_ver304": len(projected_path) >= 2 and projected_path[1] == "VER-304_FINAL_COMPLETENESS",
        "projected_reaches_release_tasks": projected_path[-2:] == ["REL-400_PACKAGE_ARCHIVE", "REL-401_HANDOFF"],
        "projected_stops_as_complete": projected.get("stop_reason") == "complete",
        "projected_has_no_approval_blockers": projected_final.get("approval_blocker_count") == 0,
        "projected_has_no_failed_tasks": projected_final.get("failed_count") == 0,
        "projected_finishes_all_tasks": projected_final.get("pending_count") == 0,
        "ver304_is_currently_blocked_only_by_ver303": current_blocked.get("VER-304_FINAL_COMPLETENESS", {}).get("dependencies") == [TASK_ID],
        "packet_proof_required": packet.get("proof_required") is True,
        "packet_single_threaded": packet.get("can_run_parallel") is False and packet.get("max_parallel") == 1,
    }

    if projected_path != [
        "VER-303_GOAL_LOOP_COMPUTE",
        "VER-304_FINAL_COMPLETENESS",
        "REL-400_PACKAGE_ARCHIVE",
        "REL-401_HANDOFF",
    ]:
        warnings.append("projected path differs from expected linear tail sequence")

    for name, passed in checks.items():
        if not passed:
            errors.append(f"check failed: {name}")

    report = {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "status": "pass" if not errors else "fail",
        "generated_at": generated_at,
        "packet_summary": {
            "goal": packet.get("goal"),
            "repo_target": packet.get("repo_target"),
            "repo_path": packet.get("repo_path"),
            "filesystem_scope": packet.get("filesystem_scope"),
            "verification_command": "python3 scripts/verify_ver303_goal_loop_compute.py",
        },
        "dependency_proofs": dependency_results,
        "current_state": {
            "generated_at": authoritative_state.get("generated_at"),
            "dispatch_packets": authoritative_state.get("dispatch_packets", []),
            "runnable_tasks": authoritative_state.get("runnable_tasks", []),
            "approval_blockers": authoritative_state.get("approval_blockers", []),
            "blocked_tasks": authoritative_state.get("blocked_tasks", []),
            "status_ver303": authoritative_state.get("statuses", {}).get(TASK_ID),
            "status_ver304": authoritative_state.get("statuses", {}).get("VER-304_FINAL_COMPLETENESS"),
        },
        "projected_run_path": projected_path,
        "projected_stop_reason": projected.get("stop_reason"),
        "projected_terminal_state": {
            "complete_count": projected_final.get("complete_count"),
            "failed_count": projected_final.get("failed_count"),
            "pending_count": projected_final.get("pending_count"),
            "approval_blocker_count": projected_final.get("approval_blocker_count"),
            "blocked_count": projected_final.get("blocked_count"),
            "dispatch_count": projected_final.get("dispatch_count"),
        },
        "checks": checks,
        "warnings": warnings,
        "errors": errors,
        "sha256": {
            relpath: "sha256:" + sha256_file(base / relpath)
            for relpath in [PACKET_PATH, STATUS_REPORT_PATH, LEDGER_PATH, *DEPENDENCY_PROOFS.values()]
        },
    }

    write_json(REPORT_PATH, report)
    write_json(LOG_PATH, report)
    write_json(
        HEARTBEAT_PATH,
        {
            "schema_version": "1.0",
            "task_id": TASK_ID,
            "status": report["status"],
            "updated_at": generated_at,
            "proof_uri": PROOF_PATH,
            "validation_report": REPORT_PATH,
        },
    )

    secret_scan_paths = [base / REPORT_PATH, base / LOG_PATH, Path(__file__)]
    secret_hits = secret_findings(secret_scan_paths)
    report["secret_scan"] = {
        "paths": [str(path.relative_to(base)) for path in secret_scan_paths],
        "findings": secret_hits,
    }
    if secret_hits:
        report["status"] = "fail"
        report["errors"].append("secret-like content detected in generated outputs: " + ", ".join(secret_hits))
        write_json(REPORT_PATH, report)
        write_json(LOG_PATH, report)
        write_json(
            HEARTBEAT_PATH,
            {
                "schema_version": "1.0",
                "task_id": TASK_ID,
                "status": report["status"],
                "updated_at": generated_at,
                "proof_uri": PROOF_PATH,
                "validation_report": REPORT_PATH,
            },
        )

    files_changed = [
        "execution-framework/scripts/verify_ver303_goal_loop_compute.py",
        "execution-framework/generated/ver303_goal_loop_compute_report.json",
        "execution-framework/state/VER-303_GOAL_LOOP_COMPUTE.heartbeat.json",
        "execution-framework/logs/VER-303_GOAL_LOOP_COMPUTE.log",
        "execution-framework/proof_records/VER-303_GOAL_LOOP_COMPUTE.proof.json",
        "execution-framework/proof_records/proof_ledger.jsonl",
        "execution-framework/state/goal_loop_state.json",
        "execution-framework/generated/status_report.json",
        "execution-framework/generated/status_from_proofs.json",
    ]
    evidence = [
        PACKET_PATH,
        STATUS_REPORT_PATH,
        LEDGER_PATH,
        REPORT_PATH,
        *DEPENDENCY_PROOFS.values(),
    ]
    proof = make_proof(
        TASK_ID,
        "completed" if report["status"] == "pass" else "failed",
        ACTOR,
        HELPER_ID,
        MODEL_TAG,
        "..",
        files_changed,
        [
            "python3 scripts/verify_ver303_goal_loop_compute.py",
            "python3 -m py_compile scripts/verify_ver303_goal_loop_compute.py",
        ],
        report,
        evidence,
        "" if report["status"] == "pass" else "; ".join(report["errors"]),
        "run VER-304_FINAL_COMPLETENESS" if report["status"] == "pass" else "fix VER-303 goal loop validation failures",
    )
    append_proof(proof)

    refreshed_proofs = read_ledger_records()
    refreshed_state = compute(rows, refreshed_proofs)
    write_json(GOAL_STATE_PATH, refreshed_state)
    write_json(STATUS_REPORT_PATH, refreshed_state)
    write_status_from_ledger(rows, refreshed_proofs)


if __name__ == "__main__":
    main()
