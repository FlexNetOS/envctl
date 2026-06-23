# TASK-0021 — node-via-bun follow-up verification · VERDICT: GO

## Trigger Check

TASK-0021 asks for a manifest truth-telling follow-up: either mark node not-applicable when a real
Node is present, or add a `node-real` component and remove the bogus `group-ai-clis -> node-via-bun`
edge. That truth has already landed in the source tree.

Source truth:

- `manifest/base.toml` contains a standalone `node-real` component.
- `manifest/base.toml` keeps `node-via-bun` as the Bun shim and does not wire it into `group-ai-clis`.
- `manifest/ai-clis.toml` defines `group-ai-clis` only over the actual CLI components.
- `crates/engine/tests/engine.rs` has focused assertions that `group-ai-clis` does not require
  `node-via-bun` and that `node-real` exists with empty `requires`.
- `manifest/envctl.lock` matches the manifest and `envctl lock --check` passes.

## Design

No code changes are needed. Close the stale card with evidence and keep the loop moving.

## Target Repos

Single repo: envctl. Sequential single-crew path.

Touched surfaces:

- `.handoff/loop/backlog.md`
- `.handoff/loop/loop_state.md`
- `.handoff/loop/cycle/*`

## Non-Goals

- Do not add a redundant manifest component if the existing `node-real` carve-out already satisfies
  the task.
- Do not touch the bun install path or the `n8n-mcp` component, which legitimately depends on
  `node-via-bun`.
- Do not introduce dependency drift just to force a manifest rewrite.
