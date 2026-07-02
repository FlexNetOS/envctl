# handoff — code-graph diff

**No prior snapshot — baseline established this run.**

This is the first `handoff` code-graph snapshot (cycle 2). The cycle-1 graph artifacts under `graph/` are for `rusty-idd`, a different target; there is no `graph/handoff.*` predecessor to diff against. The next `handoff` cycle will diff against this baseline.

## Baseline counts (snapshot @ f6abf962413bafe164d56fa26b70b0a5fdacb8a2)

| Metric | Value |
|---|---|
| symbol_count (total, incl. vendor) | 2974 |
| own symbol_count (excl. vendor/syntect) | 2128 |
| vendored symbols (crates/tui/vendor/syntect) | 846 |
| file_count | 141 |
| resolved call edges (doctor) | 7265 |
| resolved edges used for metrics (in→out, sid-matched) | 4945 |
| Cargo workspace members | 21 (16 kernel + 5 rusty-idd-* toolkit) |
| Tarjan SCC cycles (size ≥ 2) | 14 (≈9 vendor, 5 own — all same-name collisions) |
| genuine architectural cycles | 0 |
| crate-level Cargo dep cycles | 0 (strict DAG) |
| dead-code candidates (own, untriaged) | 1258 |
| binary entrypoints | 3 (hf, hf-mcp, rusty-idd-cli) |
| public-API symbols (git-kb query) | 50 |

## Top kernel hubs (baseline)

| symbol | in-degree | blast (transitive callers) |
|---|---|---|
| ledger/src/v1.rs::Ledger.open | 74 | 120 |
| hf/src/bin/hf-mcp.rs::McpServer.new | 109 | — |
| handoff-core/src/lib.rs::ledger_path | 27 | 54 |
| handoff-schema/src/lib.rs::validate_card | 5 | 40 |
| work-order/src/lib.rs::compute_intent_lock | 18 | — |

## Snapshot integrity note (for the next cycle)

The live git-kb store is `.kb/.cache/gitkb.db` (branch `feat/hftask-0072-full-kb-adoption`). The legacy `.git/gitkb/code.db` holds a **stale pre-peel `develop` snapshot** (3412 symbols, missing all `handoff-*` crates) and MUST be ignored — re-index `--force` and re-confirm `git-kb code doctor --json` shows the `handoff-*` crates in `file_breakdown` before deriving any metric.
