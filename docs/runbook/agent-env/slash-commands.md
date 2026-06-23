# Slash Commands

> **Ported from kasetto.dev/docs** (Kasetto v3.2.0, absorbed into envctl `crates/agent-env`).
> Renamed kasetto→`envctl agent`; `kasetto.yaml`→`agent-env.yaml`; mimalloc removed.
> Source: https://www.kasetto.dev/docs/slash-commands. The standalone `kasetto` binary is retired — this is the `envctl agent` surface.

Sync user-defined slash commands (prompt templates) across supported agents.

envctl can sync user-defined **slash commands** (a.k.a. custom prompt templates) into
every agent that supports them. Commands are authored once in Claude-Code-style Markdown
with YAML frontmatter, then transformed on the fly into each agent's native format.

## Source format

A command repository contains a `commands/` directory. Files are Markdown with YAML frontmatter:

```
---
description: Review the given pull request.
argument-hint: <pr-number>
allowed-tools: Bash, Read
model: claude-opus-4-7
---
Review PR $ARGUMENTS and call out anything risky.
```

Subdirectories under `commands/` become `:`-separated namespaces:

- `commands/commit.md` → `commit`
- `commands/git/commit.md` → `git:commit`
- `commands/git/work/status.md` → `git:work:status`

## Config

```yaml
commands:
  - source: https://github.com/me/my-commands
    ref: v1.0
    sub-dir: commands      # optional - subdirectory inside the source repository
    commands: "*"          # discover all, or list explicit entries below

  - source: https://github.com/acme/monorepo
    commands:
      - review-pr          # namespaced name from discovery
      - name: deploy
        path: ops          # → ops/deploy.md in the source
```

The `commands` field accepts the same shapes as `mcps`:

- `"*"` — discover every `.md` file under `commands/`.
- A list of namespaced names (`git:commit`, `review-pr`).
- A list of `{ name, path }` objects to load from custom subdirectories.

## Per-agent output formats

envctl rewrites each command into the format expected by the destination agent:

| Agent | Global directory | Project directory | Format |
| --- | --- | --- | --- |
| claude-code | `~/.claude/commands/` | `.claude/commands/` | Markdown + frontmatter |
| cursor | — | `.cursor/commands/` | Markdown (body only) |
| windsurf | `~/.codeium/windsurf/global_workflows/` | `.windsurf/workflows/` | Markdown + frontmatter |
| cline | — | `.clinerules/workflows/` | Markdown (body only) |
| opencode | `~/.config/opencode/commands/` | `.opencode/commands/` | Markdown + frontmatter |
| continue | `~/.continue/prompts/` | `.continue/prompts/` | `.prompt` file |
| github-copilot | — | `.github/prompts/` | `.prompt.md` |
| amp | `~/.config/amp/commands/` | `.agents/commands/` | Markdown + frontmatter |
| roo | `~/.roo/commands/` | `.roo/commands/` | Markdown + frontmatter |
| augment | `~/.augment/commands/` | `.augment/commands/` | Markdown + frontmatter |
| junie | — | `.junie/commands/` | Markdown + frontmatter |
| openhands | — | `.openhands/microagents/` | Markdown + frontmatter |
| codex | `~/.codex/prompts/` | — | Markdown + frontmatter |
| gemini-cli | `~/.gemini/commands/` | `.gemini/commands/` | TOML |

Agents not listed (antigravity, aider, warp, replit, openclaw, trae, kiro-cli, goose) are
skipped silently for commands.

Continue Dev's `.prompt` files get an `invokable: true` preamble injected automatically,
and envctl best-effort rewrites `$ARGUMENTS` to Continue's `{{{ input }}}` placeholder.

## Stale-file safety

envctl only deletes files it claims to own in the lock file. Pre-existing user-authored
files in the same directory are preserved even if a command is removed from your config.
