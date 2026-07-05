
# Execution Framework Install

## Additive install policy

1. Preserve the source package.
2. Add `execution-framework/` and `execution-templates/`.
3. Record prior state in `history/pre_execution_framework_manifest.json`.
4. Generate task graph and execution packets.
5. Run validation commands.
6. Record proofs and final verification.

## Required command sequence

```bash
cd execution-framework
python3 scripts/validate_task_graph.py generated/task_graph.csv
python3 scripts/task_graph_to_packets.py generated/task_graph.csv
python3 scripts/goal_loop.py generated/task_graph.csv
python3 scripts/verify_history_and_completeness.py
```

## No-downgrade rule

Existing package files remain source-of-truth. This framework is an orchestration layer and must only add or explicitly version new execution material.
