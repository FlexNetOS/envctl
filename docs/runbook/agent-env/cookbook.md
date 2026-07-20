# Cookbook

> **Ported from kasetto.dev/docs** (Kasetto v3.2.0, absorbed into envctl `crates/agent-env`).
> Renamed kasetto→`envctl agent`; `kasetto.yaml`→`agent-env.yaml`; mimalloc removed.
> Source: https://www.kasetto.dev/docs/cookbook. The standalone `kasetto` binary is retired — this is the `envctl agent` surface.

Recipes for common workflows.

**When you need this:** You want copy-paste setups for common real workflows (teams, monorepos, multiple agents, pinned rollouts).

**What you'll learn:**

- Patterns that work well in practice
- How to pin and roll out changes safely

> **envctl note:** mutating verbs are **preview-by-default**; add `--apply` to actually write. The
> recipes below add `--apply` where the upstream doc relied on kasetto's write-by-default behavior,
> and replace kasetto's `--dry-run` with the default preview (omit `--apply`). The `--instruction`
> flag is upstream-only and not in the absorbed v3.2.0 surface.

## Reproducible Team Setup (Commit The Lock)

Treat `agent-env.lock` like `Cargo.lock` or `package-lock.json`: commit it next to `agent-env.yaml` so every teammate gets identical skill versions.

```
# Maintainer: set up the config, sync, then commit both files.
envctl agent sync --scope project --apply
git add agent-env.yaml agent-env.lock
git commit -m "chore: pin agent skills"
```

```
# Teammates: clone, then use the committed v3 proofs with zero network.
envctl agent sync --locked --apply
```

```
# Maintainer rolls versions forward, then commits the updated lock.
envctl agent sync --update --apply          # re-resolve branches/HEAD, rewrite pins
# or update one entry: envctl agent sync --update code-reviewer --apply
git add agent-env.lock
git commit -m "chore: bump agent skills"
```

```
# CI verifies the checked-in lock without ever fetching new versions.
envctl agent sync --locked --scope project --apply
```

`--locked` (alias `--frozen`) errors if the config needs something the lock can't satisfy, so a stale lock fails the build instead of silently drifting. See [CI & automation](./ci.md) and [How Sync Works → The Lockfile Contract](./how-sync-works.md).

## Edit Your Config Without Leaving The Shell

`envctl agent add` / `envctl agent remove` rewrite `agent-env.yaml` (comments preserved), then sync — the cargo/uv way. Paste a tag-pinned URL with the `@<ref>` shorthand and you're done:

```
# Pin a pack to a specific tag — `@<ref>` shorthand equals `--ref <ref>`.
envctl agent add https://github.com/anthropics/skills@v1.2.0 --apply

# Add named skills from a multi-pack repo (kind-tagged, repeatable).
envctl agent add https://github.com/example/repo --skill code-reviewer --skill find-skills --apply

# Touch several sections in one edit — skills + MCPs + commands together.
envctl agent add https://github.com/example/repo --skill find --mcp github --command review --apply

# Preview the edit without writing agent-env.yaml or running sync (the default — no --apply).
envctl agent add https://github.com/example/repo

# Edit the config but skip the install (sync later or in CI).
envctl agent add https://github.com/example/repo --no-sync --apply
```

`envctl agent remove` is the symmetric reverse — same kind-tagged flags, with `*` to drop a whole entry:

```
# Drop the source from every list it appears in, prune installed assets.
envctl agent remove https://github.com/example/repo --apply

# Subtract just one skill; if it was the last name, the whole entry is dropped.
envctl agent remove https://github.com/example/repo --skill code-reviewer --apply

# Drop the whole MCP entry (the lone `*` means "this kind's entry").
envctl agent remove https://github.com/example/repo --mcp "*" --apply

# Same `@<ref>` shorthand for disambiguating when one URL appears twice.
envctl agent remove https://github.com/example/repo@v1.2.0 --apply
```

Both honor `--locked`/`--frozen` (the follow-up sync refuses to fetch), `--json` (structured output for scripts), and `--no-sync` (edit the YAML only). Deep `blob`/`tree` browse URLs work too — paste the URL you're already looking at and envctl decomposes it into `source` + `ref`/`branch` + `sub-dir`.

## Verify The Lock In CI Without Installing

`envctl agent lock --check` (alias `--frozen`) re-resolves the config and compares against `agent-env.lock` — exits non-zero on drift and never writes, but may fetch remote sources. For a fail-closed, zero-network CI audit, also pass `--locked`:

```
envctl agent lock --check --locked
```

When a single dependency needs to roll forward without re-resolving everything, target it with `-P` / `--upgrade-package` (mirrors `sync --update <name>...`):

```
envctl agent lock --upgrade-package code-reviewer
```

Other sources keep their existing lock entries; only the source providing `code-reviewer` is re-resolved.

## Team Bootstrap From A URL Config

Host a shared `agent-env.yaml` somewhere reachable over HTTPS (public or private), then have each developer run:

```
envctl agent sync --config https://example.com/team/agent-env.yaml --apply
```

For private configs hosted on git providers, set the matching token env var (see [Authentication](./authentication.md)).

## Inheriting From A Team Or Org Base

Use `extends` to compose configs. A common pattern: an org-wide base, a team overlay, and a per-project file that narrows or pins specific entries.

```yaml
# project/agent-env.yaml
extends:
  - https://github.com/acme/skills-base/raw/main/agent-env.yaml
  - https://example.com/team/overlay.yaml

scope: project

skills:
  # Same source as the base → narrows the parent's skills list to one entry.
  - source: https://github.com/anthropics/skills
    skills:
      - skill-creator

  # New source → appended on top of the inherited list.
  - source: https://github.com/acme/internal-pack
    skills: "*"
```

Top-level scalars (`scope`, `agent`, `destination`) replace. `skills` and `mcps` merge by `(source, ref-or-branch, sub-dir)` identity. See [Configuration → Extending Another Config](./configuration.md) for the full merge-instructions table.

## Monorepo: Project Scope Per Workspace

Keep one `agent-env.yaml` per workspace folder and make it project-scoped:

```yaml
scope: project
agent: cursor

skills:
  - source: https://github.com/acme/monorepo-skills
    skills:
      - code-reviewer
      - doc-coauthoring
```

Then run sync from each workspace directory:

```
envctl agent sync --apply
```

Each workspace gets its own `./agent-env.lock`.

## Multiple Agents From One Config

Install the same skills (and MCPs) into multiple agents:

```yaml
agent:
  - claude-code
  - cursor
  - codex

skills:
  - source: https://github.com/acme/skills
    skills: "*"

mcps:
  - source: https://github.com/acme/mcp-packs
    mcps: "*"
```

## MCP Packs: Pinning And Rollouts

Pin an MCP pack source to a git tag or commit SHA:

```yaml
agent: claude-code

skills:
  - source: https://github.com/acme/skills
    skills: "*"

mcps:
  - source: https://github.com/acme/mcp-packs
    ref: v2.4.1
    mcps: "*"
```

Roll forward by bumping `ref`, then preview with the default (no `--apply`):

```
envctl agent sync
```

## Explicit MCP Entries (mcps.mcps)

If a repository contains multiple MCP files or uses a non-standard layout, list entries explicitly.
Plain strings look up `mcps/<name>.json`; objects let you override the directory:

```yaml
mcps:
  # Names resolved from mcps/ dir (auto .json extension)
  - source: https://github.com/acme/monorepo
    ref: v1.4.0
    mcps:
      - github        # → mcps/github.json
      - linear        # → mcps/linear.json

  # Custom directory via { name, path }
  - source: https://github.com/acme/other
    mcps:
      - name: my-server
        path: tools   # → tools/my-server.json
```
