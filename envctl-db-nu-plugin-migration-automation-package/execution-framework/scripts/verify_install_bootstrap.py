from __future__ import annotations

import json
import shlex
from pathlib import Path

from _common import append_proof, file_checksums, now, package_root, root, write_json


TASK_ID = "REQ-044_INSTALL_BOOTSTRAP"
HELPER_ID = "helper-bootstrap-01"
MODEL_TAG = "gpt-5.3-spark"


def read_text(base: Path, relpath: str) -> str:
    return (base / relpath).read_text(encoding="utf-8")


def command_template(command_id: str, title: str, cwd: str, command: list[str], purpose: str) -> dict:
    return {
        "command_id": command_id,
        "title": title,
        "cwd": cwd,
        "argv": command,
        "shell": "bash",
        "purpose": purpose,
        "requires": ["codex", "python3", "bash"],
        "blocked_inputs": ["**/.env", "**/secrets/**", "**/private_keys/**", "**/*.pem", "**/*.key"],
    }


def build_manifest(base: Path) -> dict:
    pkg = "${PROMPT_PACKAGE_DIR:-/path/to/envctl-db-nu-plugin-migration-automation-package}"
    envctl = "${ENVCTL_REPO:?set ENVCTL_REPO}"
    nu_plugin = "${NU_PLUGIN_REPO:?set NU_PLUGIN_REPO}"
    flexnetos = "${FLEXNETOS_PACKAGE:-$PROMPT_PACKAGE_DIR/source/codex-flexnetos-migration-prompt-package}"
    packet = "generated/execution_packets/REQ-044_INSTALL_BOOTSTRAP.json"
    log = "logs/REQ-044_INSTALL_BOOTSTRAP.codex-exec.stdout.log"

    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "generated_at": now(),
        "source_files": [
            "INSTALL_IN_REPOS.sh",
            "RUN_WITH_CODEX_ENVCTL.sh",
            "execution-framework/README.md",
        ],
        "install_targets": [
            {
                "target_id": "envctl",
                "repo_env": "ENVCTL_REPO",
                "installed_paths": [
                    "AGENTS.envctl-migration-automation.md",
                    ".codex/prompts/envctl-migration-automation",
                    ".codex/prompts/envctl-migration-automation-schemas",
                    ".codex/prompts/envctl-migration-automation-sql",
                    ".codex/agents/envctl-migration-automation",
                ],
            },
            {
                "target_id": "nu_plugin",
                "repo_env": "NU_PLUGIN_REPO",
                "installed_paths": [
                    "AGENTS.nu-plugin-migration-automation.md",
                    ".codex/prompts/envctl-migration-automation",
                    ".codex/prompts/envctl-migration-automation-schemas",
                    ".codex/agents/envctl-migration-automation",
                ],
            },
        ],
        "command_templates": [
            command_template(
                "install-into-repos",
                "Install additive prompt assets into envctl and nu_plugin",
                pkg,
                [
                    "bash",
                    "INSTALL_IN_REPOS.sh",
                    "--envctl-repo",
                    envctl,
                    "--nu-plugin-repo",
                    nu_plugin,
                ],
                "Copy namespaced Codex prompts, schemas, SQL, and helper configs into both local repositories.",
            ),
            command_template(
                "run-codex-master-prompt",
                "Run Codex with the package master prompt",
                pkg,
                [
                    "bash",
                    "RUN_WITH_CODEX_ENVCTL.sh",
                    "--envctl-repo",
                    envctl,
                    "--nu-plugin-repo",
                    nu_plugin,
                    "--flexnetos-package",
                    flexnetos,
                ],
                "Start Codex from the envctl repo with the combined migration automation package prompt.",
            ),
            command_template(
                "run-single-execution-packet",
                "Run one bounded execution packet",
                f"{pkg}/execution-framework",
                ["codex", "exec", "<", packet],
                "Execute a single generated packet and require its proof record before advancing dependent tasks.",
            ),
            command_template(
                "run-background-helper",
                "Run one packet as a background helper",
                f"{pkg}/execution-framework",
                ["bash", "-lc", f"mkdir -p logs state proof_records && nohup codex exec < {shlex.quote(packet)} > {shlex.quote(log)} 2>&1 & echo $!"],
                "Launch a bounded packet in a background shell while preserving stdout/stderr evidence under logs/.",
            ),
        ],
        "proof_contract": {
            "proof_required": True,
            "proof_uri": "execution-framework/proof_records/REQ-044_INSTALL_BOOTSTRAP.proof.json",
            "heartbeat_file": "execution-framework/state/REQ-044_INSTALL_BOOTSTRAP.heartbeat.json",
            "logs_uri": "execution-framework/logs/REQ-044_INSTALL_BOOTSTRAP.log",
            "completion_gate": "proof exists and verification output passes",
        },
        "rollback_point": "history/pre_execution_framework_manifest.json",
    }


def render_docs(manifest: dict) -> str:
    lines = [
        "# Install Bootstrap",
        "",
        "This page is the verified operator entrypoint for installing the package into local `envctl` and `nu_plugin` checkouts and starting Codex/background helpers from generated execution packets.",
        "",
        "## Environment",
        "",
        "Set these variables before running the templates:",
        "",
        "```bash",
        "export PROMPT_PACKAGE_DIR=/path/to/envctl-db-nu-plugin-migration-automation-package",
        "export ENVCTL_REPO=/path/to/envctl",
        "export NU_PLUGIN_REPO=/path/to/nu_plugin",
        "export FLEXNETOS_PACKAGE=$PROMPT_PACKAGE_DIR/source/codex-flexnetos-migration-prompt-package",
        "```",
        "",
        "## Repo Install",
        "",
        "```bash",
        "cd \"$PROMPT_PACKAGE_DIR\"",
        "bash INSTALL_IN_REPOS.sh --envctl-repo \"$ENVCTL_REPO\" --nu-plugin-repo \"$NU_PLUGIN_REPO\"",
        "```",
        "",
        "The installer writes only namespaced additive files into the target repositories. The expected target paths are recorded in `generated/install_bootstrap_manifest.json`.",
        "",
        "## Codex Bootstrap",
        "",
        "```bash",
        "cd \"$PROMPT_PACKAGE_DIR\"",
        "bash RUN_WITH_CODEX_ENVCTL.sh \\",
        "  --envctl-repo \"$ENVCTL_REPO\" \\",
        "  --nu-plugin-repo \"$NU_PLUGIN_REPO\" \\",
        "  --flexnetos-package \"$FLEXNETOS_PACKAGE\"",
        "```",
        "",
        "## Bounded Packet Execution",
        "",
        "```bash",
        "cd \"$PROMPT_PACKAGE_DIR/execution-framework\"",
        "codex exec < generated/execution_packets/REQ-044_INSTALL_BOOTSTRAP.json",
        "```",
        "",
        "## Background Helper Template",
        "",
        "```bash",
        "cd \"$PROMPT_PACKAGE_DIR/execution-framework\"",
        "mkdir -p logs state proof_records",
        "nohup codex exec < generated/execution_packets/REQ-044_INSTALL_BOOTSTRAP.json \\",
        "  > logs/REQ-044_INSTALL_BOOTSTRAP.codex-exec.stdout.log 2>&1 &",
        "echo $!",
        "```",
        "",
        "Every helper must produce the task proof named by its packet before dependent work advances.",
        "",
        "## Verification",
        "",
        "```bash",
        "cd \"$PROMPT_PACKAGE_DIR/execution-framework\"",
        "python3 scripts/verify_install_bootstrap.py",
        "test -s proof_records/REQ-044_INSTALL_BOOTSTRAP.proof.json",
        "```",
        "",
        "The verifier checks the package install scripts, regenerates the command template manifest, updates the heartbeat, writes the log, and appends the proof ledger entry.",
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


def validate_manifest(base: Path, manifest: dict) -> dict:
    errors = []
    install_script = read_text(base, "INSTALL_IN_REPOS.sh")
    runner_script = read_text(base, "RUN_WITH_CODEX_ENVCTL.sh")
    framework_readme = read_text(root(), "README.md")

    required_install_tokens = [
        "--envctl-repo",
        "--nu-plugin-repo",
        "AGENTS.envctl.md.template",
        "AGENTS.nu_plugin.md.template",
        ".codex/prompts/envctl-migration-automation",
        ".codex/agents/envctl-migration-automation",
    ]
    required_runner_tokens = [
        "--envctl-repo",
        "--nu-plugin-repo",
        "--flexnetos-package",
        "--codex-bin",
        "PROMPT_PACKAGE_COMBINED.md",
        "codex/envctl-nu-plugin-migration.config.toml",
    ]
    for token in required_install_tokens:
        if token not in install_script:
            errors.append(f"INSTALL_IN_REPOS.sh missing {token}")
    for token in required_runner_tokens:
        if token not in runner_script:
            errors.append(f"RUN_WITH_CODEX_ENVCTL.sh missing {token}")
    if "scripts/verify_install_bootstrap.py" not in framework_readme:
        errors.append("execution-framework/README.md does not reference verify_install_bootstrap.py")

    ids = [template["command_id"] for template in manifest["command_templates"]]
    if len(ids) != len(set(ids)):
        errors.append("command template IDs are not unique")
    for template in manifest["command_templates"]:
        if not template.get("argv"):
            errors.append(f"{template['command_id']} has no argv template")
        joined = " ".join(template.get("argv", []))
        for blocked in template.get("blocked_inputs", []):
            if blocked.replace("**/", "") in joined:
                errors.append(f"{template['command_id']} references blocked input pattern {blocked}")

    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "status": "passed" if not errors else "failed",
        "generated_at": manifest["generated_at"],
        "summary": {
            "source_file_count": len(manifest["source_files"]),
            "install_target_count": len(manifest["install_targets"]),
            "command_template_count": len(manifest["command_templates"]),
        },
        "command_template_ids": ids,
        "errors": errors,
        "evidence": [
            "execution-framework/docs/INSTALL_BOOTSTRAP.md",
            "execution-framework/generated/install_bootstrap_manifest.json",
            "execution-framework/state/REQ-044_INSTALL_BOOTSTRAP.heartbeat.json",
            "execution-framework/logs/REQ-044_INSTALL_BOOTSTRAP.log",
        ],
    }


def write_text(relpath: str, text: str) -> None:
    path = root() / relpath
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def main() -> int:
    base = package_root()
    started_at = now()
    manifest = build_manifest(base)
    write_json("generated/install_bootstrap_manifest.json", manifest)
    write_text("docs/INSTALL_BOOTSTRAP.md", render_docs(manifest))
    validation = validate_manifest(base, manifest)
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
        "execution-framework/scripts/verify_install_bootstrap.py",
        "execution-framework/docs/INSTALL_BOOTSTRAP.md",
        "execution-framework/generated/install_bootstrap_manifest.json",
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
        "repo_path": str(base),
        "files_changed": files_changed + [f"execution-framework/proof_records/{TASK_ID}.proof.json"],
        "commands_run": ["python3 scripts/verify_install_bootstrap.py"],
        "verification_output": validation,
        "checksums": file_checksums(files_changed),
        "logs_uri": f"logs/{TASK_ID}.log",
        "rollback_point": "history/pre_execution_framework_manifest.json",
        "evidence": validation["evidence"],
        "failure_reason": "; ".join(validation["errors"]),
        "next_action": "run REQ-045_RUN_REPLAY with generated install/bootstrap command templates",
    }
    append_proof(proof)
    return 0 if validation["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
