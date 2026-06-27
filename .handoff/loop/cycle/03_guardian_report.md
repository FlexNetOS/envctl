# TASK-0078 guardian report — blocker sensitive hints

Date: 2026-06-27

## Verification commands

```bash
bash scripts/tests/test-meta-local-path-audit.sh
bash ci/gates/meta-local-policy.sh
bash ci/gates/harness-scripts.sh
```

All commands passed.

## Live runtime verification

Read-only live audit wrote:
`/tmp/envctl-dot-audit-sensitive-hints-final-20260627T184649Z`

Observed:

- exit code: 0
- `meta-local audit: PASS warnings=10 changed=0 dot_entries=79`
- blocker report header includes `sensitive_hints`
- selected blocker evidence:
  - `.pki`: `target_class=app-config-state`, `apply_safe=yes`, `canonical_target=/home/drdave/Desktop/meta/.local/share/pki`, `sensitive_hints=3`, `blocker=open-handles`, `open_handles=2`, sample `chrome/1653768`
  - `.lane`: `sensitive_hints=7`
  - `.fxapp-gh-profile`: `sensitive_hints=5`
  - `.ssh`: `sensitive_hints=1`

## Result

PASS. The slice improves surgical blocker visibility without weakening the default non-mutating policy.
