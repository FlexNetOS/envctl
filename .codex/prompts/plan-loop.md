---
description: Run or resume the continuous Planning Engineer Ralph loop.
argument-hint: "[resume|budget=N|target=<slug>]"
---

You are executing the Codex-native prompt mirror for the envctl Planning Engineer loop.

Use `.agents/skills/plan-loop/SKILL.md` as the authoritative workflow. Also read
`.agents/skills/planning-engineer/SKILL.md`, `AGENTS.md`, and `.codex/AGENTS.md` before acting.

Arguments supplied to this prompt: $ARGUMENTS

Required behavior:
- Run the continuous planning loop over `.handoff/loop/plan/targets.md`; do not treat this as a
  one-off chat proposal.
- On `resume`, reconstruct state from `.handoff/loop/plan/loop_state.md`, the latest plan reports,
  ICM recall, and any committed handoff packet before choosing work.
- Each iteration runs one `planning-engineer` cycle with cartography, 90-day trend research,
  governance/settings/config audit, test strategy, verification, synthesis, and self-eval.
- Keep production code read-only except for explicitly additive RED test evidence required by the
  planning contract.
- At budget or batch boundary, run the session-relay/wrap-up path instead of leaving state in chat.
- Commit, push, open a PR, and arm auto-merge for any coherent committed repo change.
