---
description: Alias that hands off the current envctl harness loop session.
argument-hint: "[REASON]"
---

You are executing the Codex-native prompt mirror for the legacy/Claude alias `/session-relay-handoff`.

Treat this as `/session-relay-wrap-up`: use `.agents/skills/session-relay-wrap-up/SKILL.md` as the
authoritative workflow, then write and commit the durable `.handoff/loop/HANDOFF.md` state.

Arguments supplied to this prompt: $ARGUMENTS
