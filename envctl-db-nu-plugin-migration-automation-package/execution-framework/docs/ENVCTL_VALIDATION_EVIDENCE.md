# envctl validation evidence store

Generated at: `2026-07-04T23:19:04+00:00`
Status: `passed`

## Persisted evidence

| evidence class | covered |
|---|---|
| reconciliation | yes |
| parity | yes |
| test results | yes |
| proof evidence | yes |
| scorecard view | yes |
| fail closed rejections | yes |

## Runtime smoke

- Run: `run-req025`
- Validation rows: `3`
- Evidence rows: `4`
- Hashed evidence rows: `4`
- Scorecard: `{'pass': 2, 'fail': 0, 'warn': 1, 'blocked': 0, 'unknown': 0}`
- Evidence kinds: `{'parity': 1, 'proof_record': 1, 'reconciliation': 1, 'test_result': 1}`
- Rejection cases: `3`

The smoke records reconciliation, parity, test-result, and proof evidence into the REQ-020 SQLite schema, persists validation rows linked to operations and artifacts, computes SHA-256 hashes for local evidence, reads the validation scorecard view, and confirms unsafe records fail closed.
