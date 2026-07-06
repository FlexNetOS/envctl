# Rollback Plan

- Task: `ART-122_ROLLBACK`
- Contract artifact: `artifact:07-cutover-rollback-plan-md`
- Canonical path: `migration-artifacts/07-cutover/rollback-plan.md`
- Generated at: `2026-07-05T05:06:08+00:00`
- Target: `flexnetos-vs-lifeos`
- Safety mode: `approval-gated`
- Max auto risk: `R2`
- Readiness band: `conditional`
- Execution ready now: `False`

## Gate Summary

| task | title | status | proof |
|---|---|---|---|
| REQ-024_ENVCTL_ARTIFACT_REGISTRY | Implement artifact registry | completed | proof_records/REQ-024_ENVCTL_ARTIFACT_REGISTRY.proof.json |
| REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS | Implement checkpoints and rollback handles | completed | proof_records/REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS.proof.json |
| REQ-027_ENVCTL_REPLAY_ENGINE | Implement replay/reproduce engine | completed | proof_records/REQ-027_ENVCTL_REPLAY_ENGINE.proof.json |
| REQ-040_SHARED_PROTOCOL_SCHEMAS | Implement shared protocol schemas | completed | proof_records/REQ-040_SHARED_PROTOCOL_SCHEMAS.proof.json |
| REQ-041_TWO_REPO_INTEGRATION | Implement two-repo integration | pending | proof_records/REQ-041_TWO_REPO_INTEGRATION.proof.json |
| REQ-045_RUN_REPLAY | Implement run/replay templates | pending | proof_records/REQ-045_RUN_REPLAY.proof.json |
| ART-120_WAVE_PLAN | Build Migration wave plan | completed | proof_records/ART-120_WAVE_PLAN.proof.json |
| ART-121_CUTOVER | Build Cutover checklist | completed | proof_records/ART-121_CUTOVER.proof.json |
| ART-125_RISK_REGISTER | Build Risk register | completed | proof_records/ART-125_RISK_REGISTER.proof.json |
| ART-128_READINESS_SCORECARD | Build Migration readiness scorecard | completed | proof_records/ART-128_READINESS_SCORECARD.proof.json |
| VER-300_UNIT_VALIDATION | Run unit/integration validation | pending | proof_records/VER-300_UNIT_VALIDATION.proof.json |

## Rollback Modes

| mode | use when | approval required | checkpoint |
|---|---|---|---|
| verification_only | Need to prove rollback safety before acting. | no | execution-framework/generated/rollback_checkpoints/safe-artifact.json |
| rerun_from_checkpoint | Artifact-only drift can be recovered without target restore. | no | execution-framework/generated/rollback_checkpoints/safe-artifact.json |
| task_scoped_cleanup | Only ART-122 outputs need to be removed and regenerated. | no | task-output-set |
| restore_checkpoint | Approval-gated target mutation needs pre-operation restore. | yes | history/pre_execution_framework_manifest.json |

## Rollback Steps

| step | phase | status | mode | approval | owner | trigger |
|---|---|---|---|---|---|---|
| RBK-001 | preflight | ready | verification_only | no | artifact-agent | Any operator asks whether rollback is safe to begin. |
| RBK-002 | safe_failback | ready | rerun_from_checkpoint | no | artifact-agent | Artifact content, registry linkage, or proof packaging drift is detected before target mutation. |
| RBK-003 | task_cleanup | ready | task_scoped_cleanup | no | artifact-agent | This rollback plan was generated incorrectly or must be replaced without broader failback. |
| RBK-004 | execution_abort | ready | abort_and_escalate | no | release-operator | Registry hash mismatch, missing validation evidence linkage, or unexpected exception during go-live. |
| RBK-005 | approval_gate | ready | restore_checkpoint | yes | envctl-db-agent | An approval-gated target mutation has already started or safe regeneration cannot recover the package state. |
| RBK-006 | replay_alignment | blocked | post_rollback_validation | no | validation-agent | Rollback has completed and the team is deciding whether a rerun is safe. |
| RBK-007 | closeout | ready | evidence_capture | no | artifact-agent | Any rollback branch completes or is abandoned. |

### RBK-001 - Confirm rollback preconditions before any failback action

- Phase: `preflight`
- Status: `ready`
- Rollback mode: `verification_only`
- Approval required: `no`
- Owner: `artifact-agent`
- Trigger: Any operator asks whether rollback is safe to begin.
- Success criteria:
  - Safe and risky checkpoint references are readable and hash-stable.
  - Current task/proof status is captured before any file removal or restore action.
  - The rollback mode is chosen explicitly from this plan.
- Notes:
  - Safe checkpoint hash: sha256:51c8dd88807da6208ea982acf8ab4d675662b1ae2078f09d341de34fe8de32a6.
  - Risky checkpoint hash: sha256:6bd3e9de4004907c01c8bb45e9e400f537140eadb002f3e068606276a875fb49.
- Evidence refs:
  - `proof_records/REQ-024_ENVCTL_ARTIFACT_REGISTRY.proof.json`
  - `proof_records/REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS.proof.json`
  - `generated/status_from_proofs.json`

### RBK-002 - Use the repeat-safe checkpoint for artifact-only regeneration drift

- Phase: `safe_failback`
- Status: `ready`
- Rollback mode: `rerun_from_checkpoint`
- Approval required: `no`
- Owner: `artifact-agent`
- Trigger: Artifact content, registry linkage, or proof packaging drift is detected before target mutation.
- Success criteria:
  - Rollback is limited to package-generated outputs and registry/proof alignment.
  - Operator can verify from the safe checkpoint without restoring target filesystem state.
  - Evidence shows no approval-gated target mutation has started.
- Notes:
  - Safe checkpoint ref: execution-framework/generated/rollback_checkpoints/safe-artifact.json.
  - Cutover execution-ready flag is currently False.
- Evidence refs:
  - `generated/envctl_rollback_checkpoints_report.json`
  - `migration-artifacts/art-121_cutover/cutover-checklist.json`

### RBK-003 - Remove only ART-122 outputs when rolling back this task itself

- Phase: `task_cleanup`
- Status: `ready`
- Rollback mode: `task_scoped_cleanup`
- Approval required: `no`
- Owner: `artifact-agent`
- Trigger: This rollback plan was generated incorrectly or must be replaced without broader failback.
- Success criteria:
  - All ART-122 task outputs are removed as one set.
  - Proof ledger no longer advertises ART-122 as complete after regeneration or cleanup.
  - No unrelated artifact files are touched.
- Notes:
  - Task-scoped cleanup set has 10 paths.
- Cleanup set:
  - `execution-framework/scripts/generate_art122_rollback.py`
  - `execution-framework/migration-artifacts/07-cutover/rollback-plan.md`
  - `execution-framework/migration-artifacts/art-122_rollback/rollback-plan.md`
  - `execution-framework/migration-artifacts/art-122_rollback/rollback-plan.json`
  - `execution-framework/generated/art122_rollback_report.json`
  - `execution-framework/generated/status_from_proofs.json`
  - `execution-framework/state/ART-122_ROLLBACK.heartbeat.json`
  - `execution-framework/logs/ART-122_ROLLBACK.log`
  - `execution-framework/proof_records/ART-122_ROLLBACK.proof.json`
  - `execution-framework/proof_records/proof_ledger.jsonl`
- Evidence refs:
  - `migration-artifacts/art-125_risk_register/risk-register.json`
  - `generated/execution_packets/ART-122_ROLLBACK.json`

### RBK-004 - Abort the cutover if validation or registry integrity fails mid-window

- Phase: `execution_abort`
- Status: `ready`
- Rollback mode: `abort_and_escalate`
- Approval required: `no`
- Owner: `release-operator`
- Trigger: Registry hash mismatch, missing validation evidence linkage, or unexpected exception during go-live.
- Success criteria:
  - No additional execution steps proceed after the trigger is detected.
  - The current event/proof state is captured for operator review.
  - The next rollback branch is chosen based on whether the safe or risky boundary applies.
- Notes:
  - This is the operational handoff from cutover checklist abort criteria into rollback handling.
- Evidence refs:
  - `generated/envctl_artifact_registry_report.json`
  - `migration-artifacts/art-121_cutover/cutover-checklist.md`
  - `migration-artifacts/art-125_risk_register/risk-register.md`

### RBK-005 - Escalate to the approval-gated restore checkpoint for risky rollback

- Phase: `approval_gate`
- Status: `ready`
- Rollback mode: `restore_checkpoint`
- Approval required: `yes`
- Owner: `envctl-db-agent`
- Trigger: An approval-gated target mutation has already started or safe regeneration cannot recover the package state.
- Success criteria:
  - Approval row exists before restore begins.
  - Restore uses the exact checkpoint reference and hash from REQ-026.
  - Operator records why the safe boundary was insufficient.
- Notes:
  - Risky checkpoint ref: history/pre_execution_framework_manifest.json.
  - Risky rollback status from REQ-026 smoke: planned.
- Evidence refs:
  - `generated/envctl_rollback_checkpoints_report.json`
  - `proof_records/REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS.proof.json`

### RBK-006 - Reconcile rollback with replay and validation prerequisites before reattempt

- Phase: `replay_alignment`
- Status: `blocked`
- Rollback mode: `post_rollback_validation`
- Approval required: `no`
- Owner: `validation-agent`
- Trigger: Rollback has completed and the team is deciding whether a rerun is safe.
- Success criteria:
  - REQ-041_TWO_REPO_INTEGRATION is completed before unattended rerun.
  - REQ-045_RUN_REPLAY is completed before replaying operator steps from this package.
  - VER-300_UNIT_VALIDATION is completed before declaring rollback recovery releasable.
- Notes:
  - Current statuses: REQ-041=pending, REQ-045=pending, VER-300=pending.
  - Readiness band remains conditional.
- Evidence refs:
  - `generated/status_from_proofs.json`
  - `migration-artifacts/art-128_readiness_scorecard/readiness-scorecard.json`

### RBK-007 - Record rollback outcome and residual risk after the chosen branch

- Phase: `closeout`
- Status: `ready`
- Rollback mode: `evidence_capture`
- Approval required: `no`
- Owner: `artifact-agent`
- Trigger: Any rollback branch completes or is abandoned.
- Success criteria:
  - Outcome states whether rollback was verify-only, task cleanup, or checkpoint restore.
  - Residual blockers and risks are linked back to the readiness and risk artifacts.
  - Follow-up action names the next validation or replay gate to clear.
- Evidence refs:
  - `generated/status_from_proofs.json`
  - `migration-artifacts/art-125_risk_register/risk-register.json`
  - `migration-artifacts/art-128_readiness_scorecard/readiness-scorecard.json`

## Task-Scoped Cleanup Set

Remove this exact set for an ART-122-only rollback:
- `execution-framework/scripts/generate_art122_rollback.py`
- `execution-framework/migration-artifacts/07-cutover/rollback-plan.md`
- `execution-framework/migration-artifacts/art-122_rollback/rollback-plan.md`
- `execution-framework/migration-artifacts/art-122_rollback/rollback-plan.json`
- `execution-framework/generated/art122_rollback_report.json`
- `execution-framework/generated/status_from_proofs.json`
- `execution-framework/state/ART-122_ROLLBACK.heartbeat.json`
- `execution-framework/logs/ART-122_ROLLBACK.log`
- `execution-framework/proof_records/ART-122_ROLLBACK.proof.json`
- `execution-framework/proof_records/proof_ledger.jsonl`

## Validation Links

- Depends on `REQ-024_ENVCTL_ARTIFACT_REGISTRY` for artifact hashes, evidence refs, and graph links.
- Depends on `REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS` for the repeat-safe and approval-gated checkpoint references.
- Depends on `REQ-040_SHARED_PROTOCOL_SCHEMAS` so proof and artifact payloads remain contract-compatible.
- Blocks `VER-300_UNIT_VALIDATION` until this rollback plan is registered with validation evidence.
