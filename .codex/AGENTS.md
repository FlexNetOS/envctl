# ECC for Codex CLI

This supplements the root `AGENTS.md` with a repo-local ECC baseline.

## Repo Skill

- Repo-generated Codex skill: `.agents/skills/envctl/SKILL.md`
- Claude-facing companion skill: `.claude/skills/envctl/SKILL.md`
- Keep user-specific credentials and private MCPs in `~/.codex/config.toml`, not in this repo.

## MCP Baseline

Treat `.codex/config.toml` as the default ECC-safe baseline for work in this repository.
The generated baseline enables GitHub, Context7, Exa, Memory, Playwright, and Sequential Thinking.

## Multi-Agent Support

- Explorer: read-only evidence gathering
- Reviewer: correctness, security, and regression review
- Docs researcher: API and release-note verification

## Workflow Files

- `/forge-loop` - `.codex/prompts/forge-loop.md`
- `/prompt:forge-loop` - `.codex/prompts/prompt:forge-loop.md`
- `/feature-forge` - `.codex/prompts/feature-forge.md`
- `/session-relay` - `.codex/prompts/session-relay.md`
- `/prompt:session-relay-wrap-up` - `.codex/prompts/prompt:session-relay-wrap-up.md`
- `/session-relay-handoff` - `.codex/prompts/session-relay-handoff.md`
- `/prompt:session-relay-handoff` - `.codex/prompts/prompt:session-relay-handoff.md`
- `/session-relay-resume` - `.codex/prompts/session-relay-resume.md`
- `/session-relay-wrap-up` - `.codex/prompts/session-relay-wrap-up.md`

Use these workflow files as thin shims only; the authoritative workflow bodies remain in
`.agents/skills/*/SKILL.md` and the durable state remains in `.handoff/`.
