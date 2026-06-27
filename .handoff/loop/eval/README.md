# Loop Eval — durable, cross-cycle evaluation assets

This folder is the planning loop's **stable evaluation store**. Unlike `.handoff/loop/plan/`
(per-cycle, branch-scoped working state that the artifact gate validates), `eval/` holds
**cross-cycle, referenceable** reports — model-lane comparisons, quality A/Bs, cost-parity studies,
retrospectives — that outlive any single cycle and should be cited by future cycles and by the
`evolution-steward`.

Conventions:
- One file per distinct study; stable, descriptive kebab-case names (no cycle number in the name when
  the asset spans cycles).
- Each report states its scope, method, evidence, and a dated verdict at the top.
- Append, don't overwrite — supersede with a new dated section or a `-v2` file and link back.
- Reaches `master` via the normal envctl plan-PR flow; do not commit binary ledgers here.

## Index
| asset | scope | added |
|---|---|---|
| [model-lane-comparison-codex-vs-opus.md](model-lane-comparison-codex-vs-opus.md) | Codex sub-agents vs Opus sub-agents running the same planning crew (cycle 7 icm / Opus vs cycle 8 harness-hub / Codex): mechanism, measured cost, good/bad, recommendations, hybrid verdict | 2026-06-27 |
