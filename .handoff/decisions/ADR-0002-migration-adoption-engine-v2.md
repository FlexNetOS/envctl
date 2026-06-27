# ADR-0002 — Upgrade migration/adoption into a parity-proven meta layout engine

- **Status:** proposed — 2026-06-26
- **Plane:** env-control / migration-adoption
- **Scope:** `envctl migrate`, `envctl env`, manifest path resolution, `.handoff` task tracking, CI gates, and future GUI/runtime migration UX.
- **Relates:** `.handoff/tasks/TASK-0078.task.json`; `.handoff/loop/plan/reports/TASK-0078-migration-adoption-engine-v2-design.md`; `docs/MIGRATION-ADOPTION.md`; `crates/engine/src/{migration,layout}.rs`; `crates/cli/src/main.rs`; `manifest/**/*.toml`.
- **Refines:** PR #252 migration/adoption skeleton. This ADR keeps the skeleton's safe dry-run foundation and specifies the missing end-to-end adoption, parity, activation, quarantine, and purge model.

## Context

`envctl` is the meta workspace environment manager. Its purpose is not to make a separate repo-shaped installation island; it must make every meta-hosted install resolve through a system-shaped meta root: `.local/bin`, `.local/lib`, `.local/share`, `.local/state`, `.local/cache`, `.local/tmp`, component-owned toolchain roots, and compatibility shims only when required. Manual path edits are the failure mode this project exists to eliminate.

The current migration/adoption engine has the right direction, but it is not sufficient for broad migration. The current scan evidence from the migration skeleton showed the scale of the problem: hundreds of discovered paths still need migration, with debt across user-global locations, legacy manifest paths, and system-global locations. The existing engine can audit and bootstrap safely, but it does not yet provide a typed ownership registry, parity proof, activation gate, quarantine flow, or no-new-debt ratchet strong enough to purge old paths without risking Codex/agent configuration loss or toolchain downtime.

The owner requirement is strict:

1. **Upgrade only. Never downgrade.** Existing working tools stay working until a meta-owned replacement is installed, configured, and parity-proven.
2. **No manual path surgery.** Envctl must define and activate the correct paths automatically.
3. **Meta root is the environment boundary.** User-global paths should become shims or launchers into meta, not canonical install roots.
4. **Preserve critical continuity/config.** Codex/agent configs, `.handoff`, `.kb`, `loop_lib`, and peer-repo expectations must be adopted or wrapped, not deleted or rewritten blindly.

## Decision

We will upgrade migration/adoption into a parity-proven **meta layout engine v2** with this lifecycle:

```text
scan -> classify -> plan -> adopt -> verify -> activate -> quarantine -> verify-again -> purge
```

Broad migration or purge is **not ready** until the v2 lifecycle is implemented and green on representative components. The only safe current behavior is audit/bootstrap and explicit, component-scoped adoption.

### 1. Canonical layout registry

Envctl owns a canonical layout rooted at `$META_ROOT`, with system-shaped directories under `$META_ROOT/.local` and component roots where needed:

- `$META_ROOT/.local/bin`
- `$META_ROOT/.local/lib`
- `$META_ROOT/.local/share`
- `$META_ROOT/.local/state`
- `$META_ROOT/.local/cache`
- `$META_ROOT/.local/tmp`
- `$META_ROOT/.toolchains/<component>` for component-owned payloads that need internal structure
- user-global locations only as compatibility shims/launchers into meta

The registry must answer: "for this component and artifact kind, what is the canonical path, what legacy paths are recognized, and which activation hooks put it on PATH or in config?"

### 2. Typed component ownership

Every migration candidate must map to a typed owner before mutation:

- `component_id`
- `manifest_path`
- `artifact_kind` (`binary`, `library`, `model`, `config`, `cache`, `state`, `service`, `shim`, `toolchain_root`, `agent_asset`)
- `source_kind` (`envctl_manifest`, `agent_env`, `handoff`, `meta_peer_repo`, `host_prereq`, `legacy_user_global`, `legacy_system_global`, `unknown`)
- `legacy_path`
- `canonical_path`
- `risk` (`low`, `medium`, `high`, `protected`)
- `adoption_method`
- `verifier`
- `purge_policy`

Unknown or protected owners are read-only findings. They cannot be moved, rewritten, or purged.

### 3. Adoption methods

The executor supports explicit adoption methods only:

- `copy_preserve_mode`
- `symlink_to_meta`
- `hardlink_when_same_device_and_safe`
- `rebuild_into_meta`
- `rewrite_manifest_reference`
- `preserve_only`
- `host_prereq_report_only`
- `agent_asset_sync`
- `handoff_export_import`

No generic `rm -rf`, no broad directory moves, and no shell-generated rewrite outside the engine's typed plan.

### 4. Evidence ledger v2

Each adoption produces an append-only evidence record with enough data to prove parity and rollback:

- component id and artifact kind
- legacy path and canonical path
- before/after checksum where meaningful
- before/after version or identity probe
- before/after PATH resolution
- verifier commands and exit status
- activation changes
- quarantine path if any
- rollback command/plan
- purge eligibility and timestamp

The ledger must be exportable in text/JSON so `.handoff` and PR review can prove what happened without relying on local caches.

### 5. Parity verification before activation

A migration is not complete when bytes move. It is complete only when the canonical path resolves first and the component verifier proves the same or upgraded behavior. Examples:

- binaries: `which`, `--version`, smoke command, path precedence
- libraries: cargo/ld/pkg-config resolution if relevant
- configs: consumer reads from meta path without hand-editing user-global files
- models/caches: owner process can read from meta path
- services: unit/env points at meta path and health check passes
- agent assets: generated lock/check proves no drift
- `.handoff`: `hf doctor`, `hf gitignore --check`, and `ci/gates/p7.sh` pass

### 6. Activation gate

Envctl activation (`envctl env`, dashboard launchers, shell snippets, and generated agent environments) must prepend meta-owned paths and record the resolution proof. This is where the project stops being "a manifest repo" and becomes the path-defining environment manager.

### 7. Quarantine before purge

Legacy paths are moved to a quarantine root only after canonical parity passes. Purge is a later, separate operation and must re-run verification immediately before deletion. Protected paths are never purged by the migration engine.

### 8. CI ratchet

Add a no-new-debt migration gate that fails if manifests or env surfaces introduce new hardcoded user-global/system-global install paths without a typed adoption rule. The ratchet should allow existing debt through a baseline until the component is migrated, then lock the improvement.

## Non-goals

- Do not remove or downgrade `loop_lib`.
- Do not wholesale delete `.toolchains` or user-global directories.
- Do not rebuild Codex/agent configs from scratch when they can be adopted or synced.
- Do not migrate host prerequisites that must remain host-owned; report them as prerequisites.
- Do not replace the existing safe audit/bootstrap skeleton with a less strict tool.

## Consequences

- Migration becomes slower per component but safe enough to purge later.
- The engine must carry more structured metadata and tests.
- The first implementation should be component-scoped, not a one-shot full-machine migration.
- Envctl becomes the source of truth for PATH, registry, activation, and adoption evidence inside meta.

## Implementation phasing

1. Planner v2: typed ownership, canonical layout registry, risk classification, JSON plan.
2. Evidence ledger v2: append/export/readback model with tests.
3. Safe adoption executor: dry-run default, component-scoped apply, rollback/quarantine data.
4. Manifest rewrite support: move manifest path literals to layout registry references.
5. Activation/PATH gate: prove canonical path precedence in `envctl env` and dashboard shells.
6. CI ratchet: no-new-debt path gate.
7. Quarantine/purge: delayed purge with re-verification and protected-path refusal.

## Acceptance reference

This ADR is implemented by TASK-0078. The design/spec artifact is `.handoff/loop/plan/reports/TASK-0078-migration-adoption-engine-v2-design.md`.
