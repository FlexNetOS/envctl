---
description: Run one evidence-backed Planning Engineer architecture cycle.
argument-hint: "<planning-target>"
allowed-tools: "*"
---

# /planning-engineer — single Planning Engineer cycle

Use `.codex/prompts/planning-engineer.md` as the canonical prompt body for this repository. Read it
first, then execute exactly one planning cycle with the arguments below.

Arguments supplied to this command: $ARGUMENTS

Claude Code execution requirements:
- Treat this as one `planning-engineer` cycle. For continuous backlog execution, route to `/plan-loop` instead.
- Use `.claude/skills/planning-engineer/SKILL.md` as the authoritative Claude-side workflow mirror and preserve the PromptHub source alignment named by the canonical Codex prompt.
- Launch the five required Opus background lanes through weave while keeping the foreground chat interactive.
- Keep production code read-only except permitted planning artifacts and explicitly additive RED test evidence required by the planning contract.
- Verify claims adversarially before they enter the plan; unverified claims stay out of the roadmap.
- Preserve the owner contract: strict upgrade only, no downgrades/destructive resets, and commit/push/PR/auto-merge for every coherent repo artifact change.
