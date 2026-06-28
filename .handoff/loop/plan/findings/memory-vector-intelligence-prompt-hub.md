# Findings — memory-vector-intelligence — target: prompt-hub

axis: memory-vector-intelligence
target: prompt-hub
repo: /home/drdave/Desktop/meta/prompt_hub (workspace member: `prompt-hub`)
mode: read-only audit (planning artifact only)
date: 2026-06-27

Verdict: prompt-hub is a genuine persistent store with three-tier search (FTS5 +
native libsql vector + optional Qdrant), but its **memory is store-local**. It has
no code-level binding to fleet persistent memory (ICM) or the witnessed handoff
ledger; goal/intent provenance is not recall-informed; and one learning surface
(`learn_from_feedback`) is fully ephemeral. The repo itself is onboarded to the
fleet (`.handoff/`, `.kb/`) at the harness layer, but the intent store does not
read or write those substrates.

---

## 1. Memory inventory

prompt-hub's durable memory is a local libsql/SQLite database (`Builder::new_local`,
WAL), schema from versioned migrations.

| Memory surface | Where | Persistence | Evidence |
|----------------|-------|-------------|----------|
| Prompt store (the intent/prompt memory) | `prompts` table | durable libsql, soft-delete | `prompt-hub/migrations/0001_initial.sql:5-25` |
| Version lineage as memory | `versions(prompt_id,parent_id,version,changelog,diff)` | durable | `prompt-hub/migrations/0001_initial.sql:40-50` |
| In-memory lineage graph (fork/ancestry) | `LineageTracker` HashMap | **process-local, rebuilt** | `prompt-hub/src/lineage.rs:13-18`, `register_version` `:67-116` |
| Usage metrics memory (recall-informing signal) | `metrics(usage_count,success_rate,last_used,...)` | durable | `prompt-hub/migrations/0001_initial.sql:53-62` |
| Vector embeddings memory | `embeddings(prompt_id, embedding F32_BLOB(384))` ON DELETE CASCADE | durable | `prompt-hub/migrations/0001_initial.sql:64-69`; write `prompt-hub/src/storage.rs:350` |
| FTS index memory | `prompts_fts` fts5 + sync triggers | durable, trigger-maintained | `prompt-hub/migrations/0001_initial.sql:71-100` |
| Audit trail memory | `audit` (migration 0002) | durable | `prompt-hub/src/storage.rs:192` |
| Learned-feedback memory | `LearningEngine` HashMap | **ephemeral — never persisted** | `prompt-hub/src/learn.rs:14-39`; hub wiring `prompt-hub/src/hub.rs:1897` |
| Migration ledger | `_migrations` | durable | `prompt-hub/src/storage.rs:173` |

Fleet memory pointers present at the **repo** layer (harness substrate, not store code):
- `.handoff/` continuity kernel exists with `ledger.db`, `policy.toml`, `loop/`,
  `packets/` (`/home/drdave/Desktop/meta/prompt_hub/.handoff/`).
- `.kb/` gitkb present but **embeddings disabled** (`.kb/config.toml`:
  `[embeddings] enabled = false`).
- Registered as a fleet member: `/home/drdave/Desktop/meta/.meta.yaml:168` (`prompt_hub`).

ICM / handoff recall+store hooks in store code: **none.** A repo-wide grep for
`icm|git-kb|recall|witnessed ledger|kb_recall` across `prompt-hub/src` returns
zero matches; the only `handoff` hits are prompt-hub's *own domain concept*
(swarm role-to-role handoff templates, `prompt-hub/src/swarm.rs:179-228`,
`prompt-hub/src/defaults.rs:157`) — unrelated to the meta handoff ledger.

## 2. Vector intelligence map

prompt-hub ships a real, pluggable vector/semantic stack — not FTS-only.

| Index / engine | Kind | Freshness / update | Failure behavior | Evidence |
|----------------|------|--------------------|------------------|----------|
| `FastEngine` | FTS5 lexical (BM25-capable; score hardcoded 1.0) | auto via fts5 triggers on prompt write | empty/punct query → no terms, returns none | `prompt-hub/src/search.rs:54-216` (score note `:194`) |
| `SmartEngine` | ONNX `all-MiniLM-L6-v2` 384-d embeddings, cosine; hybrid rank `0.6*cos+0.3*perf+0.1*recency` | embedding written to `embeddings` table on `index()` | model not cached → download + sha256 verify; `HashEmbedder` deterministic fallback for tests/offline | `prompt-hub/src/search.rs:794-995`, `cosine_similarity:907`, `hybrid_score:923` |
| `HashEmbedder` | deterministic hash → 384-d vector (NOT semantic) | n/a | reproducible; **this is the default backend** | `prompt-hub/src/search.rs:260-292`; default `prompt-hub/src/config.rs:14` (`Hash`), test `:130` |
| `QdrantEngine` | external Qdrant vector DB over HTTP REST | `upsert` on index; `ensure_collection` auto-create | `feature="qdrant"`-gated; network errors → `HubError::Network` | `prompt-hub/src/qdrant.rs:104-303,490-585`; gate `prompt-hub/Cargo.toml:81` |
| `HybridEngine` | merges Fast + Smart results | composes both | inherits child-engine behavior | `prompt-hub/src/search.rs:1083-1136`, `merge_results:1573` |
| ONNX model manifest | `models.json` in `dirs::cache_dir()` | checksum-verified fetch | missing dir → seed default manifest; bad sha → error | `prompt-hub/src/search.rs:326-345` (manifest), session `:136-150` |

Owner/update command: indexing is implicit — `Hub::create_prompt` calls
`self.search_engine.index(&prompt)` (`prompt-hub/src/hub.rs:934`); the engine kind
is chosen at construction from `config.embedding_backend` (`prompt-hub/src/hub.rs:388-403,
537-588`). There is **no standalone reindex/refresh command** for backfilling
embeddings after a backend switch (gap — see Upgrade U3).

RAG: **N/A — no retrieval-augmented-generation pipeline exists.** `gather.rs` /
`context_gatherer.rs` do filesystem context collection (`prompt-hub/src/gather.rs:199-210`),
not embedding-grounded RAG retrieval; grep for `RAG|retrieval.augment` in
`prompt-hub/src` finds only unrelated "retrieve" method names. The vector stack is
a prompt *search/discovery* index, not a RAG context feeder.

git-kb code-graph snapshot of the store: **N/A for the store's own runtime** — gitkb
is a workspace harness tool over the repo; `.kb/config.toml` has `embeddings
enabled = false`, so no semantic code-graph snapshot is maintained for this target
yet (planning-engineer cartography would supply it, not prompt-hub itself).

## 3. Recall guarantees

- Session-start recall: **N/A — prompt-hub is a library/server, not a session
  agent.** It exposes `Hub::search` / `get_prompt` (`prompt-hub/src/hub.rs:965,1103-1124`)
  as on-demand recall over durable libsql; there is no agent "session" concept inside it.
- Background-agent recall: callers can recall prompts by intent/role
  (`get_prompt_for` family) and by vector/FTS search — durable across process
  restarts because state is in libsql, **not** conversation memory. This satisfies
  "no plan depends on chat history" at the data layer.
- Cold-start resume proof: durable — DB reopens via migrations (`storage.rs:173-227`),
  FTS rebuilds via triggers, embeddings persist in `embeddings`. **But** two
  recall surfaces do NOT survive cold start: (a) `LineageTracker` is an in-memory
  graph rebuilt from `versions` on demand (`lineage.rs:13`), and (b)
  `learn_from_feedback` constructs a throwaway `LearningEngine::default()` per call
  and drops it (`hub.rs:1897-1907`) — learned corrections are lost immediately,
  so no cold-start recall of learning exists.
- Fleet recall (ICM) binding: **absent.** Goal/artifact provenance (who asked for a
  prompt, why, which decision) is not stored to or recalled from ICM; the `metadata`
  JSON column (`prompts.metadata`, migration `:16`) is the only provenance slot and
  is not recall-indexed against fleet memory.

## 4. Upgrade rows

| id | axis | upgrade | evidence | acceptance | risk | reversibility |
|----|------|---------|----------|-----------|------|---------------|
| U1 | memory-vector-intelligence | Persist learned feedback: write `UserCorrection` rows to a new `corrections` libsql table instead of a per-call throwaway engine; load on `Hub::new` | `prompt-hub/src/hub.rs:1897-1907`, `prompt-hub/src/learn.rs:156-162` (export/import already exist) | corrections survive restart; `learn_from_feedback` then `get_improved_prompt` recalls them in a new process | low (additive table + migration) | high (drop table / revert migration) |
| U2 | memory-vector-intelligence | Make default search semantically real or explicit: ship a config preset where default `embedding_backend` is OnnxRuntime when `smart`/model present, else clearly label Hash as lexical-only | `prompt-hub/src/config.rs:14` (default `Hash`), `prompt-hub/src/search.rs:260-292` | docs/config state that out-of-box "Smart" without ONNX is non-semantic; opt-in real embeddings documented | low | high (config-only) |
| U3 | memory-vector-intelligence | Add a `reindex`/`embed-backfill` command to (re)populate `embeddings` after a backend/dimension change; today index runs only on create | `prompt-hub/src/hub.rs:934` (index on create only), `storage.rs:350,712` | running it embeds all existing prompts; dimension mismatch fails closed | medium (batch over store) | high (idempotent, re-runnable) |
| U4 | memory-vector-intelligence | Bind goal/intent provenance to fleet memory: emit an ICM store on prompt create/learn and recall prior decisions for the same intent before serving | no ICM calls in `prompt-hub/src` (repo-wide grep empty); `.meta.yaml:168` shows fleet membership | provenance for a served prompt is recoverable via `icm recall` in a fresh session | medium (new optional dep/boundary; keep feature-gated for portability) | high (feature flag off) |
| U5 | memory-vector-intelligence | Persist `LineageTracker` derivation or guarantee rebuild-from-`versions` on open so fork/ancestry recall is cold-start safe | `prompt-hub/src/lineage.rs:13-116` (in-memory), durable source `migrations/0001_initial.sql:40-50` | after restart, `ancestry`/fork queries return same results without replay by caller | low | high |

## 5. Gate handoff (fail-closed additions)

So missing memory/vector surfaces fail closed rather than silently degrade:

1. **Persistence test for learning** — add a test that calls `learn_from_feedback`,
   drops the `Hub`, reopens from the same `db_path`, and asserts the correction is
   recalled. Today this would fail (proves U1 gap). Anchor: `hub.rs:1888`.
2. **Embedding-coverage gate** — add a check/test asserting every non-deleted
   `prompts` row has an `embeddings` row when a vector backend is active; fail the
   gate on orphan/missing embeddings (extends `delete_orphaned_embeddings`,
   `storage.rs:712`). This catches the "index-on-create-only" gap (U3).
3. **Backend-honesty assertion** — test that `EmbedderBackend::Hash` is reported as
   `lexical/deterministic` (not `semantic`) in any capability/health output, so a
   Hash default cannot masquerade as vector search (`config.rs:14`).
4. **Fleet-recall presence gate (when U4 lands)** — feature-gated test that a created
   prompt produces an ICM-recallable provenance record; absence fails closed under
   the `icm` feature. Until U4, record this as an explicit, asserted N/A so the gate
   does not silently pass on a non-existent binding.
5. **Cold-start lineage gate** — test that `LineageTracker` answers ancestry/fork
   queries after reopening from `versions` alone (U5).

---

3-line summary:
prompt-hub is a real persistent store: libsql `memory` for prompts/versions/metrics,
a genuine three-tier search (FTS5 + native 384-d `vector` embeddings + optional Qdrant;
no `RAG` pipeline, and `git-kb` embeddings are disabled at `.kb` level). Its store code
has no `ICM`/`handoff`-ledger binding and no `recall`-informed goal provenance — learned
feedback and the lineage graph are in-memory/ephemeral, so cross-session recall and
fleet convergence are the load-bearing gaps (upgrades U1, U4, U5).
