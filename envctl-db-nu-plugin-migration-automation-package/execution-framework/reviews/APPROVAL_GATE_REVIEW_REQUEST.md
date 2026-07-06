# Approval Gate Review Request

Reviewer target: Claude Opus.

If Claude Opus is unavailable, fallback reviewer target: GPT-5.5.

## Scope

Review the current approval-gated state of the envctl + nu_plugin migration automation package and decide whether these formerly human-gated packets may be unlocked for agent execution:

- `REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS`
- `REQ-028_ENVCTL_AGENT_CONTROL_API`
- `REQ-033_PLUGIN_HUMAN_APPROVAL`

## Required Evidence To Inspect

- `execution-framework/generated/task_graph.csv`
- `execution-framework/generated/execution_packets/REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS.json`
- `execution-framework/generated/execution_packets/REQ-028_ENVCTL_AGENT_CONTROL_API.json`
- `execution-framework/generated/execution_packets/REQ-033_PLUGIN_HUMAN_APPROVAL.json`
- `execution-framework/generated/status_report.json`
- `execution-framework/generated/final_verification_report.json`
- `execution-framework/docs/AGENT_APPROVAL_GATE.md`
- `execution-framework/docs/NEXT_SESSION_PROMPT.md`
- `execution-framework/scripts/goal_loop.py`
- `execution-framework/scripts/agent_approval_gate.py`

## Review Contract

Return one of:

```json
{"decision":"approved","reason":"...","approved_tasks":["REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS","REQ-028_ENVCTL_AGENT_CONTROL_API","REQ-033_PLUGIN_HUMAN_APPROVAL"],"required_changes":[]}
```

or:

```json
{"decision":"denied","reason":"...","proof":["..."],"required_changes":["..."]}
```

Do not approve if the next session prompt does not force CSV graph plus JSON packet execution. Do not approve if approval can be bypassed without a tracked review artifact. Do not approve if denial instructions are not explicit.
