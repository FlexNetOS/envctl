# Findings — memory-vector-intelligence — target: rusty-idd

Axis: `memory-vector-intelligence`. Target: `rusty-idd` (a fabric AXIS — the intent/control-plane
organ). Scope: how rusty-idd **stores and recalls knowledge** today; whether its own
`crates/knowledge` index duplicates the fleet **ICM**/**git-kb** organs; and the convergence path to
unified fabric **memory** (cross-organ **recall** across ICM / git-kb / **vector**).

Mode: READ-ONLY on target code; cites file:line. Markers present: memory, vector, git-kb, RAG, ICM,
handoff, recall.

---

## Verdict (one line)

rusty-idd runs a **fourth, parallel memory organ** of its own — the `.idd/knowledge` code-graph index
(a 47 MB committed `index.json`) — that **duplicates git-kb's code-intelligence function but with
NO embeddings/RAG, NO semantic recall, and NO ICM decision/error memory**; the fleet ICM memory organ
is **absent from product code** (appears only as a *described* harness-checker contract + a peer-repo
registry name + tests), and rusty-idd has **no `.kb/` of its own**, so it is not a first-class git-kb
member. Convergence target: make `.idd/knowledge` a *producer into* the fabric recall plane, not a
rival store.

---

## 1. Memory inventory

| surface | what it is | where | recall/store path |
|---|---|---|---|
| **`.idd/knowledge/` (rusty-idd's OWN store)** | code-graph + architecture/integration artifacts. `index.json` is **47 MB** (`-rw 47392622 .idd/knowledge/index.json`) and is **git-tracked** (`git ls-files .idd/knowledge/` lists `index.json` + 17 siblings). Schema = `KnowledgeIndex { schema_version, workspace_fingerprint, files, nodes, edges, imports, hotspots, failures }` (`crates/knowledge/src/lib.rs:147-158`). | `.idd/knowledge/*.json|*.md` | written by `refresh_workspace` (`crates/knowledge/src/lib.rs:1221`); read by `load_index` (`:1256`); recall is **exact-match only** via `query_knowledge_index` (`:1213`) over `KnowledgeQuery::{Symbol, File, Impact(u64)}` (`:140-144`). |
| **ICM (fleet memory organ)** | persistent cross-session decision/error/preference memory. **0 product references.** Appears only as: (a) a *described* verify-stage checker contract rusty-idd scaffolds — `"icm-checker"`, `"icm-recall-context-compare"`, `"icm-comparison-contract"` (`crates/cli/src/commands/harness.rs:208,219,233,254,265`); (b) a peer-repo capability **anchor** in the knowledge registry (`crates/knowledge/src/lib.rs:3643,3724-3727`); (c) test assertions (`crates/cli/tests/harness_cli.rs:119-155`). | none in product | **no `icm recall` / `icm store` call anywhere in `crates/`.** rusty-idd *names* ICM as a thing a downstream harness should consult, but never consults it. |
| **git-kb (fleet vector/code-intelligence organ)** | meta-root `.kb/` with embeddings ON (`/home/drdave/Desktop/meta/.kb/config.toml` → `[embeddings] enabled = true`, index at `.kb/cache/index`, store `.kb/.cache/gitkb.db`). | `meta/.kb/` (fleet-level) | rusty-idd has **NO `.kb/` of its own** (`ls .kb` → "NO .kb in rusty-idd"); `.gitignore:3` explicitly treats imported `.kb/.handoff` as out-of-tree. rusty-idd is therefore **not a separately-indexed git-kb member** — agents recall it only if the fleet daemon indexed the path. **0 `git_kb`/`.kb/` references in `crates/`.** |
| **handoff / `.handoff` continuity** | filesystem + schema contract (not a lib dep). `codex.rs` reads `.handoff/tasks` (`crates/cli/src/commands/codex.rs:593`); `crates/work-order` IS the `handoff.task.v1` envelope (`crates/work-order/src/lib.rs:35-45`, intake `intake.rs:178`). | `.handoff/`, `work-order` crate | no `hf` kernel lib/IPC dep — coupling is the file+schema contract only (matches codemap). |
| **ruvector (fleet vector runtime)** | referenced **only** as a peer-repo name / capability anchor: `capability:vector-runtime` "Vector and agentic runtime" `repo_names: ["ruvector","database_hub","icm"]` (`crates/knowledge/src/lib.rs:3639-3645`); drift-sentinel comment `ruvector-verified` (`crates/work-order/src/lib.rs:7,71`). | none in product | no integration — descriptive registry data only. |
| **vendored codegraph vector subsystem** | `codegraph-core` ships a full vector store (`integration/graph_vector.rs` `GraphVectorIntegrator`, `store_embeddings`, `InMemoryVectorStore` — `:295,328,338,471,546`; trait `store_embeddings` `traits.rs:13`; `embedding_config` module `lib.rs:9`). | `crates/external/codegraph-core/` | **dead in product**: `knowledge` depends on it `default-features = false` and imports **only parsing types** (`CodeNode, EdgeRelationship, EdgeType, Language, NodeType` — `crates/knowledge/src/lib.rs:7-12`); **0** refs to `store_embeddings|VectorStore|GraphVector|embed|semantic_search|cosine` in `knowledge/src/lib.rs`; `graph_vector`/`GraphVectorIntegrator` have **0 importers outside `external/codegraph-core`**. |

**Recall/store hooks in discipline docs:** `AGENTS.md:24` ("Before implementation, create or refresh
the relevant `.idd/knowledge/*` … `rusty-idd knowledge plan-context`"), `:28` (validation must refresh
`.idd/knowledge/*`), `:32` (use `report.md`/`architecture.md`/`plan-context.md` before manual
rescans). `codex.rs` lists the knowledge artifacts as the agent-facing context bundle
(`crates/cli/src/commands/codex.rs:190-204,324,399,497`). This is rusty-idd's **only** documented
recall protocol — and it points exclusively at its own `.idd/knowledge`, never at ICM or git-kb.

---

## 2. Vector intelligence map

| index | exists? | freshness | owner | update command | failure behavior |
|---|---|---|---|---|---|
| `.idd/knowledge/index.json` (code graph) | **yes** — but it is a **symbol/edge graph, not a vector index**. No embeddings, no nearest-neighbor; recall is exact `Symbol`/`File`/`Impact` lookup (`crates/knowledge/src/lib.rs:1213`, `1024`-region query, CLI `Query` `crates/cli/src/commands/knowledge.rs:499-501`). | committed blob, regenerated on demand (HEAD `5a55284` 2026-06-26). No staleness gate — `workspace_fingerprint` field exists (`:149`) but nothing in product compares it before serving a stale `load_index`. | `crates/knowledge` (`rusty_idd_knowledge`), blast 105 (codemap). | `rusty-idd knowledge refresh` → `refresh_workspace` (`crates/cli/src/commands/knowledge.rs:505`, lib `:1221`). | `load_index` fails closed on missing/corrupt file (`:1256-1262` `with_context`); but a **stale-but-parseable** index is served silently (no fingerprint check) — RAG-equivalent staleness risk. |
| embeddings / RAG / vector DB (semantic recall) | **NO** — feature flags `knowledge-vector`, `knowledge-surrealdb`, `knowledge-cloud` exist (`crates/knowledge/Cargo.toml:36-39`) but are **pure placeholders**: `grep cfg(feature="knowledge-vector"…)` over `crates/` returns **nothing** — no code is gated on them. | N/A — does not exist. | N/A | N/A | N/A — there is no semantic/RAG recall surface in rusty-idd; the vendored codegraph vector store (§1) is the only candidate and is dead. |
| fleet git-kb embeddings index (`meta/.kb/cache/index`) | yes, fleet-level (`meta/.kb/config.toml [embeddings] enabled=true`) | fleet-owned | meta/git-kb daemon | `git kb index <path>` (per code-intelligence rules) | rusty-idd does not own or update it; no in-repo `.kb/`. |

**Net:** rusty-idd's "knowledge" is **graph intelligence, not vector intelligence**. The codemap's
"vector intelligence" expectation maps to a capability that is **declared (feature flags, vendored
crate) but never wired**.

---

## 3. Recall guarantees

| guarantee | status | evidence |
|---|---|---|
| session-start recall | **partial / manual** — `AGENTS.md:24,32` instruct agents to read `.idd/knowledge/*` first; no hook enforces it. No ICM `recall` at session start (ICM absent, §1). | `AGENTS.md:24,32`; 0 icm calls in `crates/`. |
| background-agent recall | **N/A — no live recall API.** Background agents get a *file bundle* (codex.rs context list `crates/cli/src/commands/codex.rs:190-204`), not a query interface; no ICM/git-kb recall surface is invoked. | `codex.rs:190-204` |
| wrap-up store | **N/A — no store path.** rusty-idd persists graph artifacts via `refresh_workspace` (deterministic regeneration), but has **no decision/error/preference store** (ICM's job, absent). Durable continuity is delegated to `.handoff`/`work-order` envelopes, not to a memory organ. | `crates/knowledge/src/lib.rs:1221`; `crates/work-order/src/lib.rs:35-45` |
| cold-start resume proof | **graph-only.** A fresh agent can rebuild full code/architecture context from `.idd/knowledge` (committed) + OpenSpec — but the *why* (decisions, resolved errors, gotchas) is **not** captured anywhere recallable; it would live in ICM, which rusty-idd never writes. | `git ls-files .idd/knowledge`; absence of ICM store |
| "no plan depends on chat memory alone" | **met for code/architecture facts** (committed `.idd/knowledge`), **NOT met for decision/error memory** (no ICM, no semantic recall). | combined above |

**Duplication assessment (key question):** `crates/knowledge`'s `index.json` **duplicates git-kb's
code-graph function** (symbols/edges/impact — git-kb `kb_callers`/`kb_impact` cover the same shape) but
is a *separate, divergent* store with **no embeddings** (git-kb has them) and **no semantic recall**.
It does **not** duplicate ICM at all (ICM = decision/error/preference memory, which rusty-idd has zero
of). So the fleet has, for rusty-idd: a redundant graph index, a missing embeddings/RAG plane, and a
missing decision-memory plane.

---

## 4. Upgrade rows (axis · evidence · risk-tier · acceptance-criterion · reversibility)

| # | upgrade | axis · evidence | risk-tier | acceptance-criterion | reversibility |
|---|---|---|---|---|---|
| U1 | **De-duplicate code-graph: make `.idd/knowledge` a *projection* of the fabric graph organ (git-kb), not a rival.** Either (a) emit `index.json` from / reconcile it against git-kb, or (b) register rusty-idd as a first-class git-kb member so `kb_*` recall covers it. | memory-vector-intelligence · §3 duplication; `crates/knowledge/src/lib.rs:147-158` vs `meta/.kb/config.toml`; no in-repo `.kb/` | medium (knowledge blast 105) | a single documented source-of-truth for rusty-idd's code graph; `kb_callers`/`kb_impact` and `rusty-idd knowledge query` return consistent symbol/edge sets for ≥1 golden symbol. | high — additive reconciliation; existing `index.json` retained until parity proven. |
| U2 | **Stop committing the 47 MB `index.json`; add a freshness gate.** Move generated `index.json` out of git (it's a 47 MB blob at `git ls-files .idd/knowledge/`) OR gzip+LFS it; enforce `workspace_fingerprint` (`:149`) check in `load_index` so a stale index fails closed instead of serving silently (§2). | memory-vector-intelligence · `ls -la` 47392622 bytes; `:1256-1262` no fingerprint check | low | repo no longer carries a 47 MB plaintext blob; `query`/`load_index` refuses a fingerprint-mismatched index with a clear error. | high — gitignore + a guard fn; recoverable by `knowledge refresh`. |
| U3 | **Add real semantic/RAG recall — wire the vendored codegraph vector store behind `knowledge-vector` (currently an empty flag) OR delegate to git-kb embeddings.** Today `knowledge-vector` gates nothing (`Cargo.toml:36`, 0 `cfg` sites) and `graph_vector.rs`/`store_embeddings` are dead (§1). | memory-vector-intelligence/vector · `crates/knowledge/Cargo.toml:36`; `crates/external/codegraph-core/src/integration/graph_vector.rs:295,338`; 0 importers | medium-high (introduces async + embedding deps, NO-C trust-boundary review for any native embedder) | `rusty-idd knowledge query --semantic "<nl>"` returns ranked symbols/files; OR a documented decision to delegate RAG to git-kb (the empty placeholder flags removed). | medium — feature-gated; off by default. |
| U4 | **Make ICM recall/store a real seam, not just a *described* checker.** rusty-idd already *names* ICM in its harness contract (`harness.rs:208,233`) and lists it as a capability anchor — close the loop so the control plane can recall prior decisions/errors and store the *why* of each merge/spec at wrap-up. | memory-vector-intelligence/ICM/handoff · `crates/cli/src/commands/harness.rs:208,219,233`; 0 `icm` calls in product | medium | at least one product path performs `icm recall` before planning and `icm store` on a completed spec/merge (or a deliberate ADR records why rusty-idd stays ICM-free and routes memory through `.handoff`). | high — additive, gracefully no-ops if ICM absent (per icm-memory skill contract). |
| U5 | **Unify the three planes behind one recall facade (the convergence path).** Define a fabric-recall contract that fans out to git-kb (code/vector), ICM (decisions/errors), and `.idd/knowledge` (architecture/integration), so any agent gets one `recall` across all organs instead of three disjoint stores. | memory-vector-intelligence · §1 three-store split; codemap "THREE memory systems … no unified recall" | high (cross-organ contract) | a single recall entry point returns merged, provenance-tagged hits from ≥2 organs for a test query; documented as an ADR. | medium — facade is additive; underlying stores untouched. |

---

## 5. Gate handoff (fail-closed artifact/test additions)

So missing memory/vector surfaces fail closed rather than silently degrade:

1. **Freshness gate (RED test):** `load_index` must reject a `workspace_fingerprint` mismatch
   (`crates/knowledge/src/lib.rs:149,1256`). Add a unit test that mutates the workspace, calls
   `query` against a stale `index.json`, and asserts a typed staleness error — currently it would
   silently serve stale graph data.
2. **Blob-size gate (CI):** add a check that fails CI if any tracked `.idd/knowledge/*.json` exceeds
   a threshold (today `index.json` = 47 MB, tracked). Forces U2.
3. **Placeholder-feature gate:** assert that `knowledge-vector`/`knowledge-surrealdb`/`knowledge-cloud`
   either gate real code (`cfg(feature=…)` sites > 0) or are removed — prevents capability flags that
   imply a vector/RAG plane that does not exist (`crates/knowledge/Cargo.toml:36-39`).
4. **ICM-contract honesty gate:** if `harness.rs` continues to *declare* `icm-checker` /
   `icm-recall-context-compare` (`crates/cli/src/commands/harness.rs:208,233`), CI should assert that
   either ICM is actually invokable from a product path OR the contract is marked advisory-only — so
   the scaffolded harness contract can't claim ICM coverage that the engine never exercises.
5. **Recall-provenance gate:** any future unified recall (U5) must tag each hit with its organ
   (git-kb / ICM / `.idd/knowledge`) and fail closed if an organ it claims to cover is unreachable.

---

## Confidence

High on the inventory facts (every row cites a path/line, grep counts, `git ls-files`, and config
files). Medium on the *fleet* embeddings claim for git-kb (read from `meta/.kb/config.toml`, not from
rusty-idd code — rusty-idd itself has no `.kb/`). The "ICM absent from product" and "vendored vector
store dead in product" claims are strong: 0 call sites under non-test, non-`external/` `crates/`.

Artifact: `/home/drdave/Desktop/meta/.worktrees/plan-fleet-convergence/envctl/.handoff/loop/plan/findings/memory-vector-intelligence-rusty-idd.md`
