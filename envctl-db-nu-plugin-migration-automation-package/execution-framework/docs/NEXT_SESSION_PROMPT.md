# Next Session Prompt

Use this prompt from a fresh session. Start in:

```bash
cd /home/flexnetos/FlexNetOS/src/envctl
```

Then read this file:

```bash
envctl-db-nu-plugin-migration-automation-package/execution-framework/docs/NEXT_SESSION_PROMPT.md
```

Prompt:

```text
You are Codex continuing the envctl + nu_plugin migration automation package execution.

Do not invent a new implementation path.

Use the package execution framework as the control plane:
- Package path: /home/flexnetos/FlexNetOS/src/envctl/envctl-db-nu-plugin-migration-automation-package
- Task graph source: execution-framework/generated/task_graph.csv
- JSON executable packets: execution-framework/generated/execution_packets/*.json
- Status truth: execution-framework/generated/status_report.json
- Proof truth: execution-framework/proof_records/proof_ledger.jsonl

Required target repos:
- ENVCTL_REPO=/home/flexnetos/FlexNetOS/src/envctl-db-nu-migration-req020
- NU_PLUGIN_REPO=/home/flexnetos/FlexNetOS/src/nu_plugin

Use the Nix/Yazelix RTK path for routine discovery:
- /home/flexnetos/.nix-profile/bin/rtk

Do not rely on ~/.local/bin/rtk. For raw verification evidence, run the command raw and preserve logs.

Start by running from execution-framework:
1. python3 scripts/validate_task_graph.py generated/task_graph.csv
2. python3 scripts/task_graph_to_packets.py generated/task_graph.csv
3. python3 scripts/status_from_proofs.py || true
4. python3 scripts/merge_proofs.py || true
5. python3 scripts/goal_loop.py generated/task_graph.csv
6. python3 scripts/verify_history_and_completeness.py
7. python3 scripts/validate_agent_approval_gate.py

Execution method:
- Read dispatch packets from generated/status_report.json.
- Execute each dispatch packet with:
  codex exec < generated/execution_packets/<TASK_ID>.json
- Do not stop between runnable waves.
- After every packet wave, merge proofs and rerun goal_loop.

Approval-gate rule:
- HUMAN APPROVAL is replaced by AGENT APPROVAL.
- At every approval gate, first commit all current changes, push the branch, open/update a PR, and enable auto-merge when allowed.
- Request review and approval from Claude Opus through the available reviewer path.
- If Claude Opus cannot be reached through weave or the local reviewer path, launch a GPT-5.5 review with codex exec and record the fallback.
- If the reviewer denies approval, the denial must include proof and exact instructions for what must change before execution continues.
- If approved, record approval with execution-framework/scripts/agent_approval_gate.py, stage the approval artifacts, then rerun goal_loop so the JSON packets become dispatchable.
- goal_loop must reject approvals unless the review artifact exists, the approval proof exists, checksums match, fallback reason is present when Claude Opus was unavailable, and approval/review artifacts are in Git's index.

Current expected state:
- Completed proofs: 64
- Agent approval proofs: 3
- Failed packets: 0
- Runnable packets after agent approval: 3
- Approval blockers: 0
- Runnable packets: REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS, REQ-028_ENVCTL_AGENT_CONTROL_API, REQ-033_PLUGIN_HUMAN_APPROVAL
- Blocked dependents: 13
- Final verification status remains pass_with_external_blocker until the three runnable JSON packets execute and write their task proofs.

Continue until the final product is delivered, health is 100% verified, and all live tests have run or exact blockers are recorded with proof.
```
