---
description: Hand off or resume an envctl harness loop across sessions.
argument-hint: ARGS="handoff | resume | reason / checkpoint path"
---

Execute the envctl session-relay workflow from Codex.

Arguments: $ARGS

Use the Codex mirror first:

1. Read `.agents/skills/session-relay/SKILL.md`.
2. If that file is missing or incomplete, compare with `.claude/skills/session-relay/SKILL.md`.
3. For a fuller closeout, prefer `/session-relay-wrap-up`; for a fresh process, prefer `/session-relay-resume`.
4. Treat the committed `.handoff/loop/HANDOFF.md` or `hf` packet as the resume signal. Weave is only a heartbeat.
5. Do not schedule or re-fire another loop turn after HAND OFF.
