# Codex CLI project layer

This supplements the root `AGENTS.md` with the repo-local Codex baseline. The root `AGENTS.md` and the `agent-env-config` skill supersede older ECC-generated JavaScript-convention guidance.

## Repo Skill

- Canonical envctl convention skill: `.agents/skills/agent-env-config/SKILL.md` and `.codex/skills/agent-env-config/SKILL.md`.
- Harness skills live under `.agents/skills/*`; prompt shims live under `.codex/prompts/*`.
- Keep user-specific credentials, provider redirects, and private MCP auth in `~/.codex/config.toml`, not in this repo.

## MCP Baseline

Treat `.codex/config.toml` as the default ECC-safe baseline for work in this repository.
The generated project baseline currently enables only the remote Exa server. Local-launcher
servers remain retired until their commands have Yazelix-compatible profile-owned frontdoors.
Keep MCP definitions synchronized through `agent-env.yaml`/`agent-skills`, not ad-hoc edits.
This repo-local baseline is not authority to expand the active home Codex
runtime with extra plugin marketplaces, duplicate command inventories, or
cached/not-installed plugin families. Do not infer that `superhuman`,
`digitalocean`, `openai-curated`, or temp plugin cache content should be
restored just because they appear in catalog output or marketplace listings.

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

- Legacy repo-local Codex lifecycle hook sources from the pre-clean-room
  baseline are purged, retired, and archived evidence only. Do not restore
  `.codex/hooks.json`, `.codex/hooks/`, hook install scripts, or regenerated
  hook sources from this repo as active root policy.
- Hooks remain mandatory for the control plane, but the replacement must be a
  clean-room design owned by the root lifecycle contract. Until that rebuild
  lands, envctl-derived Codex config must keep hook features disabled and purge
  stale generated hook state.
- Root lifecycle policy is owned by `/home/flexnetos/AGENTS.md`,
  `/home/flexnetos/.codex/RULES.md`, and the active home/runtime config
  `/home/flexnetos/.codex/config.toml`.
- Runtime copies under `/home/flexnetos/workspace/.codex`,
  `/home/flexnetos/lifeos/.codex`, and `/home/flexnetos/FlexNetOS/.codex`
  are retired. They must not carry or grow independent hook, plugin, MCP,
  marketplace, or instruction policy. If one reappears, archive it and route
  the update through envctl `agent-env.yaml`/`agent-skills` or the active
  `/home/flexnetos/.codex` runtime config.
