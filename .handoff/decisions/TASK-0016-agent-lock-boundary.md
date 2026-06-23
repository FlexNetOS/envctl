# TASK-0016 — agent lock boundary

- **Status:** accepted — 2026-06-22
- **Scope:** envctl agent-env lock placement

## Decision

Keep two committed locks with separate domains:

- `manifest/envctl.lock` remains the component lock for declarative envctl components, using the
  existing FNV-1a component content hash.
- `agent-env.lock` remains the agent-asset lock for skills, commands, and MCP assets, using the
  kasetto-compatible SHA-256 asset hash.

Do not embed the agent-asset lock into `manifest/envctl.lock` in this cycle.

## Rationale

The two locks are not duplicates. They track different resources, use different schemas and hash
algorithms, and serve different no-downgrade contracts. Folding the SHA-256 agent assets into the
FNV-1a component lock would blur the trust boundary without adding reproducibility.

The manifest component still participates in the normal component lock as `kasetto` so the agent
environment remains visible to `doctor`/`auto-detect`/`envctl lock`; the agent assets themselves are
drift-gated by `envctl agent lock --config agent-env.yaml --check --locked`.
