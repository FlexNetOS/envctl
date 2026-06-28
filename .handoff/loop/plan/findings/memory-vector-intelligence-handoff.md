# Findings — memory-vector-intelligence — target: handoff (cycle 2, union with rusty-idd)

Axis: `memory-vector-intelligence`. Target: **handoff** — the fleet's primary **operational-truth
memory organ** (the witnessed `.handoff` ledger + `hf`). Scope: what handoff stores/recalls today; what
RuVector/RVF actually provides as the in-tree **vector** plane; whether that is the fleet's vector-memory
organ; and how the union reconciles handoff's ledger+RVF with rusty-idd's 47 MB `.idd/knowledge` store,
the fleet **ICM** decision-memory organ, and the fleet **git-kb** code-intelligence organ.

Mode: READ-ONLY on target code; every CLAIM cites file:line. Worktree:
`/home/drdave/Desktop/meta/.worktrees/plan-handoff-cycle2/handoff` @ `f6abf96`.
Markers present: memory, vector, git-kb, RAG, ICM, handoff, recall.

---

## Verdict (one line)

handoff is a **real, durable memory organ** — a witnessed append-only event ledger whose **committed
truth is a text JSONL** (`.handoff/ledger.events.jsonl`) re-derived fail-closed into a gitignored redb
cache — and, unlike rusty-idd, it **is a first-class git-kb member** (committed `.kb/`) and **drives
git-kb** for doc-sync; **BUT** its advertised "RVF semantic recall" **vector** plane is *built, paid-for
on every write, and never read*: `query_by_intent` has **zero callers**, there is **no `hf recall`
verb**, the embeddings are **hash-based pseudo-embeddings (SHA3-256), not learned** (so "semantic" is a
misnomer), and the RuVector path dep is **unresolved in the worktree** — while **ICM** (the fleet's
decision/why memory) is **entirely absent from product code (0 refs)**. The witness chain is **SHAKE-256
hash-linked, not blake3+ed25519-signed** (ed25519 is an *optional, un-wired* rvf-crypto capability).

---

## 1. Memory inventory

| surface | what it is | where | recall/store path |
|---|---|---|---|
| **`.handoff/ledger.events.jsonl` (THE committed continuity truth)** | deterministic text export of the witnessed ledger — one JSON object per event, seq order, git-diffable. The binary redb store is a **gitignored local rebuild cache**. | committed (`git ls-files` → `.handoff/ledger.events.jsonl`); cache `.handoff/**/ledger.db` ignored (`.gitignore:11`) | STORE: `export_jsonl`; RECALL/cold-start: `rebuild_from_jsonl` re-appends every event through the authoritative `Ledger::append` and **fails closed if the rebuilt witness chain count differs** (`ledger/src/export.rs:1-13`, `lib.rs:32-33`). |
| **redb authoritative event store (`v1`)** | pure-Rust ACID single-writer serializable KV; witnessed append-only hash-chain, replay, atomic lease CAS, rollup provenance. `redb = { version="4", optional=true }` behind `redb-store` (`ledger/Cargo.toml`). | `.handoff/ledger.db` (local cache, gitignored) | STORE: `Ledger::append` (`ledger/src/v1.rs:497`). RECALL: `replay_latest_status` (`ledger/src/v1.rs:828`) — structured status replay, the basis of `hf resume`/status. |
| **witness chain (tamper-evidence)** | per-event `prev_hash`→`action_hash` chain; **SHAKE-256 hash binding** (`rvf-crypto/src/witness.rs:4`). Verified fail-closed by `verify_witness_chain` (`ledger/src/v1.rs:848`). | inside redb store | `v1.rs:20` imports **only** `witness::{WitnessEntry, create_witness_chain, verify_witness_chain}` — **no `sign` import** → the chain is hash-linked, **not ed25519-signed**. ed25519 segment-signing exists but is `optional` and un-wired (`rvf-crypto/src/sign.rs:1`, `Cargo.toml` `ed25519-dalek optional`). |
| **RVF vector overlay (`v2`, DEFAULT) — the in-tree "vector" plane** | `rvf-runtime::RvfStore` HNSW index layered over redb "for semantic recall over session history" (`ledger/src/lib.rs:10-12`). 384-dim, cosine. `default = ["v2"]` (`ledger/Cargo.toml`). | `.handoff/**/*.db.rvf` sidecar (gitignored `.gitignore:14`); path `rvf_path` = `{path}.db.rvf` (`ledger/src/v2.rs:58-60`) | STORE: every `append` ingests a vector (`v2.rs:304-343`, `ingest_batch` :340). RECALL: `query_by_intent` (`v2.rs:344-346`) — **0 callers** (see §2). |
| **ICM (fleet decision/error/preference memory organ)** | persistent cross-session "why" memory (semantic recall over decisions/errors/prefs). | none in handoff | **0 product references** — `grep -rni '\bicm\b' --include=*.rs` = **0**. handoff never `recall`s or `store`s to ICM. (The only nearby string is seed-task prose, not a call.) |
| **git-kb (fleet code-intelligence + embeddings organ)** | meta-root `.kb/` with embeddings on. handoff **IS a first-class member**: it carries a **committed `.kb/` store** (17 tracked files incl. `context/extensible/{product,tech}.md`) AND **drives git-kb as a subprocess** for doc-sync, degrading gracefully when absent. | `.kb/` (committed); `hf sync` shells `git-kb` | `hf/src/sync.rs:217-230` (`run_git_kb`/`git_kb_available`), `:295-301` (`git-kb show/checkout`); contract note `sync.rs:11` "Degrades … when `.kb`/`git-kb` is absent". **Contrast: rusty-idd has NO `.kb/` and no git-kb call** (cycle-1 finding). |
| **lifecycle hooks (deterministic store/recall binding)** | typed hook contract incl. `SessionStart`/`SessionEnd` (`handoff-hooks/src/lib.rs:40-43`). SessionEnd runs checkpoint+handoff+export+sync (intent recorded `hf/src/main.rs:3117`). | `.handoff/hooks/` | store-at-session-end is **kernel-bound**, not agent-memory-bound; recall-at-start = `hf resume` (replay). |
| **RuVector / RVF (the vector substrate)** | `../../RuVector/crates/rvf/{rvf-runtime,rvf-index,rvf-types,rvf-crypto}` + `ruvector-verified` (formal-verification attestation, ADR-0011) + `ruvector-domain-expansion` (Thompson-bandit next-task routing, ADR-0012) + `cognitum-gate-tilezero` (coherence gate, opt). | path deps in `ledger/Cargo.toml`, `hf/Cargo.toml` | substrate the ledger overlay + hf governors link; registered fleet-wide as `ruvector`→`meta-ruvector` (`meta/.meta.yaml:292-294`). |

**Recall protocol of record:** cold-start = `rebuild_from_jsonl` from the **committed** JSONL (fail-closed
witness re-verify); warm recall of task status = `replay_latest_status`. Neither uses the vector plane.

---

## 2. Vector intelligence map

| index | exists? | freshness | owner | update command | failure behavior |
|---|---|---|---|---|---|
| **RVF event overlay** (`.db.rvf`, HNSW, 384-dim cosine) | **built + written, never read.** `query_by_intent` (`ledger/src/v2.rs:346`) has **0 callers** outside its own module + tests (`grep -rn query_by_intent --include=*.rs` → only the lib.rs doc comment). **No `hf` verb** named recall/search/query (`grep "\"recall\"|\"search\"|\"query\"" hf/src/main.rs` = 0). | written on **every** `append` (default `v2`) → write amplification: 384-dim vector + sidecar per event, **zero read amortization**. | `ledger` crate (`rvf-runtime::RvfStore`). | implicit: re-embedded on open if ingest fails (`v2.rs:304-307` "best-effort … re-embedded on a later open"). | **best-effort sidecar** — RVF ingest failure is swallowed (`let _ = … ingest_batch`, `v2.rs:338-340`); the authoritative redb append still succeeds. No recall path can fail because none exists. |
| **embeddings quality** | **hash-based pseudo-embeddings, NOT learned/semantic.** `encode_event_to_vector` = SHA3-256 → 384 floats (`v2.rs:42-56`). Crate doc: *"small input changes produce uncorrelated vectors"* (`v2.rs:45-46`) → it is a **content fingerprint**, not a semantic embedding. | N/A | N/A | N/A | even if wired, recall would only group **near-identical** event content; true intent-similarity (**RAG**-style) is **not** achievable with this encoder — the "semantic recall" label overstates capability. |
| **RuVector path dep resolution** | **UNRESOLVED in the worktree.** `../../RuVector` from `…/plan-handoff-cycle2/handoff` = `…/plan-handoff-cycle2/RuVector`, which **does not exist** (`ls ../../RuVector` → absent). Resolves only because **CI clones `FlexNetOS/meta-ruvector` as sibling `RuVector/`** (per `hf/Cargo.toml` comment + `.github/workflows/ci.yml`). meta-root copy exists at `meta/RuVector` + registered `meta/.meta.yaml:292`. | N/A | RuVector repo (meta-ruvector). | CI clone. | **standalone-build blocker** for a union @ `$META_ROOT + handoff` (matches codemap §5): no vendor/publish strategy → the vector substrate is a dangling path dep. |
| **fleet git-kb embeddings** (`meta/.kb/cache/index`) | yes, fleet-level. handoff participates via committed `.kb/` + `hf sync` subprocess (§1). | fleet-owned. | meta/git-kb daemon. | `git kb index <path>` / `hf sync`. | handoff degrades-and-says-so when git-kb absent (`sync.rs:11`). |

**Net:** the only **in-tree vector store is dead weight today** — written on every event, read by nothing,
encoded by a non-semantic hash. handoff's *working* intelligence is **structured replay + committed
JSONL + git-kb code intelligence**, not vector recall. This mirrors rusty-idd's dead vendored vector
store (cycle-1 §1) — **both forks declare a vector/RAG plane neither consumes.**

---

## 3. Recall guarantees

| guarantee | status | evidence |
|---|---|---|
| session-start recall | **strong (structured).** `hf resume` recalls task status via `replay_latest_status` over the witnessed ledger; `current_statuses` warns + falls back to card defaults on replay failure (fail-loud). | `handoff-core/src/lib.rs:72-78`; `ledger/src/v1.rs:828` |
| background-agent recall | **partial.** A background agent recalls via the same ledger replay + the committed JSONL; **no semantic/query API** is exposed (`query_by_intent` unwired). It cannot ask "what prior events resemble this intent." | `v2.rs:346` (0 callers); no `hf` query verb |
| wrap-up store | **strong + deterministic.** SessionEnd hook runs checkpoint+handoff+export+sync (kernel-bound, not agent-remembered); events are witnessed; truth is exported to committed JSONL. | `handoff-hooks/src/lib.rs:40-43`; `hf/src/main.rs:3117`; `ledger/src/export.rs:1-13` |
| cold-start resume proof | **best-in-fleet.** A fresh clone carries the full ledger as text; `rebuild_from_jsonl` re-derives the redb cache and **fails closed if the rebuilt witness chain does not verify to the same count** — no trust in chat history or local binaries. | `export.rs:1-13`; `lib.rs:32-33`; `git ls-files .handoff/ledger.events.jsonl` |
| decision/"why" recall (ICM-class) | **MISSING.** handoff records *operational events* (what happened, witnessed) but has **no decision/error/preference memory** — ICM is absent (0 refs). The "why" lives in commit prose / seed-task strings, not a recallable organ. | 0 `icm` refs in `--include=*.rs` |
| "no plan depends on chat memory alone" | **met for operational truth** (committed JSONL + witness chain), **met for code intelligence** (committed `.kb/` + git-kb), **NOT met for decision/why memory** (no ICM) and **vacuously unmet for vector/semantic recall** (plane exists but is unread). | combined above |

**Union reconciliation (the key question).** The fleet now carries **five** memory/recall surfaces with
**no unified recall facade**:

1. **handoff ledger** (redb + committed JSONL + SHAKE-256 witness) — *operational truth / events / status*.
2. **handoff RVF overlay** (RuVector) — *vector plane, dead/pseudo-embedded*.
3. **ICM** — *decisions/errors/prefs (semantic)* — **absent from both forks' product code**.
4. **git-kb** — *code intelligence + embeddings* — handoff IS a member (committed `.kb/`); **rusty-idd is not**.
5. **rusty-idd `.idd/knowledge`** — *47 MB committed code-graph index, exact-match only, no embeddings*
   (cycle-1 finding) — a **rival** to git-kb's code-graph function.

Reconciliation verdict for the union: handoff is the **right home for operational+continuity memory**
(witnessed, committed, fail-closed) and is **already the proper git-kb member**, so the union should
**keep handoff's ledger as the truth organ and handoff's `.kb/` as the code-intelligence seam**, and:
(a) **collapse the two dead vector planes into ONE decision** — either wire RVF with real embeddings *or*
delegate vector/**RAG** recall to git-kb/ruvector-agent-memory and delete the placeholder; (b) **stop
paying for the unread RVF write on every append**; (c) **make rusty-idd's `.idd/knowledge` a projection
into git-kb**, not a rival store (cycle-1 U1); (d) **introduce ICM** (or an ADR that routes "why" memory
onto curated ledger events — the ESAA-Conversational pattern, trends §B1/§D1) so decision recall stops
depending on commit prose.

---

## 4. Upgrade rows (axis · evidence · risk-tier · acceptance-criterion · reversibility)

| # | upgrade | axis · evidence | risk-tier | acceptance-criterion | reversibility |
|---|---|---|---|---|---|
| U1 | **Resolve the dead vector plane: wire `query_by_intent` to an `hf recall`/`hf search` verb with REAL embeddings, OR delete the v2 overlay from default and delegate semantic/RAG recall to git-kb / `ruvector-agent-memory`.** Today `default=["v2"]` writes a 384-dim pseudo-embedding per event (`ledger/Cargo.toml`; `v2.rs:304-343`) and nothing reads it (`query_by_intent` 0 callers). | memory-vector-intelligence/vector/RAG · `v2.rs:42-56,344-346`; `lib.rs:10-12`; 0-caller grep | medium-high (embedding model choice; NO-C trust-boundary review for any native embedder) | `hf recall "<intent>"` returns ranked prior events by *semantic* similarity OR an ADR records delegation + the v2 overlay is dropped from default; either way no append writes an unread vector. | high — feature-gated; redb authoritative path untouched. |
| U2 | **Stop the write-amplification: make the RVF overlay opt-in (not default), so the common build pays redb+witness only.** `default=["v2"]` forces a `.db.rvf` sidecar + ingest on every `append` for a recall path that does not exist. | speed/memory-vector-intelligence · `ledger/Cargo.toml` `default=["v2"]`; `v2.rs:338-340` per-event `ingest_batch`; `.gitignore:14` sidecar | low | default build = `redb-store` only; `v2` enabled explicitly where recall is actually consumed; existing ledgers unaffected (sidecar is a cache). | high — one feature-flag flip; rebuildable. |
| U3 | **Correct the witness-chain provenance claim and (optionally) sign it.** Docs/seed/trends describe a "blake3+ed25519 witness chain"; the code is **SHAKE-256 hash-linked, unsigned** (`rvf-crypto/src/witness.rs:4`; `v1.rs:20` imports no `sign`). Either (a) fix the claim to SHAKE-256-hash-chained, or (b) wire the already-present ed25519 segment signing (`rvf-crypto/src/sign.rs`) into the witness path for the A2A "signed agent card" posture (trends §A3/§C1). | accuracy/memory-vector-intelligence · `rvf-crypto/src/witness.rs:4`; `rvf-crypto/src/sign.rs:1`; `v1.rs:20,848` | low (docs) / medium (signing) | claim text matches implementation; OR witness entries carry verifiable ed25519 signatures and `verify_witness_chain` checks them. | high — doc-only, or additive signing behind a feature. |
| U4 | **Resolve the RuVector path dep for standalone union (vendor / path-dep map / publish).** `../../RuVector` is absent in the worktree; only CI's sibling clone makes it build (`hf/Cargo.toml` comment). A union @ `$META_ROOT + handoff` cannot build standalone. | memory-vector-intelligence/vector · `ledger/Cargo.toml` path deps; `ls ../../RuVector` absent; `meta/.meta.yaml:292` | medium (build/release) | `cargo build` succeeds from a fresh standalone checkout without an out-of-tree RuVector, via vendored crates or a documented workspace path map. | medium — vendoring is reversible; a publish step is stickier. |
| U5 | **Introduce decision/"why" memory (ICM or ledger-curated-events) — handoff has 0 ICM refs.** Operational events are witnessed but the *why* (decisions/errors/gotchas) is unrecallable. Adopt ESAA-Conversational "explicit curation records on the same append-only log" (trends §B1/§D1) OR add an ICM recall/store seam. | memory-vector-intelligence/ICM · 0 `icm` refs; trends §B1 (arXiv 2606.23752), §D1 | medium | a product path records a curated decision/error event (or `icm store`) on completion AND a recall path surfaces it next session; OR an ADR records the deliberate routing of "why" onto curated ledger events. | high — additive; ledger-curated variant needs no new dep. |
| U6 | **Unify the fleet recall plane behind one facade across handoff-ledger / git-kb / ICM / `.idd/knowledge`.** Five disjoint stores, no single `recall`. Define a fabric-recall contract that fans out and provenance-tags hits. (Shared with rusty-idd U5.) | memory-vector-intelligence · §3 five-store split; cycle-1 union | high (cross-organ contract) | one `recall` entry point returns merged, organ-tagged hits from ≥2 organs for a golden query; documented as an ADR. | medium — facade additive; underlying stores untouched. |

---

## 5. Gate handoff (fail-closed artifact/test additions)

So missing/overstated memory+vector surfaces fail closed instead of silently degrading:

1. **No-unread-vector gate (CI):** assert that if `default` includes `v2` (vector ingest on every
   `append`, `v2.rs:338-340`), at least one product caller of `query_by_intent` exists — otherwise the
   build must drop `v2` from default (forces U1/U2). Prevents paying for a recall plane nothing reads.
2. **Embedding-honesty gate:** the `ledger` crate description says "semantic recall" while
   `encode_event_to_vector` is a SHA3 fingerprint (`v2.rs:42-56`). Add a doc/test gate that the recall
   surface is labelled "content-fingerprint near-dup recall" until a real embedder is wired.
3. **Witness-claim gate (RED test):** assert the witness algorithm in code matches the documented claim
   (SHAKE-256 today, `rvf-crypto/src/witness.rs:4`) — and, if U3(b) is taken, that a tampered signature
   fails `verify_witness_chain` (`v1.rs:848`). Prevents "blake3+ed25519" drift in docs/seed/trends.
4. **Standalone-build gate (CI):** a job that builds handoff from a checkout **without** an out-of-tree
   `RuVector/` must either pass (vendored) or fail loudly with a clear "RuVector path dep unresolved"
   message — not silently rely on the sibling clone (`hf/Cargo.toml` comment; U4).
5. **Cold-start integrity gate (already partially present — keep + assert):** `rebuild_from_jsonl` must
   fail closed on witness-count mismatch (`export.rs:1-13`); add a test that a corrupted committed JSONL
   line aborts rebuild rather than producing a short chain.
6. **Recall-provenance gate (for U6):** any unified recall must tag each hit with its organ
   (handoff-ledger / git-kb / ICM / `.idd/knowledge`) and fail closed if a claimed organ is unreachable.

---

## Confidence

HIGH on inventory + wiring facts: every row cites a path/line, grep counts (ICM 0, `query_by_intent`
0 callers, no `hf recall` verb), `git ls-files`, `.gitignore`, and Cargo manifests. HIGH on "RVF overlay
unread / pseudo-embedded" and "witness chain is SHAKE-256-not-blake3/ed25519" (read directly from
`rvf-crypto` source). MEDIUM on the standalone-build claim (inferred from path-dep absence + CI comment,
not a forced standalone `cargo build`). Fleet git-kb embeddings read from `meta/.kb` config context, not
re-derived here.

Artifact: `/home/drdave/Desktop/meta/.worktrees/plan-fleet-convergence/envctl/.handoff/loop/plan/findings/memory-vector-intelligence-handoff.md`
