# TASK-0078 reviewed agent-env cache-child manifest plan

## Target
Commit the next reviewed cache-child component manifest for live `agent-env` so the existing narrow `--migrate-cache-child agent-env` path can pass its reviewed-manifest precondition in dry-run mode.

## Candidate selection
- The live validation report shows `/home/drdave/.cache/agent-env` is a direct `.cache` child with `manifest_exists=no`.
- `/home/drdave/.cache/agent-env` exists as a directory and `/home/drdave/Desktop/meta/.local/cache/agent-env` is absent, so this slice can prove would-move/would-link without touching state.
- Avoid broad `.cache` movement and avoid large/tool-active cache apply; this PR only adds the reviewed manifest stub and regression coverage.

## Contract
- Add `manifest/components.d/cache-agent-env.toml` declaring `[[component]] id = "cache-agent-env"`.
- Do not move cache state and do not run `--migrate-cache-child --apply`.
- Preserve the reviewed manifest gate: absent/wrong manifests still refuse; matching manifests only allow the existing dry-run/apply path to continue to normal safety checks.
- Add a regression fixture proving a repo-reviewed `agent-env` manifest changes the dry-run from missing-manifest refusal to would-move/would-link without touching source or target.

## Runtime surface
`runtime_verifiable? yes` — run the live audit in dry-run mode and prove `/home/drdave/.cache/agent-env` remains in place while `/home/drdave/Desktop/meta/.local/cache/agent-env` is absent.

## Verification plan
- Red: before the manifest, live dry-run refuses because `manifest/components.d/cache-agent-env.toml` is missing.
- Green: add only the agent-env manifest and the fixture; run audit tests and harness gates.
- Runtime: live dry-run emits would-move/would-link plus validation `manifest_exists=yes` / `manifest_declares_expected_id=yes` / empty `apply_command`; no `--apply` is run.
