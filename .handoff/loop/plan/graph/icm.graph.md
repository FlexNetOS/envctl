# icm — Code Graph (ASCII) + Graph Intelligence

- target: **icm** (Infinite Context Memory — persistent-memory organ of `meta`)
- repo: `/home/drdave/Desktop/meta/icm`  branch `chore/rusty-idd-fleet-adapter`  sha `5fde8fc`
- source: `git-kb code` index → `.git/gitkb/code.db` (`code_symbol` + `calls`), branch-scoped
- snapshot: 1629 symbols · 56 symbol-bearing files · 3069 resolved call edges (all intra-repo)
- generated: 2026-06-27 (baseline — no prior icm graph)

> Graph built ONLY from `git-kb code` call data (AST-resolved `name_match` edges), not grep.
> Metrics (SCC/centrality/blast-radius/layering) computed in-process over that edge list.

## 1. Crate / layer map (dependency direction = downward only)

```
 LAYER 3  ┌──────────────────────────────────────────────────────────────┐
  apps    │  icm-cli   (862 sym / 29 files)   bin "icm"  +  feature `web`  │
          │  ├─ main.rs (1572 LOC, ~40 clap verbs, MCP-config injectors)   │
          │  ├─ web.rs  (axum dashboard, 15 routes, feature-gated)         │
          │  ├─ extract*/import/summarizer/learn/cloud/tui/upgrade/config  │
          └───────┬───────────────────────┬───────────────────┬──────────┘
                  │ 40 edges               │ 35 edges          │
                  v                        v                   │
 LAYER 2  ┌───────────────────────────────────────────┐       │
  mcp     │  icm-mcp  (148 sym / 4 files)              │       │
          │  ├─ server.rs  initialize/tools_list/call  │       │
          │  ├─ tools.rs   31 icm_* tool defs+dispatch │       │
          │  └─ protocol.rs JSON-RPC envelope          │       │
          └──────┬──────────────────────────┬──────────┘       │
                 │ 17 edges                  │ 1 edge           │
                 v                           v                  v
 LAYER 1  ┌───────────────────────────────────────────────────────────────┐
  store   │  icm-store (323 sym / 3 files)                                  │
          │  ├─ store.rs (220 KB) SqliteStore: memories/concepts/feedback/  │
          │  │            sessions/messages/memoirs CRUD + search           │
          │  └─ schema.rs init_db: SQLite + sqlite-vec vec0 + FTS5          │
          │            [rusqlite bundled = C dependency in trust boundary]  │
          └──────────────────────────┬────────────────────────────────────┘
                                      │ 38 edges  (store -> core types)
                                      v
 LAYER 0  ┌───────────────────────────────────────────────────────────────┐
  core    │  icm-core (224 sym / 16 files)  — foundation, no icm-* deps     │
          │  Memory/Memoir/Concept/Feedback/Transcript models · Embedder    │
          │  trait · FastEmbedder (fastembed, optional `embeddings`) ·       │
          │  learn · wake_up · auto_link · time_fmt                         │
          └───────────────────────────────────────────────────────────────┘

 (tooling, outside crate layers: scripts/ 68 sym TS+py benches; plugins/ 4 sym TS)
```

Cross-crate edge counts (all flow high→low — **clean layering**):

```
  icm-cli  -> icm-core    40
  icm-store-> icm-core     38   (store builds on core models/Embedder)
  icm-cli  -> icm-store    35
  icm-mcp  -> icm-core     17
  icm-mcp  -> icm-store     1
  --------------------------------
  layering violations (low->high): 0
```

## 2. Surface — entrypoints & data ingress

```
 BINARIES (4)                          WEB ROUTES (axum, feature `web`, 15)
  icm-cli/src/main.rs::main            GET  /                  serve_index
  scripts/bench-quality.ts             GET  /health            api_health_check
  scripts/bench-agent-sim.ts           GET  /api/health        api_health_all
  scripts/bench-longmemeval.py         POST /api/health/decay  api_decay
                                       POST /api/health/prune  api_prune
 MCP HANDLERS (icm-mcp/server.rs)      GET  /api/memories      api_memories
  handle_initialize                    GET  /api/memories/search api_memories_search
  handle_tools_list                    DEL  /api/memories/{id} api_memory_delete
  handle_tools_call  -> 31 icm_* tools GET  /api/memoirs[/{id}]  api_memoir*
                                       GET  /api/topics[/{name}] api_topic*
 ENTRYPOINT KINDS (git-kb)            POST /api/topics/{n}/consolidate
  test 294 · public_api 120 ·          GET  /api/topics/{n}/health
  handler 16 · binary 4                GET  /api/stats         api_stats
```

## 3. Hotspots (centrality = inbound caller_count, from `query hotspots`)

```
 caller_count  symbol                         location
 ───────────── ───────────────────────────── ─────────────────────────────────
   183  SqliteStore.get .................... store.rs:926     [store accessor]
   151  make_memory (test/factory) ......... store.rs:3369    [test fixture]
   147  test_store ......................... store.rs:3365    [test fixture]
   143  SqliteStore.store .................. store.rs:907     [store mutator]
   108  protocol::error .................... mcp/protocol.rs:87
    89  call_tool (MCP dispatch) ........... mcp/tools.rs:717  [31-tool switch]
    62  test_store (mcp) ................... mcp/tools.rs:2357
    60  SqliteStore::new ................... store.rs:104
    55  protocol::text ..................... mcp/protocol.rs:77
    54  get_str (arg parse) ................ mcp/tools.rs:905
    47  Memory::new ........................ core/memory.rs:38
    35  is_icm_command ..................... cli/main.rs:2371
    34  SqliteStore.add_concept ............ store.rs:1714
```

Top BLAST-RADIUS (transitive dependents, reverse-reachability over edge list):

```
  281  SqliteStore::new      (cc 60)   — every store consumer
  232  Memory::new           (cc 47)   — every memory producer
  147  test_store            (cc 147)  — test substrate
  142  SqliteStore.get       (cc 183)
   98  make_memory           (cc 151)
   97  SqliteStore.store     (cc 143)
   97  protocol::error       (cc 108)  — MCP error path
```

Interpretation: the gravity wells are `SqliteStore` (store.rs) and the MCP
dispatch/protocol layer. The store is the single most-depended-on subsystem —
any change to `SqliteStore::{new,get,store}` or the schema has the widest blast.

## 4. Cycles (Tarjan SCC over the resolved edge list)

```
  SCCs total: 1620        cyclic SCCs (size>1): 7        self-loops: 0
  ─ size 3  icm-store   { new_cache, with_dims, new }       (ctor/cache cluster)
  ─ size 3  icm-cli     { load_memoirs, load_health, new }  (tui state ctor)
  ─ size 2  icm-core    { with_model, new }                 (FastEmbedder ctor)
  ─ size 2  icm-core    { new, new }  x2                     (overloaded ctors)
  ─ size 2  icm-store   { cache_get, get }                  (cache/get pair)
  ─ size 2  icm-store   { update, summary_hash }
```

All 7 cycles are small (2–3 nodes) and **intra-crate** — local constructor /
cache-helper clusters, not architectural cycles. **No cross-crate cycles.**

## 5. Public-API surface & dead code

```
  pub symbols: 225   (icm-cli 100 · icm-core 77 · icm-store 31 · icm-mcp 17)
  dead-code:  99 (git-kb, test-excluded)  /  326 raw caller_count==0
              ^ inflated: test helpers, trait Default impls (trait dispatch
                unresolved), TS web get/post/del (called from Svelte, cross-
                language edge unresolved). Verify per-symbol before pruning.
```

## 6. Store / recall / embed data path (the memory plane)

```
  WRITE (store):  CLI `icm store` / MCP icm_memory_store / web ─┐
                                                               v
     icm-core::Memory ──> icm-store::SqliteStore.store ──> SQLite (rusqlite,
        |                         |                          bundled C)
        |  (optional `embeddings`)|  tables: memories, concepts, concept_links,
        v                         |   feedback, sessions, messages, memoirs,
   icm-core::FastEmbedder ────────┘   hook_events, pending_extractions, icm_metadata
   (fastembed, model intfloat/multilingual-e5-base, DEFAULT_EMBEDDING_DIMS=384)
        |  Vec<f32>[384]
        v
   vec_memories  USING vec0(embedding float[384] distance_metric=cosine)  (sqlite-vec)
   + FTS5 shadow tables: memories_fts / concepts_fts / feedback_fts / messages_fts

  READ (recall):  CLI `icm recall|recall-context|recall-project`
                  / MCP icm_memory_recall / web /api/memories/search
        ──> SqliteStore.get/search ──> FTS5 keyword  ⊕  vec0 cosine (hybrid)
        ──> rank/format (recall_format.rs) ──> caller
```

## 7. Notes / gaps

- `git-kb code flows` returned **0 traced flows** for icm — the store/recall
  data path above is reconstructed from edges + routes + schema, not from a
  traced flow. (Recorded as a `- [!]` consideration; not fabricated.)
- `query routes` / `query cross-service-impact` JSON returned empty via the CLI;
  the 15 axum routes were read directly from `code_route`. cross-service-impact
  is N/A here (single self-contained workspace; cross-repo edges deferred — see
  cross-repo note in the codemap).
- rusqlite `bundled` + sqlite-vec compile C → relevant to the meta no-C trust
  boundary; flagged for the analyst/architect (memory-vector-intelligence dim).
```
