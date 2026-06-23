# TASK-0016 — agent lock boundary · VERDICT: GO

## Trigger Check

The card says "fold agent assets into envctl.lock (SHA-256 section)", but the current rust-port
handoff and research report have the later no-downgrade finding: the engine component lock and the
agent-asset lock are different domains and should remain separate.

## Decision

Route TASK-0016 as a lock-boundary decision plus manifest/gate cleanup:

- Keep `manifest/envctl.lock` as the FNV-1a component lock for manifest components.
- Keep `agent-env.lock` as the SHA-256 agent-asset lock for skills, commands, and MCP assets.
- Reframe `manifest/agent-env.toml` around the built-in `envctl agent` commands and the renamed
  `agent-env.yaml` / `agent-env.lock` files.
- Make the CI drift gate use the true zero-network command:
  `envctl agent lock --config agent-env.yaml --check --locked`.

## Target Repos

Single repo: envctl. Sequential single-crew path.

Touched surfaces:

- `crates/agent-env/src/lock.rs`
- `crates/engine/src/agent/lock.rs`
- `manifest/agent-env.toml`
- `manifest/envctl.lock`
- `ci/gates/agent-env.sh`
- `.handoff/decisions/*`

## Non-Goals

- Do not re-port kasetto.
- Do not rename the component id `kasetto` in this cycle; that would create needless component-lock
  churn and is orthogonal to the agent-asset lock boundary.
- Do not embed the SHA-256 lock into `manifest/envctl.lock`.
