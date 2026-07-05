from __future__ import annotations

import json
from pathlib import Path

from _common import append_proof, file_checksums, now, package_root, root, write_json


TASK_ID = "REQ-045_RUN_REPLAY"
HELPER_ID = "helper-replay-template-01"
MODEL_TAG = "gpt-5.3-spark"


def write_text(relpath: str, text: str) -> None:
    path = root() / relpath
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def command_template(command_id: str, title: str, cwd: str, argv: list[str], purpose: str) -> dict:
    return {
        "command_id": command_id,
        "title": title,
        "cwd": cwd,
        "argv": argv,
        "shell": "bash",
        "purpose": purpose,
        "requires": ["codex", "python3", "bash"],
        "blocked_inputs": ["**/.env", "**/secrets/**", "**/private_keys/**", "**/*.pem", "**/*.key"],
    }


def build_manifest() -> dict:
    framework = "${PROMPT_PACKAGE_DIR:-/path/to/envctl-db-nu-plugin-migration-automation-package}/execution-framework"
    packet = "generated/execution_packets/REQ-045_RUN_REPLAY.json"
    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "generated_at": now(),
        "depends_on": [
            "REQ-044_INSTALL_BOOTSTRAP",
            "REQ-027_ENVCTL_REPLAY_ENGINE",
            "APPROVAL-REQ-045_RUN_REPLAY",
        ],
        "source_files": [
            "execution-framework/generated/execution_manifest.json",
            "execution-framework/generated/execution_packets/REQ-045_RUN_REPLAY.json",
            "execution-framework/docs/ENVCTL_REPLAY_ENGINE.md",
            "execution-framework/docs/ENVCTL_ROLLBACK_CHECKPOINTS.md",
            "execution-framework/docs/INSTALL_BOOTSTRAP.md",
        ],
        "convenience_templates": [
            {
                "path": "execution-templates/README.md",
                "writable_in_this_runtime": False,
                "role": "read-only top-level convenience copy index",
            },
            {
                "path": "execution-templates/EXECUTION_TEMPLATE_INDEX.json",
                "writable_in_this_runtime": False,
                "role": "read-only top-level convenience template list",
            },
        ],
        "command_templates": [
            command_template(
                "compute-initial-plan",
                "Compute the current runnable plan from the task graph",
                framework,
                ["python3", "scripts/goal_loop.py", "generated/task_graph.csv"],
                "Regenerate runnable and approval-blocked packet state before dispatching replay-related work.",
            ),
            command_template(
                "dispatch-approved-run-replay-packet",
                "Dispatch the approved REQ-045 execution packet",
                framework,
                ["bash", "-lc", f"codex exec < {packet}"],
                "Run the bounded REQ-045 packet after its approval artifact has been recorded.",
            ),
            command_template(
                "replay-dry-run",
                "Run deterministic envctl replay dry-run",
                framework,
                [
                    "envctl",
                    "replay",
                    "dry-run",
                    "--run-id",
                    "run-req027-replay",
                    "--replay-id",
                    "replay-req045-dry-run",
                    "--requested-by",
                    HELPER_ID,
                    "--operation-ids",
                    "op-req027-replay-hash",
                    "--reason",
                    "verify deterministic replay inputs before integration validation",
                ],
                "Replay the deterministic operation from REQ-027 without mutating targets.",
            ),
            command_template(
                "replay-apply-blocked-check",
                "Prove apply replay stays approval-gated",
                framework,
                [
                    "envctl",
                    "replay",
                    "apply",
                    "--run-id",
                    "run-req027-replay",
                    "--replay-id",
                    "replay-req045-apply-blocked",
                    "--requested-by",
                    HELPER_ID,
                    "--operation-ids",
                    "op-req027-manual-cutover",
                    "--reason",
                    "prove approval-gated apply replay remains blocked without operator release",
                ],
                "Exercise the high-risk replay path in a way that must remain blocked until an operator approval exists.",
            ),
            command_template(
                "verify-rollback-checkpoints",
                "Validate rollback checkpoint safety checks",
                framework,
                ["python3", "scripts/verify_rollback_checkpoints.py"],
                "Confirm rollback handles and checkpoint protections remain valid before replay execution advances.",
            ),
            command_template(
                "verify-replay-validation",
                "Validate replay engine and task readiness",
                framework,
                ["bash", "-lc", "python3 scripts/verify_replay_engine.py && python3 scripts/verify_history_and_completeness.py"],
                "Re-run replay verification plus completeness checks after command-template generation.",
            ),
        ],
        "proof_contract": {
            "proof_required": True,
            "proof_uri": "execution-framework/proof_records/REQ-045_RUN_REPLAY.proof.json",
            "heartbeat_file": "execution-framework/state/REQ-045_RUN_REPLAY.heartbeat.json",
            "logs_uri": "execution-framework/logs/REQ-045_RUN_REPLAY.log",
            "completion_gate": "proof exists and verification output passes",
        },
        "rollback_point": "history/pre_execution_framework_manifest.json",
    }


def render_docs(manifest: dict) -> str:
    lines = [
        "# Run Replay",
        "",
        "This page is the verified operator entrypoint for computing the current plan, dispatching the approved REQ-045 packet, exercising replay checks, validating rollback protections, and re-running validation.",
        "",
        "## Preconditions",
        "",
        "- `REQ-044_INSTALL_BOOTSTRAP` proof is complete.",
        "- `REQ-027_ENVCTL_REPLAY_ENGINE` proof is complete.",
        "- `APPROVAL-REQ-045_RUN_REPLAY` approval proof is complete.",
        "",
        "## Initial Plan",
        "",
        "```bash",
        "cd \"$PROMPT_PACKAGE_DIR/execution-framework\"",
        "python3 scripts/goal_loop.py generated/task_graph.csv",
        "```",
        "",
        "Use the refreshed `state/goal_loop_state.json` before dispatching helpers.",
        "",
        "## Dispatch Approved Packet",
        "",
        "```bash",
        "cd \"$PROMPT_PACKAGE_DIR/execution-framework\"",
        "codex exec < generated/execution_packets/REQ-045_RUN_REPLAY.json",
        "```",
        "",
        "## Replay Dry-Run",
        "",
        "```bash",
        "cd \"$PROMPT_PACKAGE_DIR/execution-framework\"",
        "envctl replay dry-run \\",
        "  --run-id run-req027-replay \\",
        "  --replay-id replay-req045-dry-run \\",
        "  --requested-by helper-replay-template-01 \\",
        "  --operation-ids op-req027-replay-hash \\",
        "  --reason \"verify deterministic replay inputs before integration validation\"",
        "```",
        "",
        "## Replay Apply Blocked Check",
        "",
        "```bash",
        "cd \"$PROMPT_PACKAGE_DIR/execution-framework\"",
        "envctl replay apply \\",
        "  --run-id run-req027-replay \\",
        "  --replay-id replay-req045-apply-blocked \\",
        "  --requested-by helper-replay-template-01 \\",
        "  --operation-ids op-req027-manual-cutover \\",
        "  --reason \"prove approval-gated apply replay remains blocked without operator release\"",
        "```",
        "",
        "This command is expected to remain blocked until the operator approval path is satisfied.",
        "",
        "## Rollback Safety Checks",
        "",
        "```bash",
        "cd \"$PROMPT_PACKAGE_DIR/execution-framework\"",
        "python3 scripts/verify_rollback_checkpoints.py",
        "```",
        "",
        "## Validation",
        "",
        "```bash",
        "cd \"$PROMPT_PACKAGE_DIR/execution-framework\"",
        "python3 scripts/verify_replay_engine.py",
        "python3 scripts/verify_history_and_completeness.py",
        "test -s proof_records/REQ-045_RUN_REPLAY.proof.json",
        "```",
        "",
        "## Convenience Copies",
        "",
        "Top-level `execution-templates/` remains a read-only convenience surface in this runtime. The canonical generated run/replay artifacts for this task live under `execution-framework/` and the verifier only checks that the convenience copy index remains present.",
        "",
        "## Command Template Index",
        "",
        "| command id | purpose |",
        "|---|---|",
    ]
    for template in manifest["command_templates"]:
        lines.append(f"| `{template['command_id']}` | {template['purpose']} |")
    lines.append("")
    return "\n".join(lines)


def validate_manifest(manifest: dict) -> dict:
    errors: list[str] = []
    evidence = [
        "execution-framework/docs/RUN_REPLAY.md",
        "execution-framework/generated/run_replay_manifest.json",
        "execution-framework/state/REQ-045_RUN_REPLAY.heartbeat.json",
        "execution-framework/logs/REQ-045_RUN_REPLAY.log",
    ]
    required_paths = [
        root() / "generated" / "execution_manifest.json",
        root() / "generated" / "execution_packets" / f"{TASK_ID}.json",
        root() / "docs" / "ENVCTL_REPLAY_ENGINE.md",
        root() / "docs" / "ENVCTL_ROLLBACK_CHECKPOINTS.md",
        root() / "docs" / "INSTALL_BOOTSTRAP.md",
        root() / "approvals" / f"{TASK_ID}.agent_approval.json",
        root() / "proof_records" / "REQ-044_INSTALL_BOOTSTRAP.proof.json",
        root() / "proof_records" / "REQ-027_ENVCTL_REPLAY_ENGINE.proof.json",
        root() / "proof_records" / f"APPROVAL-{TASK_ID}.proof.json",
        package_root() / "execution-templates" / "README.md",
        package_root() / "execution-templates" / "EXECUTION_TEMPLATE_INDEX.json",
    ]
    missing = [str(path.relative_to(package_root())) for path in required_paths if not path.exists()]
    if missing:
        errors.extend(f"missing required input: {item}" for item in missing)

    readme_text = (root() / "README.md").read_text(encoding="utf-8")
    if "scripts/verify_run_replay.py" not in readme_text:
        errors.append("execution-framework/README.md does not reference verify_run_replay.py")

    ids = [template["command_id"] for template in manifest["command_templates"]]
    if len(ids) != len(set(ids)):
        errors.append("command template IDs are not unique")
    for template in manifest["command_templates"]:
        argv = template.get("argv") or []
        if not argv:
            errors.append(f"{template['command_id']} has no argv template")
            continue
        joined = " ".join(argv)
        for blocked in template.get("blocked_inputs", []):
            token = blocked.replace("**/", "")
            if token in joined:
                errors.append(f"{template['command_id']} references blocked input pattern {blocked}")

    template_index = json.loads((package_root() / "execution-templates" / "EXECUTION_TEMPLATE_INDEX.json").read_text(encoding="utf-8"))
    indexed_templates = set(template_index.get("templates") or [])
    if "TASK_GRAPH_TEMPLATE.csv" not in indexed_templates:
        errors.append("execution-templates/EXECUTION_TEMPLATE_INDEX.json is missing TASK_GRAPH_TEMPLATE.csv")

    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "status": "passed" if not errors else "failed",
        "generated_at": manifest["generated_at"],
        "summary": {
            "source_file_count": len(manifest["source_files"]),
            "command_template_count": len(manifest["command_templates"]),
            "convenience_template_count": len(manifest["convenience_templates"]),
        },
        "command_template_ids": ids,
        "convenience_templates_present": [item["path"] for item in manifest["convenience_templates"]],
        "errors": errors,
        "evidence": evidence,
    }


def main() -> int:
    started_at = now()
    manifest = build_manifest()
    write_json("generated/run_replay_manifest.json", manifest)
    write_text("docs/RUN_REPLAY.md", render_docs(manifest))
    validation = validate_manifest(manifest)
    write_json(f"logs/{TASK_ID}.log", validation)
    write_json(
        f"state/{TASK_ID}.heartbeat.json",
        {
            "task_id": TASK_ID,
            "status": validation["status"],
            "updated_at": now(),
            "helper_id": HELPER_ID,
            "evidence": validation["evidence"],
        },
    )

    files_changed = [
        "execution-framework/README.md",
        "execution-framework/scripts/verify_run_replay.py",
        "execution-framework/docs/RUN_REPLAY.md",
        "execution-framework/generated/run_replay_manifest.json",
        f"execution-framework/state/{TASK_ID}.heartbeat.json",
        f"execution-framework/logs/{TASK_ID}.log",
    ]
    proof = {
        "proof_schema_version": "1.0",
        "task_id": TASK_ID,
        "status": "completed" if validation["status"] == "passed" else "failed",
        "started_at": started_at,
        "completed_at": now(),
        "actor": "codex-cli-local",
        "helper_id": HELPER_ID,
        "model_tag": MODEL_TAG,
        "repo_path": str(package_root()),
        "files_changed": files_changed + [f"execution-framework/proof_records/{TASK_ID}.proof.json"],
        "commands_run": ["python3 scripts/verify_run_replay.py"],
        "verification_output": validation,
        "checksums": file_checksums(files_changed),
        "logs_uri": f"logs/{TASK_ID}.log",
        "rollback_point": "history/pre_execution_framework_manifest.json",
        "evidence": validation["evidence"],
        "failure_reason": "; ".join(validation["errors"]),
        "next_action": "run goal_loop.py so REQ-045 proof state is reflected in dispatch status",
    }
    append_proof(proof)
    return 0 if validation["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
