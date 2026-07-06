# envctl artifact registry

Generated at: `2026-07-04T23:14:10+00:00`
Status: `passed`

## Persisted fields

| field | persisted |
|---|---|
| paths | yes |
| hashes | yes |
| producers | yes |
| contract ids | yes |
| provenance | yes |
| validation links | yes |
| fail closed rejections | yes |

## Runtime smoke

- Artifact row: `artifact-dfdcd235057309cce430ae2a`
- Content hash: `sha256:d83a779c4f5ba4274c455c016193ad62f58d974033e53f9b11a78369cb94cded`
- Evidence rows: `3`
- Graph edges: `4`
- Validation rows: `2`
- Rejection cases: `3`

The smoke registers a concrete artifact against the SQLite schema from `REQ-020`, computes a SHA-256 hash from disk, records the producer operation, stores the artifact contract id and provenance in registry JSON, links evidence rows, graph edges, validation rows, verifies the artifact-index view, and confirms unsafe records fail closed.
