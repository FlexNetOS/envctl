# Findings — icm · dimension: CONVERGENCE (memory plane of the fabric)

Analyst: plan-analyst (cycle 7). Read-only. Evidence cites `icm` (target) + peer repos
`handoff`, `prompt_hub`, `rusty-idd` under `$META_ROOT`. Status target: `- [~]` analysed-not-verified.

Frame: meta = ONE converging system; union = handoff (continuity kernel, **no-C** redb/RVF) +
rusty-idd (intent control plane). icm = the persistent **agent-memory** organ. Question: does icm
cleanly realize the memory plane, and how does it BIND into the union?

---

## A. CLAIM rows (falsifiable, file:line / symbol)

### A1 — What icm IS (the memory plane realized)

- CLAIM: icm's durable substrate is SQLite-backed hybrid BM25⊕vector recall — one `vec0` virtual table `vec_memories` over `float[384] distance_metric=cosine` plus FTS5 mirror tables. | evidence: `icm/crates/icm-store/src/schema.rs:25-27` (`CREATE VIRTUAL TABLE vec_memories USING vec0(... embedding float[{embedding_dims}] distance_metric=cosine)`); FTS5 mirrors `memories_fts/concepts_fts/feedback_fts/messages_fts` (codemap §1) | confidence: high
- CLAIM: hybrid recall fuses keyword and vector scores with a **fixed 0.3·FTS + 0.7·vec** weighting (not RRF, not tunable). | evidence: `icm/crates/icm-store/src/store.rs:1153` (`fn search_hybrid`), `:1211` (`let combined = 0.3 * fts_score + 0.7 * vec_score`) | confidence: high
- CLAIM: default embedding is 384-dim and produced by an ONNX model via fastembed (`intfloat/multilingual-e5-base`), behind feature `embeddings`; without the feature recall degrades to FTS5-only. | evidence: `icm/crates/icm-core/src/lib.rs:18` (`pub const DEFAULT_EMBEDDING_DIMS: usize = 384`); `icm/crates/icm-core/src/fastembed_embedder.rs:36` (`DEFAULT_MODEL = "intfloat/multilingual-e5-base"`), `:112` (`TextEmbedding::try_new`) | confidence: high
- CLAIM: the memory record models agent-knowledge semantics distinct from continuity events — `topic/summary/keywords/importance{critical,high,medium,low}/source{ClaudeCode,Conversation,Manual}/weight/access_count` with decay+consolidation lifecycle. | evidence: `icm/crates/icm-core/src/memory.rs:6-30` (`struct Memory`), `:96-103` (`enum Importance`), `:132-141` (`enum MemorySource`); lifecycle ops `apply_decay/prune/consolidate_topic` in `icm/crates/icm-core/src/store.rs:26-33` (`trait MemoryStore`) | confidence: high
- CLAIM: agents reach memory over a stable MCP contract (31 `icm_*` tools) AND the server actively *nudges* persistence — after 10 non-store calls it injects a store reminder. | evidence: `icm/crates/icm-mcp/src/server.rs:17` (`const STORE_NUDGE_THRESHOLD: u32 = 10`), `:180` (`if *calls_since_store >= STORE_NUDGE_THRESHOLD && tool_name != "icm_memory_st…"`) | confidence: high
- CLAIM: `SqliteStore::new` is the highest-leverage change point of the whole repo (blast 281, 60 callers); `Memory::new` second (blast 232, 47 callers). Any change to the store ctor or the Memory shape ripples across cli+mcp+store. | evidence: `graph/icm.metrics.json` top_blast_radius `{new,icm-store,281}` + `{new,icm-core,232}`; `icm/crates/icm-store/src/store.rs:104`, `icm/crates/icm-core/src/memory.rs:38` | confidence: high

### A2 — Canonical-vs-peer (icm vs handoff capsule/ledger vs git-kb)  [Q1]

- CLAIM: handoff's continuity plane is a SEPARATE data shape from icm memory — a pure-Rust **redb** append-only witnessed event ledger (ADR-0017, no bundled C) with a **native RVF v2 vector overlay** doing HNSW `query_by_intent` semantic recall over its OWN events, not over agent memories. | evidence: `handoff/ledger/Cargo.toml:6` (`Pure-Rust handoff ledger: redb authoritative event store + native-RVF semantic recall overlay`); `handoff/ledger/src/lib.rs:11-12` (`semantic recall (HNSW query_by_intent) … RVF overlay only adds recall`); `handoff/.kb/.../context/immutable/project-brief.md:21` (`No-C trust boundary (ADR-0001) … ledger ported to pure-Rust redb, ADR-0017`) | confidence: high
- CLAIM: handoff's `context/capsule.json` (schema `handoff.context_capsule.v1`) is a north-star/doctrine POINTER record (role/plane/tier/northstar/next_command), NOT a memory store — it has no recall/store fields. | evidence: `meta/.handoff/context/capsule.json` keys = `[schema, project_name, role, plane, tier, northstar, next_command, source]`; builder `handoff/hf/src/main.rs:237-253` (`init_capsule` writes exactly those fields) | confidence: high
- CLAIM: the three "memory-ish" planes hold disjoint corpora → icm is a PEER, not a redundant store: icm = unstructured durable agent knowledge (decisions/errors/prefs/context); handoff = witnessed continuity events + doctrine capsule; git-kb = code structure (AST symbols/callers/impact). Boundary is by corpus, not by mechanism. | evidence: icm corpus `memory.rs:6-30`; handoff corpus `ledger/src/v2.rs:1-4` (`RVF vector-native ledger v2 … event ledger (append, replay, witness chain, lease state, rollup provenance)`); git-kb corpus = `.claude/rules/code-intelligence.md` (`kb_symbols/kb_callers/kb_impact`) | confidence: high
- CLAIM: the only genuine MECHANISM overlap is vector semantic-recall: handoff RVF/HNSW (no-C) vs icm sqlite-vec (C) vs prompt_hub ort+libsql (C) — three vector engines, same *capability*, three corpora. | evidence: `handoff/ledger/Cargo.toml:15` (RVF overlay), `icm/.../schema.rs:25-27` (sqlite-vec), `prompt_hub/prompt-hub/src/search.rs:313-324` (`use ort::session::Session` + `all-MiniLM-L6-v2`) | confidence: high

### A3 — Bind-as-data status (is icm read/written through a contract?)  [Q2]

- CLAIM: icm is currently bound to the union by CONVENTION + ad-hoc invocation, not as data — the binding is the CLAUDE.md/AGENTS.md mandate ("recall before work, store on triggers") plus a connected MCP server, with NO programmatic read/write contract. | evidence: `icm/AGENTS.md` (`<!-- icm:start -->` MANDATORY recall/store block); MCP surface `icm/crates/icm-mcp/src/server.rs:106-146` (`handle_initialize/handle_tools_list/handle_tools_call`); no memory pointer exists in `handoff.context_capsule.v1` (A2) | confidence: high
- CLAIM: handoff ALREADY registers icm as a fleet ENTRY (`fleet/icm/capsule.json`, plane `rtk-tooling`, tier `C`) — but this capsule is metadata-ABOUT-icm, still the doctrine schema with no db-path / MCP-endpoint / recall-contract field, so it does not make icm readable as data by the union. | evidence: `handoff/.handoff/fleet/icm/capsule.json` (role names the SQLite/sqlite-vec hybrid backend; keys identical to `context_capsule.v1`, no memory pointer) | confidence: high
- CLAIM: rusty-idd's binding to icm is also workflow-obligation, not data — it declares harness contracts that MANDATE comparing ICM recall against implementation decisions, expressed as string registry entries, not a typed read/write API. | evidence: `rusty-idd/crates/cli/src/commands/harness.rs:208` (`icm-checker`), `:219` (`icm-comparison-contract … ICM recall results must be compared against the implementation`), `:233` (`icm-recall-context-compare`); icm also self-identifies as rusty-idd-governed: `icm/.claude/rusty-idd-adapter.md:3` (`GENERATED by rusty-idd render claude`) | confidence: high
- CLAIM: "icm bound as data" is therefore an UNREALIZED design — the missing artifact is a memory-pointer block in the north-star capsule (db path / MCP endpoint / scope / recall contract) so the union reads/writes icm through a declared pointer rather than per-agent prose. | evidence: absence in `handoff/hf/src/main.rs:253` (init_capsule field list) + `meta/.handoff/context/capsule.json` (no `memory` key) | confidence: high

### A4 — C-in-trust-boundary (sidecar verdict)  [Q3]

- CLAIM: icm cannot live inside the union's no-C kernel — it links bundled C three ways: rusqlite `bundled` + `modern_sqlite`, sqlite-vec, and ONNX runtime via fastembed. | evidence: codemap §1 C-dependency note (`rusqlite { features=["bundled","modern_sqlite"] }` + `sqlite-vec`); `icm/crates/icm-store/Cargo.toml` deps `rusqlite, sqlite-vec`; ONNX via `icm/crates/icm-core/src/fastembed_embedder.rs:5` (`use fastembed::…`) | confidence: high
- CLAIM: the union kernel is explicitly no-C (handoff ADR-0001/ADR-0017: redb ACID store + RVF overlay, "no `-sys`"), so the boundary forbids icm in-process — icm must sit OUTSIDE as a sidecar reached over a contract, exactly mirroring cycle-5's grit verdict (coordination/data substrate AROUND a no-C reconciler). | evidence: `handoff/ledger/src/v1.rs:4` (`redb — a pure-Rust, ACID … embedded KV store (no -sys …)`); `handoff/.kb/.../context/extensible/tech.md:21` (`No-C trust boundary`) | confidence: high
- CLAIM: the sidecar contract surface ALREADY EXISTS (MCP stdio `icm serve` + ~40 CLI verbs + 15 REST routes) and is graceful-degradation friendly (the harness `icm-memory` skill is a "graceful no-op if ICM isn't installed"), so the sidecar pattern is buildable without touching the no-C kernel. | evidence: codemap §2 (CLI/MCP/HTTP surfaces); icm MCP server `icm/crates/icm-mcp/src/server.rs`; skill description (`icm-memory` — "Graceful no-op if ICM isn't installed") | confidence: high

### A5 — Overlap / dedup vs prompt_hub (and handoff RVF)  [Q4]

- CLAIM: icm and prompt_hub DUPLICATE the vector+FTS substrate but serve DISTINCT planes — both are local-first, C-bearing, FTS5 + 384-dim vector engines, yet icm = agent MEMORY (decay/consolidation over decisions/errors) and prompt_hub = intent/prompt STORE (the Front-Door catalog). | evidence: prompt_hub FAST engine `prompt_hub/prompt-hub/src/search.rs:51-56` (`FAST search engine backed by libsql / SQLite FTS5`), SMART engine `:313-324` (`ort` + `all-MiniLM-L6-v2`, dim 384 `:409`), `HashEmbedder::new(384)` fallback `:710`; icm planes per A1 | confidence: high
- CLAIM: the two 384-dim spaces are NOT interchangeable despite matching dimensionality — icm uses `multilingual-e5-base` (e5 prompt conventions) and prompt_hub uses `all-MiniLM-L6-v2`; same width, different model → vectors cannot be shared, so a naive "merge the vector tables" is infeasible. | evidence: `icm/crates/icm-core/src/fastembed_embedder.rs:36`; `prompt_hub/prompt-hub/src/search.rs:324` (`DEFAULT_MODEL_NAME = "sentence-transformers/all-MiniLM-L6-v2"`) | confidence: high
- CLAIM: the workspace runs THREE vector engines (icm sqlite-vec/C, prompt_hub libsql+ort/C, handoff RVF/pure-Rust) — only handoff's is no-C, making RVF/ruvector the strategically-correct convergence substrate IF a single vector core is ever consolidated; prompt_hub has 0 `rusqlite` (it is on libsql, also C). | evidence: `grep -c rusqlite prompt_hub/Cargo.lock` = 0; `handoff/ledger/Cargo.toml:15` (pure-Rust RVF); `icm/.../schema.rs:25` (sqlite-vec) | confidence: high

---

## B. GAPS (what the convergence lens reveals as missing/weak)

- GAP: No data-contract binding. The union has no typed pointer to icm; binding is prose mandate + MCP connection. A fresh union session has no machine-readable way to discover "where is agent memory, what scope, how do I recall/store" — it depends on CLAUDE.md being loaded. (A3)
- GAP: Substrate triplication. Three vector engines, two of them C-bearing, no shared embedding contract. No ADR declares which is canonical for which corpus, so drift (dims, model, fusion weights) is unmanaged across icm/prompt_hub/handoff. (A5)
- GAP: Hybrid fusion is a hard-coded magic constant (0.3/0.7), not tunable and not reconciled with handoff RVF's recall or prompt_hub's BM25 — three recall rankers with three different, unstated scoring regimes. (A1, store.rs:1211)
- GAP: C-boundary is asserted by handoff doctrine but not *gated* at the union edge — nothing mechanically prevents an integrator from trying to link icm in-process; the sidecar boundary is convention, not a build gate. (A4)
- GAP (cross-dimension hook for architect): the `memory-vector-intelligence` dimension and this convergence lens overlap on the C-dependency; the governance dimension owns whether a no-C build gate should exist. Flag for de-dup at synthesis.

---

## C. UPGRADE rows (axis: quality | speed | accuracy | convergence)

- UPGRADE: Add a `memory` pointer block to `handoff.context_capsule.v1` (db path / MCP endpoint / scope / recall+store contract version) so the union binds icm AS DATA, not by prose. | axis: convergence | target-surface: `handoff/hf/src/main.rs` `init_capsule` (`:237-253`) + capsule schema doc | rationale: turns "recall before work" from a CLAUDE.md convention into a machine-discoverable pointer the union reads; realizes Q2's unrealized design | evidence: A3 (no memory key in `meta/.handoff/context/capsule.json`; field list at `handoff/hf/src/main.rs:253`) | blast: capsule schema is fleet-wide (every member capsule) — high; touches `current_northstar_revision`/capsule_field consumers (`hf/src/main.rs:65`) | effort: M | risk-tier: PROPOSE | acceptance: `hf init` in a member repo emits a capsule whose `memory` block names a recall endpoint + scope, and a contract test asserts the field is present and well-formed | reversibility: Integrity preserved (additive field) · Reversible (drop the block) · Capability-Gain: union gains data-bound memory discovery
- UPGRADE: Author an ADR "memory/vector plane ownership" declaring icm = canonical agent-memory (sidecar), handoff RVF = continuity recall, prompt_hub = intent store, git-kb = code intelligence — one corpus per engine, no cross-write. | axis: convergence | target-surface: `handoff/docs/adr/` (new ADR) + reference from rusty-idd intent canon | rationale: kills the "redundant store?" ambiguity (Q1) and freezes the boundary so future work doesn't re-merge planes | evidence: A2 (disjoint corpora), A5 (triplication) | blast: doc-level, governs three repos — medium | effort: S | risk-tier: PROPOSE | acceptance: ADR exists, states the 4-plane ownership + no-cross-write rule, and is linked from rusty-idd's intent canon; a check asserts each engine's corpus matches its declared owner | reversibility: Integrity preserved · Reversible (supersede ADR) · Capability-Gain: stable convergence boundary
- UPGRADE: Make the icm↔kernel boundary a fail-closed BUILD gate — assert the union kernel crate graph links no rusqlite/sqlite-vec/ort/fastembed, so icm can only ever be a sidecar. | axis: convergence | target-surface: union kernel CI (handoff `.github/workflows`) + `cargo deny`/dep-graph check | rationale: converts the no-C trust boundary (A4) from doctrine to a mechanical gate; prevents an integrator from linking icm in-process | evidence: A4 (boundary asserted in docs only); `handoff/ledger/src/v1.rs:4` no-`-sys` claim | blast: CI-only, no runtime code — low | effort: S | risk-tier: PROPOSE | acceptance: a RED test/CI step fails if the kernel dep graph contains any of {rusqlite, sqlite-vec, ort, fastembed}; passes on current no-C kernel | reversibility: Integrity STRENGTHENED · Reversible (remove the gate) · Capability-Gain: enforced no-C union boundary
- UPGRADE: Define a single embedding/recall CONTRACT (model id, dims, normalization, fusion semantics) that icm, prompt_hub, and handoff RVF each declare in their capsule, so the three engines are comparable and a future consolidation onto pure-Rust RVF is mechanical. | axis: convergence | target-surface: capsule schema (`vector` block) + each repo's embedder config (`icm/crates/icm-core/src/fastembed_embedder.rs`, `prompt_hub/prompt-hub/src/search.rs`, `handoff/ledger`) | rationale: removes the silent 384d-but-different-model trap (A5) and is the precondition for collapsing 3 engines → 1 no-C core | evidence: A5 (e5-base vs MiniLM-L6, same 384 dims, not interchangeable) | blast: spans 3 repos — high | effort: L | risk-tier: PROPOSE | acceptance: each engine emits a `{model, dims, normalize, fusion}` descriptor; a cross-repo check asserts a declared canonical model per corpus and flags mismatch | reversibility: Integrity preserved · Reversible (descriptor is metadata) · Capability-Gain: comparable/consolidatable vector planes
- UPGRADE: Make icm's hybrid fusion weight (0.3/0.7) configurable + documented rather than a magic literal, and record the chosen regime in the recall contract. | axis: accuracy | target-surface: `icm/crates/icm-store/src/store.rs:1153-1212` (`search_hybrid`) | rationale: hard-coded fusion is untunable and unstated vs the two peer rankers; configurability lets recall quality be measured/compared (icm already ships eval harnesses `scripts/bench-quality.ts`) | evidence: `icm/crates/icm-store/src/store.rs:1211` (`0.3 * fts_score + 0.7 * vec_score`) | blast: `search_hybrid` feeds `recall`/MCP `icm_memory_recall` — medium (recall path, not store) | effort: S | risk-tier: APPLY | acceptance: fusion weights read from config with the current 0.3/0.7 as default; a test asserts default behavior is byte-identical to today and a non-default weight changes ranking | reversibility: Integrity preserved (default unchanged) · Reversible (revert to literal) · Capability-Gain: measurable/tunable recall

---

## D. CONVERGENCE VERDICT (recommendation for the architect)

icm cleanly realizes the **canonical agent-memory plane** and is a true PEER — not redundant —
to handoff's witnessed-continuity ledger and git-kb's code intelligence: the three hold disjoint
corpora (agent knowledge / continuity events / code structure). The ONLY mechanism overlap is
vector recall, and that is corpus-separated.

Binding: icm is **NOT yet bound as data**. Today it binds by CLAUDE.md/AGENTS.md convention + a
connected MCP server + a fleet-entry capsule + rusty-idd workflow-obligation contracts — all
ad-hoc. "Bound as data" requires a `memory` pointer block in `handoff.context_capsule.v1`
(endpoint/scope/recall contract). That is the headline upgrade (C, row 1).

C-boundary: icm links bundled C three ways (rusqlite + sqlite-vec + ONNX/fastembed) and the union
kernel is no-C (redb/RVF, ADR-0001/0017). Therefore the verdict is **SIDECAR** — icm is the
out-of-boundary memory service the no-C kernel talks to over its existing MCP/CLI contract,
mirroring cycle-5's grit verdict. Recommend a fail-closed CI gate to make that boundary mechanical.

Dedup: icm vs prompt_hub are DISTINCT planes (agent memory vs intent store) that happen to
duplicate the substrate (both C-bearing FTS5 + 384d vector). They share dims but not models, so
they are not mergeable as-is. The real convergence move is a declared embedding contract per
corpus, with handoff's pure-Rust RVF/ruvector as the strategically-correct substrate IF a single
no-C vector core is ever consolidated — a long-horizon PROPOSE, not this cycle.

Confidence: high on plane identity, C-boundary, and bind status (all code-cited); medium on the
RVF-consolidation horizon (depends on RVF maturity, out of this target's scope). Mark target `[~]`.
