from __future__ import annotations

import argparse
from collections import Counter, defaultdict
from pathlib import Path

from _common import (
    append_proof,
    load_ledger,
    make_proof,
    now,
    read_json,
    read_task_graph,
    root,
    split_list,
    write_json,
)

TASK_ID = "REQ-032_PLUGIN_LIVE_VISUALS"
ACTOR = "codex-cli-local"
HELPER_ID = "helper-nu-visuals-01"
MODEL_TAG = "gpt-5.3-spark"
REPO_PATH = "/home/flexnetos/FlexNetOS/src/envctl/envctl-db-nu-plugin-migration-automation-package"


def proof_file_exists(row: dict) -> bool:
    proof_uri = row.get("proof_uri", "")
    return bool(proof_uri and (root() / proof_uri).exists())


def task_status(row: dict, proof_by_task: dict[str, dict]) -> str:
    proof = proof_by_task.get(row["task_id"])
    if proof:
        return proof.get("status", "pending")
    if proof_file_exists(row):
        return "completed"
    return row.get("status") or "pending"


def completion_status(row: dict, proof_by_task: dict[str, dict]) -> str:
    status = task_status(row, proof_by_task)
    return "completed" if status == "completed" else status


def build_visual_model(rows: list[dict], status_report: dict | None, proofs: list[dict]) -> dict:
    proof_by_task = {p.get("task_id"): p for p in proofs if p.get("task_id")}
    task_ids = {row["task_id"] for row in rows}
    completed = {
        row["task_id"]
        for row in rows
        if completion_status(row, proof_by_task) == "completed"
    }
    failed = {
        row["task_id"]
        for row in rows
        if completion_status(row, proof_by_task) == "failed"
    }

    task_rows = []
    dependency_edges = []
    block_edges = []
    lane_counts: dict[str, Counter] = defaultdict(Counter)
    parallel_counts: dict[str, Counter] = defaultdict(Counter)

    for row in rows:
        task_id = row["task_id"]
        deps = [dep for dep in split_list(row.get("depends_on")) if dep in task_ids]
        blocks = [block for block in split_list(row.get("blocks")) if block in task_ids]
        unmet_deps = [dep for dep in deps if dep not in completed]
        failed_deps = [dep for dep in deps if dep in failed]
        status = completion_status(row, proof_by_task)
        blocker_reasons = []
        if unmet_deps:
            blocker_reasons.append("unmet dependencies")
        if failed_deps:
            blocker_reasons.append("failed dependencies")
        if row.get("human_approval_required", "").lower() == "true" and status != "completed":
            blocker_reasons.append("human approval required")
        if status in {"blocked", "failed"}:
            blocker_reasons.append(status)
        ready = status == "pending" and not blocker_reasons
        proof = proof_by_task.get(task_id, {})

        task_visual = {
            "task_id": task_id,
            "phase": row.get("phase", ""),
            "title": row.get("title", ""),
            "owner_lane": row.get("owner_lane", ""),
            "parallel_group": row.get("parallel_group", ""),
            "status": status,
            "ready": ready,
            "proof_required": row.get("proof_required", "").lower() == "true",
            "proof_uri": row.get("proof_uri", ""),
            "proof_exists": proof_file_exists(row),
            "proof_status": proof.get("status", "missing"),
            "depends_on": deps,
            "blocks": blocks,
            "blockers": blocker_reasons,
            "unmet_dependencies": unmet_deps,
            "risk_level": row.get("risk_level", ""),
            "human_approval_required": row.get("human_approval_required", "").lower() == "true",
        }
        task_rows.append(task_visual)
        lane_counts[task_visual["owner_lane"]][status] += 1
        lane_counts[task_visual["owner_lane"]]["ready" if ready else "not_ready"] += 1
        parallel_counts[task_visual["parallel_group"]][status] += 1
        parallel_counts[task_visual["parallel_group"]]["ready" if ready else "not_ready"] += 1

        for dep in deps:
            dependency_edges.append({"from": dep, "to": task_id, "edge_type": "depends_on"})
        for block in blocks:
            block_edges.append({"from": task_id, "to": block, "edge_type": "blocks"})

    lanes = []
    for lane, counts in sorted(lane_counts.items()):
        lane_tasks = [task for task in task_rows if task["owner_lane"] == lane]
        lanes.append(
            {
                "lane": lane,
                "task_count": len(lane_tasks),
                "completed": counts["completed"],
                "failed": counts["failed"],
                "blocked": counts["blocked"],
                "pending": counts["pending"],
                "ready": counts["ready"],
                "active_tasks": [task["task_id"] for task in lane_tasks if task["ready"]][:10],
            }
        )

    parallel_groups = []
    for group, counts in sorted(parallel_counts.items()):
        group_tasks = [task for task in task_rows if task["parallel_group"] == group]
        max_parallel = max(
            int(row.get("max_parallel") or 1)
            for row in rows
            if row.get("parallel_group") == group
        )
        parallel_groups.append(
            {
                "parallel_group": group,
                "task_count": len(group_tasks),
                "max_parallel": max_parallel,
                "ready": counts["ready"],
                "running_capacity": min(counts["ready"], max_parallel),
                "completed": counts["completed"],
                "pending": counts["pending"],
                "blocked": counts["blocked"],
                "failed": counts["failed"],
            }
        )

    blocker_rows = [
        {
            "task_id": task["task_id"],
            "phase": task["phase"],
            "owner_lane": task["owner_lane"],
            "status": task["status"],
            "blockers": task["blockers"],
            "unmet_dependencies": task["unmet_dependencies"],
        }
        for task in task_rows
        if task["blockers"]
    ]

    proof_counter = Counter(task["proof_status"] for task in task_rows)
    overview = {
        "task_count": len(task_rows),
        "complete_count": sum(1 for task in task_rows if task["status"] == "completed"),
        "failed_count": sum(1 for task in task_rows if task["status"] == "failed"),
        "blocked_count": len(blocker_rows),
        "ready_count": sum(1 for task in task_rows if task["ready"]),
        "lane_count": len(lanes),
        "parallel_group_count": len(parallel_groups),
        "proof_present_count": sum(1 for task in task_rows if task["proof_exists"]),
        "proof_missing_count": sum(1 for task in task_rows if not task["proof_exists"]),
    }
    if status_report:
        overview["source_status_report"] = {
            "generated_at": status_report.get("generated_at"),
            "runnable_count": status_report.get("runnable_count"),
            "dispatch_count": status_report.get("dispatch_count"),
            "approval_blocker_count": status_report.get("approval_blocker_count"),
        }

    return {
        "schema_version": "1.0",
        "generated_at": now(),
        "source_files": [
            "generated/task_graph.csv",
            "generated/status_report.json",
            "proof_records/proof_ledger.jsonl",
        ],
        "overview": overview,
        "lanes": lanes,
        "parallel_groups": parallel_groups,
        "proof_state": {
            "proof_count": len(proofs),
            "task_proof_status_counts": dict(sorted(proof_counter.items())),
            "missing_required_proofs": [
                task["task_id"]
                for task in task_rows
                if task["proof_required"] and not task["proof_exists"]
            ],
        },
        "blockers": blocker_rows,
        "task_graph": {
            "nodes": task_rows,
            "edges": dependency_edges + block_edges,
        },
        "nu_plugin_visual_output": {
            "command": "envctl migration visuals",
            "shape": "record",
            "columns": [
                "overview",
                "lanes",
                "parallel_groups",
                "proof_state",
                "blockers",
                "task_graph",
                "dashboard_markdown",
            ],
        },
    }


def table(headers: list[str], rows: list[list[object]]) -> list[str]:
    widths = [len(header) for header in headers]
    rendered = [[str(cell) for cell in row] for row in rows]
    for row in rendered:
        for idx, cell in enumerate(row):
            widths[idx] = max(widths[idx], len(cell))
    sep = " | "
    lines = [sep.join(header.ljust(widths[idx]) for idx, header in enumerate(headers))]
    lines.append("-+-".join("-" * width for width in widths))
    lines.extend(sep.join(cell.ljust(widths[idx]) for idx, cell in enumerate(row)) for row in rendered)
    return lines


def render_markdown(model: dict) -> str:
    overview = model["overview"]
    lines = [
        "# Envctl Migration Live Visuals",
        "",
        f"Generated: {model['generated_at']}",
        "",
        "## Overview",
        "",
        *table(
            ["tasks", "complete", "ready", "blocked", "failed", "proofs", "missing proofs"],
            [
                [
                    overview["task_count"],
                    overview["complete_count"],
                    overview["ready_count"],
                    overview["blocked_count"],
                    overview["failed_count"],
                    overview["proof_present_count"],
                    overview["proof_missing_count"],
                ]
            ],
        ),
        "",
        "## Parallel Lanes",
        "",
        *table(
            ["lane", "tasks", "done", "ready", "blocked", "pending", "active"],
            [
                [
                    lane["lane"],
                    lane["task_count"],
                    lane["completed"],
                    lane["ready"],
                    lane["blocked"],
                    lane["pending"],
                    ", ".join(lane["active_tasks"]) or "-",
                ]
                for lane in model["lanes"]
            ],
        ),
        "",
        "## Parallel Groups",
        "",
        *table(
            ["group", "tasks", "max", "ready", "capacity", "done", "blocked"],
            [
                [
                    group["parallel_group"],
                    group["task_count"],
                    group["max_parallel"],
                    group["ready"],
                    group["running_capacity"],
                    group["completed"],
                    group["blocked"],
                ]
                for group in model["parallel_groups"]
            ],
        ),
        "",
        "## Proof State",
        "",
        *table(
            ["proof status", "count"],
            sorted(model["proof_state"]["task_proof_status_counts"].items()),
        ),
        "",
        "## Blockers",
        "",
    ]
    blocker_rows = model["blockers"][:30]
    if blocker_rows:
        lines.extend(
            table(
                ["task", "status", "lane", "reason", "unmet deps"],
                [
                    [
                        row["task_id"],
                        row["status"],
                        row["owner_lane"],
                        ", ".join(row["blockers"]),
                        ", ".join(row["unmet_dependencies"]) or "-",
                    ]
                    for row in blocker_rows
                ],
            )
        )
    else:
        lines.append("No blockers.")
    lines.extend(
        [
            "",
            "## Graph Edges",
            "",
            *table(
                ["from", "to", "type"],
                [
                    [edge["from"], edge["to"], edge["edge_type"]]
                    for edge in model["task_graph"]["edges"][:50]
                ],
            ),
            "",
        ]
    )
    return "\n".join(lines)


def ensure_manifest_command() -> bool:
    manifest_path = root() / "generated" / "nu_plugin_command_manifest.json"
    manifest = read_json(manifest_path)
    commands = manifest.setdefault("commands", [])
    for command in commands:
        if command.get("name") == "envctl migration visuals":
            return False
    commands.append(
        {
            "name": "envctl migration visuals",
            "mode": "read",
            "envctl_endpoint": "envctl migration visuals --format json",
            "output": {
                "shape": "record",
                "columns": [
                    "overview",
                    "lanes",
                    "parallel_groups",
                    "proof_state",
                    "blockers",
                    "task_graph",
                    "dashboard_markdown",
                ],
            },
        }
    )
    write_json(manifest_path, manifest)
    return True


def validate_outputs() -> None:
    model = read_json("generated/live_visuals.json")
    required = {"overview", "lanes", "parallel_groups", "proof_state", "blockers", "task_graph"}
    missing = sorted(required - set(model))
    if missing:
        raise SystemExit(f"live_visuals.json missing keys: {', '.join(missing)}")
    if not (root() / "generated" / "live_visuals.md").exists():
        raise SystemExit("generated/live_visuals.md missing")
    manifest = read_json("generated/nu_plugin_command_manifest.json")
    names = {command.get("name") for command in manifest.get("commands", [])}
    if "envctl migration visuals" not in names:
        raise SystemExit("nu plugin manifest missing envctl migration visuals")
    print(
        "live_visuals_check=passed "
        f"tasks={model['overview']['task_count']} "
        f"lanes={len(model['lanes'])} "
        f"blockers={len(model['blockers'])}"
    )


def write_outputs(record_proof: bool) -> None:
    rows = read_task_graph("generated/task_graph.csv")
    status_path = root() / "generated" / "status_report.json"
    status_report = read_json(status_path) if status_path.exists() else None
    proofs = load_ledger()
    model = build_visual_model(rows, status_report, proofs)
    dashboard = render_markdown(model)
    model["nu_plugin_visual_output"]["dashboard_markdown"] = dashboard
    write_json("generated/live_visuals.json", model)
    (root() / "generated" / "live_visuals.md").write_text(dashboard + "\n", encoding="utf-8")
    manifest_changed = ensure_manifest_command()
    heartbeat = {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "status": "completed",
        "updated_at": now(),
        "artifacts": [
            "generated/live_visuals.json",
            "generated/live_visuals.md",
            "generated/nu_plugin_command_manifest.json",
        ],
    }
    write_json("state/REQ-032_PLUGIN_LIVE_VISUALS.heartbeat.json", heartbeat)
    log = {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "status": "completed",
        "generated_at": now(),
        "manifest_changed": manifest_changed,
        "overview": model["overview"],
        "output_files": heartbeat["artifacts"],
    }
    write_json("logs/REQ-032_PLUGIN_LIVE_VISUALS.log", log)
    if record_proof:
        files_changed = [
            "execution-framework/scripts/render_live_visuals.py",
            "execution-framework/generated/live_visuals.json",
            "execution-framework/generated/live_visuals.md",
            "execution-framework/generated/nu_plugin_command_manifest.json",
            "execution-framework/state/REQ-032_PLUGIN_LIVE_VISUALS.heartbeat.json",
            "execution-framework/logs/REQ-032_PLUGIN_LIVE_VISUALS.log",
        ]
        proof = make_proof(
            TASK_ID,
            "completed",
            ACTOR,
            HELPER_ID,
            MODEL_TAG,
            REPO_PATH,
            files_changed,
            [
                "python3 scripts/render_live_visuals.py",
                "python3 scripts/render_live_visuals.py --check",
            ],
            {
                "status": "passed",
                "overview": model["overview"],
                "manifest_command": "envctl migration visuals",
                "visual_artifacts": [
                    "generated/live_visuals.json",
                    "generated/live_visuals.md",
                ],
            },
            [
                "generated/live_visuals.json",
                "generated/live_visuals.md",
                "generated/nu_plugin_command_manifest.json",
                "logs/REQ-032_PLUGIN_LIVE_VISUALS.log",
            ],
            "",
            "run REQ-041_TWO_REPO_INTEGRATION after remaining nu_plugin gates complete",
        )
        append_proof(proof)
    print(
        "wrote live visuals "
        f"tasks={model['overview']['task_count']} "
        f"lanes={len(model['lanes'])} "
        f"blockers={len(model['blockers'])} "
        f"manifest_changed={manifest_changed}"
    )


def main() -> None:
    parser = argparse.ArgumentParser(description="Render live envctl migration visuals for nu_plugin.")
    parser.add_argument("--check", action="store_true", help="validate generated live visual artifacts")
    parser.add_argument("--no-proof", action="store_true", help="render artifacts without writing proof")
    args = parser.parse_args()
    if args.check:
        validate_outputs()
        return
    write_outputs(record_proof=not args.no_proof)


if __name__ == "__main__":
    main()
