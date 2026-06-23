# TASK-0021 — guardian report

## Verdict: PASS

TASK-0021 is already satisfied in the current source tree. The stale backlog card was the thing that
needed reconciliation.

## Findings

- PASS: `manifest/base.toml` has a standalone `node-real` component for a real Node in the n8n range.
- PASS: `group-ai-clis` does not require `node-via-bun`.
- PASS: `n8n-mcp` still requires `node-via-bun`, which is correct because it is Bun-provided node
  compat, not the real V8 runtime carve-out.
- PASS: `envctl lock --check` passes, so the generated lock matches the manifest.
- PASS: The focused engine tests prove both the truthy `node-real` component and the absence of the
  old `group-ai-clis -> node-via-bun` edge.

## Gate Results

All local gates passed:

- targeted engine tests for `group_ai_clis_does_not_require_node_via_bun` and
  `node_real_component_exists_with_empty_requires`
- engine+CLI build
- p7
- `hf test TASK-0021`

No code change was needed for this closeout.
