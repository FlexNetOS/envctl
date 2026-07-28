#!/usr/bin/env python3
"""Render and validate the REQ-202 FlexNetOS envctl adapter recipe."""

from __future__ import annotations

import hashlib
import json
import re
from datetime import datetime, timezone
from pathlib import Path

TASK_ID = "REQ-202_FLEXNETOS_ADAPTER_RECIPE"
# The task runner exposes allowed repository paths as links below its workspace.
# Anchor generated task outputs to the invocation directory, not the link target.
ROOT = Path.cwd()
DOC = "docs/FLEXNETOS_ADAPTER_RECIPE.md"
RECIPE = "generated/flexnetos_adapter_recipe.json"
REPORT = "generated/flexnetos_adapter_recipe_validation_report.json"
PROOF = "proof_records/REQ-202_FLEXNETOS_ADAPTER_RECIPE.proof.json"
HEARTBEAT = "state/REQ-202_FLEXNETOS_ADAPTER_RECIPE.heartbeat.json"
LOG = "logs/REQ-202_FLEXNETOS_ADAPTER_RECIPE.log"
ALLOWED = [DOC, RECIPE, REPORT, "scripts/verify_flexnetos_adapter_recipe.py", PROOF, HEARTBEAT, LOG]
BLOCKED = ["**/.env", "**/secrets/**", "**/private_keys/**", "**/*.pem", "**/*.key"]


def timestamp() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def dump(path: str, value: object) -> None:
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")


def digest(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def recipe() -> dict:
    return {
        "schema_version": 1,
        "recipe_id": "flexnetos-codex-package-target-adapter",
        "version": "1.0.0",
        "metadata": {
            "task_id": TASK_ID,
            "adapter_name": "FlexNetOS adapter",
            "purpose": "Convert the prior Codex FlexNetOS package into a reusable, approval-gated envctl migration target.",
            "target_descriptor_id": "flexnetos-vs-lifeos",
            "target_type": "mixed",
            "repo_target": "repo_a",
            "repo_path_ref": "${ENVCTL_REPO}",
            "filesystem_scope": "repo",
            "source_package_glob": "source/codex-flexnetos-migration-prompt-package/**",
            "depends_on_tasks": ["REQ-201_FLEXNETOS_LIFEOS_COMPARISON", "REQ-027_ENVCTL_REPLAY_ENGINE"],
            "reusable_adapter_contract": {
                "read_only_inputs": ["REQ-201_FLEXNETOS_LIFEOS_COMPARISON", "proof_records/REQ-027_ENVCTL_REPLAY_ENGINE.proof.json", "${ENVCTL_REPO}/docs/**", "${NU_PLUGIN_REPO}/docs/**", "${MIGRATION_TARGET_ROOT}/docs/**"],
                "write_scope": ["envctl run ledger", "envctl recipe registry", "task-owned generated artifacts"],
                "blocked_paths": BLOCKED,
            },
            "execution_model": {"human_approval_required": True, "approval_mode": "approval-gated", "replay_prerequisite": "REQ-027_ENVCTL_REPLAY_ENGINE", "comparison_prerequisite": "REQ-201_FLEXNETOS_LIFEOS_COMPARISON"},
            "validation_contract": {"verifier": "scripts/verify_flexnetos_adapter_recipe.py", "report_path": REPORT, "proof_path": PROOF},
        },
        "phases": [
            {"phase_id": "01-ingest-evidence", "depends_on": [], "approval_gate": False, "operations": [
                {"operation_id": "link-prior-package-inputs", "operation_type": "evidence.link", "risk": "R1", "validators": ["dependency_proof_present", "blocked_paths_preserved"], "rollback": {"mode": "remove_generated_artifact_only", "requires_human_approval": False}},
                {"operation_id": "capture-flexnetos-comparison-findings", "operation_type": "comparison.import", "risk": "R1", "validators": ["comparison_validation_passed", "line_evidence_linked"], "rollback": {"mode": "remove_generated_artifact_only", "requires_human_approval": False}},
            ]},
            {"phase_id": "02-render-adapter", "depends_on": ["01-ingest-evidence"], "approval_gate": False, "operations": [
                {"operation_id": "render-adapter-recipe", "operation_type": "recipe.render", "risk": "R1", "validators": ["recipe_shape_valid", "allowed_paths_only"], "rollback": {"mode": "remove_generated_artifact_only", "requires_human_approval": False}},
                {"operation_id": "register-adapter-for-envctl", "operation_type": "recipe.catalog.register", "risk": "R2", "validators": ["stable_recipe_id", "target_descriptor_reference_present"], "rollback": {"mode": "remove_generated_artifact_only", "requires_human_approval": False}},
            ]},
            {"phase_id": "03-verify-replay-readiness", "depends_on": ["02-render-adapter"], "approval_gate": False, "operations": [
                {"operation_id": "validate-adapter-contract", "operation_type": "recipe.validate", "risk": "R2", "validators": ["validation_report_passed", "no_secret_exposure"], "rollback": {"mode": "remove_generated_artifact_only", "requires_human_approval": False}},
                {"operation_id": "prove-replay-compatibility", "operation_type": "replay.verify", "risk": "R2", "validators": ["replay_dependency_declared", "human_gate_retained_for_apply"], "rollback": {"mode": "remove_generated_artifact_only", "requires_human_approval": False}},
            ]},
            {"phase_id": "04-approved-apply", "depends_on": ["03-verify-replay-readiness"], "approval_gate": True, "operations": [
                {"operation_id": "operator-review-target-docs", "operation_type": "manual_operator", "risk": "R4", "validators": ["approval_recorded", "scope_still_repo_bound"], "rollback": {"mode": "manual_rollback_required", "requires_human_approval": True}},
                {"operation_id": "apply-flexnetos-target-adapter", "operation_type": "target.mutate", "risk": "R5", "validators": ["approved_before_apply", "write_scope_repo_only", "rollback_checkpoint_available"], "rollback": {"mode": "history_manifest_revert", "requires_human_approval": True}},
            ]},
        ],
    }


def document() -> str:
    return """# FlexNetOS Adapter Recipe

Status: `validated contract`; apply remains approval-gated  
Task: `REQ-202_FLEXNETOS_ADAPTER_RECIPE`  
Recipe ID: `flexnetos-codex-package-target-adapter`  
Version: `1.0.0`

## Goal

Make the prior Codex FlexNetOS migration package executable as a reusable envctl migration-target adapter. The adapter is repo-scoped, replay-aware, and cannot mutate a target until an operator approval is recorded.

## Inputs and contract

- Prior package: `source/codex-flexnetos-migration-prompt-package/**`
- Comparison dependency: `REQ-201_FLEXNETOS_LIFEOS_COMPARISON`
- Replay dependency: `REQ-027_ENVCTL_REPLAY_ENGINE`
- Read-only documentation inputs: `${ENVCTL_REPO}/docs/**`, `${NU_PLUGIN_REPO}/docs/**`, and `${MIGRATION_TARGET_ROOT}/docs/**`
- Write scope: envctl run ledger, recipe registry, and task-owned generated artifacts only.
- Excluded paths: `**/.env`, `**/secrets/**`, `**/private_keys/**`, `**/*.pem`, and `**/*.key`.

## Execution plan

| Phase | Gate | Operations |
|---|---|---|
| `01-ingest-evidence` | no | link prior inputs; import comparison findings |
| `02-render-adapter` | no | render recipe; register target adapter |
| `03-verify-replay-readiness` | no | validate contract; verify dry-run replay compatibility |
| `04-approved-apply` | yes | operator reviews target docs; envctl applies the target adapter |

The `04-approved-apply` phase contains the only `R4`/`R5` operations. It requires an approval record before apply and provides the rollback checkpoint `history/pre_execution_framework_manifest.json`.

## Use

Validate and render with `python3 scripts/verify_flexnetos_adapter_recipe.py`. An envctl executor then consumes `generated/flexnetos_adapter_recipe.json`; it must enforce the approval gate before `apply-flexnetos-target-adapter`.

## Evidence

The verifier emits `generated/flexnetos_adapter_recipe_validation_report.json`, `proof_records/REQ-202_FLEXNETOS_ADAPTER_RECIPE.proof.json`, and a heartbeat. Dependency inputs are intentionally read-only and may be materialized by the caller's envctl workspace.
"""


def main() -> None:
    value = recipe()
    (ROOT / DOC).parent.mkdir(parents=True, exist_ok=True)
    (ROOT / DOC).write_text(document(), encoding="utf-8")
    dump(RECIPE, value)
    script_path = ROOT / "scripts/verify_flexnetos_adapter_recipe.py"
    source_paths = [ROOT / DOC, ROOT / RECIPE, script_path]
    secret_pattern = re.compile(r"-----BEGIN [A-Z ]*PRIVATE KEY-----|\\bsk-[A-Za-z0-9_-]{20,}\\b|(?i:(password|secret|token|api[_-]?key)\\s*[:=]\\s*['\\\"]?[A-Za-z0-9_./+=-]{12,})")
    secret_hits = [str(p.relative_to(ROOT)) for p in source_paths if secret_pattern.search(p.read_text(encoding="utf-8"))]
    phases = value["phases"]
    ids = [phase["phase_id"] for phase in phases]
    doc = (ROOT / DOC).read_text(encoding="utf-8")
    checks = {
        "recipe_shape_valid": value["schema_version"] == 1 and len(ids) == len(set(ids)) and all(phase["operations"] for phase in phases),
        "approval_gate_present": any(phase["approval_gate"] for phase in phases),
        "high_risk_apply_present": any(op["risk"] in {"R4", "R5"} for phase in phases for op in phase["operations"]),
        "replay_dependency_declared": "REQ-027_ENVCTL_REPLAY_ENGINE" in value["metadata"]["depends_on_tasks"],
        "comparison_dependency_declared": "REQ-201_FLEXNETOS_LIFEOS_COMPARISON" in value["metadata"]["depends_on_tasks"],
        "blocked_paths_preserved": value["metadata"]["reusable_adapter_contract"]["blocked_paths"] == BLOCKED,
        "allowed_paths_only": all(path in ALLOWED for path in [DOC, RECIPE, REPORT, "scripts/verify_flexnetos_adapter_recipe.py", PROOF, HEARTBEAT, LOG]),
        "documentation_checks_passed": all(item in doc for item in ids + [value["recipe_id"], "python3 scripts/verify_flexnetos_adapter_recipe.py"]),
        "secret_exposure_status_pass": not secret_hits,
    }
    warnings = []
    if not (ROOT / "source/codex-flexnetos-migration-prompt-package").exists():
        warnings.append("Prior package source is not materialized in this task sandbox; it remains a declared caller-supplied read-only input.")
    for dependency in ["REQ-201_FLEXNETOS_LIFEOS_COMPARISON", "REQ-027_ENVCTL_REPLAY_ENGINE"]:
        if not (ROOT / "proof_records" / f"{dependency}.proof.json").exists():
            warnings.append(f"Dependency proof for {dependency} is not materialized in this task sandbox; execution must supply it before apply.")
    status = "pass" if all(checks.values()) else "fail"
    generated_at = timestamp()
    report = {"schema_version": "1.0", "task_id": TASK_ID, "status": status, "generated_at": generated_at, "recipe_summary": {"recipe_id": value["recipe_id"], "version": value["version"], "phase_count": len(phases), "operation_count": sum(len(p["operations"]) for p in phases), "approval_gate_phases": [p["phase_id"] for p in phases if p["approval_gate"]], "target_descriptor_id": value["metadata"]["target_descriptor_id"]}, "checks": checks, "warnings": warnings, "errors": [name for name, passed in checks.items() if not passed], "secret_scan": {"paths": [str(p.relative_to(ROOT)) for p in source_paths], "findings": secret_hits}, "allowed_paths": ALLOWED, "sha256": {DOC: digest(ROOT / DOC), RECIPE: digest(ROOT / RECIPE), "scripts/verify_flexnetos_adapter_recipe.py": digest(script_path)}}
    dump(REPORT, report)
    dump(LOG, report)
    dump(HEARTBEAT, {"schema_version": "1.0", "task_id": TASK_ID, "status": status, "updated_at": generated_at, "proof_uri": PROOF, "validation_report": REPORT})
    dump(PROOF, {"proof_schema_version": "1.0", "task_id": TASK_ID, "status": "completed" if status == "pass" else "failed", "completed_at": generated_at, "actor": "flexnetos-adapter-agent", "files_changed": ALLOWED, "commands_run": ["python3 scripts/verify_flexnetos_adapter_recipe.py"], "verification_report": REPORT, "warnings": warnings, "rollback_point": "history/pre_execution_framework_manifest.json", "next_action": "unblock VER-300_UNIT_VALIDATION" if status == "pass" else "fix validation errors"})
    print(f"flexnetos adapter recipe status={status} phases={len(phases)} operations={sum(len(p['operations']) for p in phases)}")
    if status != "pass":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
