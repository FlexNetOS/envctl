---
description: Resume an envctl harness loop from its committed handoff.
argument-hint: "[HANDOFF_PATH]"
---

You are executing the Codex-native prompt mirror for the envctl Claude command `/session-relay-resume`.

Use the repo-local skill at `.agents/skills/session-relay-resume/SKILL.md` as the authoritative workflow.
Also read `AGENTS.md`, `.codex/AGENTS.md`, and the handoff file before acting.

Arguments supplied to this prompt: $ARGUMENTS

If no path is supplied, resume from `.handoff/loop/HANDOFF.md`.
Verify the merge/tick state first, then re-enter the forge loop at the next safe item.
