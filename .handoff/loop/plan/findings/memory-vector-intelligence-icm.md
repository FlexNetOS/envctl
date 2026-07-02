# Findings — axis: memory-vector — TARGET: icm (cycle 7)

Scope: icm is the persistent-**memory** organ of the meta fabric (recall/store across
sessions). This audit asks whether icm is the *canonical* memory/**vector**-intelligence
plane for the union, or a *peer* of git-kb code intelligence and handoff's witnessed
ledger — and how it binds into the handoff + rusty-idd union as data. Read-only on icm
source (verified at /home/drdave/Desktop/meta/icm). Versions pinned from
/home/drdave/Desktop/meta/icm/Cargo.lock.

Verdict (one line): icm is a **strong, self-contained personal-memory + hybrid-vector
plane**, correctly the canonical memory organ — but it is bound into the union only
**ad-hoc via CLI/MCP**, NOT as witnessed data; its importance/decay model is real but
crude vs the 2026 field; and it carries a documentation-drift dim bug + a stale
sqlite-vec pin. Keep it canonical for memory; do NOT let it duplicate handoff (ledger)
or git-kb (code graph); add provenance-aware **recall** to close the poisoning gap.

---

## 1. Memory model — topics / importance / decay / consolidation (real vs nominal)

CLAIM [CONFIRMED] The memory model is a flat (topic, summary, importance, weight) row,
not a 2026 three-tier episodic/semantic/procedural store.
- /home/drdave/Desktop/meta/icm/crates/icm-core/src/memory.rs:5-30 — `Memory` = topic +
  summary + keywords + importance + weight + access_count + source. No tier field.
- A *semantic* layer exists separately as memoirs/concepts/concept_links
  (icm-store/src/schema.rs:85-124), giving a partial semantic tier, but it is a
  side feature, not the primary recall path. No procedural tier at all.

CLAIM [CONFIRMED] Importance is STATIC at write time, exactly as the advisory warns.
- icm-core/src/memory.rs:96-103 — `Importance {Critical,High,Medium,Low}` enum, set by
  the caller at `Memory::new` (memory.rs:38-57) and never recomputed from retrieval
  outcomes. The only mutation is a MAX-merge on dedup
  (icm-store/src/store.rs:842-845, `max_importance`) — a write can *upgrade* importance
  but nothing closes an outcome→importance loop (no A-MAC, no dynamic admission control).

CLAIM [CONFIRMED] Decay is REAL (mutates the `weight` column) but crude — a fixed
multiplicative factor gated to fire at most once per 24h, NOT a continuous
Ebbinghaus/Weibull time function.
- icm-store/src/store.rs:128-148 — `maybe_auto_decay` claims the decay slot via
  `icm_metadata.last_decay_at` and only runs `apply_decay(0.95)` if >24h elapsed; it is
  invoked on recall (no cron). So 30 idle days still apply only ONE 0.95 step on the next
  recall — decay tracks recall *events*, not wall-clock elapsed time.
- icm-store/src/store.rs:1267-1311 — `apply_decay` is access-aware + importance-weighted:
  `weight *= 1 - (1-f)*mult/(1+min(access_count,5)*0.1)`; high=0.5x, low=2.0x, critical
  filtered out (`WHERE importance != 'critical'`). The access term is capped at 5
  (regression fix, audit #185 H7) to stop recall-count gaming. This is a sound but
  ad-hoc heuristic, not the field-standard forgetting curve.

CLAIM [CONFIRMED] Consolidation and forgetting are real operations, not nominal stubs.
- icm-store/src/store.rs:1313-1334 `prune` (weight-threshold delete, spares
  critical/high), :1378+ `consolidate_topic`, :2913-2926 `auto_consolidate` /
  `auto_consolidate_with_embedder`. extract_patterns surfaces clusters
  (icm-core/src/memory.rs:162-173 `PatternCluster`) and can promote to concepts
  (store.rs:3305 `extract_pattern_as_concept`).

GAP: No dynamic/outcome-updated importance; decay is recall-event-quantized not
time-continuous; no episodic/semantic/procedural tiering on the hot path.

UPGRADE [axis: memory-vector] Add outcome-aware reinforcement + true time-decay.
- rationale: aligns with 2026 dynamic-importance + Ebbinghaus/Weibull consensus; a
  memory that is recalled-and-used should reinforce, one never reused should decay on
  elapsed time, not on the next recall event.
- evidence: static enum at memory.rs:96-103; event-quantized 0.95 at store.rs:130-147.
- blast: medium — touches apply_decay + a new reinforcement call on recall hit; weight
  column already exists, schema-compatible.
- risk: medium — changing ranking weights can regress recall quality; gate behind a
  differential recall test and keep critical-skip invariant.

## 2. Vector / RAG path — model, dims, hybrid quality, ANN vs brute, currency

CLAIM [CONFIRMED] Hybrid **recall** = FTS5 keyword ⊕ vec0 cosine, fixed linear blend
0.3*FTS + 0.7*vector — NOT Reciprocal-Rank-Fusion.
- icm-store/src/store.rs:1153-1225 `search_hybrid`: pool=limit*4, FTS rank normalized
  `1/(1+|rank|)` (:1189), vector similarity `1 - distance` (:1147), combined
  `0.3*fts + 0.7*vec` (:1211). A query term absent from one arm scores 0 there, so the
  blend can under-rank a strong single-arm hit; RRF would be more robust.

CLAIM [CONFIRMED] Vector search is brute-force exhaustive KNN, not ANN.
- icm-store/src/store.rs:1092-1151 `search_by_embedding`: `WHERE embedding MATCH ?1 ORDER
  BY distance LIMIT ?2` against vec0. sqlite-vec 0.1.x vec0 has no ANN index — every
  query is a full cosine scan. Fine for a personal store (thousands of rows); it does not
  scale to a fabric-wide shared index without an ANN layer.

CLAIM [CONFIRMED — and a real bug] Default embedding model + dims are mis-documented.
- icm-core/src/fastembed_embedder.rs:36 `DEFAULT_MODEL = "intfloat/multilingual-e5-base"`,
  but the doc comment at :35 says "multilingual-e5-small (384d)". `model_dimensions`
  (:58-65) maps `MultilingualE5Base => 768`, so the real default-with-embeddings dim is
  **768**, not 384. The cartographer's "384d" is the *no-embeddings fallback*
  (icm-core/src/lib.rs:18 `DEFAULT_EMBEDDING_DIMS = 384`, used only when the embedder is
  absent — icm-cli/src/main.rs:1087-1093). The vec0 table is created at the *runtime*
  embedder dim, not 384 (schema.rs:17-37 `create_vec_table`, dim passed from
  main.rs:1087). So storage is correct; the comment + the const name are misleading.

CLAIM [CONFIRMED] The dim-mismatch class (model swap) is handled defensively.
- schema.rs:416-464 — on dim change the vec table is dropped, `memories.embedding`
  NULLed, table recreated, plus an idempotent self-healing sweep that NULLs any blob whose
  byte length != dims*4 (issue #200). Tested at schema.rs:698-791.

CLAIM [CONFIRMED] sqlite-vec is pinned at the version named in the advisory.
- Cargo.lock: `sqlite-vec 0.1.6`; rusqlite `0.34.0` with `bundled` (a real C/SQLite
  compile dependency — relevant to the union's "no C in the trust boundary" invariant);
  fastembed `4.9.1`. The 0.1.6→0.1.9 advisory (DELETE-on-long-metadata vec0) is partly
  mitigated because icm's vec0 carries only `memory_id` + `embedding` (schema.rs:24-29) —
  no long metadata columns — but the forget/prune/consolidate paths DO
  `DELETE FROM vec_memories` (store.rs:1316, :1384), so currency still matters.

UPGRADE [axis: memory-vector] Bump sqlite-vec to >=0.1.9 and switch the blend to RRF.
- rationale: clears the DELETE-bug advisory on the exact pinned version; RRF is the 2026
  hybrid-retrieval default and removes the zero-arm under-ranking artifact.
- evidence: pin at Cargo.lock sqlite-vec 0.1.6; linear blend at store.rs:1205-1211;
  delete paths at store.rs:1316,1384.
- blast: low (version bump) + medium (ranking change behind a recall differential test).
- risk: low — additive; keep brute-force KNN (adequate at personal scale).

UPGRADE [axis: memory-vector] Fix the default-model doc/const drift.
- rationale: a reader trusting the comment/const will mis-size an external vec index or
  mis-assume 384d; the real default is 768d (e5-base).
- evidence: fastembed_embedder.rs:35-36 comment vs const vs model_dimensions :58-65.
- blast: trivial (comment + naming).  risk: low.

## 3. Convergence — icm vs handoff ledger, git-kb, rusty-idd intent (canonical vs peer)

CLAIM [CONFIRMED] icm has ZERO code coupling to the handoff witnessed ledger.
- `grep -ril icm handoff/src handoff/crates` → no matches. handoff's
  `.handoff/context/capsule.json` (handoff/.handoff/context/capsule.json, 904B) is
  rendered from the witnessed ledger; it does not read or write icm, and icm does not
  read it. They are disjoint data planes.

CLAIM [CONFIRMED] icm binds into the union only ad-hoc via CLI/MCP, never as data.
- harness_hub/harness/skills/icm-memory/SKILL.md:9,70-73 — the harness memory skill is a
  "Graceful no-op if ICM isn't installed"; it shells `icm recall`/`icm store`
  (SKILL.md:26-28,55) and skips when `command -v icm` is empty. So icm is an *optional
  sidecar*: the loop calls it opportunistically; nothing in the union's authoritative
  state (ledger, OpenSpec/.idd artifacts, .kb graph) depends on icm rows existing.

CLAIM [CONFIRMED] The three planes are genuinely distinct data, not duplicative.
- icm = prose/semantic episodic memory (decisions, errors-resolved, preferences) →
  SQLite memories + vec0 (icm-store/src/schema.rs:50-72, vec0 :24-29).
- git-kb code intelligence = AST/call-graph (callers/callees/impact/symbols), a *code*
  graph keyed on symbols, per .claude/rules/code-intelligence.md — orthogonal to icm's
  natural-language memory; no overlap in content.
- handoff ledger = witnessed, append-only continuity events rendered into packets
  (handoff is the north-star continuity organ); rusty-idd = intent artifacts (OpenSpec /
  .idd). icm stores *why/what-we-learned*, handoff stores *what-happened-witnessed*,
  rusty-idd stores *what-we-intend*, git-kb stores *what-the-code-is*. Minimal semantic
  overlap; the risk is narrative duplication (a "decision" could land in both an icm
  `decisions-*` topic AND a handoff packet) without a canonical pointer.

RECOMMENDATION (canonical-vs-peer boundary): icm is the **canonical persistent-MEMORY +
semantic-vector plane** and should stay so; it is a **peer** (not a parent, not a child)
of git-kb (code graph) and handoff (witnessed ledger). The union should NOT fold memory
into the ledger nor vice-versa. The missing seam is a *one-directional, non-authoritative
bind*: handoff packets / rusty-idd artifacts may carry an icm memory-id reference
(provenance pointer), so a decision recorded in the ledger can `recall` its richer icm
context — but the ledger remains source-of-truth and never depends on icm being present.
This keeps icm canonical-for-memory without making it a single point of failure for the
union (preserving the harness's graceful-no-op portability).

GAP: there is no provenance pointer from ledger/intent artifacts back to icm memory-ids;
convergence today is "two systems that happen to be invoked in the same loop."

UPGRADE [axis: memory-vector] Add a non-authoritative icm memory-id reference field to
handoff packets / rusty-idd intent artifacts (pointer, not dependency).
- rationale: binds icm into the union AS DATA (cited provenance) while preserving
  portability and keeping the ledger authoritative — the "canonical memory, peer plane"
  boundary made concrete.
- evidence: zero coupling today (grep icm in handoff = none); optional-sidecar contract
  at icm-memory/SKILL.md:70-73.
- blast: medium — additive optional field on the packet/artifact schema; icm unchanged.
- risk: low — pointer is nullable; absence degrades to today's behavior.

## 4. Memory-poisoning / provenance / sanitization on the ingest path

CLAIM [CONFIRMED] Structural sanitization exists; semantic/trust sanitization does NOT.
- icm-store/src/store.rs:677-705 `validate_and_normalize` — trims topic, rejects empty
  topic/summary, NUL bytes, and newline/CR/tab in topic (display-spoofing guard); caps
  topic at 256B (MAX_TOPIC_BYTES :658) and summary at 64KB (MAX_SUMMARY_BYTES :653).
- store.rs:603-642 `sanitize_fts_query` neutralizes FTS5 operator injection and caps
  query length/token count. These are real abuse mitigations.
- BUT the *content* is ingested as fact verbatim: any agent/hook can `icm store` arbitrary
  text that a later `recall` injects into a prompt (the SessionStart `recall-context`
  hook, icm-memory/SKILL.md:36). This is precisely the May-2026 memory-poisoning surface.

CLAIM [CONFIRMED] Provenance is RECORDED but NOT used for trust weighting.
- icm-core/src/memory.rs:130-151 `MemorySource {ClaudeCode{session_id,file_path},
  Conversation{thread_id}, Manual}` is persisted (schema.rs:64-65 source_type/source_data)
  — so origin is auditable. However recall ranking (store.rs:1153-1225) ignores source
  entirely; a `Manual`/unknown-origin poisoned row ranks identically to a witnessed one.
  There is no signing/witnessing analogous to the handoff ledger.

GAP: no provenance-weighted recall, no admission policy on *who*/*what* may store at a
given importance, no content quarantine/review tier for untrusted ingest.

UPGRADE [axis: memory-vector] Provenance-aware recall + admission policy.
- rationale: closes the poisoning gap without blocking ingest — rank/penalize by
  `MemorySource` trust, optionally cap importance for non-witnessed sources, and borrow
  handoff's witnessing as the trust signal for high-importance memories.
- evidence: source recorded (memory.rs:130-151) but unused in ranking
  (store.rs:1205-1211); verbatim ingest at validate_and_normalize store.rs:677-705.
- blast: medium — recall ranking + a store-time policy hook; schema already has source.
- risk: medium — over-penalizing legitimate Manual stores; ship behind config with the
  current behavior as default-on for trusted sources.

---

## Summary rows
- icm = canonical MEMORY/semantic-vector plane; peer (not parent) of git-kb code graph &
  handoff ledger. Keep it canonical for memory; bind via a provenance pointer, not a hard
  dependency (preserve graceful-no-op portability).
- Memory model: decay + consolidation + forgetting are REAL but crude — static importance,
  recall-event-quantized 0.95 decay, no episodic/semantic/procedural tiering vs 2026 A-MAC.
- RAG path: hybrid FTS+vec0 cosine, linear 0.3/0.7 (not RRF), brute-force KNN; default
  model is e5-base **768d** (comment/const wrongly say 384d — doc-drift bug).
- Currency: sqlite-vec pinned 0.1.6 (advisory version); rusqlite 0.34 bundled = C dep;
  dim-swap handled defensively (issue #200 sweep, schema.rs:447-464).
- Poisoning: structural sanitization solid; provenance recorded but NOT trust-weighted in
  recall — the open attack surface; fix with provenance-aware recall + admission policy.

Confidence: high on code-grounded claims (every row cites file:line in icm source +
Cargo.lock); medium on the convergence recommendation (a design boundary, not a fact).
N/A — no rusty-idd↔icm code coupling exists to cite ("zero matches" is itself the finding).
