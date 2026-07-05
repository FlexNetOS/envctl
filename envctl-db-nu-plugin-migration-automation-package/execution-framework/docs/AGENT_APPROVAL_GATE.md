# Agent Approval Gate

The task graph remains the source of truth. Rows with `human_approval_required=true` are no longer bypassed by changing the CSV. They are unlocked only by an auditable agent approval record.

Approval record path:

```bash
execution-framework/approvals/<TASK_ID>.agent_approval.json
```

Required decision:

```json
{
  "task_id": "REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS",
  "reviewer": "claude-opus-4.8",
  "model": "claude-opus-4.8",
  "decision": "approved",
  "reviewed_at": "2026-07-05T00:00:00+00:00",
  "review_artifact": "reviews/<review-file>.md",
  "summary": "short approval summary",
  "evidence": ["reviews/<review-file>.md"]
}
```

Executable checks enforced by `scripts/goal_loop.py`:

- The approval record must include `task_id`, `reviewer`, `model`, `decision`, `reviewed_at`, `review_artifact`, `summary`, and `evidence`.
- The `review_artifact` and every evidence path must exist under `reviews/`, `approvals/`, or `proof_records/`.
- Fallback reviewers require `fallback_reason` explaining why Claude Opus was unavailable.
- A matching `APPROVAL-<TASK_ID>` proof record must exist with `status=completed`, `actor=agent-approval-gate`, matching reviewer/model/decision, evidence for both approval and review artifacts, and checksum bindings for both files.
- Approval and review artifacts must be present in Git's index before `goal_loop.py` dispatches the formerly human-gated packet.
- Denied approvals remain blockers and surface the reviewer's denial instructions in `status_report.json`.

Protocol at every approval gate:

1. Commit all current package changes.
2. Push the branch and open or update a PR.
3. Enable auto-merge on the PR when repository policy allows it.
4. Request review from Claude Opus through the available reviewer path.
5. If Claude Opus is unavailable, use a GPT-5.5 reviewer and record the fallback in the review artifact.
6. If the reviewer denies approval, the denial must include proof and exact change instructions.
7. If the reviewer approves, record the approval with `scripts/agent_approval_gate.py`.
8. Stage the approval record and review artifact, then rerun `scripts/goal_loop.py generated/task_graph.csv`; unlocked packets become dispatchable.

Validation command:

```bash
python3 scripts/validate_agent_approval_gate.py
```

This replaces manual human approval with a reviewable agent gate while keeping the approval decision tracked and reversible.
