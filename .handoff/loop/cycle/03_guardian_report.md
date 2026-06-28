# TASK-0078 guardian report — cache-child manifest id validation

Date: 2026-06-28

## Verification commands

```bash
bash scripts/tests/test-meta-local-path-audit.sh   # red first, then PASS after implementation
bash -n scripts/audit-meta-local-paths.sh scripts/tests/test-meta-local-path-audit.sh
bash scripts/tests/test-meta-local-path-audit.sh
git diff --check
bash ci/gates/meta-local-policy.sh
bash ci/gates/harness-scripts.sh
bash ci/gates/p7.sh
```

All final commands passed.

## Live runtime verification

Non-mutating live audit wrote:
`/tmp/envctl-cache-manifest-validation-20260628T003958Z.Hg6Q1b`

Observed:

- exit code: 0
- `meta-local audit: PASS warnings=10 changed=0 dot_entries=79`
- `--migrate-cache-child .wasm-pack` dry-ran a refusal because `manifest/components.d/cache-wasm-pack.toml` is missing
- cache-child manifest-status emitted 84 current cache-child rows
- all 84 current cache-child manifest hints reported `manifest_exists=no`
- no live cache-child apply was performed

## Result

PASS. The slice strengthens the PR #368 manifest gate from "file exists" to "matching reviewed component id exists" while preserving all non-mutating/default-fail-closed behavior.
