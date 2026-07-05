
# /goal Loop Protocol

The /goal loop converts a migration package into repeatable agent-controlled execution.

```text
Task Graph
→ normalized graph
→ executable packets
→ runnable task selection
→ agent/background-helper execution
→ proof records
→ proof ledger
→ status report
→ repeat until complete, blocked, or failed
```

## Rules

- A task is runnable only when all dependencies are complete.
- Parallel groups must respect `can_run_parallel` and `max_parallel`.
- Human approval tasks stop the loop until an approval proof appears.
- No task can be considered complete without a proof record in `proof_records/proof_ledger.jsonl`.
- Failed verification stops downstream tasks that depend on the failed node.
- Execution packets are bounded; no packet should require rereading the full package.
