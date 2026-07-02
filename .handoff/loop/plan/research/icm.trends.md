# icm — Trends & Tool-Currency Research (plan cycle 7)

- **Target:** `icm` (Infinite Context Memory) — Rust persistent agent-memory: SQLite-backed store + vector search + embeddings + MCP server + CLI.
- **Researcher:** plan-trend-researcher
- **Date generated:** 2026-06-27
- **Recency window (90d):** 2026-03-29 → 2026-06-27 (computed from today; not hardcoded). In-window sources preferred; older sources are flagged and justified.
- **Method:** fan-out web search → fetch primary sources → adversarial cross-check → cited + dated synthesis.
- **icm pins read from** `/home/drdave/Desktop/meta/icm/Cargo.toml` + `Cargo.lock` (workspace deps and resolved versions). These are facts about the target, used as the currency baseline.

## Headline

1. **Two live currency gaps with a security edge.** icm's pinned `rusqlite 0.34.0` / `libsqlite3-sys 0.32.0` bundle **SQLite 3.49.1** (2025-02-18). Upstream SQLite has shipped **four CVE fixes since 3.49.1**, including an **in-window** FTS5 heap-write overflow fixed only in **3.53.2 (2026-06-03)**. Because these are bundled-C CVEs with **no matching RustSec advisory**, `cargo audit` reports the tree **clean** — audit-green does **not** mean the bundled SQLite is patched. This is the most important finding for the architect's tool-evaluation.
2. **All three "memory-stack" crates are one stable step behind.** `sqlite-vec` 0.1 (resolved **0.1.6**) vs stable **0.1.9** (2026-03-31, in-window) — and 0.1.9 specifically fixes a **DELETE bug on `vec0` tables with long (>12-char) text metadata columns**, which is exactly icm's shape. `fastembed` 4 (resolved **4.9.1**) vs **5.17.2** (2026-06-15, in-window). `ureq` 2 (resolved **2.12.1**) vs **3.3.0** (current series).
3. **icm's memory model is sound but uses a now-superseded *static* importance scheme.** The 2026 field consensus: importance/decay should be **dynamic and outcome-updated** (Ebbinghaus/Weibull decay, multi-dimensional admission control), not assigned once at write time. icm's `critical/high/medium/low` levels are set at `store` time — aligned in spirit, trailing the frontier in mechanism.

---

## 1. Agent long-term memory architecture (field scan)

- **C1 — Three-tier (episodic / semantic / procedural) is the established 2026 pattern**, with a *consolidation pipeline* (episodic → distilled into semantic → repeated patterns automated into procedural). icm models `topics` + `importance` + `recall/store` + memoir/concepts, but does **not** expose explicit episodic/semantic/procedural tiers; its `memoir`/`concepts` and `extract_patterns`/`consolidate` MCP verbs map loosely to the consolidation idea. [S1, S2] *(S2 in-window 2026-04; S1 dated "2026" only — corroborated by S2.)*
- **C2 — Intelligent forgetting beats full retention.** Formalized policies: FIFO, LRU, **Priority-Decay** (weight by importance, decay the score over time). Benchmarks (FiFA) show naive "keep everything" *degrades* task scores vs structured forgetting; best hybrid ≈ 0.911 composite. icm has `forget` / `forget_topic` / importance levels — directionally correct. [S3, S4, S7] *(S3 in-window 2026-04.)*
- **C3 — Importance scoring is moving from static→dynamic (a gap for icm).** Early systems (icm-style) assign importance **once at write time**; 2026 work updates it by outcome and decays via **Ebbinghaus / Weibull** curves. A-MAC (Zhang et al., 2026) scores candidate memories on **five dimensions** (utility, confidence, novelty, recency, type-prior) before admission. icm's `critical/high/medium/low` is write-time-static → candidate upgrade. [S2, S4, S5] *(S5 arxiv 2603 = March 2026, flagged **just-older** — pre-window-start; used only to corroborate the dynamic-decay direction already shown in-window by S2/S4.)*
- **C4 — Memory is now an attack surface (new-but-real signal).** "Memory poisoning" reached an inflection in **May 2026**; governance primitives (provenance, right-to-be-forgotten, admission gating) are emerging best-practice. Directly relevant: icm ingests **arbitrary stored text**, then runs it through SQLite FTS5 + vector recall — see the bundled-SQLite FTS5 CVE below. [S6, S3] *(S6 in-window 2026-05.)*
- **C5 — Dedicated memory frameworks + benchmarks have matured; the value prop is proven for coding agents.** Mem0 (personalization), Zep (temporal KG), Letta (long-running), MemMachine; benchmarks LoCoMo / LongMemEval / BEAM. **GitHub Copilot's Jan-2026 production A/B reported ~+7pt PR-merge rate with agentic memory on.** This validates icm's role as `meta`'s memory organ; the comparators set the feature bar (temporal/KG edges, multi-scope). [S7, S8, S9, S10] *(dated "2026", vendor/secondary — treated as signal, cross-corroborated across 4 sources.)*

> Net for the architect: icm's model (topics, importance/decay, recall/store, consolidate/extract_patterns, memoir/concepts) is a credible local-first instance of the 2026 pattern. The two frontier deltas worth a roadmap row: (a) **dynamic/outcome-updated importance + time-decay** (vs static levels), and (b) **memory-admission + provenance** hardening (poisoning surface).

## 2. Vector search in SQLite

- **C6 — `sqlite-vec` is active again in 2026; icm is one stable step behind.** Latest **stable 0.1.9 (2026-03-31)**; the **0.1.10-alpha** line (Apr–May 2026) adds **ANN indexes (DiskANN, rescore)** — stable is still **brute-force**. icm pins `"0.1"` → **resolved 0.1.6**. [S11] *(in-window.)*
- **C7 — 0.1.9 fixes a bug that matches icm's exact data shape.** Release notes: `DELETE` on `vec0` tables with **long (>12-char) text metadata columns** erroneously reported `SQLITE_DONE`. icm stores memories with text metadata alongside vectors → on 0.1.6 it lacks this fix. Low-risk patch bump. [S11] *(in-window.)*
- **C8 — Alternatives & trade-offs for a local-first agent memory:**
  - **libSQL / Turso native vectors** — zero-setup, in-SQL, DiskANN-indexed, auto-maintained on insert; but index build can run **hours** on typical workloads (optional index mitigates). [S12, S14]
  - **`sqlite-vector` (sqliteai)** — cross-platform, no virtual tables, ~3× faster queries with quantization, "perfect recall", ~30MB default memory. Positioned as the brute-force successor for "few-million-vector" local apps. [S12, S13]
  - **LanceDB** — IVF-PQ (approximate), fastest at scale, columnar/lakehouse; heavier than an embedded extension for icm's local single-file model. [S15]
  - **Qdrant / pgvector** — server-process or Postgres dependency; contradicts icm's local-first single-binary design.

  > For icm the like-for-like choices are stay on **sqlite-vec** (bump to 0.1.9, optionally pilot 0.1.10 ANN when stable) or evaluate **sqlite-vector** if recall/latency on larger stores becomes a bottleneck. *(C8's framing leans on S12, dated **2025-09-01 — older, out-of-window**; flagged. Still authoritative: it is the canonical state-of-the-field comparison and its claims are corroborated by the live S13/S14/S15 product sources and the in-window S11 release data.)*

## 3. Embeddings for local memory

- **C9 — `fastembed`-rs is at 5.17.2 (2026-06-15); icm pins `4` (resolved 4.9.1) — one major behind.** v5 keeps the **synchronous, no-Tokio, offline-after-first-download ONNX** model and adds **Qwen3** (feature flag), **Nomic-embed-text-v2-MoE** (first general-purpose MoE embedder, 100+ languages), **EmbeddingGemma** (incl. a 4-bit `…Q4` build), and **BGE-M3** joint dense+sparse+ColBERT. Quantized model variants (append `Q`) reduce footprint — relevant since icm's embeddings feature is optional/on-device. [S16, S17] *(in-window.)*
  - Migration note for the architect: 4→5 is a major bump (model-enum/API churn likely); benefit is newer/smaller models + multilingual MoE, not a correctness fix — schedule as quality, not urgent.

## 4. MCP memory servers (landscape)

- **C10 — The MCP memory landscape splits into (a) the official knowledge-graph reference server and (b) local SQLite-backed memory servers; icm sits squarely in (b).** The official `@modelcontextprotocol/server-memory` is **knowledge-graph** (entities/observations/relations). Independent servers (e.g. `ai-memory`) use a **local SQLite DB, relevance-ranked recall, and tiered promotion (short/mid/long-term)** — the same shape as `icm-mcp` (SQLite + vector + importance tiers). Convention signal icm could adopt: **explicit entity/relation edges** (the KG axis Zep/official-server expose) on top of its topic/concept model. [S18, S19] *(S18 live reference repo, in-window; S19 dated "2026".)*

## Tool-currency & advisories

Baseline = icm's pinned/resolved versions (from its `Cargo.toml` + `Cargo.lock`). Window 2026-03-29 → 2026-06-27.

| Dependency | icm pin → resolved | Current (date) | Drift | Advisory / note |
|---|---|---|---|---|
| `rusqlite` | `0.34` → **0.34.0** | **0.40.1** (2026-06-06) | ~6 minor behind | bundles SQLite **3.49.1** vs current **3.53.2** — see C12 [S20, S21] |
| `libsqlite3-sys` | `0.32.0` | **0.38.1** | behind | bundles SQLite 3.49.1; RustSec has only RUSTSEC-2022-0090 (not applicable) [S21, S23] |
| bundled **SQLite (C)** | **3.49.1** (2025-02-18) | **3.53.2** (2026-06-03) | 4 patched CVEs ahead | **see C12 — exposure** [S22] |
| `sqlite-vec` | `0.1` → **0.1.6** | **0.1.9** (2026-03-31) | 3 patches behind | no advisory; 0.1.9 fixes long-text-metadata DELETE bug (C7) [S11, S28] |
| `fastembed` | `4` → **4.9.1** | **5.17.2** (2026-06-15) | 1 major behind | no advisory; quality/model upgrade (C9) [S16, S28] |
| `ureq` | `2` → **2.12.1** | **3.3.0** (2026-03-21) | 1 major behind | 2.x effectively superseded by 3.x; no open advisory found [S25, S26, S28] |
| `clap` | `4` → **4.5.60** | 4.5.x current | current | healthy, no advisory [S28] |
| `serde` / `serde_json` | `1` → **1.0.228 / 1.0.149** | current | current | healthy, no advisory [S28] |
| `ulid` | `1` → **1.2.1** | current | current | no advisory found [S28] |

- **C11 — rusqlite/libsqlite3-sys are materially behind.** icm `rusqlite 0.34.0` / `libsqlite3-sys 0.32.0` bundle **SQLite 3.49.1**; current `rusqlite 0.40.1` / `libsqlite3-sys 0.38.1` bundle **SQLite 3.53.2**. [S20, S21]
- **C12 — Bundled-SQLite CVE exposure (headline advisory).** SQLite CVEs fixed *after* 3.49.1, to which icm's bundled 3.49.1 is therefore exposed if the relevant features are compiled:
  - **CVE-2026-11822** — FTS5 **heap buffer write overflow** with arbitrary SQL exec when FTS5 enabled and `SQLITE_DBCONFIG_DEFENSIVE` disabled. **Fixed 3.53.2 (2026-06-03, IN-WINDOW).**
  - **CVE-2025-7709** — FTS5 integer overflow → out-of-bounds access. Fixed 3.50.3 (2025-07-17).
  - **CVE-2025-6965** — integer overflow → array-read overflow. Fixed 3.50.2 (2025-06-28).
  - **CVE-2025-70873** — zipfile-extension OOB read. Fixed 3.52.0 (2026-03-06).
  - **Exposure assessment for icm:** the **FTS5** CVEs are the relevant ones for a text-memory store with full-text recall over **arbitrary ingested content** (ties to C4 memory-poisoning). Practical risk is **moderate-not-critical** for a local single-tenant store with parameterized SQL (no attacker-controlled SQL text), but the surface is real and the fix is a routine dependency bump. **Recommend: upgrade `rusqlite`→0.40.x / `libsqlite3-sys`→0.38.x (SQLite 3.53.2).** [S22, S21]
- **C13 — `cargo audit` is blind to this.** RustSec's `libsqlite3-sys` page lists **only RUSTSEC-2022-0090** (CVE-2022-35737, affects SQLite <3.39.2 — **not** 3.49.1). The 2025–2026 SQLite CVEs have **no RustSec advisory**, so `cargo audit` returns **clean** despite the bundled-C exposure. The architect should treat "bundled SQLite currency" as a **manual** check, not an audit-gated one. [S23, S24, S28]
- **C14 — `ureq` 2→3 drift, low urgency.** icm resolves `ureq 2.12.1`; the current series is **3.x (3.3.0, 2026-03-21, Rust-2024 edition)**, with 2.x in maintenance/superseded. No open advisory found. Used only for optional cloud-sync → defer to a quality bump. [S25, S26]
- **C15/C16 — Rest of the chain is current and advisory-free.** `clap 4.5.60`, `serde 1.0.228`, `serde_json 1.0.149`, `ulid 1.2.1` are current; no RustSec advisories found for `sqlite-vec`, `fastembed`, `ulid`, `clap`, or `serde`. [S28]

## Confidence & gaps

- **High** confidence on tool-currency/version/CVE facts (primary sources: docs.rs/lib.rs version pages, sqlite.org CVE list, sqlite-vec GitHub releases, RustSec).
- **Medium** confidence on the agent-memory *frontier* framing (C3/C5): several supporting sources are vendor blogs/dated-"2026"-only; cross-corroborated but treat as direction, not benchmark-grade.
- **Gaps:** could not confirm whether icm compiles SQLite **with FTS5** (would sharpen the C12 exposure call — the cartographer/analyst should check the `modern_sqlite`/bundled feature set and any FTS5 usage in `icm-store`). Could not pin an exact publish date for several 2026-dated blogs (S1, S4, S7–S10) → conservatively marked out-of-window.

## Sources

(Full machine-readable ledger with per-source claim mapping in `sources-icm.jsonl`.)

- **S1** AppScale Blog — *Agent Memory Architecture (2026): Episodic/Semantic/Procedural Three-Tier* — https://appscale.blog/en/blog/agent-memory-architecture-episodic-semantic-procedural-the-three-tier-pattern-2026 (2026; out-of-window)
- **S2** Analytics Vidhya — *Architecture and Orchestration of Memory Systems in AI Agents* — https://www.analyticsvidhya.com/blog/2026/04/memory-systems-in-ai-agents/ (2026-04; in-window)
- **S3** arXiv 2604.12007 — *When to Forget: A Memory Governance Primitive* — https://arxiv.org/html/2604.12007 (2026-04; in-window)
- **S4** Fazm Blog — *Memory Triage for AI Agents — Why 100% Retention Is a Bug* — https://fazm.ai/blog/ai-agent-memory-triage-retention-decay (2026; out-of-window)
- **S5** arXiv 2603.11768 — *Governing Evolving Memory in LLM Agents (SSGM)* — https://arxiv.org/pdf/2603.11768 (2026-03; just-older, flagged)
- **S6** LLMS3 — *When Memory Became the Attack Surface: The May 2026 AI Agent Security Inflection* — https://llms3.com/blog/when-memory-became-the-attack-surface-may-2026 (2026-05; in-window)
- **S7** Mem0 — *State of AI Agent Memory 2026* — https://mem0.ai/blog/state-of-ai-agent-memory-2026 (2026; vendor; out-of-window)
- **S8** Mem0 — *AI Memory Benchmarks 2026: LoCoMo, LongMemEval & BEAM* — https://mem0.ai/blog/ai-memory-benchmarks-in-2026 (2026; vendor; out-of-window)
- **S9** Dev Genius — *AI Agent Memory Systems in 2026: Mem0, Zep, Hindsight, Memvid Compared* — https://blog.devgenius.io/ai-agent-memory-systems-in-2026-mem0-zep-hindsight-memvid-and-everything-in-between-compared-96e35b818da8 (2026; out-of-window)
- **S10** Medium (M. Sandelin) — *The First Controlled Benchmark of AI Memory in Coding Agents* — https://medium.com/@mrsandelin/the-first-controlled-benchmark-of-ai-memory-in-coding-agents-8e0bb776d39e (2026; out-of-window)
- **S11** GitHub — *asg017/sqlite-vec releases* — https://github.com/asg017/sqlite-vec/releases (latest 0.1.9 2026-03-31, 0.1.10-alpha May 2026; in-window)
- **S12** Marco Bambini (Substack) — *The State of Vector Search in SQLite* — https://marcobambini.substack.com/p/the-state-of-vector-search-in-sqlite (2025-09-01; older, flagged authoritative)
- **S13** GitHub — *sqliteai/sqlite-vector* — https://github.com/sqliteai/sqlite-vector (live, active 2026)
- **S14** Turso — *Native Vector Search for SQLite (libSQL)* — https://turso.tech/vector (live product page)
- **S15** LanceDB — https://www.lancedb.com/ (live product page)
- **S16** Lib.rs — *fastembed* — https://lib.rs/crates/fastembed (latest 5.17.2 2026-06-15; in-window)
- **S17** GitHub — *Anush008/fastembed-rs* — https://github.com/anush008/fastembed-rs (v5 features; live)
- **S18** GitHub — *modelcontextprotocol/servers — memory (knowledge graph)* — https://github.com/modelcontextprotocol/servers/tree/main/src/memory (live)
- **S19** mcpservers.org — *ai-memory MCP Server (local SQLite, tiered)* — https://mcpservers.org/servers/alphaonedev/ai-memory-mcp (2026)
- **S20** docs.rs — *rusqlite (latest)* — https://docs.rs/crate/rusqlite/latest (0.40.1 2026-06-06, SQLite 3.53.2; in-window)
- **S21** docs.rs — *rusqlite 0.34.0 README* — https://docs.rs/crate/rusqlite/0.34.0/source/README.md (bundled SQLite 3.49.1; older but authoritative — it is icm's pinned version)
- **S22** SQLite.org — *CVE list* — https://www.sqlite.org/cves.html (CVE-2026-11822 fixed 3.53.2 2026-06-03; in-window)
- **S23** RustSec — *Advisories for libsqlite3-sys* — https://rustsec.org/packages/libsqlite3-sys.html (only RUSTSEC-2022-0090)
- **S24** RustSec — *RUSTSEC-2022-0090 (CVE-2022-35737)* — https://rustsec.org/advisories/RUSTSEC-2022-0090 (2023-02-14; affects SQLite <3.39.2 — N/A to 3.49.1)
- **S25** Lib.rs — *ureq* — https://lib.rs/crates/ureq (latest 3.3.0 2026-03-21; 2.x superseded)
- **S26** Rustify — *reqwest vs ureq vs hyper: Which Rust HTTP Client in 2026?* — https://rustify.rs/articles/rust-reqwest-vs-ureq-vs-hyper-2026 (2026)
- **S28** RustSec — *Advisory index* — https://rustsec.org/advisories/ (live; no current advisory for sqlite-vec/fastembed/ulid/clap/serde)
