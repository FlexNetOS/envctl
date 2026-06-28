# weave — code graph + metrics (ASCII, diagram-ready)

- Target: **weave** (the fleet's A2A transport plane / "nervous system")
- Snapshot: `graph/weave.symbols.json@4fe2419` · branch `plan/weave-red-tests`
- Source: `git-kb code` (index `.`) — 2722 symbols, 9571 resolved call edges, 36 deep-indexed source files
- Derived edge list: 1717 production nodes / 4204 edges (callees keyed `file:def_line`, test call-sites excluded)
- Generated this cycle (cycle 4, parallel weave instance)

## 1. Crate layering (compiler-enforced, strictly downward)

```
                 ┌───────────────────────────────────────────────┐
   bin / CLI     │  weave            (src/main.rs, 71 CLI verbs)  │  blast→427
   (71 verbs)    │   serve · dashboard · daemon · telegram·slack  │
                 └───────┬───────────────┬───────────────┬────────┘
                         │ depends-on     │               │
                 ┌───────▼───────┐        │               │
   MCP plane     │  weave-mcp    │        │               │  blast(mcp.rs)→124
   (78 tools)    │  mcp·http·    │        │               │  blast(http.rs)→27
                 │  dashboard·   │        │               │
                 │  obscura      │        │               │
                 └───┬───────┬───┘        │               │
                     │       │ depends-on │               │
              ┌──────▼──┐    │     ┌───────▼───────────────▼─────┐
   injector   │ weave-  │    └────▶│  weave-core                 │ blast(model)→1238
   (7 muxes)  │ inject  │─────────▶│  store(SqliteStore broker)· │ blast(store)→462
              │ inject  │ depends  │  store_libsql·model(Intent)·│ blast(store_libsql)→488
              │ ·spawn  │   -on    │  config·sign·webpolicy·     │ blast(config)→345
              └─────────┘          │  memory·archive·export·llm  │
   blast(inject)→205               │  ·session·testenv           │
                                   └─────────────────────────────┘
   Manifest DAG:  weave-core (0 internal deps)
                    ▲          ▲              ▲
                weave-inject  weave-mcp     weave
   No upward Cargo dep exists → layering is CLEAN at compile time.
```

The SQLite mailbox (`SqliteStore` in `weave-core/src/store.rs`, the broker) is the hub: every
plane (CLI, MCP, injector) funnels through `weave-core`. `weave-core` itself has **zero** internal
weave dependencies — the spine all other crates point down to.

## 2. Entry points

| kind | symbol | location |
|---|---|---|
| binary `main` | `main` | `weave/src/main.rs:4489` (the `weave` bin; 71 CLI verbs) |
| MCP stdio server | `Mcp` verb → `weave-mcp::mcp` | `weave/src/main.rs` (`weave mcp`) |
| HTTP serve / dashboard | `handle_connection`, `handle_dashboard_connection` | `weave-mcp/src/http.rs:309 / :180` |
| daemon / scheduler | `Daemon`, `Tick`, `Schedule` verbs | `weave/src/main.rs` |
| host wiring | `Setup` verb → `weave/src/setup.rs` | installs MCP server + lifecycle hooks |
(index inferred 809 entrypoints total — dominated by `#[test]`/test-harness symbols.)

## 3. Hotspots — centrality (most-called production symbols)

| caller_count | symbol | role |
|---|---|---|
| 290/199* | `now` (`weave-core/src/model.rs:98`) | monotonic clock — used everywhere |
| 283/195* | `params` (`weave-core/src/store_libsql.rs`) | libsql query param builder |
| 165 | `Harness.new` (`weave-mcp/src/obscura.rs:414`) | MCP test-harness ctor |
| 112/75* | `mem` (`store.rs` / `store_libsql.rs`) | in-memory store ctor |
| 61 | `check_ident` (`weave-core/src/store.rs:1235`) | identity validation gate |
| 42 | `SqliteStore.send` (`weave-core/src/store.rs:3153`) | **the core deliver-a-message verb** |
| 30 | `SqliteStore.open` (`weave-core/src/store.rs:2469`) | broker open |
(*query `hotspots` count vs in-degree over production-edge list.)

**Top dispatch (out-degree)** — the fan-out hubs:
`call_tool` (mcp.rs:434, 50) → MCP tool router · `doctor` (main.rs:2776, 37) · `handle_dashboard_connection`
(http.rs:180, 26) · `handle_connection` (http.rs:309, 20) · `main` (main.rs:4489, 17) · `handle_hook` (main.rs:8531, 17).

## 4. Blast radius (transitive dependents, `impact --depth 5`)

```
 model.rs        ████████████████████████████████  1238   ← message schema; touch with extreme care
 store_libsql.rs ████████████                        488
 store.rs        ███████████                          462   ← the broker
 main.rs         ██████████                           427
 config.rs       ████████                             345
 inject.rs       █████                                205
 mcp.rs          ███                                  124
 sign.rs         ██                                    88
 webpolicy.rs    █                                     63
 http.rs         ▌                                     27
```
`weave-core/src/model.rs` (where `Intent`/`Message`/`Peer`/`Ask`/`Job`/`Lease` live) is the single
highest-blast file — the schema is the contract the whole mesh shares.

## 5. Cycles (Tarjan SCC, in-process)

3 multi-node SCCs, all **same-file, size ≤ 5**, 0 self-recursive:

| size | members | assessment |
|---|---|---|
| 5 | `spawn`,`kill`,`run_bounded`,`run_bounded_env`,`run_capture_env` (`weave-inject/src/inject.rs`) | overloaded `spawn`/`kill` (prod fn @714/816 vs test-mod method @1675/1690) — resolver back-edge |
| 2 | `open` ↔ `open_conn` (`weave-core/src/store.rs:2469/2499`) | `open`→`open_conn` is real; reverse edge = ambiguous resolve |
| 2 | `call_tool` ↔ `tool_meta` (`weave-mcp/src/mcp.rs:434/4854`) | `call_tool`→`tool_meta` is real; reverse edge = ambiguous resolve |

**Verdict:** No cross-crate / architectural cycles. The 3 SCCs are most consistent with resolver
back-edges among overloaded same-name helpers (index reports 774 `ambiguous` resolutions) rather
than true mutual recursion. Flagged for the verifier to confirm the back-edges.

## 6. Layering violations

**None at compile time.** `weave-core/Cargo.toml` has zero internal weave deps; the Cargo path-dep
DAG is strictly `weave-core <- weave-inject <- weave-mcp <- weave`. The call-edge list shows
"upward-apparent" edges (`weave-core`→`weave-mcp` 98, `weave-core`→`weave` 36, `weave-inject`→`weave-mcp`
16) — these are **name-ambiguity resolver artifacts** (a call to `now`/`send`/`new`/`mem`/`ident`
inside `weave-core` mis-binding to a same-named symbol in a higher crate). They cannot exist at
compile time; treat as graph noise, not architecture.

## 7. Public API surface (`query public-api`, 200 reported)

By crate: `weave-core` dominates (the shared types + broker API: `Store::send/inbox/history/search/
pull_from_store/commit_pulled`, `Intent`, `Message`, `Peer`, `Lease`, `Job`, archive/export helpers).
`weave-inject` exposes `Mux`, `Target`, `inject`, `spawn`. `weave-mcp` exposes the MCP server + 78
`weave_*` tools. `weave` is the bin (no library surface beyond `main`).

## 8. Dead code (`query dead-code-explain`, 200 reported)

**Zero genuine dead production code.** Every reported symbol is either a `#[cfg(test)]` inline test /
`benches/` fn (resolver can't link the test runner) or a serde-default helper
(`default_worktree`, `default_ask_kind`) invoked via attribute the AST doesn't traverse.

## 9. Top execution flows (`flows`, 464 traced)

| criticality | flow | depth | nodes | files |
|---|---|---|---|---|
| 0.92 | `handle_dashboard_connection` (http.rs) | 8 | 416 | 17 |
| 0.92 | `handle_connection` (http.rs) | 8 | 409 | 17 |
| 0.88 | `mcp_peer_diagnostics` | 7 | 42 | 7 |
| 0.84 | `main` (the CLI) | 6 | 731 | 27 |
| 0.80 | `resolve_mcp_circle` | 5 | 17 | 5 |

The two HTTP-serve flows are the deepest/widest — the Tier-2 cross-machine PUSH + dashboard plane is
the most interconnected behavior in weave.
