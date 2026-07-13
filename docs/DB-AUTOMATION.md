# Meta and LifeOS database automation

`envctl db` is the Rust-native, agent-facing index/query/refactor/deploy surface for two related
but distinct products:

- the **Meta control plane**, rooted at `META_ROOT`; and
- the **LifeOS product runtime**, targeted by `LIFE_OS_ROOT`.

Both are owned project sources in the same organization. The two-root model is about preserving
product identity and choosing the right output root, not about treating one source as hostile or
replacing one project with the other. The implementation lives in `envctl-engine`.
The CLI is wired to these engine APIs today. The native GUI does not yet expose a database screen.
Any GUI integration must call the same engine entry points rather than grow a second implementation.

## Root profiles

`envctl db roots` holds the current and target roots at the same time:

| Root | Role | Profile | Meaning |
|---|---|---|---|
| `META_ROOT` | `observed_current` | `current` | The active Meta control-plane root envctl observes and operates. |
| `LIFE_OS_ROOT` | `release_target` | `lifeos-release` by default | The LifeOS output/validation target. |

`LIFEOS_ROOT` is an accepted input alias. It normalizes to the canonical `LIFE_OS_ROOT`
spelling before comparison or generated output. Root-token matching recognizes `$VAR` and
`${VAR}` forms and does not confuse a longer identifier such as `META_ROOT_FALLBACK` with
`META_ROOT`.

```bash
envctl db roots --observed /home/u/meta --release /home/u/lifeos --json
```

The command is read-only. Omitting `--observed` or `--release` leaves the corresponding absolute
path unset while preserving both logical root rows.

## Index, query syntax, and presets

The CLI query syntax is intentionally small and stable:

```text
envctl db [--repo-root DIR] query --preset PRESET [--explain] [--json]
```

`--repo-root` defaults to the current directory. `--json` emits the machine contract;
`--explain` adds the resolved table/filter trace. The CLI exposes named presets rather than an
SQL parser:

| Preset | Resolved intent |
|---|---|
| `root-meta` | Symbols whose normalized name is `META_ROOT`. |
| `root-lifeos` | Symbols whose normalized name is `LIFE_OS_ROOT`. |
| `hooks-codex` | Indexed files whose path contains `codex`. |
| `wrappers-broken` | Indexed files whose `file_kind` is `shell`. |
| `mutable-unsafe` | Files whose `mutable_policy` is `never`. |
| `symbols-rust-cli` | Rust files used for the structural CLI-symbol view. |
| `paths-legacy` | Indexed files whose `absolute_path` contains the literal `legacy` substring. |

The stable names describe the intended agent workflows, but the current filters are authoritative:
`wrappers-broken` currently resolves to files whose `file_kind` is `shell`,
`mutable-unsafe` currently resolves to files whose `mutable_policy` is `never`, and
`paths-legacy` currently matches the literal `legacy` substring in `absolute_path`. The docs state
those filters explicitly so a caller never infers a stronger health or legacy-prefix analysis
than the engine currently performs.

Current query examples:

```bash
envctl db query --preset root-meta --json
envctl db --repo-root /path/to/repo query --preset symbols-rust-cli --json
```

For a durable file snapshot, `envctl db scan --json` stores the deterministically ordered index
at `<repo-root>/.envctl/db-index.json`; `--no-persist` keeps the scan in memory. `envctl db watch`
performs one content-hash delta poll against that saved baseline and reports added, changed,
removed, and unchanged rows.

## Symbol mapping and impact

The symbol index combines:

- environment-variable and path-token occurrences, including normalized root spellings;
- parsed Rust items and imports; and
- Clap-derived Rust types identified as CLI subcommands.

Each occurrence reports its file, position, context, mutable policy, and whether it is a safe
replace candidate. Use the full symbol/occurrence rows when building navigation, then request a
single-symbol blast-radius report before proposing a rewrite:

```bash
envctl db --repo-root /path/to/repo symbols --json
envctl db --repo-root /path/to/repo impact --symbol LIFE_OS_ROOT --json
```

The shorter forms are `envctl db symbols` and `envctl db impact --symbol NAME`; both still default
to the current directory.

## Safe root-alias refactor

The root-alias refactor is normalization-aware and **plan-only by default**. It rewrites matching
environment tokens rather than performing a blind substring replacement. Every candidate is
grouped by file and returned with a per-line unified diff. Protected files, `.env` files, and
occurrences whose owner policy does not allow automatic replacement are counted as refused and
are never written.

The three execution modes are:

1. No mode flag: return the plan/diffs; do not write.
2. `--render-out DIR`: write safe changes into a new tree while preserving relative paths;
   **originals are never modified**.
3. `--apply --confirm --approve WHO`: change safe files in place. Missing either confirmation or
   approval makes the engine refuse before writing. Existing files receive a `.bak`; each new
   value is written to a sibling temporary file, synced, renamed, reread, and hash-verified.

Current refactor examples:

```bash
envctl db refactor --from META_ROOT --to LIFE_OS_ROOT --json
envctl db --repo-root /path/to/repo refactor --from META_ROOT --to LIFE_OS_ROOT --render-out /tmp/rendered
envctl db --repo-root /path/to/repo refactor --from META_ROOT --to LIFE_OS_ROOT --apply --confirm --approve drdave --note 'REQ-055 migration'
```

Run the in-place form only on a clean, committed tree. The `.bak` is the inner rollback point;
version control remains the outer recovery boundary.

## Safe hook deployment

`envctl db deploy` maps regular files from a staged tree to the same relative paths under a target
root. Symlinks in the staged tree are skipped. Planning is the default and classifies every step:

- `Ready`: eligible for promotion;
- `Queued`: the target appears in a running process command line, so it is left untouched; or
- `Refused`: the indexed target is protected or has `Never` mutable policy.

Applying a deploy requires `--apply --confirm --approve WHO`. Only `Ready` steps are promoted;
`Queued` and `Refused` steps remain untouched. An existing target is copied to
`<target>.envctl-bak`, then the staged bytes are written to a sibling temporary file and renamed
over the target as the atomic promotion step.

Current hook deployment examples:

```bash
envctl db deploy --kind hooks --target /path/to/root --stage /tmp/rendered --json
envctl db deploy --kind hooks --target /path/to/root --stage /tmp/rendered --apply --confirm --approve drdave
```

An omitted `--stage` produces an empty, unapproved plan. No process is stopped or rewritten by
the planner.

## Agent JSON and compact widgets

Use `--json` for roots, queries, symbols, impact, refactor plans, and deploy plans. The engine
sorts indexed rows, changes, and promoted paths for deterministic output, and tests pin the wire
tags and output shape. Compact UI-ready projections are also available:

```bash
envctl db widget roots --json
envctl db widget refs --json
envctl db widget hooks --json
```

These widgets are read-only views of the same engine snapshot; they are not a second database or
an alternate source of truth.
