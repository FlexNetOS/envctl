# ADR-0003 — Make envctl catalog-first, with generated-file projections and bidirectional sync

- **Status:** proposed — 2026-06-28
- **Plane:** env-control / catalog-state / configuration-generation
- **Scope:** `envctl catalog`, `envctl config`, manifest components and locks, agent-env config/lock, Codex/MCP config, registries, canonical paths, env vars, secretd config surfaces, `.handoff` exports, future CLI/TUI configuration widget, and doctor drift reporting.
- **Relates:** `docs/ARCHITECTURE.md`; `docs/MIGRATION-ADOPTION.md`; `.handoff/decisions/ADR-0002-migration-adoption-engine-v2.md`; `manifest/**/*.toml`; `manifest/envctl.lock`; `agent-env.yaml`; `agent-env.lock`; `.codex/config.toml`; `.mcp.json`; `crates/engine/src/layout.rs`; `crates/engine/src/{manifest,lock,agent_env,secretd}`.

## Context

Envctl currently treats repo files as the primary shape of intent. The rough flow is:

```text
files -> structs -> reports
```

That is workable for a manifest-driven bootstrapper, but it is too weak for a workspace control plane. Settings, configs, variables, registries, paths, agent assets, migration evidence, and observed machine facts are spread across TOML/YAML/JSON/Rust code/lock files and per-command checks. The result is that ownership, precedence, generated-vs-manual status, and drift are not consistently queryable.

The target architecture is:

```text
tables -> generated files -> live system
```

The safe migration path is:

```text
existing files -> query/catalog tables -> diff/validate -> bidirectional sync -> DB/table-first generation
```

This ADR intentionally does **not** rip out TOML/YAML/JSON or ask humans to rewrite files by hand. Envctl will absorb the current files first, normalize them into queryable catalog rows/views, prove round-trip stability, then gradually promote the catalog into the source of truth.

## Decision

Envctl will add a catalog layer that makes desired state and observed state first-class table-shaped data. The catalog becomes the normalization authority first, then the render/diff authority, and eventually the DB/table-first source of truth for generated-file projections.

The control-plane lifecycle becomes:

```text
declare -> catalog -> render -> apply -> observe -> prove -> lock
```

Instead of:

```text
files everywhere -> scattered parsers -> scattered reports
```

The first implementation phase is read-only. Existing files remain accepted inputs. Later phases add deterministic render, bidirectional reconciliation, controlled row editing, and DB/table-first operation.

## Phase 1 — Query-generated tables from existing files

Build `envctl catalog` as a read-only import/report surface that reads current repo files and produces normalized tables/views.

Example command surfaces:

```bash
envctl catalog scan --json
envctl catalog table components
envctl catalog table paths
envctl catalog table settings
envctl catalog table env-vars
envctl catalog table agent-assets
envctl catalog table registries
envctl catalog table config-files
envctl catalog table observed-facts
envctl catalog table migration-evidence
envctl catalog table migration-candidates
```

Initial sources:

- `manifest/**/*.toml`
- `manifest/envctl.lock`
- `agent-env.yaml`
- `agent-env.lock`
- `.codex/config.toml`
- `.mcp.json`
- hub `registry.json` files
- `layout.rs` canonical path registry
- `secretd.toml` and env-var schema surfaces
- `.handoff` task cards, ledger exports, and rendered reports

This phase gives envctl a queryable control plane without changing ownership or behavior.

## Phase 2 — Canonical catalog schema

The catalog schema must make the following tables/views first-class. Implementations may start with in-memory views and later persist them, but the row contracts are the stable interface.

### `components`

One row per component.

| Field | Meaning |
| --- | --- |
| `component_id` | Stable component identifier. |
| `name` | Human-readable name. |
| `source_file` | File that produced the row. |
| `description` | Component description. |
| `requires` | Component dependencies. |
| `gpu_required` | Whether GPU capability is required. |
| `destructive` | Whether any lifecycle action can be destructive. |
| `has_detect` | Detect hook/implementation exists. |
| `has_install` | Install hook/implementation exists. |
| `has_verify` | Verify hook/implementation exists. |
| `has_fix` | Fix hook/implementation exists. |
| `has_remove` | Remove hook/implementation exists. |
| `status` | Desired/catalog status. |
| `lock_hash` | Lock/effective content hash. |
| `resolved_order` | Dependency-resolved order. |

### `component_hooks`

One row per lifecycle hook.

| Field | Meaning |
| --- | --- |
| `component_id` | Owning component. |
| `phase` | Lifecycle phase: detect/install/verify/fix/remove/etc. |
| `hook_kind` | Script, command, Rust-native, generated unit, or other supported hook kind. |
| `command` / `script` / `path` | Executed command or referenced artifact. |
| `args` | Hook arguments. |
| `env` | Hook-specific environment. |
| `needs_sudo` | Whether elevated privileges are required. |
| `login_shell` | Whether a login shell is required. |
| `source_file` | File that produced the hook row. |

### `paths`

One row per path envctl knows about.

| Field | Meaning |
| --- | --- |
| `path_id` | Stable path row identifier. |
| `path` | Filesystem path or templated path expression. |
| `path_kind` | Binary, library, config, cache, state, model, service, shim, toolchain root, etc. |
| `owner_component` | Component that owns or declares the path. |
| `artifact_kind` | Kind of artifact at the path. |
| `canonical` | Whether this is the canonical meta-owned path. |
| `legacy` | Whether this is a legacy/adopted path. |
| `bridge` | Whether this path is a compatibility bridge/shim/symlink. |
| `protected` | Whether mutation/purge is forbidden. |
| `source` | Source file, code registry, or observed source. |
| `verification_status` | Path verification result. |

### `settings`

One row per setting.

| Field | Meaning |
| --- | --- |
| `setting_key` | Stable dotted key. |
| `value` | Normalized value, redacted when sensitive. |
| `source_file` | File that produced the setting. |
| `source_kind` | Manifest, lock, agent-env, Codex config, MCP config, secretd config, CLI override, etc. |
| `owner_component` | Owning component or subsystem. |
| `scope` | Workspace, repo, component, user-bridge, service, session, etc. |
| `precedence` | Effective precedence/rank. |
| `sensitive` | Whether the value is sensitive and must not be rendered in plaintext reports. |
| `generated` | Whether the setting should be generated. |
| `manual_override` | Whether an explicit manual override is active. |
| `drift_status` | In-sync, changed, conflict, missing, unknown. |

### `env_vars`

One row per environment variable.

| Field | Meaning |
| --- | --- |
| `var_name` | Environment variable name. |
| `value` | Declared value, redacted when sensitive. |
| `producer` | Component/subsystem that produces it. |
| `consumer` | Component/subsystem that consumes it. |
| `scope` | Shell, service, daemon, CLI, GUI, hook, test, etc. |
| `sensitive` | Whether the value is secret/sensitive. |
| `default_value` | Default value, if any. |
| `effective_value` | Resolved effective value, redacted when sensitive. |
| `source` | File, code registry, or generated source. |
| `generated_by` | Catalog renderer or subsystem that emits it. |

### `agent_assets`

One row per skill, MCP, command, or other agent asset.

| Field | Meaning |
| --- | --- |
| `asset_kind` | Skill, MCP, command, agent, hook, setting, or generated config. |
| `name` | Asset name. |
| `source` | Source repository/path/registry entry. |
| `destination` | Rendered or synced destination. |
| `hash` | Content hash. |
| `source_revision` | Source revision/ref when applicable. |
| `lock_status` | Lock match/mismatch/missing. |
| `drift_status` | Drift status. |

### `registries`

One row per external or internal registry item.

| Field | Meaning |
| --- | --- |
| `registry_kind` | Hub, plugin, MCP, component, prompt, task, package, model, etc. |
| `entry_id` | Stable registry entry id. |
| `name` | Human-readable name. |
| `component_id` | Owning/related component id. |
| `status` | Active, inactive, deprecated, missing, conflict, etc. |
| `tier` | Trust/support/priority tier. |
| `source_file` | Source file that produced the row. |
| `drift_status` | Registry drift status. |

### `config_files`

One row per imported or rendered control-plane/config file. This prevents "configs" from being only incidental sources of settings; the file/projection itself is queryable.

| Field | Meaning |
| --- | --- |
| `config_id` | Stable config/projection identifier. |
| `path` | Source or destination path. |
| `file_kind` | Manifest, lock, agent-env, Codex config, MCP config, hub registry, secretd config, handoff export, shell snippet, dashboard layout, systemd unit, etc. |
| `format` | TOML, YAML, JSON, KDL, shell, Rust registry, rendered report, etc. |
| `owner_component` | Owning component, when component-scoped. |
| `owner_subsystem` | Owning subsystem when not component-scoped. |
| `source_file` | Existing file that produced the row, if imported. |
| `source_table` | Catalog table/view that renders the projection, if generated. |
| `generated` | Whether envctl is expected to render the file. |
| `manual_edits_allowed` | Whether human edits are accepted and importable. |
| `override_path` | Override/import path for manual control. |
| `precedence_scope` | Scope in which this file wins or loses precedence. |
| `sensitive_policy` | Redaction/reference policy for sensitive content. |
| `renderer` | Renderer responsible for producing the file, if generated. |
| `validator` | Validator/verifier for import or render acceptance. |
| `lock_hash` | Lock/effective content hash. |
| `drift_status` | File/catalog/lock drift status. |

### `migration_evidence`

One row per adoption or migration action.

| Field | Meaning |
| --- | --- |
| `component_id` | Owning component. |
| `artifact_kind` | Binary, config, cache, state, agent asset, service, registry, etc. |
| `legacy_path` | Previous path. |
| `canonical_path` | New canonical path. |
| `before_checksum` | Checksum before adoption, where meaningful. |
| `after_checksum` | Checksum after adoption, where meaningful. |
| `before_version` | Version/identity before adoption. |
| `after_version` | Version/identity after adoption. |
| `verifier` | Verifier command or Rust verifier. |
| `verifier_status` | Verifier result. |
| `activation_status` | Whether activation succeeded. |
| `quarantine_path` | Quarantine location, if any. |
| `rollback_plan` | Recovery plan. |
| `purge_eligible` | Whether later purge is allowed. |

### `observed_facts`

One row per verifier/probe result about the live machine. Desired-state tables say what envctl intends; observed facts say what the host actually proves right now.

| Field | Meaning |
| --- | --- |
| `fact_id` | Stable observation identifier for this probe run/result. |
| `subject_kind` | Component, path, setting, env var, registry entry, agent asset, config file, service, binary, migration, etc. |
| `subject_id` | Catalog row id or external subject id being observed. |
| `probe_kind` | Existence, version, PATH resolution, service status, config-consumed check, parity verifier, etc. |
| `expected_value` | Desired/effective value being checked, redacted when sensitive. |
| `observed_value` | Observed value/result, redacted when sensitive. |
| `status` | Pass, fail, missing, conflict, skipped, unknown. |
| `evidence_ref` | Log/report/probe output reference, not a secret payload dump. |
| `verifier` | Verifier command or Rust verifier that produced the fact. |
| `observed_at` | Observation timestamp. |
| `source` | Source of the observation. |
| `drift_status` | Drift interpretation when compared with desired state. |
| `remediation_hint` | Optional fix/action pointer. |

## Phase 3 — Generated files as catalog projections

Once the catalog can read current state, generated files become deterministic projections of catalog rows.

Examples:

```text
catalog tables
  -> manifest/*.toml
  -> agent-env.yaml
  -> agent-env.lock
  -> .codex/config.toml
  -> .mcp.json
  -> shell env snippets
  -> dashboard layouts
  -> systemd user units
```

Generated files must carry provenance headers where the file format permits it:

```text
Generated by envctl catalog render.
Source table: components
Manual edits allowed: yes/no
Override path: ...
```

Some files become fully generated. Some files may allow manual sections during the transition. Every generated output must be deterministic, diffable, lockable, and represented by a `config_files` row so the projection itself has ownership, precedence, manual-edit policy, and drift status.

## Phase 4 — Bidirectional sync

The rule is not "the DB overwrites files." The rule is round-trip safety.

Command surfaces:

```bash
envctl catalog import     # files -> tables
envctl catalog render     # tables -> files
envctl catalog diff       # table/file drift
envctl catalog sync       # safe bidirectional reconcile
envctl catalog lock       # update lock after accepted changes
```

Manual-file change flow:

```text
file changed manually
  -> envctl catalog import detects change
  -> normalized row changes
  -> render proves stable output
  -> lock updates
```

CLI/widget change flow:

```text
CLI/widget changes table row
  -> render updates file
  -> verifier runs
  -> lock updates
```

This preserves optional human manual control without losing machine authority.

## Phase 5 — CLI-controlled widget

The widget is a controlled table editor, not a raw file editor.

Example command surfaces:

```bash
envctl config edit component codex
envctl config edit path rustup
envctl config edit setting secretd.store.backend
envctl config edit env ENVCTL_USR_BIN
envctl config edit registry prompt_hub

envctl catalog tui
envctl config widget
envctl doctor --fix-widget
```

Widget behavior:

- Show the relevant table rows.
- Show the source file projection.
- Show the generated diff before apply.
- Block unsafe edits.
- Allow manual override only with an explicit reason.
- Write back through the catalog engine, not direct ad-hoc file surgery.
- Run verifier(s) before apply.

This is safer than humans editing TOML/YAML/JSON by hand, while still preserving manual control.

## Phase 6 — Manual override model

Manual control is explicit catalog state, not hidden file drift.

Override fields:

- `manual_override`
- `override_reason`
- `override_owner`
- `override_timestamp`
- `expires_at`
- `review_required`
- `generated_conflict_policy`

Generated files may carry preserved blocks during transition:

```text
# envctl:manual-begin component.foo.notes
...
# envctl:manual-end component.foo.notes
```

The preferred model remains:

```text
manual edit -> import -> normalized table row -> render
```

Not indefinite handwritten islands.

## Desired state versus observed state

The catalog must keep two concepts separate.

### Desired state

What envctl intends. Examples:

- components
- settings
- paths
- env vars
- registries
- agent assets
- config files/projections

### Observed state

What the machine actually has, captured as `observed_facts` rows. Examples:

- binary exists
- version probe result
- PATH resolves correctly
- config file was consumed
- service is running
- migration parity passed

Doctor becomes a table diff:

```text
desired_state - observed_facts = drift
```

That is cleaner and safer than scattered checks whose ownership and precedence are hard to inspect.

## Best migration path

1. Add read-only catalog import: files to normalized tables/views. No behavior change.
2. Add table diff/report: catalog rows versus source files versus lock files.
3. Add render without apply: tables to generated files in a temp dir, then compare against current files.
4. Make selected files generated-but-compatible. Start with low-risk outputs:
   - reports
   - lock summaries
   - path registry export
   - env var catalog
   - hub registry report
5. Make component manifests bidirectional. TOML remains accepted input, but catalog becomes the normalization authority.
6. Add CLI/TUI widget. Human edits go through controlled row mutation.
7. Move to DB/table-first operation: catalog DB is source of truth, files are projections, and manual edits are imported/reconciled.

## Non-goals

- Do not immediately replace component manifests with a database.
- Do not let DB/table state overwrite manual edits without import, diff, conflict policy, and verifier approval.
- Do not store plaintext secrets, bearer tokens, vault material, or other sensitive values in general catalog tables or reports.
- Do not add C-linked local database dependencies or weaken the no-C trust boundary.
- Do not bypass the engine-first rule by putting catalog logic only in CLI or GUI code.
- Do not make handwritten generated-file islands permanent; temporary manual blocks must be importable and reviewable.

## Invariants

- Read-only import comes first and must not change behavior.
- Generated outputs are deterministic and diffable.
- Every row carries source/provenance where practical.
- Ownership and precedence are explicit for settings, config files/projections, paths, env vars, registries, and agent assets.
- Observed machine facts carry verifier, evidence reference, timestamp, and drift interpretation before doctor treats drift as proven.
- Sensitive values are redacted or represented by references, never casually rendered in plaintext.
- Manual overrides require reason/owner/timestamp and should expire or require review.
- Migration/adoption remains strict-upgrade-only: replacement must be installed, activated, and parity-proven before quarantine or purge.
- Existing TOML/YAML/JSON remains accepted during the transition.
- CLI and GUI both use the same engine catalog API.

## Consequences

- Envctl becomes a real control plane for environment state rather than a collection of file parsers and reports.
- Doctor, lock, migration, agent-env, and dashboard generation can converge on the same catalog model.
- Implementation requires schema discipline, snapshot tests, import/render round-trip tests, and drift tests.
- The transition is incremental: low-risk read-only tables first, then generated projections, then bidirectional sync, then table-first authority.
- Review becomes easier because changes are visible as row diffs plus deterministic file projections.

## Acceptance reference

This ADR is satisfied when envctl can perform the following without changing existing behavior in the initial phase:

1. `envctl catalog scan --json` emits normalized catalog data from current repo files.
2. `envctl catalog table <name>` can print at least `components`, `component_hooks`, `paths`, `settings`, `env_vars`, `agent_assets`, `registries`, `config_files`, `migration_evidence`, and `observed_facts` views, even if early tables are partially populated.
3. `envctl catalog diff` reports file/catalog/lock drift without mutation.
4. `envctl catalog render --out <tempdir>` produces deterministic projections that can be compared against current files.
5. Later implementation adds `import`, `sync`, `lock`, and controlled `envctl config edit ...` flows with verifier-gated apply.
