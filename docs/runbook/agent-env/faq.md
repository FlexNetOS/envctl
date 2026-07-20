# FAQ

> **Ported from kasetto.dev/docs** (Kasetto v3.2.0, absorbed into envctl `crates/agent-env`).
> Renamed kasetto→`envctl agent`; `kasetto.yaml`→`agent-env.yaml`; mimalloc removed.
> Source: https://www.kasetto.dev/docs/faq. The standalone `kasetto` binary is retired — this is the `envctl agent` surface.

Frequently asked questions.

**When you need this:** You have a quick "what happens if…" question about syncing skills or MCPs.

## Will envctl Overwrite My MCP Entries?

No. MCP merges are **additive** and existing server entries are **never overwritten**. See [How Sync Works](./how-sync-works.md).

## What Happens When Two Sources Define The Same MCP Server Name?

**First write wins** based on config order. Later sources with the same server name are skipped. See [How Sync Works](./how-sync-works.md).

## Where Is The Lock File?

- **Global scope**: `$XDG_DATA_HOME/agent-env/agent-env.lock` (envctl-managed default: `$META_ROOT/.local/share/agent-env/agent-env.lock`)
- **Project scope**: `./agent-env.lock`

See [How Sync Works](./how-sync-works.md).

## Should I Commit agent-env.lock?

Yes. For project scope, commit **both** `agent-env.yaml` and the v3 `agent-env.lock`, just like
you'd commit `Cargo.lock` or `package-lock.json`. The config says what you want; the lock pins
versions/selectors and the relative ownership proofs created by a successful apply. Do the first
install with plain `sync --apply`, then commit that proof-bearing lock. Machine-local runtime state
is not committed; clean clones use the portable project proofs. See [How Sync Works → The Lockfile Contract](./how-sync-works.md).

## How Do I Update Pinned Versions?

Run `envctl agent sync --update --apply` (alias `-u`) to explicitly roll moving pins/selections
forward, or `--update <name>` for selected entries. Plain sync may also resolve/materialize
configured sources; only `--locked`/`--frozen` guarantees zero network. See [How Sync Works → How sync Honors the Lock](./how-sync-works.md).

## How Do I Uninstall Safely?

- To tear down every exact output owned by a scope: `envctl agent clean --apply`
- envctl has no agent-env-binary uninstall — the engine ships inside `envctl`. (Upstream kasetto used `self uninstall` for a full teardown of all assets + the standalone binary.)

See `envctl agent clean` in [Commands](./commands.md).

## Can I Use Multiple Agents?

Yes. Set `agent` to a list. See [Configuration](./configuration.md).

## Does envctl Run Code From Skills?

No. Skills are copied as directories. Execution is up to the agent you load them into.

## Can I Pin Sources To A Known-Good Version?

Yes. Use `ref:` with a tag or commit SHA. See [Cookbook](./cookbook.md).

## How Do I Preview Without Writing?

envctl is **preview-by-default**: run the verb *without* `--apply` and it prints what would change without writing any files. Useful in CI. (This replaces kasetto's `--dry-run`.) See [CI & automation](./ci.md).

## Why Didn't My MCP Servers Show Up?

Most common causes:

- The target agent settings file is malformed JSON/TOML
- The MCP file doesn't contain a top-level `mcpServers` object

See [How Sync Works](./how-sync-works.md) (corrupted settings file behavior).

## How Do Remote Configs (--config https://...) Authenticate?

envctl selects tokens by the URL hostname using the same instructions as skill/MCP sources. See [Authentication](./authentication.md).
