# envctl Code Graph Diff

Date: 2026-07-02

No prior `envctl` autoresearch graph snapshot was present in
`.handoff/loop/plan/graph/`, so this cycle establishes the baseline rather than a
numeric before/after delta.

## Baseline Drift Watch

- Branch-specific graph proof is currently blocked by the `kb_root` mismatch:
  the graph reports `/home/flexnetos/FlexNetOS/src/envctl` while the active
  worktree root is `/home/flexnetos/FlexNetOS/src/envctl-plan-autoresearch-20260702`.
- The next refresh should record `git rev-parse --show-toplevel`, branch, HEAD,
  `git-kb code stats --json.kb_root`, symbol count, file count, edge count,
  unresolved count, and service-edge counts in one machine-checkable row.
- Future cycles should diff at least: symbol count, file count, unresolved ratio,
  top 20 hotspots, top 20 dead-code candidates, public entrypoints, and route/client
  fact counts.
