# Run Replay

This page is the verified operator entrypoint for computing the current plan, dispatching the approved REQ-045 packet, exercising replay checks, validating rollback protections, and re-running validation.

## Preconditions

- `REQ-044_INSTALL_BOOTSTRAP` proof is complete.
- `REQ-027_ENVCTL_REPLAY_ENGINE` proof is complete.
- `APPROVAL-REQ-045_RUN_REPLAY` approval proof is complete.

## Initial Plan

```bash
cd "$PROMPT_PACKAGE_DIR/execution-framework"
python3 scripts/goal_loop.py generated/task_graph.csv
```

Use the refreshed `state/goal_loop_state.json` before dispatching helpers.

## Dispatch Approved Packet

```bash
cd "$PROMPT_PACKAGE_DIR/execution-framework"
codex exec < generated/execution_packets/REQ-045_RUN_REPLAY.json
```

## Replay Dry-Run

```bash
cd "$PROMPT_PACKAGE_DIR/execution-framework"
envctl replay dry-run \
  --run-id run-req027-replay \
  --replay-id replay-req045-dry-run \
  --requested-by helper-replay-template-01 \
  --operation-ids op-req027-replay-hash \
  --reason "verify deterministic replay inputs before integration validation"
```

## Replay Apply Blocked Check

```bash
cd "$PROMPT_PACKAGE_DIR/execution-framework"
envctl replay apply \
  --run-id run-req027-replay \
  --replay-id replay-req045-apply-blocked \
  --requested-by helper-replay-template-01 \
  --operation-ids op-req027-manual-cutover \
  --reason "prove approval-gated apply replay remains blocked without operator release"
```

This command is expected to remain blocked until the operator approval path is satisfied.

## Rollback Safety Checks

```bash
cd "$PROMPT_PACKAGE_DIR/execution-framework"
python3 scripts/verify_rollback_checkpoints.py
```

## Validation

```bash
cd "$PROMPT_PACKAGE_DIR/execution-framework"
python3 scripts/verify_replay_engine.py
python3 scripts/verify_history_and_completeness.py
test -s proof_records/REQ-045_RUN_REPLAY.proof.json
```

## Convenience Copies

Top-level `execution-templates/` remains a read-only convenience surface in this runtime. The canonical generated run/replay artifacts for this task live under `execution-framework/` and the verifier only checks that the convenience copy index remains present.

## Command Template Index

| command id | purpose |
|---|---|
| `compute-initial-plan` | Regenerate runnable and approval-blocked packet state before dispatching replay-related work. |
| `dispatch-approved-run-replay-packet` | Run the bounded REQ-045 packet after its approval artifact has been recorded. |
| `replay-dry-run` | Replay the deterministic operation from REQ-027 without mutating targets. |
| `replay-apply-blocked-check` | Exercise the high-risk replay path in a way that must remain blocked until an operator approval exists. |
| `verify-rollback-checkpoints` | Confirm rollback handles and checkpoint protections remain valid before replay execution advances. |
| `verify-replay-validation` | Re-run replay verification plus completeness checks after command-template generation. |
