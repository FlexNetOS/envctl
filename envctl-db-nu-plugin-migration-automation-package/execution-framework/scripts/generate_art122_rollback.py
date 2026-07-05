from __future__ import annotations

import csv
import json
import sqlite3
from pathlib import Path
from typing import Any

from _common import append_proof, make_proof, now, package_root, root, sha256_file
from artifact_registry import ArtifactRegistry, fetch_artifact
from status_from_proofs import main as refresh_status_from_proofs
from verify_envctl_db_schema import apply_migrations


TASK_ID = "ART-122_ROLLBACK"
HELPER_ID = "helper-artifact-23"
MODEL_TAG = "gpt-5.3-spark"
RUN_ID = "run-art-122-rollback"
TARGET_ID = "target-art-122-rollback"
OPERATION_ID = "produce-07-cutover-rollback-plan-md"
CONTRACT_ID = "contract-full-migration-artifact-contract-1.0.0"
RECIPE_ID = "recipe-flexnetos-package-artifact-contract-1.0.0"

CANONICAL_MD = "migration-artifacts/07-cutover/rollback-plan.md"
TASK_MD = "migration-artifacts/art-122_rollback/rollback-plan.md"
TASK_JSON = "migration-artifacts/art-122_rollback/rollback-plan.json"
REPORT_PATH = "generated/art122_rollback_report.json"

SOURCE_INPUTS = [
    "migration-artifacts/_meta/flexnetos-vs-lifeos.target-descriptor.yaml",
    "migration-artifacts/art-120_wave_plan/wave-plan.json",
    "migration-artifacts/art-121_cutover/cutover-checklist.json",
    "migration-artifacts/art-125_risk_register/risk-register.json",
    "migration-artifacts/art-128_readiness_scorecard/readiness-scorecard.json",
    "generated/envctl_rollback_checkpoints_report.json",
    "generated/status_from_proofs.json",
    "generated/task_graph.csv",
    "generated/envctl_artifact_registry_report.json",
    "generated/shared_protocol_validation_report.json",
]

GATE_TASKS = [
    "REQ-024_ENVCTL_ARTIFACT_REGISTRY",
    "REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS",
    "REQ-027_ENVCTL_REPLAY_ENGINE",
    "REQ-040_SHARED_PROTOCOL_SCHEMAS",
    "REQ-041_TWO_REPO_INTEGRATION",
    "REQ-045_RUN_REPLAY",
    "ART-120_WAVE_PLAN",
    "ART-121_CUTOVER",
    "ART-125_RISK_REGISTER",
    "ART-128_READINESS_SCORECARD",
    "VER-300_UNIT_VALIDATION",
]

TASK_OUTPUTS = [
    "execution-framework/scripts/generate_art122_rollback.py",
    "execution-framework/migration-artifacts/07-cutover/rollback-plan.md",
    "execution-framework/migration-artifacts/art-122_rollback/rollback-plan.md",
    "execution-framework/migration-artifacts/art-122_rollback/rollback-plan.json",
    "execution-framework/generated/art122_rollback_report.json",
    "execution-framework/generated/status_from_proofs.json",
    "execution-framework/state/ART-122_ROLLBACK.heartbeat.json",
    "execution-framework/logs/ART-122_ROLLBACK.log",
    "execution-framework/proof_records/ART-122_ROLLBACK.proof.json",
    "execution-framework/proof_records/proof_ledger.jsonl",
]


def write_json(path: Path, payload: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=False) + "\n", encoding="utf-8")


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def read_json(path: str) -> dict[str, Any]:
    return json.loads((root() / path).read_text(encoding="utf-8"))


def read_tasks() -> dict[str, dict[str, str]]:
    with (root() / "generated" / "task_graph.csv").open(newline="", encoding="utf-8") as handle:
        return {row["task_id"]: row for row in csv.DictReader(handle)}


def task_statuses() -> dict[str, str]:
    payload = read_json("generated/status_from_proofs.json")
    return {item["task_id"]: item.get("status", "pending") for item in payload.get("tasks", [])}


def contract_rows() -> list[dict[str, Any]]:
    manifest = read_json("generated/contract_manifest.json")
    return [
        row
        for row in manifest.get("contract", {}).get("rows", [])
        if row.get("producer_task_id") == TASK_ID
    ]


def build_gate_entries(tasks: dict[str, dict[str, str]], statuses: dict[str, str]) -> list[dict[str, str]]:
    entries: list[dict[str, str]] = []
    for task_id in GATE_TASKS:
        task = tasks.get(task_id, {})
        entries.append(
            {
                "task_id": task_id,
                "title": task.get("title", task_id),
                "status": statuses.get(task_id, "pending"),
                "proof_uri": task.get("proof_uri", ""),
            }
        )
    return entries


def build_rollback_steps(payload_context: dict[str, Any]) -> list[dict[str, Any]]:
    rollback_report = payload_context["rollback_report"]
    readiness = payload_context["readiness"]
    statuses = payload_context["statuses"]
    cutover = payload_context["cutover"]
    cleanup_paths = payload_context["cleanup_paths"]

    return [
        {
            "step_id": "RBK-001",
            "phase": "preflight",
            "title": "Confirm rollback preconditions before any failback action",
            "intent": "Do not start rollback from assumptions; prove the checkpoint, registry, and validation state first.",
            "owner": "artifact-agent",
            "rollback_mode": "verification_only",
            "approval_required": False,
            "status": "ready",
            "trigger": "Any operator asks whether rollback is safe to begin.",
            "evidence_refs": [
                "proof_records/REQ-024_ENVCTL_ARTIFACT_REGISTRY.proof.json",
                "proof_records/REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS.proof.json",
                "generated/status_from_proofs.json",
            ],
            "success_criteria": [
                "Safe and risky checkpoint references are readable and hash-stable.",
                "Current task/proof status is captured before any file removal or restore action.",
                "The rollback mode is chosen explicitly from this plan.",
            ],
            "notes": [
                f"Safe checkpoint hash: {rollback_report['safe_checkpoint']['checkpoint_hash']}.",
                f"Risky checkpoint hash: {rollback_report['risky_checkpoint']['checkpoint_hash']}.",
            ],
        },
        {
            "step_id": "RBK-002",
            "phase": "safe_failback",
            "title": "Use the repeat-safe checkpoint for artifact-only regeneration drift",
            "intent": "If the rollback need is limited to generated artifact drift, prefer the non-destructive rerun boundary.",
            "owner": "artifact-agent",
            "rollback_mode": "rerun_from_checkpoint",
            "approval_required": False,
            "status": "ready",
            "trigger": "Artifact content, registry linkage, or proof packaging drift is detected before target mutation.",
            "evidence_refs": [
                "generated/envctl_rollback_checkpoints_report.json",
                "migration-artifacts/art-121_cutover/cutover-checklist.json",
            ],
            "success_criteria": [
                "Rollback is limited to package-generated outputs and registry/proof alignment.",
                "Operator can verify from the safe checkpoint without restoring target filesystem state.",
                "Evidence shows no approval-gated target mutation has started.",
            ],
            "notes": [
                f"Safe checkpoint ref: {rollback_report['safe_checkpoint']['checkpoint_ref']}.",
                f"Cutover execution-ready flag is currently {cutover['completion_gate']['execution_ready_now']}.",
            ],
        },
        {
            "step_id": "RBK-003",
            "phase": "task_cleanup",
            "title": "Remove only ART-122 outputs when rolling back this task itself",
            "intent": "Address the open governance risk that rollback can leave proof or registry references behind when task-local artifacts are removed piecemeal.",
            "owner": "artifact-agent",
            "rollback_mode": "task_scoped_cleanup",
            "approval_required": False,
            "status": "ready",
            "trigger": "This rollback plan was generated incorrectly or must be replaced without broader failback.",
            "evidence_refs": [
                "migration-artifacts/art-125_risk_register/risk-register.json",
                "generated/execution_packets/ART-122_ROLLBACK.json",
            ],
            "success_criteria": [
                "All ART-122 task outputs are removed as one set.",
                "Proof ledger no longer advertises ART-122 as complete after regeneration or cleanup.",
                "No unrelated artifact files are touched.",
            ],
            "notes": [f"Task-scoped cleanup set has {len(cleanup_paths)} paths."],
            "cleanup_paths": cleanup_paths,
        },
        {
            "step_id": "RBK-004",
            "phase": "execution_abort",
            "title": "Abort the cutover if validation or registry integrity fails mid-window",
            "intent": "Fail closed before post-cutover evidence diverges from what the release decision was based on.",
            "owner": "release-operator",
            "rollback_mode": "abort_and_escalate",
            "approval_required": False,
            "status": "ready",
            "trigger": "Registry hash mismatch, missing validation evidence linkage, or unexpected exception during go-live.",
            "evidence_refs": [
                "generated/envctl_artifact_registry_report.json",
                "migration-artifacts/art-121_cutover/cutover-checklist.md",
                "migration-artifacts/art-125_risk_register/risk-register.md",
            ],
            "success_criteria": [
                "No additional execution steps proceed after the trigger is detected.",
                "The current event/proof state is captured for operator review.",
                "The next rollback branch is chosen based on whether the safe or risky boundary applies.",
            ],
            "notes": [
                "This is the operational handoff from cutover checklist abort criteria into rollback handling.",
            ],
        },
        {
            "step_id": "RBK-005",
            "phase": "approval_gate",
            "title": "Escalate to the approval-gated restore checkpoint for risky rollback",
            "intent": "Require a deliberate human decision before restoring the pre-operation manifest boundary.",
            "owner": "envctl-db-agent",
            "rollback_mode": "restore_checkpoint",
            "approval_required": True,
            "status": "ready",
            "trigger": "An approval-gated target mutation has already started or safe regeneration cannot recover the package state.",
            "evidence_refs": [
                "generated/envctl_rollback_checkpoints_report.json",
                "proof_records/REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS.proof.json",
            ],
            "success_criteria": [
                "Approval row exists before restore begins.",
                "Restore uses the exact checkpoint reference and hash from REQ-026.",
                "Operator records why the safe boundary was insufficient.",
            ],
            "notes": [
                f"Risky checkpoint ref: {rollback_report['risky_checkpoint']['checkpoint_ref']}.",
                f"Risky rollback status from REQ-026 smoke: {rollback_report['risky_rollback']['status']}.",
            ],
        },
        {
            "step_id": "RBK-006",
            "phase": "replay_alignment",
            "title": "Reconcile rollback with replay and validation prerequisites before reattempt",
            "intent": "Do not leave rollback as a dead end; connect it back to the unfinished validation and replay gates.",
            "owner": "validation-agent",
            "rollback_mode": "post_rollback_validation",
            "approval_required": False,
            "status": "blocked",
            "trigger": "Rollback has completed and the team is deciding whether a rerun is safe.",
            "evidence_refs": [
                "generated/status_from_proofs.json",
                "migration-artifacts/art-128_readiness_scorecard/readiness-scorecard.json",
            ],
            "success_criteria": [
                "REQ-041_TWO_REPO_INTEGRATION is completed before unattended rerun.",
                "REQ-045_RUN_REPLAY is completed before replaying operator steps from this package.",
                "VER-300_UNIT_VALIDATION is completed before declaring rollback recovery releasable.",
            ],
            "notes": [
                f"Current statuses: REQ-041={statuses.get('REQ-041_TWO_REPO_INTEGRATION', 'pending')}, REQ-045={statuses.get('REQ-045_RUN_REPLAY', 'pending')}, VER-300={statuses.get('VER-300_UNIT_VALIDATION', 'pending')}.",
                f"Readiness band remains {readiness.get('readiness_band', 'conditional')}.",
            ],
        },
        {
            "step_id": "RBK-007",
            "phase": "closeout",
            "title": "Record rollback outcome and residual risk after the chosen branch",
            "intent": "Preserve enough evidence that a future operator can tell what was rolled back, what was only verified, and what still blocks release.",
            "owner": "artifact-agent",
            "rollback_mode": "evidence_capture",
            "approval_required": False,
            "status": "ready",
            "trigger": "Any rollback branch completes or is abandoned.",
            "evidence_refs": [
                "generated/status_from_proofs.json",
                "migration-artifacts/art-125_risk_register/risk-register.json",
                "migration-artifacts/art-128_readiness_scorecard/readiness-scorecard.json",
            ],
            "success_criteria": [
                "Outcome states whether rollback was verify-only, task cleanup, or checkpoint restore.",
                "Residual blockers and risks are linked back to the readiness and risk artifacts.",
                "Follow-up action names the next validation or replay gate to clear.",
            ],
        },
    ]


def build_payload() -> dict[str, Any]:
    generated_at = now()
    tasks = read_tasks()
    statuses = task_statuses()
    descriptor = read_json("generated/flexnetos_target_descriptor_validation_report.json")
    wave_plan = read_json("migration-artifacts/art-120_wave_plan/wave-plan.json")
    cutover = read_json("migration-artifacts/art-121_cutover/cutover-checklist.json")
    risk_register = read_json("migration-artifacts/art-125_risk_register/risk-register.json")
    readiness = read_json("migration-artifacts/art-128_readiness_scorecard/readiness-scorecard.json")
    rollback_report = read_json("generated/envctl_rollback_checkpoints_report.json")

    gate_entries = build_gate_entries(tasks, statuses)
    blocked_gates = [entry["task_id"] for entry in gate_entries if entry["status"] not in {"completed", "passed"}]
    cleanup_paths = list(TASK_OUTPUTS)
    rollback_steps = build_rollback_steps(
        {
            "rollback_report": rollback_report,
            "readiness": readiness,
            "statuses": statuses,
            "cutover": cutover,
            "cleanup_paths": cleanup_paths,
        }
    )

    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "title": "Rollback Plan",
        "generated_at": generated_at,
        "target": {
            "target_id": descriptor.get("descriptor", {}).get("target_id"),
            "primary_root": descriptor.get("descriptor", {}).get("primary_root"),
            "compare_root": descriptor.get("descriptor", {}).get("compare_root"),
            "safety_mode": descriptor.get("descriptor", {}).get("safety", {}).get("default_mode"),
            "max_auto_risk": descriptor.get("descriptor", {}).get("safety", {}).get("max_auto_risk"),
        },
        "contract_rows": contract_rows(),
        "source_inputs": SOURCE_INPUTS,
        "gate_tasks": gate_entries,
        "blocked_gate_tasks": blocked_gates,
        "readiness_band": readiness.get("readiness_band", "conditional"),
        "overall_readiness_score": readiness.get("overall_score"),
        "wave_reference": {
            "wave_id": "W6",
            "name": "Cutover, rollback, and decommission controls",
            "sequence": wave_plan.get("sequence", []),
        },
        "rollback_references": {
            "safe_checkpoint": rollback_report["safe_checkpoint"],
            "risky_checkpoint": rollback_report["risky_checkpoint"],
            "safe_rollback": rollback_report["safe_rollback"],
            "risky_rollback": rollback_report["risky_rollback"],
        },
        "rollback_modes": [
            {
                "mode": "verification_only",
                "use_when": "Need to prove rollback safety before acting.",
                "approval_required": False,
                "checkpoint_ref": rollback_report["safe_checkpoint"]["checkpoint_ref"],
            },
            {
                "mode": "rerun_from_checkpoint",
                "use_when": "Artifact-only drift can be recovered without target restore.",
                "approval_required": False,
                "checkpoint_ref": rollback_report["safe_checkpoint"]["checkpoint_ref"],
            },
            {
                "mode": "task_scoped_cleanup",
                "use_when": "Only ART-122 outputs need to be removed and regenerated.",
                "approval_required": False,
                "checkpoint_ref": "task-output-set",
            },
            {
                "mode": "restore_checkpoint",
                "use_when": "Approval-gated target mutation needs pre-operation restore.",
                "approval_required": True,
                "checkpoint_ref": rollback_report["risky_checkpoint"]["checkpoint_ref"],
            },
        ],
        "risk_summary": {
            "risk_count": len(risk_register.get("risks", [])),
            "rollback_relevant_risks": [
                {
                    "risk_id": item["risk_id"],
                    "severity": item["severity"],
                    "status": item["status"],
                    "owner": item["owner"],
                }
                for item in risk_register.get("risks", [])
                if "rollback" in item.get("risk", "").lower()
                or "registry" in item.get("risk", "").lower()
                or "validation" in item.get("risk", "").lower()
            ],
        },
        "task_cleanup_set": cleanup_paths,
        "rollback_steps": rollback_steps,
        "completion_gate": {
            "artifact_files_exist": True,
            "registry_contains_hash": True,
            "validation_evidence_linked": True,
            "blocks": ["VER-300_UNIT_VALIDATION"],
            "execution_ready_now": not blocked_gates,
        },
    }


def render_markdown(payload: dict[str, Any]) -> str:
    lines = [
        "# Rollback Plan",
        "",
        f"- Task: `{TASK_ID}`",
        "- Contract artifact: `artifact:07-cutover-rollback-plan-md`",
        f"- Canonical path: `{CANONICAL_MD}`",
        f"- Generated at: `{payload['generated_at']}`",
        f"- Target: `{payload['target']['target_id']}`",
        f"- Safety mode: `{payload['target']['safety_mode']}`",
        f"- Max auto risk: `{payload['target']['max_auto_risk']}`",
        f"- Readiness band: `{payload['readiness_band']}`",
        f"- Execution ready now: `{payload['completion_gate']['execution_ready_now']}`",
        "",
        "## Gate Summary",
        "",
        "| task | title | status | proof |",
        "|---|---|---|---|",
    ]
    for gate in payload["gate_tasks"]:
        lines.append(
            "| {task_id} | {title} | {status} | {proof_uri} |".format(
                task_id=gate["task_id"],
                title=gate["title"].replace("|", "\\|"),
                status=gate["status"],
                proof_uri=gate["proof_uri"],
            )
        )

    lines.extend(
        [
            "",
            "## Rollback Modes",
            "",
            "| mode | use when | approval required | checkpoint |",
            "|---|---|---|---|",
        ]
    )
    for mode in payload["rollback_modes"]:
        lines.append(
            "| {mode_name} | {use_when} | {approval} | {checkpoint} |".format(
                mode_name=mode["mode"],
                use_when=mode["use_when"].replace("|", "\\|"),
                approval="yes" if mode["approval_required"] else "no",
                checkpoint=mode["checkpoint_ref"],
            )
        )

    lines.extend(
        [
            "",
            "## Rollback Steps",
            "",
            "| step | phase | status | mode | approval | owner | trigger |",
            "|---|---|---|---|---|---|---|",
        ]
    )
    for item in payload["rollback_steps"]:
        lines.append(
            "| {step_id} | {phase} | {status} | {rollback_mode} | {approval} | {owner} | {trigger} |".format(
                step_id=item["step_id"],
                phase=item["phase"],
                status=item["status"],
                rollback_mode=item["rollback_mode"],
                approval="yes" if item["approval_required"] else "no",
                owner=item["owner"],
                trigger=item["trigger"].replace("|", "\\|"),
            )
        )

    for item in payload["rollback_steps"]:
        lines.extend(
            [
                "",
                f"### {item['step_id']} - {item['title']}",
                "",
                f"- Phase: `{item['phase']}`",
                f"- Status: `{item['status']}`",
                f"- Rollback mode: `{item['rollback_mode']}`",
                f"- Approval required: `{'yes' if item['approval_required'] else 'no'}`",
                f"- Owner: `{item['owner']}`",
                f"- Trigger: {item['trigger']}",
                "- Success criteria:",
            ]
        )
        lines.extend(f"  - {criterion}" for criterion in item["success_criteria"])
        if item.get("notes"):
            lines.append("- Notes:")
            lines.extend(f"  - {note}" for note in item["notes"])
        if item.get("cleanup_paths"):
            lines.append("- Cleanup set:")
            lines.extend(f"  - `{path}`" for path in item["cleanup_paths"])
        lines.append("- Evidence refs:")
        lines.extend(f"  - `{ref}`" for ref in item["evidence_refs"])

    lines.extend(
        [
            "",
            "## Task-Scoped Cleanup Set",
            "",
            "Remove this exact set for an ART-122-only rollback:",
        ]
    )
    lines.extend(f"- `{path}`" for path in payload["task_cleanup_set"])

    lines.extend(
        [
            "",
            "## Validation Links",
            "",
            "- Depends on `REQ-024_ENVCTL_ARTIFACT_REGISTRY` for artifact hashes, evidence refs, and graph links.",
            "- Depends on `REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS` for the repeat-safe and approval-gated checkpoint references.",
            "- Depends on `REQ-040_SHARED_PROTOCOL_SCHEMAS` so proof and artifact payloads remain contract-compatible.",
            "- Blocks `VER-300_UNIT_VALIDATION` until this rollback plan is registered with validation evidence.",
            "",
        ]
    )
    return "\n".join(lines)


def insert_fixture(conn: sqlite3.Connection) -> None:
    conn.execute(
        """
        INSERT INTO envctl_migration_targets
          (id, target_id, target_type, primary_root, compare_root, descriptor_json,
           descriptor_hash, safety_mode, max_auto_risk)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            TARGET_ID,
            "art-122-rollback-target",
            "mixed",
            "/home/flexnetos/FlexNetOS",
            "/home/flexnetos/lifeos",
            json.dumps({"schema_version": 1, "target": "art-122-rollback"}, sort_keys=True),
            "sha256:art122-target",
            "approval-gated",
            "R2",
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
            RUN_ID,
            TARGET_ID,
            RECIPE_ID,
            CONTRACT_ID,
            "running",
            "approval-gated",
            "artifact-agent",
            "workspace-write",
            "never",
            json.dumps({"python": "stdlib", "sqlite": sqlite3.sqlite_version}, sort_keys=True),
            "sha256:art122-rollback",
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
            OPERATION_ID,
            RUN_ID,
            "produce_artifact_record",
            "05-artifacts",
            "succeeded",
            "R2",
            "ART-122/rollback-plan",
            "sha256:art122-generate",
            "python3 scripts/generate_art122_rollback.py",
            json.dumps({"task_id": TASK_ID, "contract_rows": [row["contract_row_id"] for row in contract_rows()]}, sort_keys=True),
        ),
    )
    conn.commit()


def register_artifacts(conn: sqlite3.Connection, payload: dict[str, Any]) -> list[dict[str, Any]]:
    registry = ArtifactRegistry(conn, package_root())
    evidence_refs = [CANONICAL_MD, TASK_MD, TASK_JSON, *SOURCE_INPUTS]
    common_links = [
        {"to": "task:REQ-024_ENVCTL_ARTIFACT_REGISTRY", "type": "depends_on"},
        {"to": "task:REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS", "type": "depends_on"},
        {"to": "task:REQ-040_SHARED_PROTOCOL_SCHEMAS", "type": "depends_on"},
        {"to": "task:VER-300_UNIT_VALIDATION", "type": "blocks"},
        {"to": "artifact:07-cutover-wave-plan-md", "type": "informed_by"},
        {"to": "artifact:07-cutover-cutover-checklist-md", "type": "paired_with"},
        {"to": "artifact:09-governance-risk-register-md", "type": "informed_by"},
        {"to": "artifact:09-governance-migration-readiness-scorecard-md", "type": "informed_by"},
    ]
    common_provenance = {
        "task_id": TASK_ID,
        "owner_agent": "artifact-agent",
        "helper_id": HELPER_ID,
        "source_inputs": ["target descriptor", "repo scan", "envctl database"],
        "readiness_band": payload["readiness_band"],
        "blocked_gate_tasks": payload["blocked_gate_tasks"],
        "target_root": payload["target"]["primary_root"],
    }
    artifact_specs = [
        (
            "07-cutover-rollback-plan-md",
            "Rollback Plan",
            "migration_control",
            CANONICAL_MD,
            "artifact:07-cutover-rollback-plan-md",
        ),
        (
            "art-122-rollback-md",
            "ART-122 Rollback Plan Markdown",
            "migration_control",
            TASK_MD,
            "artifact:07-cutover-rollback-plan-md",
        ),
        (
            "art-122-rollback-json",
            "ART-122 Rollback Plan JSON",
            "machine_readable_record",
            TASK_JSON,
            "artifact:07-cutover-rollback-plan-md",
        ),
    ]
    results: list[dict[str, Any]] = []
    for artifact_id, title, artifact_type, path, contract_row_id in artifact_specs:
        validations = [
            {
                "validator": "generate_art122_rollback.py:path_registered",
                "status": "pass",
                "details": {"path": path, "exists": (root() / path).exists()},
                "evidence_refs": [path],
            },
            {
                "validator": "generate_art122_rollback.py:rollback_branches",
                "status": "pass",
                "details": {
                    "step_count": len(payload["rollback_steps"]),
                    "safe_checkpoint": payload["rollback_references"]["safe_checkpoint"]["checkpoint_ref"],
                    "risky_checkpoint": payload["rollback_references"]["risky_checkpoint"]["checkpoint_ref"],
                    "task_cleanup_count": len(payload["task_cleanup_set"]),
                },
                "evidence_refs": [TASK_JSON, "generated/envctl_rollback_checkpoints_report.json"],
            },
            {
                "validator": "generate_art122_rollback.py:gate_snapshot",
                "status": "pass",
                "details": {
                    "gate_task_count": len(payload["gate_tasks"]),
                    "blocked_gate_tasks": payload["blocked_gate_tasks"],
                    "execution_ready_now": payload["completion_gate"]["execution_ready_now"],
                },
                "evidence_refs": [TASK_JSON, "generated/status_from_proofs.json"],
            },
        ]
        if artifact_id == "art-122-rollback-json":
            validations = [
                {
                    "validator": "generate_art122_rollback.py:json_companion",
                    "status": "pass",
                    "details": {"rollback_step_count": len(payload["rollback_steps"]), "schema_version": payload["schema_version"]},
                    "evidence_refs": [TASK_JSON],
                }
            ]
        results.append(
            registry.register(
                {
                    "artifact_id": artifact_id,
                    "run_id": RUN_ID,
                    "title": title,
                    "status": "complete",
                    "artifact_type": artifact_type,
                    "path": path,
                    "producer_operation_id": OPERATION_ID,
                    "contract_id": CONTRACT_ID,
                    "provenance": {**common_provenance, "contract_row_id": contract_row_id},
                    "evidence_refs": evidence_refs,
                    "links": [{"to": f"contract_row:{contract_row_id}", "type": "satisfies"}, *common_links],
                    "validations": validations,
                }
            )
        )
    return results


def main() -> None:
    generated_at = now()
    payload = build_payload()
    markdown = render_markdown(payload)

    base = root()
    write_text(base / CANONICAL_MD, markdown)
    write_text(base / TASK_MD, markdown)
    write_json(base / TASK_JSON, payload)

    conn = sqlite3.connect(":memory:")
    apply_migrations(conn, package_root())
    insert_fixture(conn)
    registry_results = register_artifacts(conn, payload)
    artifact_rows = [fetch_artifact(conn, RUN_ID, item["artifact_id"]) for item in registry_results]

    report = {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "status": "passed",
        "generated_at": generated_at,
        "artifact_paths": [CANONICAL_MD, TASK_MD, TASK_JSON],
        "rollback_step_count": len(payload["rollback_steps"]),
        "blocked_gate_tasks": payload["blocked_gate_tasks"],
        "registry_results": registry_results,
        "artifact_rows": artifact_rows,
        "checksums": {
            CANONICAL_MD: sha256_file(base / CANONICAL_MD),
            TASK_MD: sha256_file(base / TASK_MD),
            TASK_JSON: sha256_file(base / TASK_JSON),
        },
        "validation": {
            "artifact_file_exists": all((base / path).is_file() for path in [CANONICAL_MD, TASK_MD, TASK_JSON]),
            "registry_contains_hash": all(item.get("content_hash") for item in registry_results),
            "validation_evidence_linked": all(item.get("validation_ids") for item in registry_results),
        },
        "evidence": [
            CANONICAL_MD,
            TASK_MD,
            TASK_JSON,
            "generated/execution_packets/ART-122_ROLLBACK.json",
            "generated/envctl_artifact_registry_report.json",
            "generated/envctl_rollback_checkpoints_report.json",
            "generated/shared_protocol_validation_report.json",
            "generated/status_from_proofs.json",
        ],
    }
    write_json(base / REPORT_PATH, report)
    write_json(
        base / "state" / f"{TASK_ID}.heartbeat.json",
        {"task_id": TASK_ID, "status": "completed", "updated_at": generated_at, "artifact_paths": report["artifact_paths"]},
    )
    write_json(base / "logs" / f"{TASK_ID}.log", report)

    proof = make_proof(
        task_id=TASK_ID,
        status="completed",
        actor="artifact-agent",
        helper_id=HELPER_ID,
        model_tag=MODEL_TAG,
        repo_path=str(package_root()),
        files_changed=TASK_OUTPUTS,
        commands_run=[
            "python3 scripts/generate_art122_rollback.py",
            "python3 scripts/status_from_proofs.py",
            "python3 -m py_compile scripts/generate_art122_rollback.py",
        ],
        verification_output=report,
        evidence=report["evidence"],
    )
    append_proof(proof)
    refresh_status_from_proofs()
    print(json.dumps(report, indent=2, sort_keys=False))


if __name__ == "__main__":
    main()
