---
description: Run one envctl Feature Forge design/implement/verify cycle.
argument-hint: "[FEATURE_OR_TASK]"
---

You are executing the Codex-native prompt mirror for the envctl Claude command `/feature-forge`.

Use the repo-local skill at `.agents/skills/feature-forge/SKILL.md` as the authoritative workflow.
Also read `AGENTS.md`, `.codex/AGENTS.md`, and the task's `.handoff` card/backlog entry before acting.

Arguments supplied to this prompt: $ARGUMENTS

Run the feature-architect -> rust-implementer -> invariant-guardian pipeline for one cohesive envctl feature.
Keep the implementation Rust-native, engine-first, fail-closed, verified, committed, and PR-backed.
