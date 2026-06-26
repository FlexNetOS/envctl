---
description: Orchestrate strict-upgrade-only meta/envctl compliance recovery with background Opus agents.
argument-hint: "[GOAL_OR_RECOVERY_REQUEST]"
allowed-tools: "*"
---

# /goal — meta/envctl strict-upgrade-only recovery

Use `.codex/prompts/goal.md` as the canonical prompt body for this repository. Read it first, then execute it with the arguments below.

Arguments supplied to this command: $ARGUMENTS

Claude Code execution requirements:
- Use Opus for subagents (`model: "opus"`, or explicit `claude-opus-4-8` if this runtime supports full model IDs).
- Spawn the five background agents named by the canonical prompt with `run_in_background: true`.
- Keep read-only researchers read-only; use isolated worktrees for any mutation.
- Preserve the owner contract: strict upgrade only, no downgrades, commit/push/PR/auto-merge for every coherent chunk.
- For handoff/ledger/p7 claims, verify `meta/handoff` source and ADRs first. Current verified contract: committed `.handoff/ledger.events.jsonl` plus rendered text; `.handoff/ledger.db`/RVF are gitignored per-worktree rebuild caches unless `meta/handoff` changes the contract.
