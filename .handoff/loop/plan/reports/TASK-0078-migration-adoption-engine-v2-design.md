# TASK-0078 design/spec — migration/adoption engine v2

- **Task:** TASK-0078
- **ADR:** `.handoff/decisions/ADR-0002-migration-adoption-engine-v2.md`
- **Date:** 2026-06-26
- **Status:** planned / not yet implemented
- **Build-hf note:** the `hf` kernel was rebuilt from `meta/handoff` before this task was authored; this document keeps the implementation tracked in `.handoff` rather than as an untracked chat plan.

## 1. Problem statement

The migration/adoption skeleton gives envctl a safe audit foothold, but the repo is not ready for broad migration or purge. Envctl must become the meta path-defining registry and installer: anything meta consumes should resolve through `$META_ROOT` and system-shaped directories such as `.local/bin`, `.local/lib`, `.local/share`, `.local/state`, `.local/cache`, and `.local/tmp`.

The hard part is not copying files. The hard part is preserving working configs, agent state, peer-repo expectations, and toolchains while moving canonical ownership into meta. The migration engine therefore needs a typed plan, a strict parity proof, and a delayed purge flow.

## 2. Readiness conclusion

**Not ready for broad migration/purge.** Ready only for audit/bootstrap and component-scoped adoption after the v2 plan/evidence/verification gates exist.

A full migration run must not be exposed as a one-shot destructive command until all of these are true:

1. Every candidate has a typed owner or is read-only `unknown`/`protected`.
2. Every mutating plan has a rollback/quarantine path.
3. Canonical path activation is verified before legacy purge.
4. Protected continuity surfaces (`loop_lib`, `.handoff`, `.kb`, agent/Codex configs) have explicit preserve/adopt policies.
5. CI prevents new unmanaged path debt.

## 3. End-to-end workflow

### A. Scan

Command:

```bash
envctl migrate scan --json
```

Responsibilities:

- Discover manifest install paths, binaries on PATH, toolchain roots, agent assets, config roots, cache/state roots, and known meta peer repo hooks.
- Emit candidates without mutation.
- Mark unknowns as findings, not work items.

### B. Classify

Engine maps each candidate to a `MigrationOwner`:

```rust
pub struct MigrationOwner {
    pub component_id: ComponentId,
    pub manifest_path: Option<PathBuf>,
    pub artifact_kind: ArtifactKind,
    pub source_kind: SourceKind,
    pub legacy_path: PathBuf,
    pub canonical_path: PathBuf,
    pub risk: MigrationRisk,
    pub adoption_method: AdoptionMethod,
    pub verifier: VerifierSpec,
    pub purge_policy: PurgePolicy,
}
```

Unknown/protected classifications are non-mutating.

### C. Plan

Command:

```bash
envctl migrate plan --json [--component <id>] [--baseline <file>]
```

Responsibilities:

- Produce a stable, reviewable plan.
- Group candidates by owner/component.
- Include before/after path resolution expectations.
- Include exact verifier commands.
- Mark whether apply is allowed.

### D. Adopt

Command:

```bash
envctl migrate apply [--component <id>] [--group <name>] [--apply]
```

Responsibilities:

- Dry-run by default.
- Mutate only when `--apply` is present.
- Execute only typed adoption methods.
- Write evidence records as each step completes.
- Never downgrade versions; if canonical is older than legacy, fail and require an upgrade plan.

### E. Verify

Command:

```bash
envctl migrate verify [--component <id>] [--strict-path]
```

Responsibilities:

- Confirm canonical path wins resolution.
- Run component verifier.
- Compare version/identity before and after.
- Confirm agent/handoff gates where relevant.

### F. Activate

Command surfaces:

```bash
envctl env
envctl dashboard
# future GUI/runtime surfaces call the same Engine API
```

Responsibilities:

- Export PATH/env with meta roots first.
- Avoid manual per-user path edits.
- Generate shell/dashboard surfaces from the registry.
- Record resolution proof.

### G. Quarantine and purge

Commands:

```bash
envctl migrate quarantine --component <id> --apply
envctl migrate purge --component <id> --confirm --apply
```

Rules:

- Quarantine only after verify passes.
- Purge only after a second verify pass.
- Protected paths refuse purge forever.
- Unknown paths refuse mutation.
- Purge evidence must be exported.

## 4. Data model

### ArtifactKind

- `Binary`
- `Library`
- `ShareData`
- `State`
- `Cache`
- `Tmp`
- `Model`
- `Config`
- `Service`
- `Shim`
- `ToolchainRoot`
- `AgentAsset`
- `HandoffLedgerExport`

### SourceKind

- `EnvctlManifest`
- `AgentEnv`
- `Handoff`
- `MetaPeerRepo`
- `HostPrereq`
- `LegacyUserGlobal`
- `LegacySystemGlobal`
- `Unknown`

### MigrationRisk

- `Low`
- `Medium`
- `High`
- `Protected`

### AdoptionMethod

- `CopyPreserveMode`
- `SymlinkToMeta`
- `HardlinkWhenSameDeviceAndSafe`
- `RebuildIntoMeta`
- `RewriteManifestReference`
- `PreserveOnly`
- `HostPrereqReportOnly`
- `AgentAssetSync`
- `HandoffExportImport`

### EvidenceRecordV2

Fields:

- `schema = "envctl.migration.evidence.v2"`
- `component_id`
- `artifact_kind`
- `legacy_path`
- `canonical_path`
- `legacy_checksum`
- `canonical_checksum`
- `version_before`
- `version_after`
- `path_resolution_before`
- `path_resolution_after`
- `adoption_method`
- `activation_changes`
- `verifier_commands`
- `verifier_results`
- `quarantine_path`
- `rollback_plan`
- `purge_allowed`
- `created_at`

## 5. Protection policy

Protected or preserve-only unless a future ADR narrows the rule:

- `loop_lib` and meta peer repo loop surfaces
- `.handoff` ledger export and task cards
- `.kb` knowledge surfaces
- Codex/agent config that is generated by agent-env
- secrets material and vault state
- host prerequisites that cannot live under meta

## 6. CLI contract

Add/extend CLI tests for these contracts:

1. `scan --json` contains typed candidates and protected findings.
2. `plan --json` refuses mutation for unknown/protected candidates.
3. `apply` without `--apply` is dry-run and writes no filesystem changes.
4. `apply --apply` records evidence before activation.
5. `verify --strict-path` fails if legacy path resolves before meta path.
6. `purge` refuses without successful evidence and second verify.
7. `purge` refuses protected candidates even with `--apply`.
8. `envctl env` includes canonical meta `.local/bin` and component roots in deterministic order.

## 7. CI gate

Add:

```bash
bash ci/gates/migration-debt.sh
```

Gate rules:

- Parse manifests and env-generation surfaces.
- Fail on new hardcoded `/usr/local`, `$HOME/.local`, `~/.local`, or legacy `.toolchain` install targets unless covered by a typed adoption rule or host-prereq exception.
- Compare against a checked-in baseline so current debt does not block unrelated work, but improvements are ratcheted.

## 8. Implementation plan

1. Introduce layout registry module and canonical path helpers.
2. Extend migration scan model to typed owners.
3. Add plan v2 JSON fixture tests.
4. Implement evidence ledger v2 append/readback/export.
5. Implement component-scoped adoption executor, dry-run default.
6. Wire activation proof into `envctl env` and dashboard launchers.
7. Add quarantine/purge commands with re-verification.
8. Add migration debt CI gate and baseline.
9. Document operator workflow in `docs/MIGRATION-ADOPTION.md`.
10. Run one low-risk component migration as proof before any broad migration.

## 9. Acceptance criteria

- ADR, design, task, and backlog entry are tracked under `.handoff`.
- `hf doctor --json`, `hf gitignore --check`, and `ci/gates/p7.sh` pass.
- Planner v2 emits typed ownership and canonical layout for scan findings.
- Evidence ledger v2 records before/after identity, path resolution, verifier results, and rollback/quarantine data.
- Apply is dry-run by default and component-scoped when mutating.
- Purge is refused until canonical parity and a second verify pass succeed.
- `loop_lib`, `.handoff`, `.kb`, Codex/agent assets, and secrets surfaces are protected by default.
- CI has a no-new-debt path ratchet.

## 10. Verification commands for the implementation PR

```bash
hf doctor --json
hf gitignore --check
bash ci/gates/p7.sh
cargo test -p envctl-engine migration
cargo test -p envctl --test cli_contract migration
bash ci/gates/migration-debt.sh
cargo fmt --all --check
cargo clippy --workspace -- -D warnings
```
