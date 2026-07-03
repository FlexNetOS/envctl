# Codex CLI project layer

This supplements the root `AGENTS.md` with the repo-local Codex baseline. The root `AGENTS.md` and the `agent-env-config` skill supersede older ECC-generated JavaScript-convention guidance.

## Repo Skill

- Canonical envctl convention skill: `.agents/skills/agent-env-config/SKILL.md` and `.codex/skills/agent-env-config/SKILL.md`.
- Harness skills live under `.agents/skills/*`; prompt shims live under `.codex/prompts/*`.
- Keep user-specific credentials, provider redirects, and private MCP auth in `~/.codex/config.toml`, not in this repo.

## MCP Baseline

Treat `.codex/config.toml` as the default ECC-safe baseline for work in this repository.
The project baseline enables GitHub, Context7, Exa, Memory, Playwright, Sequential Thinking, and n8n-mcp. Keep MCP definitions synchronized through `agent-env.yaml`/`agent-skills`, not ad-hoc edits.

## Multi-Agent Support

- Explorer: read-only evidence gathering.
- Reviewer: correctness, security, and regression review.
- Docs researcher: API and release-note verification.
- Harness agents: feature-forge, rust-port, continuity, and build-health roles under `.codex/agents/*.toml`.

## Workflow Files

- `/goal` - `.codex/prompts/goal.md` (canonical) and `.claude/commands/goal.md` (Claude Code shim)
- `/planning-engineer` - `.codex/prompts/planning-engineer.md`
- `/plan-loop` - `.codex/prompts/plan-loop.md`
- `/plan-engineering-loop` - `.codex/prompts/plan-engineering-loop.md` (compatibility alias to `/plan-loop`)
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

## Runtime Gates

- The pre-cleanroom Codex runtime gate and `hooks.json` are archived evidence,
  not active lifecycle wiring. Do not restore
  `.codex/hooks/install-flexnetos-runtime-hooks.sh`, `.codex/hooks.json`, or
  `.codex/hooks/flexnetos-runtime-gate.sh` from the archive.
- Hooks remain mandatory for the control plane, but the replacement must be a
  clean-room design owned by the root lifecycle contract. Until that rebuild
  lands, envctl must keep `[features].hooks = false` and purge stale generated
  `~/.codex/hooks.json` state.
- Runtime copies under `/home/flexnetos/workspace/.codex` and
  `/home/flexnetos/FlexNetOS/.codex` must not carry independent lifecycle hook
  policy. Root lifecycle hooks are owned by
  `/home/flexnetos/FlexNetOS/.codex/hooks.json` when the clean-room gate is
  deliberately reintroduced.
