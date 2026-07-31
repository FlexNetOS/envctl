# Kasetto Feature Catalog

> Catalog of **kasetto v3.2.0** (source-of-truth: `github.com/pivoshenko/kasetto`, git tag
> `v3.2.0`, `ec01cca`; mirrored at `meta/kasetto`) — a Rust "declarative AI agent environment
> manager." Source: ~49 `src/**/*.rs` files plus `README.md`, `CLAUDE.md`,
> `kasetto.example.yaml`, `Cargo.toml`, `justfile`, and `site/content/docs/*.mdx`.
>
> Each feature lists: **what it is**, **how kasetto implements it** (module + mechanism),
> and **relevance to envctl** — a declarative, GPU-aware, source-building environment manager
> (`meta/envctl`) with components / manifest / drift / reset / auto-fix / add-repo.
>
> **⚠️ Historical framing:** the per-section "relevance to envctl" tags
> (**ADOPT** / **PARTIAL** / **N/A**) were written before the absorption and describe what envctl
> *should* adopt. As of **2026-06-14 every ADOPT/PARTIAL item is ABSORBED** — see §0. Read the tags
> as "what was adopted," not "what is still missing."

---

## 0. Status: ABSORBED into envctl (Epic C — parity-complete 2026-06-14)

kasetto v3.2.0 is **no longer an external dependency** — its behavior is reproduced natively inside
envctl, verified at parity and shipped end-to-end:

- **`crates/agent-env`** — 18-module pure-Rust port of kasetto v3.2.0 (config model, `extends`
  composition, source resolver, fsops, SHA-256 asset lock, MCP additive-never-clobber merge, the
  5 command-format transforms).
- **`crates/engine/src/agent/*`** — the shared, non-printing `Engine::agent_{sync,add,remove,lock,list,clean}`
  methods (the 11 kasetto verbs folded to envctl's 6; preview-default fail-closed).
- **Front-ends** — `envctl agent {sync,add,remove,lock,list,clean}` (CLI) + the GUI Agent panel,
  both driving the identical Engine API. Shipped engine → CLI (PR #90/#91) → GUI (PR #93/#94),
  promoted to `master` via PR #96.
- **Parity:** **102 `[x]` verified / 0 `[~]` / 13 `[≠]` front-end** (the `[≠]` rows are kasetto's
  terminal presentation + clap wiring, which map to envctl's own CLI/GUI — intentional divergence,
  not a downgrade). Authoritative ledger: `.handoff/loop/rust-port/parity-ledger.md`.
- **External binary retired** (TASK-0018): envctl no longer installs/symlinks the `kasetto`/`kst`
  binaries — `envctl agent` is the replacement. The `meta/kasetto` repo remains the upstream
  reference, not a runtime requirement.

**Invariants preserved through the absorption:** no C in the trust boundary (libSQL-remote /
ring-only rustls / flate2 `rust_backend`), engine stays non-printing (kasetto's `ui.rs`/`banner.rs`/
`process::exit` dropped — front-ends own rendering), destructive verbs fail-closed + dry-run by
default.

### v3.1 → v3.2 delta (absent from the original v3.0.0 extract, now covered)

The catalog below was first written against v3.0.0; the v3.1/v3.2 line added — and the absorption
reproduces — these:

- **`add` / `remove` source mutation with sync-after** (`config_edit.rs`): edit `agent-env.yaml`
  then re-sync, `--no-sync` to skip; remote-config rejection (local-only edit target).
- **`lock --check`** — audit-only lock verification (exit 1 on drift) distinct from rewrite.
- **`--upgrade-package` / per-package `--update <NAME>`** — re-resolve named packages' refs and
  re-pin, vs the all-packages refresh.
- **Hardened source resolver** — tar-slip (`..`) guard, `ref > branch > default(main→master)`
  precedence with the deferred `master` fallback, browser-URL → raw rewrite, host-aware auth hints.
- **MCP merge discipline** — additive, never-clobber, never-prune-on-failure.

### Intentional envctl security superset: agent lock v3

Parity is a floor, not a ceiling. Envctl's current agent-env lock is schema **v3** and deliberately
rejects several ambiguous behaviors accepted by kasetto's v2 lock:

- **Desired state is not ownership.** `skills`, `assets`, and `source_selectors` describe what a
  run wants. They do not authorize replacement or deletion. A project-scope apply writes separate
  `installed_outputs` attestations only after the exact output commits; removed outputs remain as
  tombstones until a proof-checked sync/clean removes them and compacts the lock. `agent lock` and
  `lock --check` preserve existing attestations but never manufacture new ones.
- **Selectors and identities are injective.** v3 length-frames asset IDs and multi-value lists, and
  binds each source selector to its selection form, scope, complete target-path set, and native
  output format. Commas, path changes, format changes, or a wildcard/object-selector change cannot
  alias the old lock entry.
- **One immutable tree drives hash and install.** `TreeSnapshot` captures effective paths, entry
  kinds, file bytes, modes, empty directories, and lossless native names. Contained source symlinks
  become ordinary effective entries; escapes, cycles, special files, and destination symlinks are
  refused. Global hashes retain exact platform/mode semantics. Project hashes normalize to Git's
  reproducible subset (directories `0755`, files `0644` or executable `0755`) and reject empty
  directories that a fresh clone cannot carry.
- **Locked means zero network.** `sync --locked`/`--frozen` and `lock --check --locked` use the
  zero-network config loader (remote root configs or remote `extends` are refused), re-hash local
  sources, and validate remote hashes, revisions, selectors, and installed copies against v3.
  Missing remote bytes fail closed instead of fetching. Plain sync may resolve/materialize sources.
- **Migration is evidence-preserving.** A versionless lock is v2. Locked mode rejects it, and a
  direct lock rewrite refuses to relabel its old hashes. A plain `sync --apply` may migrate only
  exact v2-named outputs whose configured identity, destination/unit, and current hash all match;
  the output changes, v3 lock, and runtime ledger commit in one rollback-capable transaction.
- **Scope-specific ownership.** Portable `installed_outputs` are project-only and relative to the
  project root. The machine-local `managed_outputs` ledger is authoritative for global scope and is
  a required second factor for a retired project custom root. Global custom-root cleanup likewise
  requires an exact secure runtime proof. Forged/out-of-root claims, incomplete proof sets,
  symlinked paths, foreign ownership, or content drift are refused.
- **State files are roots of trust.** Lock/runtime reads are no-follow and current-user-owned;
  writes use owned parent chains and atomic sibling replacement. Existing modes are preserved;
  new project locks default to `0644`, and new global locks/runtime files default to `0600`.

---

## 1. Declarative YAML Config + Schema

**What:** One `kasetto.yaml` describes the entire desired state — target agents (or a raw
`destination`), scope, and three asset kinds (`skills`, `commands`, `mcps`), each a list of
sources.

**How kasetto implements it:**
- `model/config.rs` — `Config` struct (serde `Deserialize`): `destination: Option<String>`,
  `scope: Option<Scope>`, `agent: Option<AgentField>`, `skills: Vec<SourceSpec>`,
  `mcps: Vec<McpSourceSpec>`, `commands: Vec<CommandSourceSpec>`.
- Each source has `source`, optional `branch`, `ref` (renamed from `git_ref`), `sub-dir`
  (with a `sub_dir` serde alias), and a typed asset-selection field.
- Asset selection uses `#[serde(untagged)]` enums so YAML can be either a wildcard string
  `"*"` or a list of names / `{ name, path }` objects: `SkillsField`/`SkillTarget`,
  `McpsField`/`McpEntry`, `CommandsField`/`CommandEntry` — all three mirror each other.
- `agent` accepts a single value or a list (`AgentField::One` / `AgentField::Many`).
- Config discovery precedence (`lib.rs::resolve_config_path`): `$KASETTO_CONFIG` env →
  `./kasetto.yaml` → `source:` key in `$XDG_CONFIG_HOME/kasetto/config.yaml` (a separate
  *preferences* file) → `$XDG_CONFIG_HOME/kasetto/kasetto.yaml` → fallback to local filename.

**Relevance to envctl:** **PARTIAL.** envctl is already declarative but uses **TOML
components** in a `manifest/` dir with `components.d/` drop-ins, not a single file. Worth
adopting from kasetto: (a) the `untagged` "wildcard OR explicit list" pattern for selecting
sub-items, and (b) the layered config-discovery precedence (env var → local → user prefs →
global), which envctl only partially has via `ENVCTL_MANIFEST_DIR`.

---

## 2. Lock File + Content Hashing + Diffing (the core idea)

**What:** A committed `kasetto.lock` records exactly what was installed (hash + source +
revision + relative destination). Plain `sync` is authoritative against the lock; only what
changed gets touched. This is the cargo/uv "lock-first" model applied to agent assets.

**How kasetto implements it:**
- `lock.rs` — `LockFile { version: u8, skills: BTreeMap<id, SkillEntry>, assets: BTreeMap<id,
  AssetEntry> }`. `version: 2` schema. `BTreeMap` keeps output deterministic/diff-friendly.
- **Portable paths:** `destination` is stored *relative to the scope root* (project root for
  Project, `$HOME` for Global) via `fsops::relativize_dest` / `resolve_dest`, so the
  committed lock is machine-independent. Legacy absolute paths are still honored.
- **No run-specific data** in the lock (no timestamps) — those live in machine-local state
  (see §16). Saving re-stamps `version` so older locks migrate forward.
- **Hashing:** `fsops/hash.rs` — `hash_dir` walks files, sorts them, and SHA-256s
  `relpath\0 <contents> \0` per file. Path separators are normalized `\` → `/` so digests
  are **OS-invariant** (critical for a committed lock across Windows/Unix). `hash_file` for
  single files (commands/MCPs), `hash_str` for keying state by lock path.
- **Diffing / `needs_fetch` gate** (`commands/sync/skills.rs`, `mcps.rs`, `commands.rs`):
  before any download, each source re-hashes its on-disk destinations and compares to the
  locked hash + expected revision. If all destinations match, the fetch is **skipped
  entirely** — a plain sync with everything in place does **zero network I/O**.
- **Self-repair without fetch:** if one destination is missing/tampered but another holds a
  hash-verified-good copy, kasetto repairs by local copy instead of re-downloading. Only when
  no good local copy exists does it fetch.

**Envctl status:** **ABSORBED and hardened.** `agent-env.lock` carries the source/revision/content
contract, while schema v3 adds selector/target binding, tree-v1 hashes, framed lists/identities,
and portable project ownership attestations. Unlike kasetto v2, envctl does not treat a desired
entry or an old-version hash as proof that it owns existing bytes. See the v3 security-superset
contract in §0.

---

## 3. `extends` Config Composition (org → team → project)

**What:** A config can inherit from one or more parent configs (local paths or URLs), so an
org base, a team overlay, and a project config compose into one effective config.

**How kasetto implements it:**
- `model/extend.rs` + `fsops/config.rs`. The loader (`load_config_any` → `load_config_recursive`)
  works at the **YAML `Value` level before deserialization**: it extracts the `extends` field
  (string or list), recursively loads each parent, and merges.
- **Merge rules:** top-level scalars (`destination`, `scope`, `agent`) **replace**;
  `skills`/`mcps`/`commands` lists **merge by identity** where identity =
  `(source, ref|branch, sub-dir)`. Same identity → overlay replaces wholesale; new identity →
  appended. (`merge_source_list` / `identity_of`.)
- **Safety:** cycle detection via a `visited` set of canonical IDs, and a depth cap
  (`MAX_EXTENDS_DEPTH = 8`). Relative `extends` resolve against the parent config's directory;
  relative extends from an HTTP origin are an error.

**Relevance to envctl:** **ADOPTED for component manifests (TASK-0017).** envctl already has
`components.d/` drop-ins, which are a flat overlay; `extends`-style **layered inheritance with
identity-keyed merge** now lets a shared base manifest (org/box-class defaults) be overlaid by a
machine-specific manifest without copy-paste. envctl's manifest loader keeps this local-only for
component manifests: parent paths are relative or absolute TOML files, cycles/depth overflow fail
closed, and `[[component]]` arrays merge by component `id`.

---

## 4. Global vs Project Scope

**What:** Assets install either globally (user-wide agent dirs under `$HOME`) or project-locally
(under the project root), each with its own lock file.

**How kasetto implements it:**
- `model/config.rs` — `enum Scope { Global (default), Project }`. Resolution order
  (`resolve_scope`): **CLI flag (`--project`/`--global`) → config `scope:` field → Global default**.
- `fsops/mod.rs::scope_root` returns the project root (Project) or `$HOME` (Global), used as the
  base for portable lock paths.
- `lock.rs::lock_path`: Project lock lives at `<project>/kasetto.lock`; Global lock at
  `$XDG_DATA_HOME/kasetto/kasetto.lock`.
- Per-agent install paths differ by scope: `Agent::global_path(home)` vs
  `Agent::project_path(project_root)` (and likewise for MCP/command targets).

**Envctl status:** **ABSORBED with scope-specific trust rules.** Agent assets support both scopes
with the **CLI flag → config field → Global default** ladder. Project locks are commit-friendly
and may carry relative `installed_outputs` attestations. Global locks/runtime are private `0600`,
and global mutation relies on the machine-local ownership ledger rather than portable project
proofs. Project destinations must stay beneath the project root; current custom roots are allowed
only inside that boundary, while retired custom-root tombstones require their matching runtime
second factor.

---

## 5. Asset Kinds: Skills, Commands, MCPs (one source, three kinds)

**What:** Three distinct asset kinds, each fetched from the same git-source mechanism but
installed differently. Each "agent" can consume all three.

**How kasetto implements it:**
- **Skills** — directories containing a `SKILL.md`. Discovered by convention (`source/mod.rs::
  discover`): the source root, its `skills/` subdir, or a root-level `SKILL.md` (named by the
  repo/dir name). Copied verbatim into the agent's skills dir.
- **Commands** (user-defined slash-command prompt templates) — Markdown-with-frontmatter files
  under `commands/`. Nested dirs become `:`-namespaced names (`commands/git/commit.md` →
  `git:commit`). Transformed per agent (see §8).
- **MCPs** (Model Context Protocol servers) — JSON pack files (`.mcp.json`, `mcp.json`, or
  `mcps/*.json`). Merged into agent-native settings (see §7).
- Selection per kind: `"*"` wildcard (auto-discover) OR explicit list of names / `{name, path}`.
  Name→file conventions: `mcps: [github]` → `mcps/github.json`; commands resolve via the
  namespaced discovery map or an explicit `{name, path}`.

**Relevance to envctl:** **PARTIAL / N/A.** The specific kinds (skills/commands/MCPs) are
AI-agent-specific and N/A. But the *architecture* — **one source resolver feeding multiple
"asset kinds," each with a kind-specific installer + tracker** — is ADOPT-worthy: envctl
components already vary (apt package vs build-from-source vs config wiring); modeling them as
typed kinds sharing one fetch/track pipeline would reduce per-component bespoke code.

---

## 6. Supported Agents (the full preset table)

**What:** A closed enum of 21 known agents; setting `agent:` auto-resolves install paths +
native MCP/command targets. Unknown targets fall back to a raw `destination`.

**How kasetto implements it:** `model/agent.rs` — `enum Agent` with serde renames; a
`AGENT_PRESETS` const array; per-agent methods `global_path`, `project_path`,
`mcp_settings_target`, `mcp_project_target`, `commands_global_path`, `commands_project_path`.
Adding an agent = one enum variant + path mappings (exhaustive `match`, so the compiler forces
completeness).

**Full supported-agent list (config value → global skills path):**

| Agent | Config value | Global skills path |
|---|---|---|
| Amp | `amp` | `~/.config/agents/skills/` |
| Antigravity | `antigravity` | `~/.gemini/antigravity/skills/` |
| Augment | `augment` | `~/.augment/skills/` |
| Claude Code | `claude-code` | `/home/flexnetos/meta/var/lib/claude/skills/` |
| Cline | `cline` | `~/.agents/skills/` |
| Codex | `codex` | `/home/flexnetos/meta/var/lib/codex/skills/` |
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

(Project paths and MCP/command targets differ per agent; e.g. Codex uses TOML, Copilot uses
VS Code `mcp.json`, OpenCode uses its own JSON shape — see §7/§8. Not every agent supports
commands — unsupported ones return `None` and are silently skipped.)

**Relevance to envctl:** **PARTIAL.** The "exhaustive enum of known targets, each mapping to
canonical paths, with a raw-`destination` escape hatch" is a strong pattern envctl can mirror
for **known tool families** (e.g. CUDA toolchains, shells). envctl's equivalent is its
component registry; adopting the *compiler-enforced exhaustiveness* + a generic fallback is the
takeaway.

---

## 7. Supported Sources + Source Resolvers (GitHub/GitLab/Bitbucket/Codeberg/Gitea, public+private, self-hosted)

**What:** Sources can be local paths or remote repos across multiple git hosts, public or
private, including self-hosted/enterprise instances. Browser URLs are accepted and rewritten.

**How kasetto implements it (`source/`):**
- **URL classification** (`hosts.rs`): `is_gitlab_host` (`gitlab.com`, `*.gitlab.com`,
  `gitlab.*`), `is_bitbucket_host` (`bitbucket.org`), `is_gitea_style_host` (Codeberg, Gitea,
  Forgejo). `extract_host` strips scheme.
- **Parsing** (`parse.rs`): `enum RepoUrl { GitHub, GitLab, Bitbucket, Gitea }`. GitHub
  Enterprise = any 2-segment host not matching the others; GitLab supports nested subgroups
  (3+ segments). Trims `.git` and trailing `/`.
- **Archive download** (`remote.rs`): builds host-specific tarball URLs —
  - GitHub web `…/archive/refs/heads/<branch>.tar.gz` **or** the API
    `api.github.com/repos/.../tarball/<ref>` when a token is set (web archive doesn't auth
    private repos); refs URL-encode `/` → `%2F`.
  - GitLab API `…/api/v4/projects/<encoded path>/repository/archive.tar.gz?sha=<ref>`.
  - Bitbucket `…/get/<branch>.tar.gz`; Gitea/Codeberg `…/archive/<branch>.tar.gz`.
  - `download_extract` streams the gzip tar, strips the top-level dir, and **rejects unsafe
    archive paths** (any `..` component) — a tar-slip guard.
- **Browser-URL rewriting** (`rewrite_browse_to_raw_url`): paste a `github.com/.../blob/...`,
  Codeberg `/src/branch/...`, or GitLab `/-/blob/...` URL and it's rewritten to the raw-content
  endpoint. Used for both skill sources and remote `--config` URLs.
- **Default-branch fallback** (`materialize_source`): with no `ref`/`branch`, tries `main` then
  `master`. `ref` > `branch` > default precedence (`SourceSpec::git_pin`).
- **Local sources**: any `source` without `://` is treated as a local path
  (`~` expansion, relative-to-config resolution), no cleanup, revision label `local`.

**Supported source hosts:** GitHub + GitHub Enterprise; GitLab + self-hosted GitLab (incl.
subgroups); Bitbucket Cloud; Codeberg; Gitea; Forgejo; plus arbitrary local directories.

**Relevance to envctl:** **ADOPT (high).** envctl's `add-repo` already "turns an upstream git
repo into a managed build-from-source component." kasetto's **multi-host resolver layer**
(classify → parse → host-specific tarball URL → safe extract → default-branch fallback →
browser-URL rewrite → tar-slip guard) is almost directly portable and would harden + broaden
`add-repo` beyond GitHub. The `..`-path rejection and `ref > branch > default` precedence are
must-have safety/UX details.

---

## 8. Per-Agent Native-Format Transforms (commands) + Auto-Merge (MCPs)

**What:** One canonical asset is transformed into each target's native on-disk format; MCP
packs are *merged* (not overwritten) into each agent's settings file.

**How kasetto implements it:**
- **Command transforms** (`prompts/`): `parse.rs` splits Markdown frontmatter/body;
  `transform.rs::render` emits one of **5 formats** keyed by `CommandFormat`:
  - `MarkdownFrontmatter` — keep `---`/body, nested `:`→subdir paths (Claude, OpenCode, Roo…).
  - `MarkdownPlain` — strip frontmatter, flatten names with `-` (Cursor, Cline).
  - `PromptMd` — `.prompt.md` (GitHub Copilot).
  - `PromptFile` — Continue `.prompt` with a YAML preamble + `invokable: true`, rewriting
    `$ARGUMENTS` → `{{{ input }}}`.
  - `GeminiToml` — `.toml` with `description` + `prompt = """..."""` (Gemini CLI).
  - `derive_relpath` chooses nested vs flattened filenames per format.
- **MCP merge** (`mcps/`): `merge_mcp_config` dispatches on `McpSettingsFormat` — **4 formats**:
  - `McpServers` (standard `{"mcpServers": {...}}` JSON — Claude, Cursor, Gemini, etc.).
  - `VsCodeServers` (`{"servers": {...}}`, injects `type: stdio|http` — Copilot).
  - `OpenCode` (`{"mcp": {...}}` with `type: local|remote`, `command` as array, `environment`).
  - `CodexToml` (`/home/flexnetos/meta/var/lib/codex/config.toml` `[mcp_servers]`, stdio vs remote, `http_headers`).
  - **Additive, non-destructive:** `merge_into_json_key` only inserts keys not already present
    (`if !dst_map.contains_key(...)`), so user-authored servers and secrets are never
    overwritten (test: real `AIRFLOW_PASSWORD` is preserved over a placeholder from the pack).
  - Codex TOML merge preserves unrelated keys (e.g. `model = "gpt-5.1"`).
- **Settings I/O** (`fsops/settings.rs`): `SettingsFile::load/save` creates parent dirs, treats
  a missing file as `{}`, rejects invalid JSON.

**Relevance to envctl:** **PARTIAL→ADOPT.** The *exact* formats are AI-specific (N/A), but two
ideas are strong ADOPTs: (a) **format-dispatch via an enum** so one canonical artifact emits
many native shapes — useful if envctl wires the same tool into multiple config systems
(shell rc, systemd, env files); (b) the **additive/non-destructive merge** discipline — merge
into existing config files inserting only what's missing, never clobbering user content — which
maps directly to envctl's "back up before clobber, never touch user data" safety model and
should govern any config-wiring component.

---

## 9. Sync / Apply Pipeline + Dry-Run

**What:** The `sync` command reads config, resolves scope/destinations, then installs / updates /
removes each asset to make disk match config — with a `--dry-run` preview.

**How kasetto implements it (`commands/sync/`):**
1. `mod.rs::run` loads config (with `extends` recursion), resolves scope + per-agent
   destinations, creates dirs (unless dry-run), builds a `SyncContext`, loads lock + machine
   state.
2. Three phases in order: `skills::sync_skills`, then `commands::sync_commands`, then
   `mcps::sync_mcps`. Each: derive desired set → per-source `needs_fetch` gate → either honor
   from lock (no network) or fetch+materialize → classify each item as
   installed/updated/unchanged → apply → record in lock.
3. **Stale removal** (`remove_stale`): assets in the lock but no longer desired are removed
   (dirs/files deleted, MCP server entries scrubbed from settings, lock entries dropped) — but
   **skipped if any source failed**, to avoid destroying still-locked assets on a partial error.
4. A `Report { run_id, config, destination, dry_run, summary{installed,updated,removed,
   unchanged,broken,failed}, actions[] }` is produced; lock + machine state saved (unless dry-run).
5. **Dry-run:** every mutating branch is gated on `ctx.dry_run`; statuses become
   `would_install` / `would_update` / `would_remove`; nothing is written.

**Envctl status:** **ABSORBED and made transactional.** Preview still classifies the complete plan
without writing. Apply stages and revalidates every output, then commits output bytes, the v3 lock,
and the runtime ownership/report ledger as one strict rollback boundary. A failed commit restores
the prior coherent state; only after a successful commit do in-memory ownership claims advance.
`clean` likewise enumerates exact proof units (including tombstones), refuses incomplete or drifted
proof sets, and keeps ownership evidence intact if output cleanup cannot complete. The shared
engine remains non-printing and exposes the uniform structured result to both front-ends.

---

## 10. `--json` Structured Output

**What:** Most commands emit machine-readable JSON for scripting/CI instead of human tables.

**How kasetto implements it:** `--json` flag on `sync`, `list`, `doctor`, `clean`, `self update`.
`ui::print_json` serializes a per-command `#[derive(Serialize)]` struct (`Report`,
`DoctorOutput`, `CleanOutput`, `UpdateOutput`, list's JSON object with `merged_scopes`). JSON
mode suppresses banners, spinners, and the update notice.

**Relevance to envctl:** **PARTIAL (already has it).** envctl's `auto-detect --json` emits an
`EnvReport`. ADOPT the breadth: make **every** verb support `--json` with a stable per-command
schema (kasetto does this uniformly), so `install`/`reset`/`auto-fix` are all CI-consumable, not
just `auto-detect`.

---

## 11. Exit Codes + CI Ergonomics

**What:** Real exit codes and CI-friendly flags so pipelines can gate on results.

**How kasetto implements it:**
- `sync` exits `1` when `report.summary.failed > 0` (source/config read failures); broken
  *individual* skills are non-fatal (kasetto keeps going) and only bump the `broken` counter.
- `--locked`/`--frozen` (CI mode): never fetch; **error** if the lock can't satisfy the config
  (named asset absent, or source absent). `--locked --update` is rejected as contradictory.
  `--locked` no-op runs print `Audited N items`.
- Flags: `--dry-run`, `--json`, `--color <auto|always|never>` (honors `NO_COLOR`,
  `CLICOLOR_FORCE`; auto-detects non-TTY), `-q/--quiet` (repeatable), `-v/--verbose` (`-v`..`-vvv`).
- `site/docs/ci.mdx` recommends `kst sync --project --dry-run --json` in GitHub Actions and
  `--locked` for reproducible installs.

**Envctl status:** **ABSORBED and fail-closed.** Locked source/config/selector/ownership failures
surface as `locked_error`, increment `failed`, are cached in the coherent top-level `SyncResult`
for `doctor`, and produce a non-zero front-end exit. `lock --check --locked` is the read-only
zero-network drift gate; it writes neither outputs nor the lock.

---

## 12. Update / Lock-Refresh Semantics (`--update`, `--locked`, plain)

**What:** Three precise modes controlling when moving refs are re-resolved.

**How kasetto implements it (`CLAUDE.md` + `commands/sync/*`):**
- **Plain `sync`** — lock is authoritative; honors locked hashes, **no network** if disk
  matches; wildcard sources hold to the *locked set* (don't pick up newly-added upstream skills).
- **`--update [name...]`** — the only path that re-resolves branches/default HEAD and rewrites
  locked hashes + revisions. Selective `--update foo bar` re-resolves only sources providing
  those names (`update_active_for_source`). For a wildcard source, `--update` re-runs discovery
  and **prunes** upstream-removed items.
- **`--locked`/`--frozen`** — never fetch; error if unsatisfiable (see §11).
- **Revision tracking:** `expected_revision()` produces `ref:<r>` / `branch:<b>` / `branch:main`
  / `local`; if the lock's `source_revision` differs (user retargeted a `ref`/`branch`), a
  fetch is forced even when the old content still hashes correctly.

**Envctl status:** **ABSORBED with a stricter zero-network boundary.** Only `--locked`/`--frozen`
guarantees zero network. Plain sync is allowed to rebuild/re-resolve/materialize the configured
sources; `--update [name...]` controls explicit moving-pin/selection update intent and selective
resolution, but is not the only network-enabling mode. Locked mode accepts only v3 and verifies
local source bytes plus remote source/revision/selector bindings from the committed lock; it never
constructs a remote fetch to fill a missing input.

---

## 13. Install / Distribution (binaries, curl|sh, Homebrew, Scoop, Cargo)

**What:** Single static binary per platform, multiple install channels, dual binary names.

**How kasetto implements it:**
- **Two binaries from one crate** (`Cargo.toml`): `kasetto` (`src/main.rs`, `default-run`) and
  `kst` (`src/bin/kst.rs`). Both share all code; the program name is detected at runtime
  (`app::current_program_name`) so help/output reflect how it was invoked.
- **Release profile:** `lto="fat"`, `codegen-units=1`, `panic="abort"`, `strip="symbols"`,
  `mimalloc` allocator → small fast static binary. `unsafe_code = "forbid"`.
- **Channels:** `curl -fsSL kasetto.dev/install | sh` (+ PowerShell `install.ps1`),
  `brew install pivoshenko/tap/kasetto` (`Formula/kasetto.rb`, symlinks `kst`→`kasetto`),
  Scoop bucket, `cargo install kasetto`.
- **Release workflow** (`release.yaml`): builds a 6-target matrix (linux/macos/windows ×
  x86_64/aarch64; cross-compiles aarch64-linux), emits `checksums.txt`, cuts a GitHub Release,
  `cargo publish`, and regenerates the Homebrew formula + Scoop manifest.

**Relevance to envctl:** **PARTIAL.** envctl is a personal single-box tool (cargo-build), so
broad multi-channel distribution is lower priority. ADOPT-worthy if envctl is ever shared: the
**dual-binary-from-one-crate** trick (short alias + full name), the hardened release profile,
and the **checksummed release artifacts** consumed by self-update (§15).

---

## 14. `doctor` Diagnostics

**What:** Local health check: version, scope, lock path, install paths, last-sync status,
inventory counts, writability checks, failed-skill detail, update status.

**How kasetto implements it (`commands/doctor.rs`):** loads lock + machine state; reports
`Environment` (scope, lock file, install path, last sync, update status), `Inventory`
(skills/MCPs/commands counts), `Checks` (lock readable, install path writable, no failed skills,
N-of-M command dirs writable — `is_writable` walks up to the first existing ancestor and probes
`readonly()`), a `Command directories` panel scoped to the config's agents, and a `Failures`
detail section sourced from the last sync's machine-local `Report`. `--json` emits `DoctorOutput`.

**Relevance to envctl:** **ADOPT.** This is a direct analog to envctl's `auto-detect`, but with
**actionable health checks** (writability probes, "last op status," failed-component detail
from a persisted report). envctl's detect could grow these: surface the last `install`/`auto-fix`
report and per-target writability so users see *why* a component is broken before acting.

---

## 15. Self-Update with Checksum Verification (`self update`)

**What:** Fetch the latest GitHub release, verify SHA-256 against `checksums.txt`, and replace
the running binary in place with rollback on failure.

**How kasetto implements it (`commands/self_update.rs`):** `fetch_latest_release` hits the
GitHub releases API; `is_newer` does numeric semver compare; picks the asset matching the
current `arch-os` target; downloads, **verifies SHA-256 against `checksums.txt`** (aborts on
mismatch), extracts (tar-slip-guarded, only `kasetto`/`kst` entries), backs up the old binary to
`.old`, copies in the new one (`chmod 0755` on unix), and **restores the backup if the swap
fails**. `self uninstall` removes assets, XDG config/data dirs, and the binary.

**Relevance to envctl:** **N/A→PARTIAL.** A personal cargo-built tool self-updates via
`cargo install`/`git pull`, so binary self-update is mostly N/A. But the **download → checksum-
verify → atomic swap with backup-and-rollback** pattern is exactly envctl's safety doctrine and
is reusable for *any* binary/artifact a component builds and installs (verify before clobber,
roll back on failure).

---

## 16. Machine-Local Runtime State (separate from the committed lock)

**What:** Per-machine, throwaway state (last run time, last sync `Report`, per-skill install
timestamps) kept *out* of the committed lock — mirroring how uv separates its cache from
`uv.lock`.

**How kasetto implements it (`state.rs`, `CLAUDE.md`):** JSON under
`$XDG_CACHE_HOME/kasetto/runtime/<hash-of-lock-path>.json`. Holds `last_run`, the latest sync
`Report` (so `doctor` can show failures), and per-skill `updated_at` (so `list` shows
"updated N ago"). Safe to delete; regenerated on next sync. Keyed by a SHA-256 of the lock path
so multiple projects don't collide.

**Envctl status:** **ABSORBED with a deliberate ownership split.** Telemetry remains disposable,
but v3 runtime state also carries `managed_outputs`: the authoritative ownership ledger for global
scope and the second factor for retired project custom roots. Deleting that cache is harmless to
ordinary project installs because their relative `installed_outputs` live in the committed lock;
it intentionally removes authority to mutate global or historical custom-root outputs. Envctl
does not reconstruct that authority from matching pre-existing bytes; an absent output may be
installed anew by a safe plain apply. A successful sync stores the final top-level
`SyncResult`; a coherent failed transaction stores its failure report without advancing ownership.

---

## 17. Graph / Dependency / Topology Logic

**What:** Ordering and dependency resolution between assets.

**How kasetto implements it:** Essentially **none** at the asset level. Sync order is fixed
(skills → commands → MCPs), and within a kind, sources are processed in config order. There's no
skill-to-skill dependency graph or topological sort. The only "graph-ish" structure is the
**`extends` config DAG** (parents merged before children, with cycle detection + depth cap) and
**file-tree walks** for discovery (skills dirs, namespaced commands, MCP files).

**Relevance to envctl:** **N/A (kasetto) — but envctl is *ahead* here.** envctl already does
"install components in **dependency order**" — a real topological concern kasetto doesn't have.
So envctl should *not* look to kasetto for this; rather, note that kasetto's flat model works
because skills are independent. The reusable crumb is only the **`extends` cycle/depth guard**
pattern (§3), applicable to envctl's component-dependency graph for cycle detection.

---

## 18. Security Model

**What:** A constrained, tracked-only, no-code-execution, env-var-credentials threat model.

**How kasetto implements it (`site/docs/security.mdx` + code):**
- **Does not run skill/asset code** — it only copies files and merges JSON/TOML.
- **Tracked-only mutation:** only lock-tracked skills/MCP servers are removed during cleanup;
  user-authored content is never touched. MCP merges are additive (never overwrite existing
  servers — §8).
- **Credentials via env vars only** (`source/auth.rs`): `GITHUB_TOKEN`/`GH_TOKEN`,
  `GITLAB_TOKEN`/`CI_JOB_TOKEN`, `BITBUCKET_EMAIL`+`BITBUCKET_TOKEN` (or
  `BITBUCKET_USERNAME`+`BITBUCKET_APP_PASSWORD`, sent as HTTP Basic),
  `GITEA_TOKEN`/`CODEBERG_TOKEN`/`FORGEJO_TOKEN`. **No credentials file** is read or written.
  Token selection is host-based; the same tokens authenticate remote `--config` URLs.
- **Helpful auth errors:** on 401/403/404 or an HTML-instead-of-tarball response, kasetto prints
  a host-specific "set <TOKEN>" hint (`auth_env_inline_help`, `http_fetch_auth_hint`).
- **Archive safety:** tar extraction rejects any `..` path component (tar-slip guard), for both
  source archives and self-update.
- **Self-update integrity:** SHA-256 verification before binary swap (§15).
- `unsafe_code = "forbid"` crate-wide.

**Envctl status:** **ABSORBED and strengthened.** Env-var-only credentials, additive MCP merge,
tar-slip refusal, and host-aware auth diagnostics remain. V3 further requires exact ownership
proof before replacement/deletion, rejects symlink/foreign-owner/path-escape ambiguity, binds
selectors and target formats, and commits outputs plus both ledgers transactionally. A synthetic
or freshly rebuilt v3 desired lock with empty `installed_outputs` cannot authorize a locked
mutation: the first authorized install into absent destinations is a plain `sync --apply`, which
records proofs. Commit that resulting project lock; a clean clone can then use
`sync --locked --apply`. V2 exact-output
bootstrap exists only for that one migration path, never as a general first-install shortcut.

---

## 19. UI / Output System (cargo/uv-aligned)

**What:** A polished, terminal-theme-respecting CLI UX with spinners, trees, and uv-style
summaries.

**How kasetto implements it:** `colors.rs` uses the basic ANSI-16 palette (so hues inherit the
user's terminal theme, like cargo/uv); only the brand banner (`banner.rs`) uses 24-bit color and
only on bare `kst`/`kst init`. `ui.rs` provides spinners (`with_spinner`, braille frames),
action glyphs (` + ` / ` ~ ` / ` - ` / ` = ` / ` ! `), source-grouped trees, `print_json`,
and uv-style summary verbs (`Installed N items in Xms`, `Audited`, `Removed`). A background
`update_notifier` thread refreshes a 24h-TTL update cache and prints one yellow line at end of
run (suppressed for json/quiet/non-TTY/etc.). Shell completions (`completions <bash|zsh|fish|
powershell>`) via `clap_complete`.

**Relevance to envctl:** **PARTIAL.** envctl has both a CLI and an egui GUI. ADOPT the
**ANSI-16/terminal-theme-respecting** approach for the CLI, the **uv-style verb summaries**, the
**action-glyph diff rows** for showing what changed, and **shell completions** via
`clap_complete`. The background update-notifier is lower value for a local tool.

---

## 20. Build / Dev Tooling

**What:** Unified `just` recipes and a strict lint posture.

**How kasetto implements it:** `justfile` — `just check` = format + lint + test + build for both
the Rust crate and the Next.js docs `site/`. `Cargo.toml [lints]`: clippy `all` denied, `perf`
warned, `dbg!`/`todo!` warned, `unsafe_code` forbidden. CI (`ci.yaml`) runs two parallel jobs
(rust + site). `git-cliff` drives conventional-commit-based versioning + changelog in
`release.yaml`.

**Relevance to envctl:** **PARTIAL.** envctl already uses cargo + a pinned toolchain. ADOPT the
**strict lint gate** (deny clippy::all, forbid unsafe — though envctl's source-building nature
may need targeted `unsafe` exceptions) and **conventional-commit-driven changelog/versioning**
via git-cliff if it wants automated releases.

---

## Cross-Cutting Notes

- **Parallelism:** kasetto is essentially **sequential** (per-source, per-asset loops). The
  "speed" claim comes from the **no-fetch-when-hashes-match** gate, not threading. The only
  threads are the spinner animation and the background update check. (envctl's
  dependency-ordered install is inherently more concurrency-sensitive.)
- **Caching:** No content cache of downloaded archives — each fetch downloads to a temp stage,
  extracts, and deletes. The lock's hash gate *avoids* fetches entirely rather than caching
  their results. The only persistent cache is the update-check JSON (24h TTL).
- **Idempotency:** Strong — re-running plain `sync` on a converged tree is a no-op
  (all-`unchanged`, zero writes). This matches envctl's "idempotent install" goal.

---

## Executive Summary — Top Features (ranked) · STATUS: all ABSORBED (§0)

> Originally "top features envctl should adopt." As of 2026-06-14 **all ten are absorbed** into
> `crates/agent-env` + `Engine::agent_*` at parity (§0). Kept ranked as the rationale record.


1. **Committed lock file with reproducible, scope-aware content hashing (§2).** The single
   highest-value idea became `agent-env.lock` v3: source/revision/content pins plus selector/target
   binding and separate ownership proofs. Project tree hashes normalize to Git-reproducible mode
   semantics; global hashes deliberately retain exact platform/mode semantics.
2. **Lock-driven sync modes — plain / `--update` / `--locked` (§12, §11).** Plain may resolve
   configured sources; `--update` expresses moving-pin/selection update intent; only `--locked`
   is a zero-network CI gate that fails closed on drift or missing proof. Plus crisp exit codes.
3. **Multi-host source resolver layer (§7).** Port kasetto's classify→parse→host-tarball→safe-
   extract→default-branch-fallback→browser-URL-rewrite pipeline to harden `add-repo` beyond
   GitHub, including the tar-slip (`..`) guard and `ref > branch > default` precedence.
4. **`extends` layered config composition with identity-keyed merge + cycle/depth guards (§3).**
   Let a shared base manifest be overlaid by machine-specific configs without copy-paste; the
   value-level merge and cycle detection are directly reusable for envctl's component graph.
5. **Additive, non-destructive config merge discipline (§8, §18).** Any envctl component that
   wires into an existing config file (shell rc, TOML, JSON) should insert only missing keys and
   never clobber user content/secrets — exactly kasetto's MCP-merge rule, and a perfect fit for
   envctl's "back up before clobber, never touch user data" doctrine.
6. **Separate committed lock from machine-local runtime state (§16).** Keep the reproducible
   record diff-clean; put last-run, last report, and timings in `$XDG_CACHE_HOME`, keyed by a
   hash of the lock path.
7. **Universal `--json` + structured per-command reports (§10, §9).** Make *every* verb
   (`install`/`reset`/`auto-fix`, not just `auto-detect`) emit a stable JSON schema with uniform
   `installed/updated/removed/unchanged/broken/failed` counters for CI consumption.
8. **`doctor`-style actionable diagnostics (§14).** Extend envctl's `auto-detect` with
   writability probes, last-op status, and persisted failure detail so users see *why* a
   component is broken before acting.
9. **Env-var-only credentials with host-aware auth hints (§7, §18).** No secrets on disk;
   host-based token selection; print "set <TOKEN>" hints on 401/403/404 — useful for any private
   source `add-repo` pulls from.
10. **CLI UX polish (§19).** ANSI-16 terminal-theme-respecting colors, uv-style verb summaries,
    action-glyph diff rows, and `clap_complete` shell completions.

---

*Catalog of the kasetto **v3.2.0** source tree (`meta/kasetto`, upstream
`github.com/pivoshenko/kasetto` `ec01cca`). All feature behavior is absorbed into envctl's
`crates/agent-env` + `Engine::agent_*` at parity (§0; ledger
`.handoff/loop/rust-port/parity-ledger.md`). Module paths in the per-section "how kasetto
implements it" notes refer to the kasetto source, not envctl's port.*
