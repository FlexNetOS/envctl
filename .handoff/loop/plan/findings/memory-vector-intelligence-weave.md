# Findings — persistent memory + vector/code intelligence — target: weave (cycle 4)

- Axis: `memory-vector-intelligence`
- Target code (read-only): `/home/drdave/Desktop/meta/.worktrees/plan-weave-red/weave` @ codemap snapshot `@4fe2419`, branch `plan/weave-red-tests`
- Context consumed: `reports/codemap-weave.md` (present, read). `research/weave.trends.md` (ABSENT — see Gap G5; no trend evidence available this cycle, so no web/vendor currency claims are made here rather than inventing them).
- Verdict (one line): weave's SQLite store is a **transport/event log, not semantic memory** (CONFIRMED), and weave has **zero vector/embeddings/RAG** (CONFIRMED). BUT the fleet's "weave = transport, not a 6th memory store" framing is **PARTIALLY REFUTED**: weave-core ships `memory.rs`, a real (if primitive) filesystem-backed scoped persistent-memory organ that auto-injects context into A2A messages — a 6th memory surface overlapping ICM's role.

---

## 1. Memory inventory

### 1a. The SQLite store = TRANSPORT log, not recall memory (CONFIRMED)

weave's broker DB is an append-mostly event/mailbox log. Schema (all in `weave-core/src/store.rs`):

| table | line | role | memory or transport? |
|---|---|---|---|
| `messages` | store.rs:1502 | the mailbox (A2A traffic) | TRANSPORT |
| `reads` | store.rs:1517 | per-recipient read receipts | TRANSPORT (delivery state) |
| `wake_acks` | store.rs:1523 | wake acknowledgements | TRANSPORT |
| `peers` | store.rs:1527 | agent registry/discovery | TRANSPORT (directory) |
| `outbox` | store.rs:1547 | Tier-2 cross-store delivery intents | TRANSPORT |
| `pull_cursor` | store.rs:1561 | per-source high-water dedup mark | TRANSPORT |
| `keys` / `identity_keys` / `revocations` | store.rs:1565/1569/1575 | ed25519 identity material | SECURITY (not memory) |
| `asks` / `ask_groups` | store.rs:1583/1600 | request-response correlation | TRANSPORT |
| `jobs` | store.rs:1608 | job queue + fencing token | TRANSPORT (work queue) |
| `delivery_log` | store.rs:1647 | metadata-only delivery audit | TRANSPORT (trace) |
| `presence` / `schedules` / `reviews` / `leases` | store.rs:1658/1664/1679/1693 | liveness / cron / review queue / resource reservation | TRANSPORT/COORDINATION |
| `summaries` | store.rs:1702 | thread summaries | borderline (derived transport artifact, not recall) |

This is a **traceable transport ledger**: every row is keyed by message/outbox id, deduped on `(source, id)` via `pull_cursor` (codemap §"Message schema"), and read-only-federatable (`federated_peers`/`federated_sessions`, store.rs:23). It is NOT a recall organ — there is no semantic index over agent knowledge, no decisions/preferences topic model, no cross-session "why" store. **CLAIM-1 CONFIRMED: the SQLite store is transport, not memory.** Evidence: schema is mailbox/queue/receipt tables (store.rs:1502–1702); the only query surface is FTS5 over message bodies (§2), i.e. searching *traffic*, not recalled facts.

### 1b. weave DOES ship a memory organ — `weave-core/src/memory.rs` (the refutation)

This is the load-bearing finding. weave-core contains a standalone persistent-memory module, **separate from the SQLite transport DB**:

- Module doc (memory.rs:1–4): *"Filesystem-backed scoped memory under `~/.config/weave/memory/`. Plain markdown files with YAML frontmatter; simple substring search; no SQLite/FTS, no async. All I/O is synchronous std::fs."*
- Persistence path: `config_dir()/memory` (memory.rs:415–416).
- Scopes (memory.rs:37–53): `Global`, `Project(name)`, `Persona(name)`, `Orchestrator(circle)` — i.e. it is namespaced like ICM topics.
- Full CRUD + search API: `memory_write` (memory.rs:89), `memory_read` (145), `memory_search` (160), `memory_list` (202), `memory_delete` (229), `memory_scopes` (241).
- Exposed on BOTH surfaces:
  - CLI verb `weave memory {write,read,search,list,delete,scopes}` — `MemoryCmd` (main.rs:1135), dispatched `Cmd::Memory` (main.rs:6713) → `dispatch_memory` (main.rs:7092).
  - 5 MCP tools: `weave_memory_write` / `_read` / `_search` / `_list` / `_delete` (`weave-mcp/src/mcp.rs:520–524`, handlers ~4606–4645).

**CLAIM-2 CONFIRMED: weave has a first-class persistent memory store (filesystem notes), not just transport.** This **partially refutes** the cycle-1/2 "5 disjoint surfaces; weave is transport, not a 6th store" finding: the *mailbox* is transport, but `memory.rs` is a genuine 6th persistent-memory surface co-located inside the transport plane. See Upgrade U1 (separation-of-concerns).

### 1c. Auto-injection: weave's de-facto "recall" path

weave actively performs a recall-and-inject step on outbound A2A traffic. `build_context_prefix(identity, circle, body, top_n)` (memory.rs:332) extracts keywords from the message body (memory.rs:334, `extract_keywords` with a stop-word list memory.rs:22–30), lexically scores memory entries across Global+Project+Persona+Orchestrator scopes, and prepends the top hits as a `<weave-memory>…</weave-memory>` block ahead of `<original body follows>` (memory.rs:398–408). Cap is `MAX_CONTEXT_PREFIX_ENTRIES = 5` (memory.rs:20, enforced memory.rs:333); the call site uses `top_n = 3` (main.rs:1766). Senders opt out per-message via `--no-memory` on send/reply/ask (main.rs:253, 367, 667).

So weave's **recall** mechanism is: lexical keyword match over its own filesystem notes, auto-stuffed into outgoing messages. This is a *message-augmentation* recall, NOT a session-start/cold-start agent recall (contrast ICM `recall` / handoff resume — see §3).

### 1d. Continuity / handoff membership

- weave HAS `.handoff/` (loop kernel: `context/ decisions/ hooks/ loop/ packets/ policies/ tasks/`, `policy.toml`, `HARNESS-CHANGELOG.md`). This is the **handoff** continuity kernel deployed *into* weave, i.e. weave is an hf fleet member — its cross-session continuity comes from the handoff ledger, not from weave's own memory.
- `.meta.yaml` entry: `weave` → `tags: [mcp, orchestration]` (line 181) — **not** tagged `memory`. The fabric does not register weave as a memory organ, consistent with intent even though `memory.rs` exists.

---

## 2. Vector intelligence map

**CLAIM-3 CONFIRMED: weave has NO vector / embeddings / RAG / ANN index of any kind.**

Evidence (read-only repo-wide scan of `*/src/`):
- No `embedding`, `cosine`, `faiss`, `qdrant`, `pgvector`, `hnsw`, `sentence-transformer`, `candle`, `onnx`, `tch`, `fastembed`, or `RAG` symbol appears in any source or `Cargo.toml`. The only matches are false positives in prose comments: `provider_switch.rs:3` ("…instead of *embedding* the…") and `weave-core/Cargo.toml:30` ("no *embedded-replica* sync"). The word "vector" in `store.rs` is Rust `Vec`/"attack vector"/"censorship vector" comments (store.rs:639, 5444); "semantic" in `model.rs:1379` refers to a default *circle* (namespace), not semantic search.
- weave's only search is **lexical**:
  - Message search = SQLite **FTS5** virtual table `messages_fts` (store.rs:2378, triggers store.rs:2386/2390) backing `Store::search` (trait store.rs:99, impl store.rs:3311). Full-text, not vector.
  - Memory search = substring `contains` scoring: `relevance_score` returns 100 on tag/quoted match, 10 on body substring, else 0 (memory.rs:586–600); `memory_search` (memory.rs:160) reads dirs and ranks by that score. No similarity vectors.

| index | exists? | kind | freshness/update | failure behavior |
|---|---|---|---|---|
| `messages_fts` (FTS5) | YES | lexical full-text over message bodies | auto via INSERT/DELETE triggers (store.rs:2386/2390) | created only when FTS5 available; on sqlite/libsql builds (store.rs:2375–2376) |
| memory note search | YES | lexical substring/tag scoring | recomputed per query from `~/.config/weave/memory` (no index) | non-fatal; `build_context_prefix` returns original body on any error (memory.rs doc + 367/394) |
| vector/embeddings/RAG | **NO (N/A)** | — | — | — |
| `git-kb` code-graph snapshot of weave | **NO in-repo** | — | — | — |

**git-kb / code-intelligence:** weave has **no `.kb/` directory** (not a git-kb member; verified absent at repo root). The `reports/codemap-weave.md` graph (2722 symbols / 9571 edges, codemap §5) was produced by running `git-kb code` against weave *from the plan worktree externally* — weave itself ships no persisted code-graph snapshot and no embeddings. So code-intelligence over weave is a **planner-side, ephemeral** artifact, not an organ weave owns.

---

## 3. Recall guarantees

| guarantee | weave status | evidence | who actually provides it for weave |
|---|---|---|---|
| session-start recall | **N/A in weave** — weave has no session-start hook that recalls its own memory | no startup recall path; memory is read only on `weave memory search` or on send-time `build_context_prefix` (memory.rs:332) | ICM (`icm recall` discipline) + `.handoff` resume |
| background-agent recall | PARTIAL — an agent that *sends* a message gets lexical memory auto-prefixed (memory.rs:332, main.rs:1766), unless `--no-memory` | main.rs:253/367/667, 1766 | weave (message-augmentation only) |
| wrap-up store | **N/A in weave** — `memory_write` is manual/opt-in (CLI/MCP), no automatic end-of-session capture | memory_write (memory.rs:89) is caller-invoked only | ICM store + handoff packet render |
| cold-start resume proof | **N/A in weave's memory** — weave cannot reconstruct "why/plan" from its store; transport is replayable but is traffic, not intent | store schema §1a; codemap §"Federation" (read-only pull) | handoff ledger (weave has `.handoff/`); HANDOFF.md |

**No plan should depend on weave's memory for recall.** weave's `memory.rs` is best-effort, lexical, and opt-in; the authoritative recall/store organs remain **ICM** (recall/store discipline) and the **handoff** ledger (witnessed continuity). weave's correct role is to *transport* between those organs, plus a thin message-augmentation cache. This is the separation-of-concerns the Upgrade rows protect.

---

## 4. Upgrade rows (axis: memory-vector-intelligence)

**U1 — Resolve the dual-store separation-of-concerns: classify or quarantine `weave-core/memory.rs`.**
- axis: memory-vector-intelligence · separation-of-concerns
- evidence: `memory.rs:1–4, 89–241, 332` (a 2nd persistent store inside the transport plane) vs ICM being the fabric memory organ; `.meta.yaml:181` does not tag weave `memory`. This contradicts the 5-disjoint-surfaces invariant.
- risk-tier: MEDIUM (silent memory fragmentation — agents may write durable facts into weave notes that ICM/handoff never see; no decay, no cross-agent consolidation).
- reversibility: HIGH (decision + docs/ADR; no code deletion required to start).
- acceptance-criterion: an ADR exists stating weave memory's bounded role (per-agent send-time augmentation cache ONLY, not a fleet recall organ); CLAUDE.md/ARCHITECTURE.md explicitly point durable recall/store to ICM+handoff; a test or doc-gate asserts weave memory is excluded from the "memory surfaces" count or is explicitly enumerated as the 6th, bounded surface.

**U2 — Make weave's send-time recall and ICM consistent (or wire one to the other).**
- axis: memory-vector-intelligence · accuracy
- evidence: `build_context_prefix` (memory.rs:332) recalls only weave's own filesystem notes via lexical scoring (memory.rs:586) — it cannot surface ICM decisions/preferences, so injected "context" can diverge from the fabric's real memory.
- risk-tier: MEDIUM (stale/partial context injected into A2A traffic, presented authoritatively in `<weave-memory>`).
- reversibility: HIGH (additive; gated behind existing `--no-memory` opt-out).
- acceptance-criterion: either (a) `build_context_prefix` reads from / defers to ICM, or (b) docs state the augmentation is explicitly a local cache and weave memory is periodically reconciled to ICM; a RED test asserts the documented contract (e.g. `--no-memory` fully suppresses, and injected entries carry a provenance/scope label — already present at memory.rs:402).

**U3 (N/A-but-recorded) — No vector/RAG upgrade proposed for weave.**
- axis: memory-vector-intelligence · vector
- rationale (genuine N/A): weave is transport; semantic retrieval is the job of the memory organs (ICM/git-kb/ruvector), not the nervous system. Adding embeddings to weave would duplicate fabric capability and worsen U1's separation problem. Evidence of correct current state: no vector deps anywhere (§2). Recommendation: keep FTS5 lexical for message search (transport-appropriate); do not add a vector index to weave.

---

## 5. Gate handoff (fail-closed additions)

So that missing/over-reaching memory-vector surfaces fail closed rather than drift silently:

1. **ADR gate (U1):** require an ADR enumerating weave memory's bounded role; a `.handoff` policy/doc check fails closed if `weave-core/src/memory.rs` exists but no ADR classifies it. (RED test handoff to plan-test-strategist: assert ADR presence when `memory.rs` is present.)
2. **No-vector invariant test:** an additive RED test / `deny`-style grep gate asserting weave introduces **no** embedding/vector/RAG dependency (guards the intended transport-only role; today it passes — §2 — so it is a regression fence, not a fix).
3. **Provenance/opt-out test for injection (U2):** RED test asserting `--no-memory` suppresses the `<weave-memory>` block (memory.rs:398) and that every injected line carries a `scope::key` label (memory.rs:402) — so auto-recalled context is never anonymous/unattributable in A2A traffic.
4. **Trend-evidence gate (G5):** flag that `research/weave.trends.md` was absent this cycle; the synthesis step must not assert tool-currency claims for the memory/vector axis without that input (fail-closed: no source = no claim).

---

## Gaps / N-A ledger
- **G5 (missing input):** `research/weave.trends.md` and a `research/` dir do not exist at the target path; no 90-day trend evidence consumed. Recorded, not fabricated.
- **N/A — vector/RAG:** genuinely absent and correctly so (transport plane); see U3.
- **N/A — session-start/wrap-up recall in weave:** genuinely absent; provided by ICM + handoff, not weave (§3).
- weave is NOT a git-kb member (no `.kb/`); IS a handoff member (`.handoff/` present).

---
Artifact path: `/home/drdave/Desktop/meta/.worktrees/plan-weave/envctl/.handoff/loop/plan/findings/memory-vector-intelligence-weave.md`
