---
description: Run the envctl Feature Forge loop over the durable backlog.
argument-hint: "[resume|budget=N|GOAL]"
---

You are executing the Codex-native prompt mirror for the envctl Claude command `/forge-loop`.

Use the repo-local skill at `.agents/skills/forge-loop/SKILL.md` as the authoritative workflow.
Also read `AGENTS.md` and `.codex/AGENTS.md` before acting.

Arguments supplied to this prompt: $ARGUMENTS

Required behavior:
- Run the Feature Forge loop, not a one-off proposal.
- Use ICM recall before work and ICM store before wrap-up when the skill triggers require it.
- Start from the durable `.handoff` state: `hf resume --json` when available, then task cards, then `.handoff/loop/backlog.md`.
- Honor tick-on-merged: only mark a task done after the PR is confirmed `MERGED`.
- Work in fresh meta-managed worktrees for implementation cycles.
- Verify, commit, push, open PRs, and merge/auto-merge according to the loop gates.
- If cycle budget or batch boundary is reached, invoke the session relay workflow instead of leaving state in chat.

If the user supplied no arguments, resume from `.handoff/loop/HANDOFF.md` and continue the next safe task.
