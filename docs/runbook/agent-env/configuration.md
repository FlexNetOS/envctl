# Configuration

> **Ported from kasetto.dev/docs** (Kasetto v3.2.0, absorbed into envctl `crates/agent-env`).
> Renamed kasetto→`envctl agent`; `kasetto.yaml`→`agent-env.yaml`; mimalloc removed.
> Source: https://www.kasetto.dev/docs/configuration. The standalone `kasetto` binary is retired — this is the `envctl agent` surface.

**envctl note — the `instructions` asset kind:** The live kasetto.dev docs describe a **four-kind**
model (skills / commands / MCPs / **instructions**). envctl absorbed **kasetto v3.2.0**, whose schema
is **6 keys + `extends`**: `destination`, `scope`, `agent`, `skills`, `mcps`, `commands`. The
`instructions:` config key, the `--instruction` flag, and the per-agent instruction-file transforms
are a **post-v3.2.0 kasetto addition that was NOT absorbed into envctl**. The instructions sections
below are reproduced for fidelity to the source doc but **do not apply to the current `envctl agent`
surface** — treat them as upstream reference, not as envctl behavior.

Configure your `agent-env.yaml` — sources, agents, scope, and extends.

When `--config` is omitted, envctl looks for config in this order:

1. `$ENVCTL_AGENT_CONFIG` env var
2. `./agent-env.yaml`
3. `source:` key in `$XDG_CONFIG_HOME/agent-env/config.yaml`
4. `$XDG_CONFIG_HOME/agent-env/agent-env.yaml` (or `~/.config/agent-env/agent-env.yaml`)

Point it at a specific file or URL with `--config`, or run `envctl agent init` for a local
`./agent-env.yaml` (`envctl agent init --scope global` writes the global config file).
To persist a remote URL as your default, add a `source:` key to `~/.config/agent-env/config.yaml`.

## Example

```yaml
# Option A: preset destination by agent (see README for supported agent values)
agent:
  - codex
  - claude-code

# Option B: manual destination (takes precedence if both are set)
# destination: ./.agents/skills

skills:
  # "*" syncs every skill in the source — each is a directory with a SKILL.md,
  # discovered in the source root or its skills/ subdirectory
  - source: https://github.com/vercel-labs/next-skills
    # ref: v1.0.0   # pin to a tag or commit; omit to track the default branch
    skills: "*"

  # or list skills by name
  - source: https://github.com/anthropics/skills
    skills:
      - doc-coauthoring
      - frontend-design
      - pptx

  # sub-dir: resolve the named skills under this path, e.g. skills/productivity/grill-me/
  - source: https://github.com/mattpocock/skills
    sub-dir: skills/productivity
    skills:
      - grill-me
      - caveman

  # path: a skill in a non-standard location → <path>/<name>/, here skills/engineering/improve-codebase-architecture/
  - source: https://github.com/mattpocock/skills
    skills:
      - name: improve-codebase-architecture
        path: skills/engineering

commands:
  # names resolve to commands/<name>.md in the source (nested dirs namespace, e.g. git:commit)
  - source: https://github.com/gsd-build/get-shit-done
    commands:
      - gsd:explore
      - gsd:fast

mcps:
  # names resolve to mcps/<name>.json in the source
  - source: https://github.com/pivoshenko/pivoshenko.ai
    branch: main   # track a specific branch (use ref: to pin a tag or commit)
    mcps:
      - github
      - vercel
      - kaggle
```

> **envctl note:** an `instructions:` block (shown in the upstream doc) is **not part of the
> absorbed v3.2.0 schema** and is omitted from the example above.

## Reference

### Top-Level Fields

| Key | Required | Description |
| --- | --- | --- |
| `agent` | no | One or more supported agent presets — string or list |
| `destination` | no | Explicit install path — overrides `agent` if both are set |
| `scope` | no | `"global"` (default) or `"project"` — where to install |
| `skills` | **yes** | List of skill sources |
| `mcps` | no | List of MCP server sources |
| `commands` | no | List of slash-command sources |
| `instructions` | no | _(upstream-only; not in absorbed v3.2.0)_ List of instruction sources (CLAUDE.md / .cursor/rules / AGENTS.md …) |
| `extends` | no | Path or URL of a parent config to inherit from (string or list) |

### Skill Source Fields

| Key | Required | Description |
| --- | --- | --- |
| `source` | **yes** | Git host URL or local path (GitHub, GitLab, Bitbucket, Codeberg/Gitea) |
| `branch` | no | Branch for remote sources (default: `main`, falls back to `master`) |
| `ref` | no | Git tag, commit SHA, or ref — takes priority over `branch` |
| `sub-dir` | no | Relative subdirectory within the source used as the discovery root (`sub_dir` alias supported) |
| `skills` | **yes** | `"*"` for all, or a list of names / `{ name, path }` objects |

### Skill Entry Fields

Each entry in the `skills` list can be a string (the skill name) or an object:

| Key | Required | Description |
| --- | --- | --- |
| `name` | **yes** | Name of the skill directory to install |
| `path` | no | Parent directory containing `<name>/SKILL.md`, resolved relative to the source root (or `sub-dir` if set). Absolute paths are honored as-is. |

| Form | Resolves to |
| --- | --- |
| `- code-reviewer` | discovered (root or `skills/`) |
| `- { name: x }` | discovered (root or `skills/`) |
| `- { name: x, path: dir }` | `dir/x/SKILL.md` |
| `- { name: x, path: nested/dir }` | `nested/dir/x/SKILL.md` |

### MCP Source Fields

| Key | Required | Description |
| --- | --- | --- |
| `source` | **yes** | Git host URL or local path containing MCP server config |
| `branch` | no | Branch for remote sources (default: `main`, falls back to `master`) |
| `ref` | no | Git tag, commit SHA, or ref — takes priority over `branch` |
| `mcps` | **yes** | `"*"` to discover all, or a list of names / `{ name, path }` objects |

When `mcps: "*"`, envctl auto-discovers MCP config files in this order:

1. `.mcp.json` at the source root
2. `mcp.json` at the source root
3. Any `.json` file inside the `mcps/` subdirectory

### MCP Entry Fields

Each entry in the `mcps` list can be a plain string (name) or an object — mirrors skill entries:

| Form | Resolves to |
| --- | --- |
| `- github` | `mcps/github.json` |
| `- github.json` | `mcps/github.json` |
| `- { name: x }` | `mcps/x.json` |
| `- { name: x, path: dir }` | `dir/x.json` |
| `- { name: x, path: nested/dir }` | `nested/dir/x.json` |

Paths are resolved relative to the source root (or `sub-dir` if set); absolute paths are honored as-is. `.json` is appended automatically when the name has no extension.

MCP config files must contain a `mcpServers` object with server definitions. Servers are merged
into each agent's native settings file (e.g., `.claude.json` for Claude Code, `.cursor/mcp.json`
for Cursor). See [how sync works](./how-sync-works.md) for merge behavior details.

### Command Source Fields

Commands are slash-command prompt templates: Markdown-with-frontmatter files under `commands/`
in the source. Nested directories become `:`-namespaced names (`commands/git/commit.md` →
`git:commit`). They are transformed into each agent's native command format on sync.

| Key | Required | Description |
| --- | --- | --- |
| `source` | **yes** | Git host URL or local path containing a `commands/` directory |
| `branch` | no | Branch for remote sources (default: `main`, falls back to `master`) |
| `ref` | no | Git tag, commit SHA, or ref — takes priority over `branch` |
| `sub-dir` | no | Relative subdirectory within the source used as the discovery root (`sub_dir` alias supported) |
| `commands` | **yes** | `"*"` to discover all, or a list of names / `{ name, path }` objects |

---

> **envctl note:** Everything from here to the end of the "Reference" section
> (**Instruction Source Fields**, **Instruction Entry Fields**, **How Instructions Land Per Agent**)
> documents the upstream `instructions` kind, which is **not in the absorbed v3.2.0 surface**. Kept
> verbatim for reference only.

### Instruction Source Fields _(upstream-only — not in envctl)_

Instructions wire each agent's instruction file — `CLAUDE.md`, `.cursor/rules/*.mdc`, `AGENTS.md`,
`.github/copilot-instructions.md`, and so on — from a single source. An instruction is Markdown with
optional YAML frontmatter, discovered in the source's `instructions/` directory.

| Key | Required | Description |
| --- | --- | --- |
| `source` | **yes** | Git host URL or local path containing an `instructions/` directory |
| `branch` | no | Branch for remote sources (default: `main`, falls back to `master`) |
| `ref` | no | Git tag, commit SHA, or ref — takes priority over `branch` |
| `sub-dir` | no | Relative subdirectory within the source used as the discovery root (`sub_dir` alias supported) |
| `instructions` | **yes** | `"*"` to discover all, or a list of names / `{ name, path }` objects |

When `instructions: "*"`, envctl walks `instructions/**/*.{md,mdc}` (nested directories namespace with `:`,
e.g. `house:security`).

### Instruction Entry Fields _(upstream-only — not in envctl)_

Each entry in the `instructions` list can be a plain string (name) or an object — mirrors command entries:

| Form | Resolves to |
| --- | --- |
| `- style` | `instructions/style.{md,mdc}` |
| `- house:security` | `instructions/house/security.{md,mdc}` |
| `- { name: x, path: dir }` | `dir/x.{md,mdc}` |

### How Instructions Land Per Agent _(upstream-only — not in envctl)_

Each instruction is transformed into the target agent's native format. Two destination shapes exist:

- **Aggregate file** — many instructions merge into one shared file (`CLAUDE.md`, `AGENTS.md`,
`GEMINI.md`, `.github/copilot-instructions.md`, …). Each instruction is wrapped in a managed
`<!-- envctl:instruction:ID … -->` comment block, so your own hand-written content and other instructions in
the same file are preserved. `remove`/`clean` strip only envctl's blocks — they never delete the
file.
- **Per-instruction directory** — one file per instruction. Cursor (`.cursor/rules/<name>.mdc`) gets reconstructed
MDC frontmatter (`description`, `globs`, `alwaysApply`); other directory agents
(`.windsurf/rules`, `.clinerules`, `.continue/rules`, …) get the Markdown body only.

`globs`/`alwaysApply` are only meaningful for Cursor; they are dropped for agents that don't scope
instructions. See [supported agents](./agents.md) for the per-agent destinations.

## Extending Another Config

Use `extends` to inherit from a parent config. Local relative paths resolve against the extending file's directory; HTTPS URLs are fetched with the same auth env vars as `--config`.

```yaml
# child.yaml
extends: ./team-base.yaml
scope: project
skills:
  - source: https://github.com/example/extra-pack
    skills: "*"
```

`extends` accepts a single string or a list. With a list, parents merge left-to-right; the child overrides them all.

```yaml
extends:
  - ./org-base.yaml
  - https://example.com/team-overlay.yaml
```

### Merge instructions

| Field | Instruction |
| --- | --- |
| `destination` | Replace — child wins |
| `scope` | Replace — child wins |
| `agent` | Replace — child wins |
| `skills` | Merge by `(source, ref-or-branch, sub-dir)` identity. Same identity replaces; new entries append. |
| `mcps` | Same as `skills` (`sub-dir` is always empty for MCP sources, so identity is `(source, ref-or-branch)`). |
| `commands` | Same as `skills`. |
| `instructions` | _(upstream-only)_ Same as `skills`. |

Identity-based merging lets a child *narrow* a parent's `skills: "*"` to a specific list, or pin a different `ref`, while still adding new sources.

Cycles are detected and rejected. Maximum chain depth is 8.

## Remote Configs

envctl can fetch configs from any HTTPS URL:

```
envctl agent sync --config https://example.com/team-skills.yaml
```

Great for sharing a single config across a team without checking it into every repository.

### Real-world example

[pivoshenko/pivoshenko.ai](https://github.com/pivoshenko/pivoshenko.ai) is a public config that pulls skills from several community packs for Claude Code and OpenCode:

```
envctl agent sync --config https://github.com/pivoshenko/pivoshenko.ai/blob/main/agent-env.yaml
```

envctl recognises browser URLs from GitHub, GitLab, and Gitea / Codeberg / Forgejo, and auto-rewrites them to the matching raw-content endpoint. You can paste any of these directly:

- `https://github.com/owner/repo/blob/main/agent-env.yaml`
- `https://gitlab.com/group/repo/-/blob/main/agent-env.yaml`
- `https://codeberg.org/owner/repo/src/branch/main/agent-env.yaml`

envctl prints a short `note: rewrote browser URL to raw content: ...` line so you can see what was fetched. Authentication is resolved against the rewritten host, so the same tokens that work for raw URLs apply here too.

If the URL points to a private repository, envctl uses the same token-based authentication as skill sources. See [authentication](./authentication.md) for the full list of supported environment variables.

## Multiple Agents

The `agent` field accepts a single value or a list. With a list, envctl installs skills and commands
to every agent's directory and merges MCPs into every agent's settings file:

```yaml
agent:
  - claude-code
  - cursor
  - codex

skills:
  - source: https://github.com/org/skill-pack
    skills: "*"
```

Handy when you juggle multiple agents and want them all to share the same skill set.

## Agent vs Destination

If you set both, `destination` wins. Use `agent` for convenience with supported presets, or `destination` when you need full control over the install path.

Use `destination` when targeting an agent that isn't in the supported list.

## Scope: Global vs Project

By default, skills are installed globally into the agent's home-directory path. Add `scope: project` to your config, or pass `--project` on the command line, to install into the current project directory instead.

The `--project` / `--global` flags always override whatever `scope` is set in the config file.

## Environment Variables

These environment variables affect envctl's output behavior:

| Variable | Effect |
| --- | --- |
| `NO_COLOR` | Disables colored output. Set to any value. |
| `CLICOLOR_FORCE` | Forces color even when stdout is not a TTY. Set automatically by `--color always`. |
| `ENVCTL_AGENT_CONFIG` | Overrides config discovery to use the exact path or URL. |
