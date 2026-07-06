# CODEX FINAL EXECUTION PROMPT — envctl + nu_plugin migration automation

You are Codex running locally on Ubuntu 26.04+.

Package root to use:

```bash
cd ~/envctl-db-nu-plugin-migration-automation-package
```

Mission:
Build the final product by executing the package's database-backed migration automation design against the real local repos. This is real execution only. No simulation. No demo. No fabricated repo structure. No destructive migration without explicit approval.

Critical navigation order:
1. Read `README.md` first, especially `Agent navigation + backtrace metadata`.
2. Confirm `history/v0` through `history/v5` exist and preserve upgrade-only/no-downgrade lineage.
3. Use `execution-framework/generated/task_graph.csv` as the source of task truth.
4. Use `execution-framework/generated/execution_manifest.json` to find the JSON executable packets.
5. Execute bounded JSON packets from `execution-framework/generated/execution_packets/`.
6. Start at runnable task `REQ-010_CONTRACT_LOCK` unless a newer `status_report.json` says otherwise.
7. Write proof records to `execution-framework/proof_records/` and merge into `proof_ledger.jsonl` after every completed task.
8. Recompute status with the framework scripts before reporting completion.

Known no-gap state:
- Live Drive bookkeeping gaps from `DEEP_VERSION_GAP_ANALYSIS_2026-07-04_envctl_package` are closed.
- Maintenance packets/proofs exist for README backtrace navigation, proof template restore, live manifest/verification sync, and final Codex handoff.
- `execution-framework/proof_templates/PROOF_RECORD_TEMPLATE.json` is present.
- `final_verification_report.json` status is `pass_no_gaps_drive_live_synchronized`.
- envctl/nu_plugin implementation tasks are still pending by design; that is your work.

Required local repos:
- envctl repo path: inspect the user's local target, do not invent it.
- nu_plugin repo path: inspect the user's local target, do not invent it.

If repo paths are not provided by command-line args or environment variables, stop with `HARD STOP — REPO_PATHS_NOT_PROVIDED` and list the exact missing values. Do not create fake repos.

Execution commands to run from package root before implementation:

```bash
cd execution-framework
python3 scripts/validate_task_graph.py generated/task_graph.csv
python3 scripts/task_graph_to_packets.py generated/task_graph.csv
python3 scripts/goal_loop.py generated/task_graph.csv
python3 scripts/verify_history_and_completeness.py
```

If paths differ after extraction, adapt only the path prefix; do not change task IDs or contract semantics.

Implementation objective:
Make envctl database features perform the migration process as built-in, agent-controllable CLI/database tooling, with nu_plugin as the live human/agent control and visualization surface.

Minimum product deliverables:
1. envctl migration automation database schema/migrations.
2. envctl target descriptor registry/parser/validator.
3. envctl migration recipe loader.
4. envctl operation/event ledger.
5. envctl artifact contract registry.
6. envctl approval/checkpoint/rollback/replay model.
7. envctl adapter to import/use this package and the prior FlexNetOS package as real fixtures.
8. nu_plugin commands for live status, visual tables, graph views, approvals, replay, artifacts, and run control.
9. Shared protocol schemas between envctl and nu_plugin.
10. Tests proving run creation, event append, artifact registration, approval flow, replay, and plugin output.
11. Final verification showing no package/task/packet/proof gaps and no downgrade.

Policy:
- Upgrade-only. No downgrade.
- Additive first. Preserve source history.
- Do not expose secrets.
- Every task requires a proof record.
- Every claimed completion must cite changed files, commands run, verification output, and rollback path.
- If a command fails because dependencies are missing, record the exact command and exact blocker in the proof record.

Completion gate:
The job is not done until:
- `execution-framework/generated/status_report.json` shows all required implementation tasks complete or explicitly blocked with evidence.
- `execution-framework/generated/final_verification_report.json` reports no unresolved gaps.
- `execution-framework/proof_records/proof_ledger.jsonl` includes every completed task.
- envctl and nu_plugin repo checks have been run or exact blockers recorded.
- README, manifest, task graph, execution manifest, JSON packets, and proof ledger agree.

Final response format:
Return concise completion report with changed files, commands run, tests/verification, blockers if any, and next executable task if not complete.
