# Introduction — envctl agent (agent-env)

> **Ported from kasetto.dev/docs** (Kasetto v3.2.0, absorbed into envctl `crates/agent-env`).
> Renamed `kst`/`kasetto`→`envctl agent`; `kasetto.yaml`→`agent-env.yaml`; mimalloc removed.
> Source: https://www.kasetto.dev/docs. The standalone `kst`/`kasetto` binary is retired — this
> is the `envctl agent` surface (a declarative AI-agent environment manager).

Get started with **`envctl agent`** — the declarative skills / MCP / slash-command sync engine.

**When you need this:** you want a quick, predictable path to "working sync" and you're not sure
what envctl will modify.

**What you'll learn:**
- How to create a config and run your first sync
- How to preview changes and script CI runs
- Where to go next for the exact sync/merge guarantees

Confirm the surface is available:

```console
$ envctl agent --help
Manage agent assets (skills / MCP servers / commands) declaratively over the shared engine.

Usage: envctl agent <COMMAND>
```

Mutating verbs (`sync` / `add` / `remove` / `clean`) are **PREVIEW by default** — pass `--apply`
to write. `list` and `lock --check` are read-only. Pipe stdout or pass `--color never` to strip
styling; `--json` emits the typed value.

## Creating a Config

Run `envctl agent init` to generate a starter config:

```bash
envctl agent init
```

Use `envctl agent init --scope global` to create the global `agent-env.yaml`.

Or let envctl edit the config for you — `envctl agent add <source>` appends a source (preserving
your comments) and syncs it in, and `envctl agent remove <source>` reverses it, the cargo/uv way:

```bash
envctl agent add https://github.com/org/skill-pack                       # add every skill
envctl agent add https://github.com/org/skill-pack@v1.2.0                # `@<ref>` shorthand (cargo/uv-style)
envctl agent add https://github.com/org/skill-pack --skill code-reviewer # or just named ones
envctl agent add https://github.com/org/skill-pack --dry-run             # preview the edit, don't write
envctl agent remove https://github.com/org/skill-pack --skill code-reviewer  # reverses the add
```

Or create an `agent-env.yaml` by hand:

```yaml
agent: claude-code

skills:
  - source: https://github.com/org/skill-pack
    branch: main
    skills:
      - code-reviewer
      - name: design-system
```

Use the `agent` field to target any of the [supported agents](agents.md), or use the
`destination` field for a custom install path.

## Syncing Skills

Run `envctl agent sync --apply` and envctl does the rest:

```console
$ envctl agent sync --apply
Syncing skills from 1 source...
  ✓ code-reviewer (installed)
  ✓ design-system (installed)
Synced 2 skills in 1.2s
```

envctl pulls the skills and installs them into the right agent directory. Next time you run
sync, only what changed gets updated.

For the exact contract of what gets copied/removed, read [How Sync Works](how-sync-works.md).

## Syncing from a Remote Config

Got a shared team config? Pass it as a URL:

```bash
envctl agent sync --config https://example.com/team-skills.yaml --apply
```

For private configs hosted on git providers, set the right env-var token first (see
[Authentication](authentication.md)).

You can also inherit from another config with `extends:` — a child config pulls in everything
from a team base and overrides or extends it (see [Configuration → Extending Another Config](configuration.md)).

## Previewing Changes

Not ready to commit? `sync` is preview by default (omit `--apply`), or use `--dry-run`:

```console
$ envctl agent sync --dry-run
Would install: code-reviewer, design-system
Would remove: old-skill
```

See [CI & Automation](ci.md) for recommended CI patterns.

## MCP Servers

envctl can also manage MCP server configs. Add an `mcps` section to your config:

```yaml
agent: claude-code

skills:
  - source: https://github.com/org/skill-pack
    skills: "*"

mcps:
  - source: https://github.com/org/mcp-pack
```

MCP servers are discovered and **additively merged** into the agent's MCP config (e.g. Claude's
`.mcp.json`, Codex's `.codex/config.toml`) — existing entries are preserved. See
[How Sync Works → MCP servers: discovery and additive merge](how-sync-works.md).

## Exploring What's Installed

```bash
envctl agent list                 # all installed assets
envctl agent list --kind skills   # filter by kind (skills | mcps | commands)
```

## Using JSON Output

Every command supports `--json` for scripting / CI:

```bash
envctl agent list --json
envctl agent sync --json --apply
```

## Next Steps

- [Installation](installation.md) — how envctl ships the agent-env engine
- [Configuration](configuration.md) — the full `agent-env.yaml` schema + `extends`
- [Commands](commands.md) — every subcommand + flag
- [How Sync Works](how-sync-works.md) / [Sync Flow](sync-flow.md) — the exact sync/merge guarantees
- [Authentication](authentication.md) — tokens for private/remote configs
- [Security](security.md) — the trust model
