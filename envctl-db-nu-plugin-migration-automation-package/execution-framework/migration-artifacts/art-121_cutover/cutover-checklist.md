# Cutover Checklist

- Task: `ART-121_CUTOVER`
- Contract artifact: `artifact:07-cutover-cutover-checklist-md`
- Canonical path: `migration-artifacts/07-cutover/cutover-checklist.md`
- Generated at: `2026-07-05T05:02:33+00:00`
- Target: `flexnetos-vs-lifeos`
- Safety mode: `approval-gated`
- Readiness band: `conditional`
- Execution ready now: `False`

## Gate Summary

| task | title | status | proof |
|---|---|---|---|
| REQ-024_ENVCTL_ARTIFACT_REGISTRY | Implement artifact registry | completed | proof_records/REQ-024_ENVCTL_ARTIFACT_REGISTRY.proof.json |
| REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS | Implement checkpoints and rollback handles | completed | proof_records/REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS.proof.json |
| REQ-040_SHARED_PROTOCOL_SCHEMAS | Implement shared protocol schemas | completed | proof_records/REQ-040_SHARED_PROTOCOL_SCHEMAS.proof.json |
| REQ-041_TWO_REPO_INTEGRATION | Implement two-repo integration | pending | proof_records/REQ-041_TWO_REPO_INTEGRATION.proof.json |
| REQ-045_RUN_REPLAY | Implement run/replay templates | pending | proof_records/REQ-045_RUN_REPLAY.proof.json |
| ART-120_WAVE_PLAN | Build Migration wave plan | completed | proof_records/ART-120_WAVE_PLAN.proof.json |
| ART-122_ROLLBACK | Build Rollback plan | pending | proof_records/ART-122_ROLLBACK.proof.json |
| ART-125_RISK_REGISTER | Build Risk register | completed | proof_records/ART-125_RISK_REGISTER.proof.json |
| ART-128_READINESS_SCORECARD | Build Migration readiness scorecard | completed | proof_records/ART-128_READINESS_SCORECARD.proof.json |
| VER-300_UNIT_VALIDATION | Run unit/integration validation | pending | proof_records/VER-300_UNIT_VALIDATION.proof.json |

## Checklist

| step | phase | status | blocking | title | intent | owner |
|---|---|---|---|---|---|---|
| CUT-001 | pre_cutover | ready | no | Confirm control-plane gating artifacts are complete | Do not start go-live work until registry, shared schemas, and rollback checkpoint capabilities are proven. | artifact-agent |
| CUT-002 | pre_cutover | blocked | yes | Hold execution until validation and replay prerequisites clear | Make the current package state explicit: cutover planning exists, but live execution is still gated by unfinished validation and replay work. | validation-agent |
| CUT-003 | pre_cutover | ready | no | Review migration wave ordering and owner assignments | Use the registered wave plan, risk register, and readiness scorecard as the single checklist source before issuing a go-live window. | lane_d_filesystem |
| CUT-004 | execution_window | ready | no | Freeze artifact inputs and record the exact release boundary | Avoid a moving target by pinning the descriptor, task graph, and proof/status projections used to justify the cutover. | artifact-agent |
| CUT-005 | execution_window | ready | no | Publish the operator start signal with rollback checkpoint references | Every go-live attempt should name the safe rerun boundary and the approval-gated restore boundary before any irreversible action starts. | envctl-db-agent |
| CUT-006 | execution_window | blocked | yes | Run the validated migration sequence only after gate clearance | Execute the move order from W6 after validation says the package is actually releasable. | release-operator |
| CUT-007 | stabilization | ready | no | Capture post-cutover validation evidence and parity outcome | Document whether the execution achieved the intended state and whether any rollback or exception path was needed. | validation-agent |
| CUT-008 | abort_criteria | ready | no | Abort and escalate if rollback triggers fire | Stop the cutover if registry integrity, validation, or operator approval expectations are violated. | release-operator |

### CUT-001 - Confirm control-plane gating artifacts are complete

- Phase: `pre_cutover`
- Status: `ready`
- Blocking: `no`
- Owner: `artifact-agent`
- Success criteria:
  - REQ-024, REQ-026, and REQ-040 show completed proof records.
  - Artifact registry hashes and validation links exist for new cutover outputs.
- Evidence refs:
  - `proof_records/REQ-024_ENVCTL_ARTIFACT_REGISTRY.proof.json`
  - `proof_records/REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS.proof.json`
  - `proof_records/REQ-040_SHARED_PROTOCOL_SCHEMAS.proof.json`

### CUT-002 - Hold execution until validation and replay prerequisites clear

- Phase: `pre_cutover`
- Status: `blocked`
- Blocking: `yes`
- Owner: `validation-agent`
- Success criteria:
  - REQ-041_TWO_REPO_INTEGRATION is completed.
  - REQ-045_RUN_REPLAY is completed.
  - VER-300_UNIT_VALIDATION is completed.
- Notes:
  - Current statuses: REQ-041=pending, REQ-045=pending, VER-300=pending.
  - Readiness band remains conditional.
- Evidence refs:
  - `generated/status_from_proofs.json`
  - `migration-artifacts/art-128_readiness_scorecard/readiness-scorecard.md`

### CUT-003 - Review migration wave ordering and owner assignments

- Phase: `pre_cutover`
- Status: `ready`
- Blocking: `no`
- Owner: `lane_d_filesystem`
- Success criteria:
  - Wave W6 still reflects the intended cutover ordering.
  - Open high-severity risks have a named owner and mitigation.
  - Conditional or blocked readiness domains are acknowledged in the release decision.
- Evidence refs:
  - `migration-artifacts/art-120_wave_plan/wave-plan.md`
  - `migration-artifacts/art-125_risk_register/risk-register.md`
  - `migration-artifacts/art-128_readiness_scorecard/readiness-scorecard.md`

### CUT-004 - Freeze artifact inputs and record the exact release boundary

- Phase: `execution_window`
- Status: `ready`
- Blocking: `no`
- Owner: `artifact-agent`
- Success criteria:
  - The target descriptor path set matches the execution window.
  - The task graph and packet list have no unreviewed changes.
  - The status snapshot used for sign-off is archived with the cutover evidence.
- Evidence refs:
  - `migration-artifacts/_meta/flexnetos-vs-lifeos.target-descriptor.yaml`
  - `generated/task_graph.csv`
  - `generated/status_from_proofs.json`

### CUT-005 - Publish the operator start signal with rollback checkpoint references

- Phase: `execution_window`
- Status: `ready`
- Blocking: `no`
- Owner: `envctl-db-agent`
- Success criteria:
  - Safe checkpoint reference is available for repeatable regeneration.
  - High-risk restore checkpoint is identified for operator escalation.
  - Approval requirement for risky rollback is recorded in the cutover notes.
- Notes:
  - Safe checkpoint: execution-framework/generated/rollback_checkpoints/safe-artifact.json.
  - Risky checkpoint: history/pre_execution_framework_manifest.json.
- Evidence refs:
  - `generated/envctl_rollback_checkpoints_report.json`
  - `proof_records/REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS.proof.json`

### CUT-006 - Run the validated migration sequence only after gate clearance

- Phase: `execution_window`
- Status: `blocked`
- Blocking: `yes`
- Owner: `release-operator`
- Success criteria:
  - All blocking tasks in the checklist are complete.
  - The release operator confirms the exact run and replay instructions to use.
  - No new unreviewed exceptions were added after sign-off.
- Notes:
  - This step is intentionally blocked in the current package snapshot because validation and replay tasks are still pending.
- Evidence refs:
  - `migration-artifacts/art-120_wave_plan/wave-plan.md`
  - `generated/status_from_proofs.json`

### CUT-007 - Capture post-cutover validation evidence and parity outcome

- Phase: `stabilization`
- Status: `ready`
- Blocking: `no`
- Owner: `validation-agent`
- Success criteria:
  - Validation evidence links are attached to the release run.
  - Parity and shadow-traffic results are summarized in the handoff packet.
  - Any deviation from the planned cutover is logged with owner and follow-up.
- Evidence refs:
  - `proof_records/REQ-025_ENVCTL_VALIDATION_EVIDENCE.proof.json`
  - `migration-artifacts/06-testing-validation/parity-dashboard.md`
  - `migration-artifacts/06-testing-validation/shadow-traffic-comparison-report.md`

### CUT-008 - Abort and escalate if rollback triggers fire

- Phase: `abort_criteria`
- Status: `ready`
- Blocking: `no`
- Owner: `release-operator`
- Success criteria:
  - Abort immediately on missing artifact hash registration.
  - Abort immediately on failed validation evidence linkage.
  - Escalate before any risky rollback that requires approval is attempted.
- Evidence refs:
  - `generated/envctl_artifact_registry_report.json`
  - `generated/envctl_rollback_checkpoints_report.json`
  - `migration-artifacts/art-125_risk_register/risk-register.md`

## Rollback Anchors

- Safe checkpoint: `execution-framework/generated/rollback_checkpoints/safe-artifact.json`
- Risky checkpoint: `history/pre_execution_framework_manifest.json`
- Risky rollback approval required: `True`

## Validation Links

- Depends on `REQ-024_ENVCTL_ARTIFACT_REGISTRY` for content hashes and registry rows.
- Depends on `REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS` for replay-safe and approval-gated rollback references.
- Depends on `REQ-040_SHARED_PROTOCOL_SCHEMAS` for shared proof and artifact compatibility.
- Blocks `VER-300_UNIT_VALIDATION` until this checklist is registered with validation evidence.
