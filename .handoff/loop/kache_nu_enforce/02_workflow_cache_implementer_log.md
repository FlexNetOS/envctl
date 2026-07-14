# Workflow-cache implementer log

## Outcome

- Removed every remote/non-Kache cache directive from the retained workflow YAML.
- Disabled every legacy non-Nushell workflow by moving it outside the active workflow directory.
- Added `.github/workflows/automation_policy.yml` as the sole active workflow.
- Added `ci/gates/automation_policy.nu`; it rejects non-Kache cache actions/inputs/wrappers and any unported active workflow.
- No Rust symbol was edited. Nothing was pushed or activated on the host.

## Exact workflow moves

- `.github/workflows/ci.yml` → `.github/workflows_disabled/ci.yml`
- `.github/workflows/sync-master.yml` → `.github/workflows_disabled/sync-master.yml`

Additional files:

- `.github/workflows_disabled/README.md`
- `.github/workflows/automation_policy.yml`
- `ci/gates/automation_policy.nu`

## Verification

- Policy gate: PASS.
- Negative probe containing a cache input: correctly rejected with exit 1.
- Active workflow actionlint: PASS.
- Disabled YAML syntax: PASS (2 files).
- Banned directive scan under `.github`: zero matches.
- `git diff --check`: PASS.

## Blocker

The disabled workflows must stay inactive until each automatic command is ported to native Nushell. The policy gate intentionally refuses reactivation before that migration.

