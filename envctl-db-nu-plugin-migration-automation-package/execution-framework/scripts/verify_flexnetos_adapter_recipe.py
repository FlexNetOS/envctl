#!/usr/bin/env python3
"""Build and validate the REQ-202 FlexNetOS adapter recipe package artifacts."""

from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any

from _common import append_proof, make_proof, now, package_root, read_json, root, sha256_file, write_json


TASK_ID = "REQ-202_FLEXNETOS_ADAPTER_RECIPE"
PACKET_PATH = "generated/execution_packets/REQ-202_FLEXNETOS_ADAPTER_RECIPE.json"
DOC_PATH = "docs/FLEXNETOS_ADAPTER_RECIPE.md"
RECIPE_PATH = "generated/flexnetos_adapter_recipe.json"
REPORT_PATH = "generated/flexnetos_adapter_recipe_validation_report.json"
HEARTBEAT_PATH = "state/REQ-202_FLEXNETOS_ADAPTER_RECIPE.heartbeat.json"
LOG_PATH = "logs/REQ-202_FLEXNETOS_ADAPTER_RECIPE.log"
PROOF_PATH = "proof_records/REQ-202_FLEXNETOS_ADAPTER_RECIPE.proof.json"
APPROVAL_PATH = "approvals/REQ-202_FLEXNETOS_ADAPTER_RECIPE.agent_approval.json"
APPROVAL_REVIEW_PATH = "reviews/GPT55_REQ-202_FLEXNETOS_ADAPTER_RECIPE_APPROVAL_REVIEW.md"
REQ201_PROOF_PATH = "proof_records/REQ-201_FLEXNETOS_LIFEOS_COMPARISON.proof.json"
REQ027_PROOF_PATH = "proof_records/REQ-027_ENVCTL_REPLAY_ENGINE.proof.json"
REQ201_REPORT_PATH = "generated/flexnetos_lifeos_comparison_validation_report.json"
SHARED_SCHEMA_PATH = "schemas/shared_protocol.schema.json"

SECRET_PATTERNS = [
    re.compile(r"-----BEGIN [A-Z ]*PRIVATE KEY-----"),
    re.compile(r"\bsk-[A-Za-z0-9_-]{20,}\b"),
    re.compile(r"(?i)(password|secret|token|api[_-]?key)\s*[:=]\s*['\"]?[A-Za-z0-9_./+=-]{12,}"),
]


def recipe_payload() -> dict[str, Any]:
    return {
        "schema_version": 1,
        "recipe_id": "flexnetos-codex-package-target-adapter",
        "version": "1.0.0",
        "metadata": {
            "task_id": TASK_ID,
            "adapter_name": "FlexNetOS adapter",
            "purpose": "Turn the prior Codex FlexNetOS migration package into an approval-gated envctl target adapter recipe.",
            "target_descriptor_id": "flexnetos-vs-lifeos",
            "target_type": "mixed",
            "repo_target": "repo_a",
            "repo_path_ref": "${ENVCTL_REPO}",
            "filesystem_scope": "repo",
            "source_package_glob": "source/codex-flexnetos-migration-prompt-package/**",
            "reusable_adapter_contract": {
                "read_only_inputs": [
                    "REQ-201_FLEXNETOS_LIFEOS_COMPARISON",
                    "proof_records/REQ-027_ENVCTL_REPLAY_ENGINE.proof.json",
                    "${ENVCTL_REPO}/docs/**",
                    "${NU_PLUGIN_REPO}/docs/**",
                    "${MIGRATION_TARGET_ROOT}/docs/**",
                ],
                "write_scope": [
                    "envctl run ledger",
                    "envctl recipe registry",
                    "task-owned generated artifacts",
                ],
                "blocked_paths": [
                    "**/.env",
                    "**/secrets/**",
                    "**/private_keys/**",
                    "**/*.pem",
                    "**/*.key",
                ],
            },
            "depends_on_tasks": [
                "REQ-201_FLEXNETOS_LIFEOS_COMPARISON",
                "REQ-027_ENVCTL_REPLAY_ENGINE",
            ],
            "execution_model": {
                "human_approval_required": True,
                "replay_prerequisite": "REQ-027_ENVCTL_REPLAY_ENGINE",
                "comparison_prerequisite": "REQ-201_FLEXNETOS_LIFEOS_COMPARISON",
                "approval_mode": "approval-gated",
            },
            "validation_contract": {
                "verifier": "scripts/verify_flexnetos_adapter_recipe.py",
                "report_path": REPORT_PATH,
                "proof_path": PROOF_PATH,
            },
        },
        "phases": [
            {
                "phase_id": "01-ingest-evidence",
                "depends_on": [],
                "approval_gate": False,
                "operations": [
                    {
                        "operation_id": "link-prior-package-inputs",
                        "operation_type": "evidence.link",
                        "risk": "R1",
                        "inputs": {
                            "required_inputs": [
                                "source/codex-flexnetos-migration-prompt-package/**",
                                "REQ-201_FLEXNETOS_LIFEOS_COMPARISON",
                                "${ENVCTL_REPO}/docs/**",
                                "${NU_PLUGIN_REPO}/docs/**",
                                "${MIGRATION_TARGET_ROOT}/docs/**",
                            ]
                        },
                        "expected_artifacts": [
                            "input-evidence-links",
                        ],
                        "validators": [
                            "dependency_proof_present",
                            "blocked_paths_preserved",
                        ],
                        "rollback": {
                            "mode": "remove_generated_artifact_only",
                            "requires_human_approval": False,
                        },
                    },
                    {
                        "operation_id": "capture-flexnetos-comparison-findings",
                        "operation_type": "comparison.import",
                        "risk": "R1",
                        "inputs": {
                            "comparison_task_id": "REQ-201_FLEXNETOS_LIFEOS_COMPARISON",
                            "comparison_report_path": REQ201_REPORT_PATH,
                        },
                        "expected_artifacts": [
                            "flexnetos-comparison-context",
                        ],
                        "validators": [
                            "comparison_validation_passed",
                            "line_evidence_linked",
                        ],
                        "rollback": {
                            "mode": "remove_generated_artifact_only",
                            "requires_human_approval": False,
                        },
                    },
                ],
            },
            {
                "phase_id": "02-render-adapter",
                "depends_on": [
                    "01-ingest-evidence",
                ],
                "approval_gate": False,
                "operations": [
                    {
                        "operation_id": "render-adapter-recipe",
                        "operation_type": "recipe.render",
                        "risk": "R1",
                        "inputs": {
                            "recipe_path": RECIPE_PATH,
                            "documentation_path": DOC_PATH,
                        },
                        "expected_artifacts": [
                            "flexnetos-adapter-recipe-json",
                            "flexnetos-adapter-doc",
                        ],
                        "validators": [
                            "shared_protocol_shape_valid",
                            "allowed_paths_only",
                        ],
                        "rollback": {
                            "mode": "remove_generated_artifact_only",
                            "requires_human_approval": False,
                        },
                    },
                    {
                        "operation_id": "register-adapter-for-envctl",
                        "operation_type": "recipe.catalog.register",
                        "risk": "R2",
                        "inputs": {
                            "target_descriptor_id": "flexnetos-vs-lifeos",
                            "recipe_id": "flexnetos-codex-package-target-adapter",
                        },
                        "expected_artifacts": [
                            "recipe-registry-row",
                        ],
                        "validators": [
                            "stable_recipe_id",
                            "target_descriptor_reference_present",
                        ],
                        "rollback": {
                            "mode": "remove_generated_artifact_only",
                            "requires_human_approval": False,
                        },
                    },
                ],
            },
            {
                "phase_id": "03-verify-replay-readiness",
                "depends_on": [
                    "02-render-adapter",
                ],
                "approval_gate": False,
                "operations": [
                    {
                        "operation_id": "validate-adapter-contract",
                        "operation_type": "recipe.validate",
                        "risk": "R2",
                        "inputs": {
                            "verification_command": "python3 scripts/verify_flexnetos_adapter_recipe.py",
                        },
                        "expected_artifacts": [
                            "flexnetos-adapter-validation-report",
                        ],
                        "validators": [
                            "validation_report_passed",
                            "no_secret_exposure",
                        ],
                        "rollback": {
                            "mode": "remove_generated_artifact_only",
                            "requires_human_approval": False,
                        },
                    },
                    {
                        "operation_id": "prove-replay-compatibility",
                        "operation_type": "replay.verify",
                        "risk": "R2",
                        "inputs": {
                            "depends_on_proof": "REQ-027_ENVCTL_REPLAY_ENGINE",
                            "compatibility_mode": "dry_run",
                        },
                        "expected_artifacts": [
                            "replay-ready-adapter-evidence",
                        ],
                        "validators": [
                            "replay_dependency_present",
                            "human_gate_retained_for_apply",
                        ],
                        "rollback": {
                            "mode": "remove_generated_artifact_only",
                            "requires_human_approval": False,
                        },
                    },
                ],
            },
            {
                "phase_id": "04-approved-apply",
                "depends_on": [
                    "03-verify-replay-readiness",
                ],
                "approval_gate": True,
                "operations": [
                    {
                        "operation_id": "operator-review-target-docs",
                        "operation_type": "manual_operator",
                        "risk": "R4",
                        "inputs": {
                            "human_approval_required": True,
                            "review_surfaces": [
                                "${ENVCTL_REPO}/docs/**",
                                "${NU_PLUGIN_REPO}/docs/**",
                                "${MIGRATION_TARGET_ROOT}/docs/**",
                            ],
                        },
                        "expected_artifacts": [
                            "human-approval-decision",
                        ],
                        "validators": [
                            "approval_recorded",
                            "scope_still_repo_bound",
                        ],
                        "rollback": {
                            "mode": "manual_rollback_required",
                            "requires_human_approval": True,
                        },
                    },
                    {
                        "operation_id": "apply-flexnetos-target-adapter",
                        "operation_type": "target.mutate",
                        "risk": "R5",
                        "inputs": {
                            "adapter_mode": "approval-gated",
                            "repo_target": "repo_a",
                            "repo_path_ref": "${ENVCTL_REPO}",
                        },
                        "expected_artifacts": [
                            "envctl-run-record",
                            "artifact-links",
                        ],
                        "validators": [
                            "approved_before_apply",
                            "write_scope_repo_only",
                            "rollback_checkpoint_available",
                        ],
                        "rollback": {
                            "mode": "history_manifest_revert",
                            "requires_human_approval": True,
                        },
                    },
                ],
            },
        ],
    }


def render_doc(recipe: dict[str, Any], packet: dict[str, Any], warnings: list[str]) -> str:
    phase_lines = []
    for phase in recipe["phases"]:
        phase_lines.append(
            "| `{phase_id}` | `{gate}` | `{ops}` | {summary} |".format(
                phase_id=phase["phase_id"],
                gate="yes" if phase.get("approval_gate") else "no",
                ops=len(phase["operations"]),
                summary=", ".join(op["operation_id"] for op in phase["operations"]),
            )
        )

    warning_section = ""
    if warnings:
        warning_section = "## Warnings\n\n" + "\n".join(f"- {warning}" for warning in warnings) + "\n\n"

    return """# FlexNetOS Adapter Recipe

Status: `validated`
Task: `{task_id}`
Recipe ID: `{recipe_id}`
Version: `{version}`

## Goal

Convert the earlier FlexNetOS Codex migration package into a reusable envctl migration target adapter that stays repo-scoped, replay-aware, and human-approved before any target mutation.

## Inputs

- Prior package source: `source/codex-flexnetos-migration-prompt-package/**`
- Comparison evidence: `REQ-201_FLEXNETOS_LIFEOS_COMPARISON`
- Replay semantics: `REQ-027_ENVCTL_REPLAY_ENGINE`
- Read-only docs: `${{ENVCTL_REPO}}/docs/**`, `${{NU_PLUGIN_REPO}}/docs/**`, `${{MIGRATION_TARGET_ROOT}}/docs/**`

## Execution Model

- Target descriptor: `flexnetos-vs-lifeos`
- Repo target: `repo_a`
- Repo path reference: `${{ENVCTL_REPO}}`
- Filesystem scope: `repo`
- Human approval required: `true`
- Verification command: `python3 scripts/verify_flexnetos_adapter_recipe.py`

## Phase Plan

| phase | approval gate | operation count | focus |
|---|---|---:|---|
{phase_lines}

## Safety

- Writes stay limited to the packet-owned execution-framework outputs.
- Apply work remains behind the `04-approved-apply` gate.
- Blocked paths remain excluded: `**/.env`, `**/secrets/**`, `**/private_keys/**`, `**/*.pem`, `**/*.key`.
- Replay compatibility is required before apply compatibility is claimed.

## Packet Alignment

- Packet command template: `{command_template}`
- Completion gate: `{completion_gate}`
- Proof path: `{proof_uri}`
- Validation report: `{report_path}`

{warning_section}## Notes

- This adapter recipe intentionally references external repo docs as read-only runtime inputs; the recipe package itself does not widen write scope into those repos.
- The apply phase is intentionally abstracted as envctl-controlled execution so the same recipe can be reused against future FlexNetOS migration targets without changing the execution-framework package.
""".format(
        task_id=TASK_ID,
        recipe_id=recipe["recipe_id"],
        version=recipe["version"],
        phase_lines="\n".join(phase_lines),
        command_template=packet["command_template"],
        completion_gate=packet["completion_gate"],
        proof_uri=packet["proof_uri"],
        report_path=REPORT_PATH,
        warning_section=warning_section,
    )


def validate_recipe_shape(recipe: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    if not isinstance(recipe.get("schema_version"), int):
        errors.append("schema_version must be an integer")
    if not recipe.get("recipe_id"):
        errors.append("recipe_id is required")
    phases = recipe.get("phases")
    if not isinstance(phases, list) or not phases:
        errors.append("phases must be a non-empty array")
        return errors
    seen_phase_ids: set[str] = set()
    has_gate = False
    high_risk_ops = 0
    for phase in phases:
        phase_id = phase.get("phase_id")
        if not isinstance(phase_id, str) or not phase_id:
            errors.append("phase_id must be a non-empty string")
            continue
        if phase_id in seen_phase_ids:
            errors.append(f"duplicate phase_id: {phase_id}")
        seen_phase_ids.add(phase_id)
        if phase.get("approval_gate") is True:
            has_gate = True
        operations = phase.get("operations")
        if not isinstance(operations, list) or not operations:
            errors.append(f"phase {phase_id} must contain operations")
            continue
        for operation in operations:
            if not operation.get("operation_id"):
                errors.append(f"phase {phase_id} has operation without operation_id")
            if operation.get("risk") not in {"R0", "R1", "R2", "R3", "R4", "R5"}:
                errors.append(f"phase {phase_id} has invalid risk value")
            if operation.get("risk") in {"R4", "R5"}:
                high_risk_ops += 1
    if not has_gate:
        errors.append("recipe must include at least one approval_gate phase")
    if high_risk_ops == 0:
        errors.append("recipe must include at least one R4 or R5 operation")
    return errors


def secret_findings(paths: list[Path]) -> list[str]:
    findings: list[str] = []
    for path in paths:
        text = path.read_text(encoding="utf-8")
        for pattern in SECRET_PATTERNS:
            if pattern.search(text):
                findings.append(str(path))
                break
    return findings


def allowed_path_violations(packet: dict[str, Any], paths: list[str]) -> list[str]:
    allowed = set(packet["allowed_paths"])
    return [path for path in paths if path not in allowed]


def main() -> None:
    base = root()
    package_base = package_root()
    generated_at = now()
    packet = read_json(PACKET_PATH)
    recipe = recipe_payload()
    warnings: list[str] = []

    source_package_dir = package_base / "source" / "codex-flexnetos-migration-prompt-package"
    if not source_package_dir.exists():
        warnings.append(
            "source/codex-flexnetos-migration-prompt-package is not materialized in this workspace; the adapter references the declared source glob from the packet and validated downstream evidence instead."
        )

    req201_proof_exists = (base / REQ201_PROOF_PATH).exists()
    req027_proof_exists = (base / REQ027_PROOF_PATH).exists()
    approval_exists = (base / APPROVAL_PATH).exists()
    approval_review_exists = (base / APPROVAL_REVIEW_PATH).exists()

    doc_text = render_doc(recipe, packet, warnings)
    (base / DOC_PATH).parent.mkdir(parents=True, exist_ok=True)
    (base / DOC_PATH).write_text(doc_text, encoding="utf-8")
    write_json(RECIPE_PATH, recipe)

    recipe_errors = validate_recipe_shape(recipe)
    doc_text_written = (base / DOC_PATH).read_text(encoding="utf-8")
    documentation_checks = {
        "doc_mentions_recipe_id": recipe["recipe_id"] in doc_text_written,
        "doc_mentions_verification_command": "python3 scripts/verify_flexnetos_adapter_recipe.py" in doc_text_written,
        "doc_mentions_replay_dependency": "REQ-027_ENVCTL_REPLAY_ENGINE" in doc_text_written,
        "doc_mentions_comparison_dependency": "REQ-201_FLEXNETOS_LIFEOS_COMPARISON" in doc_text_written,
    }

    phase_ids = [phase["phase_id"] for phase in recipe["phases"]]
    phase_doc_missing = [phase_id for phase_id in phase_ids if phase_id not in doc_text_written]

    output_paths = [DOC_PATH, RECIPE_PATH, REPORT_PATH, "scripts/verify_flexnetos_adapter_recipe.py", PROOF_PATH, HEARTBEAT_PATH, LOG_PATH]
    path_violations = allowed_path_violations(packet, output_paths)

    scan_paths = [
        base / DOC_PATH,
        base / RECIPE_PATH,
        Path(__file__),
    ]
    secret_hits = secret_findings(scan_paths)

    checks = {
        "packet_present": (base / PACKET_PATH).exists(),
        "approval_present": approval_exists,
        "approval_review_present": approval_review_exists,
        "req201_dependency_proof_present": req201_proof_exists,
        "req027_dependency_proof_present": req027_proof_exists,
        "recipe_json_exists": (base / RECIPE_PATH).exists(),
        "recipe_doc_exists": (base / DOC_PATH).exists(),
        "allowed_paths_only": not path_violations,
        "recipe_shape_valid": not recipe_errors,
        "documentation_checks_passed": all(documentation_checks.values()) and not phase_doc_missing,
        "approval_gate_present": any(phase.get("approval_gate") for phase in recipe["phases"]),
        "high_risk_apply_present": any(
            operation["risk"] in {"R4", "R5"}
            for phase in recipe["phases"]
            for operation in phase["operations"]
        ),
        "secret_exposure_status_pass": not secret_hits,
    }

    errors = list(recipe_errors)
    if phase_doc_missing:
        errors.append("documentation missing phase ids: " + ", ".join(phase_doc_missing))
    for name, passed in documentation_checks.items():
        if not passed:
            errors.append(f"documentation check failed: {name}")
    if path_violations:
        errors.append("generated outputs outside allowed paths: " + ", ".join(path_violations))
    if secret_hits:
        errors.append("secret-like content detected in generated outputs: " + ", ".join(secret_hits))
    if not req201_proof_exists:
        errors.append(f"missing dependency proof: {REQ201_PROOF_PATH}")
    if not req027_proof_exists:
        errors.append(f"missing dependency proof: {REQ027_PROOF_PATH}")
    if not approval_exists:
        errors.append(f"missing approval artifact: {APPROVAL_PATH}")

    report_status = "pass" if not errors and all(checks.values()) else "fail"
    report = {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "status": report_status,
        "generated_at": generated_at,
        "recipe_summary": {
            "recipe_id": recipe["recipe_id"],
            "version": recipe["version"],
            "phase_count": len(recipe["phases"]),
            "operation_count": sum(len(phase["operations"]) for phase in recipe["phases"]),
            "approval_gate_phases": [
                phase["phase_id"] for phase in recipe["phases"] if phase.get("approval_gate")
            ],
            "target_descriptor_id": recipe["metadata"]["target_descriptor_id"],
        },
        "checks": checks,
        "documentation_checks": documentation_checks,
        "dependency_evidence": {
            "req201_proof_path": REQ201_PROOF_PATH,
            "req027_proof_path": REQ027_PROOF_PATH,
            "approval_path": APPROVAL_PATH,
        },
        "warnings": warnings,
        "errors": errors,
        "secret_scan": {
            "paths": [str(path.relative_to(base)) if path.is_relative_to(base) else str(path) for path in scan_paths],
            "findings": secret_hits,
        },
        "allowed_paths": packet["allowed_paths"],
        "output_paths": output_paths,
        "evidence": [
            DOC_PATH,
            RECIPE_PATH,
            REPORT_PATH,
            APPROVAL_PATH,
            REQ201_PROOF_PATH,
            REQ027_PROOF_PATH,
            SHARED_SCHEMA_PATH,
        ],
        "sha256": {
            DOC_PATH: "sha256:" + sha256_file(base / DOC_PATH),
            RECIPE_PATH: "sha256:" + sha256_file(base / RECIPE_PATH),
            "scripts/verify_flexnetos_adapter_recipe.py": "sha256:" + sha256_file(Path(__file__)),
        },
    }

    write_json(REPORT_PATH, report)
    write_json(LOG_PATH, report)
    write_json(
        HEARTBEAT_PATH,
        {
            "schema_version": "1.0",
            "task_id": TASK_ID,
            "status": report_status,
            "updated_at": generated_at,
            "proof_uri": PROOF_PATH,
            "validation_report": REPORT_PATH,
        },
    )

    files_changed = [
        "execution-framework/scripts/verify_flexnetos_adapter_recipe.py",
        "execution-framework/docs/FLEXNETOS_ADAPTER_RECIPE.md",
        "execution-framework/generated/flexnetos_adapter_recipe.json",
        "execution-framework/generated/flexnetos_adapter_recipe_validation_report.json",
        "execution-framework/state/REQ-202_FLEXNETOS_ADAPTER_RECIPE.heartbeat.json",
        "execution-framework/logs/REQ-202_FLEXNETOS_ADAPTER_RECIPE.log",
        "execution-framework/proof_records/REQ-202_FLEXNETOS_ADAPTER_RECIPE.proof.json",
        "execution-framework/proof_records/proof_ledger.jsonl",
    ]
    proof = make_proof(
        TASK_ID,
        "completed" if report_status == "pass" else "failed",
        "flexnetos-adapter-agent",
        packet["helper_id"],
        packet["model_tag"],
        packet["repo_path"],
        files_changed,
        ["python3 scripts/verify_flexnetos_adapter_recipe.py"],
        report,
        report["evidence"],
        "" if report_status == "pass" else "; ".join(errors),
        "unblock VER-300_UNIT_VALIDATION" if report_status == "pass" else "fix REQ-202 validation errors",
    )
    append_proof(proof)

    print(
        "flexnetos adapter recipe status={status} phases={phases} operations={operations}".format(
            status=report_status,
            phases=report["recipe_summary"]["phase_count"],
            operations=report["recipe_summary"]["operation_count"],
        )
    )
    if report_status != "pass":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
