# Commands

> **Ported from kasetto.dev/docs** (Kasetto v3.2.0, absorbed into envctl `crates/agent-env`).
> Renamed kasetto→`envctl agent`; `kasetto.yaml`→`agent-env.yaml`; mimalloc removed.
> Source: https://www.kasetto.dev/docs/commands. The standalone `kasetto` binary is retired — this is the `envctl agent` surface.

**envctl note — the act-flag is inverted from kasetto.** Kasetto mutates by default and previews
with `--dry-run`. **envctl is fail-closed: mutating verbs (`sync` / `add` / `remove` / `clean`) are
PREVIEW by default and write only with `--apply`.** Everywhere the upstream doc says `--dry-run`,
the envctl equivalent is "the default (no `--apply`)", and where it shows a plain mutating command
you must add `--apply` to actually write. Tables below have been corrected to the real `envctl agent`
flags.

**envctl note — scope flag.** envctl uses `--scope global` / `--scope project` (a value-enum) to
override scope on `sync` / `add` / `remove` / `lock` / `list` / `clean` / `doctor`. (`init` is the
one exception and uses the boolean `--global`, matching the upstream example.)

**envctl note — global output flags.** `--json`, `-q/--quiet`, `--color <when>`, and `-v/--verbose`
are **global** envctl flags (placed before/after any subcommand), not re-declared per agent verb.
They behave as the upstream doc describes.

**envctl note — the `instructions` asset kind / `--instruction` flag are upstream-only.** envctl
absorbed kasetto v3.2.0 (three kinds: skills / commands / MCPs). The `--instruction` flag and the
`instructions` list/`list --kind instructions` shown in the live docs are a post-v3.2.0 addition and
are **not present in `envctl agent`**. References to them below are upstream reference only.

Reference for `envctl agent` add, remove, lock, sync, list, doctor, init, clean, and the top-level
`envctl self` family.

## envctl agent init

Generates a starter config file — local `./agent-env.yaml` by default, or the global config with `--global`.

```
envctl agent init [OPTIONS]
```

### Options

| Flag | Description |
| --- | --- |
| `--global` | Write `$XDG_CONFIG_HOME/agent-env/agent-env.yaml` (or `~/.config/agent-env/agent-env.yaml`) |
| `-f`, `--force` | Overwrite an existing config file without prompting |

## envctl agent add

Appends a source to your local `agent-env.yaml` and (unless `--no-sync`) syncs it in — the cargo/uv-style way to grow a config without hand-editing YAML. Comments and formatting in the file are preserved; only the targeted lists gain a new entry. **PREVIEW by default — pass `--apply` to write.**

```
envctl agent add <SOURCE> [OPTIONS]
```

Kind-tagged flags `--skill`, `--mcp`, and `--command` (each repeatable) name the entries to add. Because skills, MCPs, and commands are separate lists, a single `add` can write to several of them at once — handy for a repo that ships more than one kind. A lone `*` value (`--skill "*"`) is a wildcard. With **no** kind flags, the source is added as `skills: "*"`.

```
envctl agent add https://github.com/anthropics/skills --apply                 # every skill in the pack
envctl agent add https://github.com/anthropics/skills@v1.2.0 --apply          # `@<ref>` shorthand (cargo/uv-style)
envctl agent add https://github.com/anthropics/skills --skill pptx --skill pdf --apply
envctl agent add https://github.com/example/repo --skill find --mcp github --command review --apply
envctl agent add https://github.com/example/pack --branch develop --no-sync --apply
envctl agent add https://github.com/example/pack                              # preview the edit (no --apply = don't write)
```

**Deep browse URLs.** Paste the URL you're looking at on GitHub/Gitea/GitLab — `add` decomposes a `blob`/`tree` URL into the repo, ref, and sub-directory (and, for a `SKILL.md` link, the skill name):

```
envctl agent add https://github.com/mattpocock/skills/blob/main/skills/personal/edit-article/SKILL.md --apply
```

writes:

```yaml
skills:
  - source: https://github.com/mattpocock/skills
    branch: main
    sub-dir: skills/personal
    skills:
      - edit-article
```

A 40-char hex ref is pinned as `ref:`; any other ref (e.g. `main`) becomes `branch:` so `--update` keeps tracking it. Explicit `--ref`/`--branch`/`--sub-dir` override the derived pieces — e.g. add a `tree/` URL for the sub-dir and name the skills yourself with `--skill`.

### Options

| Flag | Description |
| --- | --- |
| `--skill <name>` | Skill to add (repeatable; `*` for all). Default kind when none given |
| `--mcp <name>` | MCP server to add (repeatable; `*` for all) |
| `--command <name>` | Command to add (repeatable; `*` for all) |
| `--ref <ref>` | Pin to a git tag, commit SHA, or ref (conflicts with `--branch`) |
| `--branch <branch>` | Track a specific branch (conflicts with `--ref`) |
| `--sub-dir <dir>` | Subdirectory inside the source to use as the root (skills/commands) |
| `--config <path>` | Config file to edit (default: `./agent-env.yaml`); must be a local file |
| `--scope <global\|project>` | Override the scope used for the follow-up sync |
| `--apply` | Write the change (else preview / zero writes) |
| `--no-verify` | Skip the upfront fetch that validates the source resolves |
| `--no-sync` | Edit the config without installing |
| `--locked`, `--frozen` | During the follow-up sync, never fetch; honor the lock (requires `--no-sync` on `add`) |
| `-u`, `--update [NAME...]` | Re-resolve named (or all) moving refs during the follow-up sync |

Before writing, `add` fetches the source once to confirm it resolves (and, for a named skill list, that the names exist) — skip with `--no-verify`. Adding a source whose `(source, ref/branch, sub-dir)` identity already exists in a list is rejected; edit it directly or `envctl agent remove` it first. MCP entries never carry `sub-dir` (the schema has none there). Remote configs (HTTPS) can't be edited in place.

The `<SOURCE>@<REF>` shorthand (cargo/uv-style) splits the trailing `@<ref>` off the positional and is equivalent to `--ref <REF>`. Passing both the shorthand and `--ref`/`--branch` is rejected. SSH (`git@host:org/repo`) and URLs with userinfo (`https://user@host/repo`) round-trip unchanged.

## envctl agent remove

Deletes entries from your local `agent-env.yaml`, then (unless `--no-sync`) syncs so the now-unconfigured skills, MCPs, and commands are pruned from disk and the lock. Aliased as `envctl agent rm`. **PREVIEW by default — pass `--apply` to write.**

```
envctl agent remove <SOURCE> [OPTIONS]
```

`remove` mirrors `add`. The kind-tagged flags `--skill`, `--mcp`, and `--command` (each repeatable) **subtract** named entries from a list; when the last name goes, the whole entry is dropped. A lone `*` value drops that kind's entry outright. With **no** kind flags, the source is removed from every list it appears in.

```
envctl agent remove https://github.com/anthropics/skills --apply               # drop the whole source
envctl agent remove https://github.com/anthropics/skills@v1.2.0 --apply         # disambiguate via `@<ref>` shorthand
envctl agent remove https://github.com/vercel-labs/skills --skill find-skills --apply
envctl agent remove https://github.com/example/repo --mcp github --command review --apply
envctl agent remove https://github.com/example/pack --mcp "*" --apply           # drop the whole mcps entry
envctl agent remove https://github.com/example/pack                             # preview (no --apply = don't write)
envctl agent rm ./local/pack --no-sync --apply                                  # edit config only
```

Deep `blob`/`tree` browse URLs are accepted too — they resolve to the same repo-root source `add` would have written, so you can paste the URL you're looking at.

### Options

| Flag | Description |
| --- | --- |
| `--skill <name>` | Skill to remove (repeatable; `*` drops the whole skills entry) |
| `--mcp <name>` | MCP to remove (repeatable; `*` drops the whole mcps entry) |
| `--command <name>` | Command to remove (repeatable; `*` drops the whole commands entry) |
| `--ref <ref>` | Disambiguate by pinned ref when a URL appears more than once |
| `--branch <branch>` | Disambiguate by tracked branch |
| `--sub-dir <dir>` | Disambiguate by sub-directory |
| `--config <path>` | Config file to edit (default: `./agent-env.yaml`); must be local |
| `--scope <global\|project>` | Override the scope used for the follow-up sync |
| `--apply` | Write the change (else preview / zero writes) |
| `--no-sync` | Edit the config without pruning installed assets |
| `--locked`, `--frozen` | During the follow-up sync, never fetch; honor the lock |
| `-u`, `--update [NAME...]` | Re-resolve named (or all) moving refs during the follow-up sync |

Subtracting a name from a `"*"` wildcard entry is an error (there are no named entries to drop — use `--<kind> "*"` to remove the whole entry); so is naming a skill that isn't in the list. When several entries share a source URL, pass `--ref` or `--branch` to pick one. Entries inherited via `extends:` live in the parent config and must be removed there.

## envctl agent sync

Reads your config, fetches any remote skills, and brings your local install up to date. **PREVIEW by default — pass `--apply` to write.**

```
envctl agent sync [OPTIONS]
```

### Options

| Flag | Description |
| --- | --- |
| `--config <path-or-url>` | Path or HTTPS URL to a YAML config (default order: `$ENVCTL_AGENT_CONFIG`, `./agent-env.yaml`, `source:` in `config.yaml`, `$XDG_CONFIG_HOME/agent-env/agent-env.yaml`) |
| `--scope <global\|project>` | Override the scope resolved from the config |
| `--apply` | Write changes (else preview / zero writes) |
| `--locked`, `--frozen` | Audit against the lock with **zero** network fetch (fail-closed if the lock can't satisfy the config) |
| `-u`, `--update [NAME...]` | Re-resolve the named packages' moving refs (no names = all) and rewrite the lock |

Missing skills are reported as broken but won't stop the rest of the run. The exit code is non-zero only for source-level failures.

The default (no `--apply`) preview is great in CI — verify your config without touching anything on disk. For a reproducible CI gate that fails on drift, use `--locked`.

## envctl agent lock

Re-resolves every source and pins it into `agent-env.lock` **without installing** to your agent directories — the equivalent of `cargo generate-lockfile` / `uv lock`. Like `sync --update`, it re-resolves moving refs (branches and the default HEAD).

```
envctl agent lock [OPTIONS]
```

### Options

| Flag | Description |
| --- | --- |
| `--config <path-or-url>` | Path or HTTPS URL to a YAML config (same default order as `sync`) |
| `--scope <global\|project>` | Override the resolved scope |
| `--check`, `--frozen` | Verify the lock matches the config without writing; **exit 1 on drift** |
| `--locked` | With `--check`: make the audit zero-network |
| `-P`, `--upgrade-package <name>...` | Only re-resolve sources providing these skills (mirrors `sync --update <name>...`) |

Skills are hashed from the materialized **source tree**. Because a skill installs as a verbatim copy, that hash equals the one a later `sync` computes at the destination — so after `envctl agent lock` a plain `envctl agent sync --locked` succeeds with zero fetches. MCP and command entries can't be hashed without applying their merge/transform, so `lock` only refreshes their resolved revision pins; their content hash fills in on the next real `sync`. Any source that fails to resolve aborts the lock before writing.

`envctl agent lock` reads remote configs fine (it only writes the lock, never the config). `envctl agent add` / `envctl agent remove`, which rewrite the config, require a local file.

> **envctl note:** Unlike kasetto, envctl's `lock` keeps only the `--frozen` alias on `--check`
> (not `--locked`), because envctl's `lock` has a distinct `--locked` flag (the zero-network audit
> knob) — a `--locked` alias on `--check` would collide.

## envctl agent list

Prints a uv-style table of installed assets from the lock file — name, scope, and source per row. Read-only.

```
envctl agent list [OPTIONS]
```

### Options

| Flag | Description |
| --- | --- |
| `--kind <kind>` | `all` (default), `skills`, `mcps`, or `commands` |
| `--scope <global\|project>` | Restrict to a single scope's lock (else merges both) |

With **no** `--scope`, envctl merges **both** scopes so you can see global and project installs together. Each row includes a **scope** column. The `--json` shape always includes top-level `skills`, `mcps`, `commands`, and `merged_scopes` keys; `--kind` filters the contents of each list but never drops the key.

> **envctl note:** the upstream doc shows `--type` and an `instructions` kind; envctl uses `--kind`
> with kinds `all|skills|mcps|commands` (no `instructions` in the absorbed v3.2.0 surface).

## envctl agent doctor

Prints a local health check: your version, lock file location, install paths, last sync time, command-directory writability, and any skills that failed. Read-only.

```
envctl agent doctor [OPTIONS]
```

### Options

| Flag | Description |
| --- | --- |
| `--scope <global\|project>` | Override the scope resolved from the config |

## envctl agent clean

Removes installed assets that are no longer referenced by the config — skills, commands, MCP configs — and resets the corresponding lock entries. **PREVIEW by default — pass `--apply` to write.**

```
envctl agent clean [OPTIONS]
```

### Options

| Flag | Description |
| --- | --- |
| `--config <path>` | Config file path (else default resolution / `$ENVCTL_AGENT_CONFIG`) |
| `--scope <global\|project>` | Override the resolved scope |
| `--apply` | Write changes — actually remove (else preview / zero writes) |

> **envctl note:** kasetto's `clean` is a full teardown of *everything* installed for the scope and
> previews with `--dry-run`. envctl's `clean` **prunes only assets orphaned from the config** and is
> preview-by-default (`--apply` to write). For a full teardown, remove the sources from the config
> first, then `clean --apply`.

## envctl self

Manage the envctl binary itself.

> **envctl note (does not apply as upstream):** Kasetto's `self update` downloads the latest
> GitHub-release binary, verifies its SHA-256 against `checksums.txt`, and swaps it in place; `self
> uninstall` tears down all installed assets + data dirs + the binary. **envctl's agent-env engine
> ships *inside* the `envctl` binary** (the standalone `kasetto`/`kst` binary was retired), so there
> is no agent-env-binary self-update. To upgrade, rebuild from the meta Cargo workspace
> (`git pull && cargo build -p envctl`). envctl does expose a top-level `envctl self` command family
> governing the envctl binary's own lifecycle; consult `envctl self --help` for its current verbs.
> The upstream `self update` / `self uninstall` reference is preserved below for fidelity only.

### kasetto `self update` (upstream reference — not the envctl mechanism)

Fetches the latest release from GitHub, verifies the SHA256 checksum against `checksums.txt` from the same release, and swaps out the binary in-place.

```
# upstream kasetto behavior; envctl rebuilds from source instead
self update [OPTIONS]
```

#### Update notifications (upstream reference)

Kasetto printed a yellow `New version available: x.y.z → a.b.c` line at the end of any command when a newer release existed on GitHub. The check ran in a background thread at most once every 24 hours and cached the result under `$XDG_CACHE_HOME/agent-env/update-check.json`. The notice was suppressed for `--json`, `--color never`, `--quiet`, machine-readable commands (`completions`), and non-TTY stdout.

### kasetto `self uninstall` (upstream reference — not the envctl mechanism)

A full teardown: removes installed skills, commands, and MCP configs, clears the data directories, and deletes the binary. (Not applicable to envctl — the engine is part of `envctl`.)

## Shell completions

> **envctl note:** completions are generated by the top-level `envctl` binary, not under `agent`.

```
envctl completions <SHELL>
```

Supported shells: `bash`, `zsh`, `fish`, `powershell`.

Example for Fish: `envctl completions fish > ~/.config/fish/completions/envctl.fish`
