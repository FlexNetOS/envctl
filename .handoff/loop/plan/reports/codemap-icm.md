# Codemap — icm (Infinite Context Memory)

- **Target:** `icm` — the persistent-memory organ of the `meta` workspace
  (recall/store across sessions; the planning loop itself calls `icm store`/`icm recall`).
- **Repo:** `/home/drdave/Desktop/meta/icm` · meta member · remote `git@github.com:FlexNetOS/icm.git`
- **Snapshot:** branch `chore/rusty-idd-fleet-adapter` @ `5fde8fc`; 1629 symbols / 56 files / 3069 edges.
- **Build surface:** Rust workspace, 4 crates, resolver 2; release profile `lto=true codegen-units=1 panic=abort strip=true`. One binary `icm` (icm-cli). Edition/rust-version per crate manifests.
- **Read-only map** built from `git-kb code` JSON; README/ARCHITECTURE claims are intent-to-verify, not facts.

> North-star frame: `meta` converges on handoff + rusty-idd UNION (one continuity+intent
> control plane). icm is the candidate **canonical memory plane** — this cycle maps what it
> *is* so the analyst/architect can decide whether it is canonical or a peer of git-kb /
> .handoff context. (Binding decision is downstream; not asserted here.)

---

## 1. Crates (role · public surface · entrypoints)

### icm-core  (L0 foundation — 224 sym / 16 files; no `icm-*` deps)
Domain models + the embedding abstraction. Pub re-exports (`lib.rs`):
- **Models:** `Memory`, `Memoir`/`Concept`/`ConceptLink`/`Label`/`Relation`, `Feedback`/`FeedbackStats`, `Message`/`Role`/`Session`/`TranscriptHit`.
- **Stores (thin, core-level):** `MemoryStore` (`store.rs`), `MemoirStore`, `FeedbackStore`, `TranscriptStore` — note: these are core-level façades distinct from icm-store's `SqliteStore`.
- **Embedding:** `Embedder` trait (`embedder.rs`: `embed`/`embed_batch`/`dimensions`) + `FastEmbedder` (`fastembed_embedder.rs`, behind feature `embeddings` = `["fastembed","directories"]`). Default model `intfloat/multilingual-e5-base`; `DEFAULT_EMBEDDING_DIMS=384` (`lib.rs:18`).
- **Logic:** `learn::{learn_project,LearnResult}` (`learn.rs`, 25 KB), `wake_up` (24 KB), `auto_link::{auto_link_memory,add_backrefs,AutoLinkOptions}`, `time_fmt::format_local`.
- 77 pub symbols. No binary. Hotspot: `Memory::new` (`memory.rs:38`, cc 47, blast 232).

### icm-store  (L1 — 323 sym / 3 files; deps icm-core)
The durable substrate. Largest single file in the repo: `store.rs` (220 KB).
- **`SqliteStore`** (`store.rs:104` `new`): CRUD + search over memories, concepts, feedback, sessions, messages, memoirs. THE hotspot crate (`get` cc 183, `store` cc 143, `new` cc 60, blast 142–281).
- **`schema.rs`** (`init_db` / `init_db_with_dims`): builds the SQLite schema.
  - **Tables:** `memories`, `concepts`, `concept_links`, `feedback`, `sessions`, `messages`, `memoirs`, `hook_events`, `pending_extractions`, `icm_metadata`.
  - **Vector:** `CREATE VIRTUAL TABLE vec_memories USING vec0(embedding float[<dims>] distance_metric=cosine)` (sqlite-vec; dims 64–4096, default 384, stored in `icm_metadata`).
  - **FTS:** FTS5 virtual tables `memories_fts` / `concepts_fts` / `feedback_fts` / `messages_fts`.
  - Migration path: `ALTER TABLE memories ADD COLUMN embedding BLOB` for older DBs.
- **C-dependency note:** `rusqlite { features=["bundled","modern_sqlite"] }` + `sqlite-vec` compile C — sits in the trust boundary (relevant to meta's no-C invariant; flagged for the memory-vector-intelligence dimension). 31 pub symbols.

### icm-mcp  (L2 — 148 sym / 4 files; deps icm-core + icm-store)
The MCP (Model Context Protocol) server surface — how agents reach memory.
- **`server.rs`:** `handle_initialize` (`:106`), `handle_tools_list` (`:142`), `handle_tools_call` (`:146`). Includes a STORE-nudge counter (`calls_since_store >= STORE_NUDGE_THRESHOLD`) to push agents to persist.
- **`tools.rs`** (125 KB): the 31 `icm_*` tool definitions + `call_tool` dispatch (`:717`, cc 89) + arg helpers (`get_str` cc 54).
- **`protocol.rs`:** JSON-RPC envelope helpers (`error` cc 108, `text` cc 55).
- **31 MCP tools:** memory: `icm_memory_store`, `_recall`, `_update`, `_forget`, `_forget_topic`, `_consolidate`, `_list_topics`, `_stats`, `_health`, `_embed_all`, `_extract_patterns`; learn: `icm_learn`; wake: `icm_wake_up`; memoir: `icm_memoir_{create,add_concept,link,refine,show,inspect,list,search,search_all,export}`; feedback: `icm_feedback_{record,search,stats}`; transcript: `icm_transcript_{record,search,show,start_session,stats}`. 17 pub symbols.

### icm-cli  (L3 apps — 862 sym / 29 files; deps all icm-*; bin `icm`)
The human/agent CLI plus the optional web dashboard.
- **`main.rs`** (307 KB, 1572 LOC `main`): clap `Commands` enum (`:60`) with ~40 verbs:
  - memory: `Store`, `Recall`, `RecallContext`, `RecallProject`, `SaveProject`, `Update`, `Forget`, `List`, `Consolidate`, `Embed`, `ExtractPatterns`, `ExtractPending`, `Extract`.
  - intelligence: `Learn`, `WakeUp`, `Memoir`, `Transcript`, `Feedback`.
  - ops: `Health`, `Decay`, `Prune`, `Stats`, `Topics`, `Doctor`, `Config`, `Init`, `Upgrade`, `Uninstall`, `Hook`/`HookLog`/`HookStats`, `Import`, `Cloud`.
  - surfaces: `Serve` (MCP stdio), `Dashboard`/web, `Tui` (`tui.rs`, 54 KB), `Bench*`.
  - MCP-config injectors (handlers): `inject_{mcp,zed,copilot_cli,continue,codex,opencode}_mcp_server` — wire icm into 6 agent hosts.
- **`web.rs`** (axum, feature `web`): 15 REST routes (see §2). Cloud sync via `ureq` (`cloud.rs`). `is_icm_command` (`main.rs:2371`, cc 35) gates command routing. 100 pub symbols.

### scripts/ + plugins/  (tooling, outside crate layers)
`scripts/bench-{quality.ts,agent-sim.ts,longmemeval.py}` (eval harnesses, 3 binaries); `plugins/opencode-icm.ts` (opencode plugin). 70 TS + 11 py symbols indexed.

---

## 2. External interface surface

| Surface | Where | Detail |
|---|---|---|
| **CLI** | `icm` bin, `icm-cli/src/main.rs` | ~40 verbs (store/recall/learn/wake-up/memoir/transcript/feedback/serve/dashboard/...) |
| **MCP** | `icm serve` → `icm-mcp` | 31 `icm_*` tools over JSON-RPC stdio; `initialize`/`tools/list`/`tools/call` |
| **HTTP** | `icm dashboard` → `icm-cli/src/web.rs` (feature `web`) | 15 axum routes: `/`, `/health`, `/api/health[/decay,/prune]`, `/api/memories[/search,/{id}]`, `/api/memoirs[/{id}]`, `/api/topics[/{name}[/consolidate,/health]]`, `/api/stats` |
| **Cloud sync** | `icm-cli/src/cloud.rs` | `ureq` HTTP client (outbound sync) |
| **Hosts** | injectors in `main.rs` | claude, zed, copilot-cli, continue, codex, opencode |

---

## 3. Memory-plane data path (store / recall / embed)

```
STORE: CLI `icm store` | MCP icm_memory_store | POST web
   -> icm-core::Memory  (+ optional FastEmbedder -> Vec<f32>[384])
   -> icm-store::SqliteStore.store
   -> SQLite (rusqlite bundled C): memories(+embedding BLOB) / concepts / feedback / ...
      + vec_memories vec0(float[384] cosine)  + *_fts FTS5

RECALL: CLI `icm recall|recall-context|recall-project` | MCP icm_memory_recall | GET /api/memories/search
   -> SqliteStore.get/search  ->  FTS5 keyword ⊕ vec0 cosine (hybrid)
   -> rank + recall_format.rs  ->  caller
```
Default embedding model `intfloat/multilingual-e5-base` (384d); dims persisted in `icm_metadata`. Embeddings are **optional** (feature `embeddings`); without it, recall is FTS5-only.

---

## 4. Graph intelligence (summary; full in `graph/icm.metrics.json` + `graph/icm.graph.md`)

- **Hotspots:** `SqliteStore.{get,store,new}`, `make_memory`/`test_store` (fixtures), MCP `call_tool`/`protocol::error`.
- **Blast radius:** `SqliteStore::new` 281, `Memory::new` 232 — store + core models are the highest-leverage change points.
- **Cycles:** 7, all intra-crate, size 2–3 (ctor/cache clusters). No cross-crate cycles.
- **Layering:** 0 violations; edges flow cli→mcp→store→core only.
- **Public API:** 225 pub symbols. **Dead code:** 99 (test-excluded) / 326 raw (inflated by test helpers, trait impls, TS web fns — verify before pruning).

---

## 5. Claims to verify (downstream) & gaps

- **CLAIM (README/intent):** icm is the canonical cross-session memory plane. → analyst/architect to decide vs git-kb code-graph memory + `.handoff` witnessed ledger; not asserted here.
- **GAP `[!]`:** `git-kb code flows` traced **0 flows** — the data path in §3 is reconstructed from edges/routes/schema, not a traced flow.
- **GAP `[!]`:** `query routes` / `query cross-service-impact` returned empty JSON via CLI; routes read directly from `code_route` table. cross-service-impact N/A (single workspace).
- **Cross-repo edges:** deferred — icm is a self-contained crate workspace; no intra-repo `cross-service-impact` edges resolved. Its fleet bindings (how handoff/rusty-idd/the loop call `icm store`/`recall`) are *runtime CLI/MCP invocations*, not source call-edges, so they live in the target-DAG (`graph/target-dag.md`), not an icm-internal cross-repo edge list. (Per cartography skill: one-line note, skipped for this self-contained crate.)
- **C-dependency:** rusqlite(bundled)+sqlite-vec compile C in the trust boundary — flagged for memory-vector-intelligence + governance dimensions.

---

## 6. Dimensions seeded

Pre-seeded in `dimensions.md` (preserved): governance-config, filesystem-layout,
test-strategy, **memory-vector-intelligence** (primary for icm), autoresearch,
rules-policy-org, distributed-compute, prompt-architecture, **test-coverage**
(always-seeded, plan-test-strategist). All `[ ]` unexamined → analyst work items.
