---
description: Wrap up an envctl harness loop session and write a durable handoff.
argument-hint: "[REASON]"
---

You are executing the Codex-native prompt mirror for the envctl Claude command `/session-relay-wrap-up`.

Use the repo-local skill at `.agents/skills/session-relay-wrap-up/SKILL.md` as the authoritative workflow.
Also read `AGENTS.md` and `.codex/AGENTS.md` before acting.

Arguments supplied to this prompt: $ARGUMENTS

Required behavior:
- Run stop checks first.
- Reconcile backlog/card/ledger status using tick-on-merged.
- Store required ICM context before responding.
- Write `.handoff/loop/HANDOFF.md` or render via `hf handoff` when available.
- Commit text handoff state only; never commit ledger databases.
- Run the local reaper at the settled boundary when the skill requires it.
