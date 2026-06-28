# icm — Code Graph Diff

**Baseline — no prior icm snapshot.** This is the first code-graph snapshot for
the `icm` target (cycle 7). There is no previous committed `graph/icm.symbols.json`
to diff against, so this file records the baseline surface; subsequent cycles will
diff new/removed symbols, edge churn, and metric movement against it.

## Baseline snapshot facts

| metric | value |
|---|---|
| branch / sha | `chore/rusty-idd-fleet-adapter` / `5fde8fc` |
| symbols | 1629 |
| symbol-bearing files | 56 (index touched 119 incl. non-symbol) |
| resolved call edges | 3069 (all intra-repo, `name_match`) |
| unresolved calls | 12771 (no_match 6309 · skip_list 4890 · ambiguous 1184 · stdlib 388) |
| crates | icm-cli 862 · icm-store 323 · icm-core 224 · icm-mcp 148 (+scripts 68, plugins 4) |
| public-API symbols | 225 |
| cyclic SCCs | 7 (all intra-crate, size 2–3) · self-loops 0 |
| layering violations | 0 |
| dead-code (test-excluded) | 99 (raw caller_count==0: 326, inflated) |
| MCP tools | 31 (`icm_*`) |
| web routes (axum) | 15 |
| binaries | 4 (icm + 3 bench scripts) |
| traced flows | 0 |

## Surface summary (for next-cycle diffing)

- **Gravity wells:** `icm-store::SqliteStore` (`store.rs`, 220 KB, hotspots
  `get`=183, `store`=143, `new`=60, blast 142–281) and the MCP dispatch/protocol
  (`call_tool`=89, `protocol::error`=108).
- **Memory plane data path:** Memory → SqliteStore → SQLite(rusqlite bundled) +
  sqlite-vec `vec0(float[384] cosine)` + FTS5; embeddings via icm-core
  FastEmbedder (fastembed, optional `embeddings` feature, e5-base/384d).
- **Clean architecture:** dependency edges flow downward only
  (cli→mcp→store→core); zero layering violations; cycles are local ctor/cache
  clusters only.

> Next cycle: re-run `git-kb code index --force`, regenerate `icm.symbols.json` /
> `icm.callgraph.json` / `icm.metrics.json`, and populate this file with the delta
> (Δ symbols, Δ edges, Δ hotspots, Δ cycle/violation counts) vs this baseline.
