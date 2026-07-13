# Sync Flow

> **Ported from kasetto.dev/docs** (Kasetto v3.2.0, absorbed into envctl `crates/agent-env`).
> Renamed kasetto→`envctl agent`; `kasetto.yaml`→`agent-env.yaml`; mimalloc removed.
> Source: https://www.kasetto.dev/docs/sync-flow. The standalone `kasetto` binary is retired — this is the `envctl agent` surface.

> **Porting note:** the upstream page renders its content as **client-side Mermaid flowcharts**
> (one per section). Those diagrams are drawn in the browser and are not present in the server-side
> HTML, so they could not be captured verbatim. The section structure below is preserved, and each
> diagram's content corresponds to the textual pipeline already documented in
> [How Sync Works → Sync Flow](./how-sync-works.md) — refer there for the step-by-step pipeline,
> change-detection, merge behavior, and removal semantics. The upstream `instructions` asset kind is
> not part of the absorbed v3.2.0 surface (envctl syncs skills / commands / MCPs).

Complete reference for how `envctl agent sync` resolves sources, discovers skills, commands, and MCP files, diffs against the lock, and writes to agent environments.

---

## Top-Level Pipeline

The end-to-end sync: load config + v3 lock/runtime proof → build the complete immutable plan →
stage skills/commands/MCPs + lock + runtime → revalidate → commit or roll back (only on
`--apply`). See the numbered pipeline in
[How Sync Works → Sync Flow](./how-sync-works.md).

---

## Source Materialization

Shared by all asset kinds — skills, commands, and MCPs. Plain/update mode may materialize local or
remote sources while rebuilding desired pins. Only `--locked`/`--frozen` guarantees zero network:
it refuses remote root configs/remote `extends`, re-hashes local inputs in place, and validates
remote hash/revision/selector pins without constructing a fetch. Archive extraction in networked
modes rejects unsafe `..` paths (tar-slip guard); default-branch resolution tries `main` then
`master`, with `ref` > `branch` > default precedence.

---

## Skills Sync Flow

### Discovery

Discover skills from the resolved source root (optionally via `sub-dir`): a top-level `SKILL.md`,
root-level subdirectories containing `SKILL.md`, or `skills/<name>/SKILL.md`.

### Target Selection

Resolve each skill's destination from the active scope + agent preset (or an explicit
`destination:`).

### Hash, Diff & Copy

Snapshot the source skill directory once (lock-v3 SHA-256 over length-framed native paths,
effective entry types, file bytes, modes, and empty directories), compare it to the desired lock
and destination, then install that same immutable snapshot. Project snapshots normalize to Git's
portable mode subset and reject empty directories; global snapshots retain exact mode/platform
semantics. Existing outputs are replaced only after exact ownership proof succeeds.

### Stale Removal

Remove skill directories that are no longer listed in the config only from exact v3 ownership
tombstones. Missing, incomplete, forged, or drifted proof fails closed; envctl never derives delete
authority from desired lock entries alone.

---

## MCP Sync Flow

### File Resolution

Discover MCP pack files: `.mcp.json` at root, then `mcp.json` at root, then any `.json` under
`mcps/` — or pick specific files via an `mcps:` list.

### Parse, Hash & Diff

Parse each pack's `mcpServers` object, hash the pack file, and compare to the lock. Skip if the hash
matches **and** all server names are still present in the target settings.

### Merge Into Agent Settings

Additive, non-destructive merge into each agent's native settings file (McpServers JSON / VS Code
servers JSON / OpenCode JSON / Codex TOML): new entries are inserted; existing entries are **never**
overwritten (first write wins).

---

## Scope & Destinations

Scope resolves as **CLI `--scope` → config `scope:` field → Global default**. The scope root is the
project root (Project) or `$HOME` (Global), and the lock + per-agent destinations are derived from
it. Lock locations: Global → `$XDG_DATA_HOME/agent-env/agent-env.lock`; Project → `./agent-env.lock`.
Portable `installed_outputs` are project-only and relative. Global ownership is machine-local;
historical project custom roots require the portable tombstone plus its matching runtime proof.

---

## Dry Run (preview)

> **envctl note:** kasetto previews with `--dry-run`. **envctl previews by default** — running
> `envctl agent sync` *without* `--apply` is the preview; pass `--apply` to write.

The preview (default, no `--apply`) skips all writes. Actions report `would_install`,
`would_update`, `would_remove`. The lock file is never modified.

Apply stages and revalidates the entire output set plus the resulting lock/runtime state before the
first rename. A strict commit failure restores prior outputs and ledgers. A successful commit is
the only point at which new ownership attestations become authoritative.
