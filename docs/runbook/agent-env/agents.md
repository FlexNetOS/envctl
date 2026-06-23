# Supported Agents

> **Ported from kasetto.dev/docs** (Kasetto v3.2.0, absorbed into envctl `crates/agent-env`).
> Renamed kasetto→`envctl agent`; `kasetto.yaml`→`agent-env.yaml`; mimalloc removed.
> Source: https://www.kasetto.dev/docs/agents. The standalone `kasetto` binary is retired — this is the `envctl agent` surface.

The 21 agent presets envctl can sync to.

Set the `agent` field in your config and envctl figures out where to put things. Each preset maps to the directory that agent expects.

## Agent Presets

| Agent | Config value | Install path |
| --- | --- | --- |
| Amp | `amp` | `~/.config/agents/skills/` |
| Antigravity | `antigravity` | `~/.gemini/antigravity/skills/` |
| Augment | `augment` | `~/.augment/skills/` |
| Claude Code | `claude-code` | `~/.claude/skills/` |
| Cline | `cline` | `~/.agents/skills/` |
| Codex | `codex` | `~/.codex/skills/` |
| Continue | `continue` | `~/.continue/skills/` |
| Cursor | `cursor` | `~/.cursor/skills/` |
| Gemini CLI | `gemini-cli` | `~/.gemini/skills/` |
| GitHub Copilot | `github-copilot` | `~/.copilot/skills/` |
| Goose | `goose` | `~/.config/goose/skills/` |
| Junie | `junie` | `~/.junie/skills/` |
| Kiro CLI | `kiro-cli` | `~/.kiro/skills/` |
| OpenClaw | `openclaw` | `~/.openclaw/skills/` |
| OpenCode | `opencode` | `~/.config/opencode/skills/` |
| OpenHands | `openhands` | `~/.openhands/skills/` |
| Replit | `replit` | `~/.config/agents/skills/` |
| Roo Code | `roo` | `~/.roo/skills/` |
| Trae | `trae` | `~/.trae/skills/` |
| Warp | `warp` | `~/.agents/skills/` |
| Windsurf | `windsurf` | `~/.codeium/windsurf/skills/` |

## Instruction Destinations

> **envctl note:** the entire `instructions` asset kind is **upstream-only** — it is a post-v3.2.0
> kasetto feature not present in the absorbed `envctl agent` surface. The table below is reproduced
> for reference; envctl does not write to these instruction destinations.

`instructions` land in each agent's native instruction file or directory rather than the skills path
above. Paths are verified against each agent's official docs. **Aggregate** files (one shared file,
many instructions merged via managed comment blocks so your own edits survive) and **directories** (one
file per instruction) are noted below; project paths are relative to the repo root, global paths to `$HOME`.

| Agent | Project instructions path | Global instructions path | Shape |
| --- | --- | --- | --- |
| Claude Code | `CLAUDE.md` | `~/.claude/CLAUDE.md` | aggregate |
| Codex | `AGENTS.md` | `~/.codex/AGENTS.md` | aggregate |
| OpenCode | `AGENTS.md` | `~/.config/opencode/AGENTS.md` | aggregate |
| Amp | `AGENTS.md` | `~/.config/amp/AGENTS.md` | aggregate |
| Gemini CLI | `GEMINI.md` | `~/.gemini/GEMINI.md` | aggregate |
| Antigravity | `GEMINI.md` | `~/.gemini/GEMINI.md` | aggregate |
| GitHub Copilot | `.github/copilot-instructions.md` | `~/.copilot/copilot-instructions.md` | aggregate |
| Junie | `.junie/AGENTS.md` | `~/.junie/AGENTS.md` | aggregate |
| Goose | `.goosehints` | `~/.config/goose/.goosehints` | aggregate |
| Warp | `WARP.md` | — (UI-managed) | aggregate |
| Replit | `replit.md` | — | aggregate |
| OpenHands | `.openhands/microagents/repo.md` | — | aggregate |
| OpenClaw | — (no project instructions) | `~/.openclaw/workspace/AGENTS.md` | aggregate |
| Cursor | `.cursor/rules/<name>.mdc` | `~/.cursor/rules/<name>.mdc` | directory (MDC) |
| Windsurf | `.windsurf/rules/<name>.md` | `~/.codeium/windsurf/memories/global_rules.md` | directory / global aggregate |
| Cline | `.clinerules/<name>.md` | `~/Documents/Cline/Rules/` | directory |
| Continue | `.continue/rules/<name>.md` | — | directory |
| Roo Code | `.roo/rules/<name>.md` | `~/.roo/rules/` | directory |
| Augment | `.augment/rules/<name>.md` | `~/.augment/rules/` | directory |
| Kiro CLI | `.kiro/steering/<name>.md` | `~/.kiro/steering/` | directory |
| Trae | `.trae/rules/<name>.md` | — (UI-managed) | directory |

Only Cursor (`.cursor/rules/*.mdc`) gets reconstructed `description`/`globs`/`alwaysApply`
frontmatter; every other agent receives the Markdown body. Many of these agents also read the
cross-tool `AGENTS.md` standard as a fallback. See the
[configuration reference](./configuration.md) for how instructions are transformed and merged.

## Custom Paths

Don't see your agent? Use the `destination` field to point at any path:

```yaml
destination: ~/.my-custom-agent/skills
```

If both `agent` and `destination` are set, `destination` wins. See the
[configuration reference](./configuration.md) for details.
