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
1. Load config and resolve scope/targets.
   - Plain/update may load local or remote configs and materialize sources.
   - --locked uses the zero-network loader; remote root configs and remote extends fail closed.
2. Load agent-env.lock and the machine-local runtime ownership/report ledger.
3. Build and validate the complete desired snapshot.
   - Plain/update re-resolves and materializes configured sources.
   - --locked re-hashes local sources and validates remote hash/revision/selector pins without fetch.
4. Snapshot skills once; parse/render commands and MCP target fragments; compare every live output.
5. Require exact ownership proof before replacing/removing an existing output; plan stale tombstones.
6. Stage every changed output plus the resulting v3 lock and runtime state; revalidate live inputs.
7. On --apply, commit the staged set as one rollback boundary. Preview reports the same plan but
   writes nothing. A successful commit advances ownership; a failed coherent commit restores the
   previous bytes and records only the failure report.
```

## Skills: Copy and Replace

Skills are plain directories with a `SKILL.md` file (see [writing skills](./writing-skills.md)). On each sync, envctl:

- **Resolves** the source root (optionally via `sub-dir`).
- **Discovers** skills from:

  - top-level `SKILL.md` in the resolved root,
  - root-level subdirectories with `SKILL.md`, and
  - `skills/<name>/SKILL.md`.

- **Snapshots** the source skill directory once, then hashes and installs that immutable snapshot.
- **Compares** the hash to the lock file. If unchanged, the skill is skipped.
- **Copies** the entire effective directory to the destination, replacing only an output whose
  ownership is proven. Contained source symlinks are captured as ordinary entries; escapes,
  cycles, special entries, and symlinks in an installed destination are refused.
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

envctl uses schema-v3 SHA-256 hashes so output writes happen only when something actually changed:

- **Skills:** tree-v1 length-frames path components, entry kind, mode, and file bytes, and includes
  empty directories. Project scope normalizes directories to `0755` and files to Git's `0644` or
  executable `0755`; an empty directory is rejected because a clean Git clone cannot reproduce it.
  Global scope keeps exact platform/mode semantics.
- **MCP packs:** The pack file is hashed. If the hash matches **and** all server names are still
present in the target settings, the pack is skipped.

The comparison still reads local source/destination bytes. Only `--locked`/`--frozen` guarantees
zero network; plain sync is allowed to resolve and materialize configured sources.

## The Lockfile Contract

`agent-env.lock` is a real lockfile, in the same spirit as `Cargo.lock`, `package-lock.json`, or
`go.sum`. Schema v3 separates two facts:

- `skills`, `assets`, and `source_selectors` pin **desired state**: source, revision, selection,
  complete target paths/formats, and content hash.
- Project-only `installed_outputs` pin **ownership evidence**: the exact relative destination,
  native format/unit, and installed hash committed by a successful apply. Inactive entries are
  retained as tombstones until proof-checked removal succeeds.

A desired entry never proves envctl installed a pre-existing file. `agent lock` and
`lock --check` preserve existing ownership attestations but never synthesize them.

Where it lives depends on the scope:

| Scope | Location |
| --- | --- |
| Global | `$XDG_DATA_HOME/agent-env/agent-env.lock` (envctl-managed default: `$META_ROOT/.local/share/agent-env/agent-env.lock`) |
| Project | `./agent-env.lock` in the project root |

The lock is also how envctl knows what to remove when you drop a source from the config or run `envctl agent clean`. You generally won't need to touch it by hand.

**Skills and assets** use length-framed v3 identities; selectors and multi-value destination/server
lists are also length-framed so commas or concatenated names cannot alias another identity.

**MCPs** are tracked as assets plus one ownership unit per installed server fragment. Only exact,
hash-matching proven fragments are removed during sync or cleanup.

envctl never touches entries it didn't install. Manually added servers, skills from other tools, or entries from a different scope are always left alone.

### Commit It

For project scope, commit **both** `agent-env.yaml` and `agent-env.lock`. The config says *what* you want; the lock says *exactly which versions* everyone gets. The lock is deterministic and portable by design:

- Project ownership destinations are stored **relative to the project root**, so the proof remains
  valid after cloning at another absolute path.
- It contains **no timestamps or machine-specific data** — nothing run-specific leaks in.

Machine-local runtime state (last-run report, per-skill install times, and `managed_outputs`) lives
separately under `$XDG_CACHE_HOME/agent-env/runtime/` and is **never committed**. For project-native
targets, the committed portable proof is authoritative across a clean clone. The runtime ledger is
authoritative for global scope and is a required second factor for a retired project custom root;
deleting it intentionally removes authority to mutate those outputs until a safe plain apply
installs an absent output anew; matching pre-existing bytes are never silently re-adopted.

Project destinations must stay beneath the project root. Current project custom roots inside that
boundary can receive portable proof; historical custom-root tombstones require the exact matching
runtime proof as a second factor. Global custom roots are non-portable and may be removed only from
an exact current-user-owned, hash-matching runtime proof.

### How sync Honors the Lock

| Command | Behavior |
| --- | --- |
| `envctl agent sync` | Preview by default. Plain mode may re-resolve/materialize configured sources and rebuild desired pins; `--apply` commits the complete plan. |
| `envctl agent sync --update` (`-u`) | Re-resolves branch / default-HEAD sources, downloads the latest, and rewrites the pins (hash + revision) in the lock. |
| `envctl agent sync --update <name>` | Updates only the selected entries; everything else is honored from the lock. (Naming one skill re-resolves its whole source.) |
| `envctl agent sync --locked` / `--frozen` | Strict v3 mode: performs **zero network I/O**, validates selectors/proofs, and errors if local/installed verified bytes cannot satisfy the lock. |

`--update` controls moving-pin/selection update intent; it is not the only mode allowed to use the
network. `--update` and `--locked` together are contradictory and rejected.

### First Install, Clean Clone, and v2 Migration

A synthetic/fresh v3 lock with empty `installed_outputs` cannot authorize `--locked` to overwrite
pre-existing bytes. Run the first install into absent destinations as plain
`envctl agent sync --apply`; it records portable proofs only for outputs that actually commit.
Commit the resulting project lock. A clean clone can then run
`envctl agent sync --locked --apply` without a machine cache, using those relative proofs and
verified local source bytes.

A versionless lock is treated as v2. Locked mode and direct lock rewrite reject it. The only
bootstrap is a plain apply migration, and only when the configured identity, destination/unit, and
current hash name the exact v2 output. The v3 output, lock, and runtime ledger commit together.

### Wildcards In Plain Versus Locked Mode

When a source uses `skills: "*"`, plain/update mode may re-materialize and rediscover the source,
then rewrite the desired set. Locked mode freezes the wildcard to the exact v3 identities and
selector binding already committed; it never discovers newly added remote items.

**V2 upgrade requires an apply migration.** Envctl does not silently restamp old hash bytes as
tree-v1. Run a plain `sync --apply`, review the exact migration actions, and commit the v3 lock.

## Removal Behavior

### Removing a Source from Config

Remove a source from `agent-env.yaml` and re-sync:

- **Skills:** The skill directory is deleted from disk.
- **MCPs:** The specific server entries that envctl installed are removed from the agent's settings
file. The file itself is preserved.

### envctl agent clean

Tears down every output owned by the selected scope (preview-by-default; `--apply` to write):

- Enumerates exact skill, command, and MCP ownership units, including retained tombstones
- Refuses a v2 lock, an incomplete/forged proof set, a symlink/foreign owner, or content drift
- Removes only exact proven output units, then clears the corresponding lock/runtime evidence

The default (no `--apply`) prints what would be removed (skill destinations and MCP pack lines) without changing disk.

Anything you added by hand outside the ownership proofs is never touched. If output cleanup fails,
the proofs remain intact for a recoverable retry; strict transaction failures roll back committed
replacements.

> `clean` is a full tracked teardown. Ordinary `sync` handles only assets that became stale relative
> to the config.

## Edge Cases

**Conflicting server names across sources.** If source A and source B both define a server named `"my-server"`, source A wins (it's processed first). If you later remove source A, `"my-server"` gets removed — even though source B also wanted it. Re-sync will then install source B's version.

**Renamed servers in upstream.** If an upstream MCP pack renames a server from `"old"` to `"new"`, envctl treats it as: remove `"old"` (no longer in pack) + add `"new"` (not yet in settings). The old entry is cleaned up and the new one is added.

**Corrupted settings file.** If an agent's settings file is malformed JSON/TOML, the merge for that file fails and is reported as an error. The lock file isn't updated for that pack, so the next sync retries the merge automatically.
