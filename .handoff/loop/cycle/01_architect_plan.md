# TASK-0078 reviewed starship cache-child manifest plan

## Target
Commit the next reviewed cache-child component manifest for live `starship` so the existing narrow `--migrate-cache-child starship` path can pass its reviewed-manifest precondition in dry-run mode.

## Candidate selection
- `zellij` initially passed the missing-manifest red probe, but live dry-run with a manifest present refused because `/home/drdave/Desktop/meta/.local/cache/zellij` already exists.
- Switch to `starship`, whose live source exists and meta target is absent, so the reviewed-manifest slice can prove would-move/would-link without touching state.

## Contract
- Add `manifest/components.d/cache-starship.toml` declaring `[[component]] id = "cache-starship"`.
- Do not move cache state and do not run `--migrate-cache-child --apply`.
- Preserve the reviewed manifest gate: absent/wrong manifests still refuse; matching manifests only allow the existing dry-run/apply path to continue to normal safety checks.
- Add a regression fixture proving a repo-reviewed `starship` manifest changes the dry-run from missing-manifest refusal to would-move/would-link without touching source or target.

## Runtime surface
`runtime_verifiable? yes` — run the live audit in dry-run mode and prove `/home/drdave/.cache/starship` remains in place while `/home/drdave/Desktop/meta/.local/cache/starship` is absent.

## Verification plan
- Red: before the manifest, live dry-run refuses because `manifest/components.d/cache-starship.toml` is missing; zellij target-collision is avoided rather than committed.
- Green: add only the starship manifest and the fixture; run audit tests and harness gates.
- Runtime: live dry-run emits would-move/would-link plus validation `manifest_exists=yes` / `manifest_declares_expected_id=yes` / empty `apply_command`; no `--apply` is run.
