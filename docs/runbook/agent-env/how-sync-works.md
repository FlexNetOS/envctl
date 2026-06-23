# How Sync Works

> **Ported from kasetto.dev/docs** (Kasetto v3.2.0, absorbed into envctl `crates/agent-env`).
> Renamed kasetto→`envctl agent`; `kasetto.yaml`→`agent-env.yaml`; mimalloc removed.
> Source: https://www.kasetto.dev/docs/how-sync-works. The standalone `kasetto` binary is retired — this is the `envctl agent` surface.

Internals of the sync process.

A look at what `envctl agent sync` actually does — how skills are installed, how MCP configs are merged, what gets overwritten, and what's always left alone.

> **envctl note:** mutating runs are **preview-by-default**; the writes described below happen on
> `--apply`. Wherever the upstream doc says `--dry-run`, the envctl equivalent is "the default
> (omit `--apply`)". The upstream `instructions` asset kind is not part of the absorbed v3.2.0
> surface; envctl syncs three kinds (skills / commands / MCPs).

## Sync Flow

```
1. Load config (`$ENVCTL_AGENT_CONFIG` → `./agent-env.yaml` → saved preference → `$XDG_CONFIG_HOME/agent-env/agent-env.yaml`, or explicit `--config` path/URL)
2. Load lock file (agent-env.lock)
3. Sync skills
   a. For each skill source: decide whether a fetch is needed (the lock pins it and the destinations already match → no network)
   b. Materialize (download if remote) only when a fetch is needed
   c. Discover skills in source directory
   d. For each skill: hash → compare to lock → copy if changed
   e. Remove skills no longer in config
4. Sync commands
   a. For each command source: decide whether a fetch is needed (the lock pins it and the destination files already exist → no network)
   b. Materialize (download if remote) only when a fetch is needed
   c. Discover commands in source, transform into each agent's native format
   d. For each command: hash → compare to lock → write if changed
   e. Remove commands no longer in config
5. Sync MCPs
   a. For each MCP source: decide whether a fetch is needed (the lock pins it and its servers are already present in the agent settings → no network)
   b. Materialize (download if remote) only when a fetch is needed
   c. Discover MCP pack files in source
   d. For each pack: hash → compare to lock → merge into agent settings if changed
   e. Remove MCP entries no longer in config
6. Save lock file and report (only when --apply; the default preview writes nothing)
```

## Skills: Copy and Replace

Skills are plain directories with a `SKILL.md` file (see [writing skills](./writing-skills.md)). On each sync, envctl:

- **Resolves** the source root (optionally via `sub-dir`).
- **Discovers** skills from:

  - top-level `SKILL.md` in the resolved root,
  - root-level subdirectories with `SKILL.md`, and
  - `skills/<name>/SKILL.md`.

- **Hashes** the source skill directory.
- **Compares** the hash to the lock file. If unchanged, the skill is skipped.
- **Copies** the entire directory to the destination, replacing any previous version (including
symlinked files/directories inside the skill).
- **Removes** skill directories that are no longer listed in the config.

Skills are fully replaced on update — no partial merges.

## MCP Servers: Discovery and Additive Merge

envctl auto-discovers MCP pack files in the source:

1. `.mcp.json` at the source root
2. `mcp.json` at the source root
3. Any `.json` file inside the `mcps/` subdirectory

Each file must contain a `mcpServers` JSON object. Use a `mcps:` list to pick specific files instead of discovering all.

Server entries are merged into each agent's native settings file (e.g., `.claude.json`, `.cursor/mcp.json`). The merge follows two simple instructions:

1. **New entries are added.** If the settings file doesn't have a server with that name, it's
inserted.
2. **Existing entries are never overwritten.** If a server name already exists — whether added by
envctl or by hand — envctl leaves it untouched.

In practice, this means:

- **Manual edits are safe.** If you add API keys, environment variables, or tweak server settings
after a sync, those changes survive future syncs.
- **Re-sync is idempotent.** Running `sync` twice with the same config produces the same result.
- **First write wins.** If two sources define a server with the same name, only the first one is
installed.

### Supported Config Formats

envctl auto-detects the right format for each agent:

| Format | Used by | Target file example |
| --- | --- | --- |
| McpServers JSON | Claude Code, Cursor, Gemini CLI, Roo, and others | `.claude.json`, `.cursor/mcp.json` |
| VS Code servers JSON | GitHub Copilot | `.vscode/mcp.json` |
| OpenCode JSON | OpenCode | `.config/opencode/opencode.json` |
| Codex TOML | Codex | `.codex/config.toml` |

All formats follow the same additive-merge instructions — the underlying behavior is identical.

## Change Detection

envctl uses SHA-256 hashes so it only does work when something actually changed:

- **Skills:** The entire skill directory is hashed. If the hash matches the lock file, the skill is
skipped.
- **MCP packs:** The pack file is hashed. If the hash matches **and** all server names are still
present in the target settings, the pack is skipped.

This keeps re-sync fast — unchanged sources require no file I/O beyond reading the lock.

## The Lockfile Contract

`agent-env.lock` is a real lockfile, in the same spirit as `Cargo.lock`, `package-lock.json`, or `go.sum`. It pins exactly which skills, commands, and MCP assets were installed — by source, revision, destination, and content hash — so that anyone who runs `envctl agent sync` against the same config and lock gets an identical setup.

Where it lives depends on the scope:

| Scope | Location |
| --- | --- |
| Global | `$XDG_DATA_HOME/agent-env/agent-env.lock` (default: `~/.local/share/agent-env/agent-env.lock`) |
| Project | `./agent-env.lock` in the project root |

The lock is also how envctl knows what to remove when you drop a source from the config or run `envctl agent clean`. You generally won't need to touch it by hand.

**Skills** are tracked by a composite key (`source::skill-name`) along with their destination path and hash.

**MCPs** are tracked as assets with the server names they installed (e.g., `destination: "git-tools,airflow"`). Only these tracked names are removed during cleanup.

envctl never touches entries it didn't install. Manually added servers, skills from other tools, or entries from a different scope are always left alone.

### Commit It

For project scope, commit **both** `agent-env.yaml` and `agent-env.lock`. The config says *what* you want; the lock says *exactly which versions* everyone gets. The lock is deterministic and portable by design:

- Destination paths are stored **relative to the scope root**, so the file is identical on every machine and diffs cleanly in review.
- It contains **no timestamps or machine-specific data** — nothing run-specific leaks in.

Machine-local runtime state (last-run report, per-skill install times used by `envctl agent list` and `envctl agent doctor`) lives separately under `$XDG_CACHE_HOME/agent-env/runtime/` and is **never committed**. It is safe to delete and regenerates on the next sync.

A custom **absolute** `destination:` stays non-portable by design — an absolute path can't be relativized, so it's stored as-is. Use a relative `destination:` (or the defaults) if you want the lock to travel between machines.

### How sync Honors the Lock

| Command | Behavior |
| --- | --- |
| `envctl agent sync` | Installs exactly what the lock pins. If the on-disk destinations already match the locked hashes, **no network fetch happens** — moving refs like a branch are not re-resolved. (Add `--apply` to write.) |
| `envctl agent sync --update` (`-u`) | Re-resolves branch / default-HEAD sources, downloads the latest, and rewrites the pins (hash + revision) in the lock. |
| `envctl agent sync --update <name>` | Updates only the selected entries; everything else is honored from the lock. (Naming one skill re-resolves its whole source.) |
| `envctl agent sync --locked` / `--frozen` | Strict mode for CI: installs from the lock and **never fetches**. Errors only if the config needs something the lock can't satisfy. |

A plain `envctl agent sync` always re-hashes the destination contents locally (cheap, no network) and repairs any tampered or missing copy from the lock — but it will not phone home to check whether an upstream branch moved. That is what `--update` is for. `--update` and `--locked` together are contradictory and rejected.

### Wildcards Are Frozen to the Locked Set

When a source uses `skills: "*"`, a plain `envctl agent sync` installs exactly the set recorded in the lock — not whatever the source currently contains. New skills added upstream and skills removed upstream are only reflected after `envctl agent sync --update`, which re-discovers the source and prunes accordingly. This keeps the lock authoritative: the wildcard resolves the same way for every teammate until someone deliberately updates.

**One-time `updated` flood after upgrading.** A release normalizes the skill content hash across operating systems (path separators are unified before hashing). The first `envctl agent sync` after upgrading will therefore show every skill as `updated` once as it rewrites the lock with the new hash format. This is expected and self-heals — subsequent syncs report `unchanged` again.

## Removal Behavior

### Removing a Source from Config

Remove a source from `agent-env.yaml` and re-sync:

- **Skills:** The skill directory is deleted from disk.
- **MCPs:** The specific server entries that envctl installed are removed from the agent's settings
file. The file itself is preserved.

### envctl agent clean

Prunes assets orphaned from the config for the given scope (preview-by-default; `--apply` to write):

- Deletes tracked skill directories no longer referenced by the config
- Removes tracked MCP server entries no longer referenced, from every agent's settings file
- Resets the corresponding lock entries

The default (no `--apply`) prints what would be removed (skill destinations and MCP pack lines) without changing disk.

Anything you added by hand outside the lock is never touched.

> **envctl note:** kasetto's `clean` was a full teardown of *everything* tracked in the lock for the
> scope. envctl's `clean` prunes only what's **orphaned from the config**. For a full teardown,
> empty the config first, then `clean --apply`.

## Edge Cases

**Conflicting server names across sources.** If source A and source B both define a server named `"my-server"`, source A wins (it's processed first). If you later remove source A, `"my-server"` gets removed — even though source B also wanted it. Re-sync will then install source B's version.

**Renamed servers in upstream.** If an upstream MCP pack renames a server from `"old"` to `"new"`, envctl treats it as: remove `"old"` (no longer in pack) + add `"new"` (not yet in settings). The old entry is cleaned up and the new one is added.

**Corrupted settings file.** If an agent's settings file is malformed JSON/TOML, the merge for that file fails and is reported as an error. The lock file isn't updated for that pack, so the next sync retries the merge automatically.
