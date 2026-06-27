# TASK-0078 implementer log — blocker sensitive hints

Date: 2026-06-27

## Changes made

- Extended `--migration-blockers-report` with a `sensitive_hints` column.
- Reused the existing `path_sensitive_hint_count` scanner for every residual blocker row.
- Updated TDD coverage to lock the 13-column schema and fixture hint counts:
  - `.pki` = 3 (`cert9.db`, `key4.db`, `pkcs11.txt`)
  - `.mcp-auth` = 1 (`oauth_tokens.json`)
  - `.lane` = 3 (`*.pem`, `*.key`)
  - `.fxapp-gh-profile` = 0 in the fixture
- Strengthened `ci/gates/meta-local-policy.sh` so this schema/test coverage cannot silently regress.

## Notes

- This is report-only/read-only. No live real-home dot entry was moved.
- The live audit now makes `.pki` show `sensitive_hints=3` while still failing closed on Chrome open handles.
