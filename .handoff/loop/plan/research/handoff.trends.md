# handoff — best-practices + latest trends + tool currency (trends note)

Target: **handoff** — continuity / witnessed-ledger kernel (`hf`), pure-Rust workspace.
Researcher: plan-trend-researcher. Cycle: 2. Date: **2026-06-26**.
Recency window (90 days): **2026-03-28 → 2026-06-26**. Findings outside the window are flagged.
Frame: meta is ONE converging system. handoff = **continuity** (witnessed event ledger → rendered
packets); rusty-idd = **intent** (spec/why-what). They are converging into one control plane. Every
finding is tied to that union path (event-ledger ⊕ intent/spec on a single witnessed log).

Method: deep-research (fan-out search → fetch → adversarial refute → cited synthesis). Every
load-bearing claim carries a URL + date. **Reuses cycle-1 notes (cited inline, NOT duplicated):**
`research/rusty-idd.trends.md` (shared stack: clap/serde/ratatui/crossterm/tokio/toml; A2A v1.0 §D1),
`research/agentic-planning-trends-2026-06.md`, `research/sources-rusty-idd.jsonl`.

Verified target pins (from `Cargo.lock` + manifests at target_root `…/plan-handoff-cycle2/handoff`,
2026-06-26): **redb 4.1.0** (authoritative pure-Rust event store, ADR-0017 no-C cutover) · **blake3
1.8.5** (witness chain) · **ed25519-dalek 2.2.0** + **curve25519-dalek 4.1.3** + **signature 2.2.0**
+ **subtle 2.6.1** + **zeroize 1.8.2** (signed witness/cards) · **sha2 0.10.9** · **rusqlite 0.31.0 /
libsqlite3-sys 0.28.0** (OPTIONAL — `legacy-sqlite` migration importer ONLY, never in default no-C
build) · **toml 0.8.23** · **time 0.3.49** · **chrono 0.4.45** · **syntect 5.3.0** (vendored TUI).
Shared-with-rusty-idd pins (current; see cycle-1 §A): clap 4.6.1 · serde 1.0.228 · serde_json 1.0.150 ·
ratatui 0.30.1 (one patch behind rusty-idd's 0.30.2 — trivial) · crossterm 0.29.0 · tokio 1.52.3 ·
anyhow 1.0.102 · thiserror 1.0.69 + 2.0.18.

Confidence legend: HIGH = corroborated by primary source (release/advisory/repo) + in-window;
MED = single strong source or in-window blog; LOW = single secondary source / could not corroborate.

---

## A. Tool-currency & advisories (architect R7 input)

### A1. redb 4.1.0 — CURRENT, in-window, no advisory. The ledger store is on the latest line. [HIGH · in-window]
- handoff's authoritative event store is **redb** (pure-Rust, copy-on-write B+trees, **fully ACID**,
  **crash-safe by default**, MVCC concurrent readers + single writer, savepoints/rollback; **stable file
  format** with a committed upgrade path). handoff pins **4.1.0** = the **latest published** version,
  released **2026-04-19** (in-window). 4.1 delivered ~**1.5x speedup** (dynamic read/write cache
  partitioning) plus a large batch of bug fixes **surfaced by AI coding agents**.
- redb **4.0.0** was the breaking major: `Drop` added to `AccessGuardMut`/`AccessGuardMutInPlace` to fix
  a critical data-loss bug, and the `Legacy` type removed. handoff is already on 4.1.0 → past that cut.
  (Exact 4.0.0 GA date not pinned to one primary changeset — a small-model fetch returned an ambiguous
  year; not load-bearing since handoff is on 4.1.0.)
- **No RustSec advisory** for redb (searched the advisory DB index — no redb entry).
- Source: https://lib.rs/crates/redb (4.1.0 = 2026-04-19, "Stable and maintained", crash-safe ACID,
  stable file format; accessed 2026-06-26); https://www.phoronix.com/news/Redb-4.1-Released (2026,
  1.5x + AI-found bugfixes); https://github.com/cberner/redb (pure-Rust ACID embedded KV; accessed
  2026-06-26); https://rustsec.org/advisories/ (no redb advisory; accessed 2026-06-26).
- Action: **none for currency** — handoff is on the latest, crash-safe, stable-format line. Keep
  `cargo audit`/`cargo deny` in CI so any future redb advisory surfaces. Best-practice fit: redb's
  crash-safe-by-default + stable-format guarantees are exactly what a continuity ledger needs.
- Refute: is redb production-grade for a ledger? cberner/redb is the canonical pure-Rust ACID embedded
  KV (redb.org), 4.x stable-and-maintained, file-format stability promised → corroborated, not a single
  blog. PASS. (Caveat: AI-found bug volume in 4.1 is a *signal of churn* — keep pinning + CI audit.)

### A2. blake3 1.8.5 — CURRENT (latest), no advisory. [HIGH · in-window]
- handoff hashes the witness chain with **blake3 1.8.5** = the **latest published** version. No RustSec
  advisory for blake3. (1.8.x added the `hazmat` low-level module; not used by handoff's chain.)
- Source: https://docs.rs/crate/blake3/latest (1.8.5 latest; accessed 2026-06-26);
  https://github.com/BLAKE3-team/BLAKE3/releases (accessed 2026-06-26); https://rustsec.org/advisories/
  (no blake3 advisory). Action: none.

### A3. ed25519-dalek 2.2.0 + curve25519-dalek 4.1.3 — CURRENT, past ALL signing advisories. [HIGH · in-window]
- The signing stack is on the **fixed** versions of every known advisory:
  - **RUSTSEC-2022-0093** (ed25519-dalek "Double Public Key Signing Oracle") is fixed in **2.0**;
    handoff is on **2.2.0** → unaffected.
  - **RUSTSEC-2024-0344** (curve25519-dalek timing variability in `Scalar29/52::sub`, can leak scalars)
    is patched in **4.1.3** — handoff pins **exactly 4.1.3** → unaffected (on the patched edge).
- Source: https://rustsec.org/advisories/RUSTSEC-2022-0093 ; https://rustsec.org/advisories/RUSTSEC-2024-0344.html
  (affected < 4.1.3, patched 4.1.3; accessed 2026-06-26); https://rustsec.org/packages/ed25519-dalek.html .
- Action: **none**, but **pin-floor `curve25519-dalek >= 4.1.3`** explicitly (it currently resolves to
  the patched version transitively — a lockfile drift below 4.1.3 would reintroduce a key-leak timing
  bug). Confirm `cargo deny` would catch a regression. Best-practice: signed witness cards are exactly
  the A2A "signed agent card" direction (§C1) — handoff is on-trend.
- Refute: any newer 2026 ed25519-dalek/curve25519-dalek advisory? RustSec package pages show none past
  2024-0344. PASS.

### A4. rusqlite 0.31.0 / libsqlite3-sys 0.28.0 — feature-gated migration-only; NOT in the no-C runtime; past advisories. [MED · status: known-residual]
- These are the **only C dependency** in the tree and are **`optional = true`**, pulled in **solely** by
  the **`legacy-sqlite`** feature (`ledger/Cargo.toml:23,37` → the one-time legacy C-SQLite → redb
  `hf migrate` importer). They are **never** part of the default no-C runtime build (ADR-0017 cutover).
- Both pins sit **past** their advisories: rusqlite **RUSTSEC-2021-0128** (closure lifetime, fixed 0.26)
  and **RUSTSEC-2020-0014** (memory-safety, fixed 0.23) → 0.31.0 unaffected; libsqlite3-sys
  **RUSTSEC-2022-0090** (bundles CVE-2022-35737 SQLite, fixed by bundling patched SQLite in 0.25.1) →
  0.28.0 unaffected.
- These pins are **older than the current rusqlite/libsqlite3-sys lines** (currency lag), but currency is
  low-priority here because the crates are migration-only + feature-gated behind the trust boundary.
- Source: https://rustsec.org/packages/rusqlite.html ; https://rustsec.org/advisories/RUSTSEC-2022-0090.html
  (accessed 2026-06-26); target manifest `ledger/Cargo.toml:23,37`.
- Action: keep `legacy-sqlite` **out of the default feature set** (it is). Optional (governance): once the
  fleet's legacy C-SQLite ledgers are all migrated, consider **removing** the `legacy-sqlite` feature +
  crates entirely to delete the last C dependency from the repo (a strict no-C upgrade). Until then,
  document as an accepted, gated, advisory-clear residual.
- Refute: any 2026 SQLite/rusqlite advisory hitting 0.31/0.28? Most recent advisories are 2020–2022, all
  pre-pin. PASS (MED — absence-of-evidence beyond the listed advisories, not a per-CVE SQLite sweep).

### A5. toml 0.8.23 — minor currency lag (0.9 line is out); no advisory. [LOW-MED · in-window]
- handoff is on **toml 0.8.23**; the current line is **0.9.x** (rusty-idd already pins **0.9.6**, the
  latest — see cycle-1 §A). No advisory at either version. Pure currency gap, no security/correctness gate.
- Source: cycle-1 `research/rusty-idd.trends.md` §A (toml 0.9.6 = current); target manifest.
- Action: optional alignment to `toml 0.9` for fleet consistency with rusty-idd — **not** a gate. Note:
  toml 0.9 had config/format-feature changes; verify before bumping. Low blast-radius.

### A6. Shared stack (clap/serde/serde_json/ratatui/crossterm/tokio/anyhow/thiserror) — CURRENT; see cycle-1, not re-derived. [HIGH · carried]
- All match or are within one patch of cycle-1's verified-current pins (clap 4.6.1, serde 1.0.228,
  serde_json 1.0.150, ratatui 0.30.x, crossterm 0.29.0, **tokio 1.52.3 — 2026 advisories are tokio-0.1
  legacy, do NOT apply**; anyhow 1.0.102). No supersession; no action. See `research/rusty-idd.trends.md`
  §A1/A2/A3/A6. (handoff ratatui 0.30.1 vs rusty-idd 0.30.2 = one patch behind, trivial.)

---

## B. Continuity / ledger / witnessed-state kernels for agents (state of the art, in-window)

### B1. Event-sourcing IS the converged best-practice for agent continuity/handoff — and ESAA describes handoff's exact architecture. [HIGH · in-window]
- **ESAA-Conversational** (arXiv 2606.23752, submitted **2026-06-22**, in-window) treats cross-agent
  continuity as an **event-sourcing problem**: mechanically capture turns into an **append-only immutable
  log** (`activity.jsonl`) → **deterministic projection** (handoff guidance, state summaries, decision
  records, task lists) generated **without LLM inference** → a **verifiable materialized view**; agents
  apply judgment only for **explicit curation** (decision/task records). It enables collaboration across
  heterogeneous agents (Codex/Claude/Grok) **"without a direct agent-to-agent channel" via a common log**,
  with workspace isolation + write-path locking. Predecessor **ESAA** (arXiv 2602.23193, Feb 2026,
  baseline/older) frames the same for autonomous SWE agents: separate cognitive intention from state
  mutation; orchestrator validates structured-JSON intentions, persists events, applies effects,
  projects a verifiable view.
- Source: https://arxiv.org/abs/2606.23752 (2026-06-22); https://arxiv.org/abs/2602.23193 (2026-02);
  https://arxiv.org/pdf/2511.03690 (OpenHands Agent SDK — composable, event-based production agents).
- **Relevance (load-bearing for the union):** this is **handoff's design, externally validated and named
  in-window** — handoff's witnessed event ledger → **rendered packets** = ESAA's append-only log →
  deterministic projection. The architect can cite ESAA as best-practice corroboration that
  packets-are-rendered-not-hand-written and continuity-without-direct-A2A are the right invariants. Best-
  practice to **affirm/adopt**: deterministic, inference-free projection of read-models from the log;
  explicit curation commands for durable decisions/tasks (the seam where intent/rusty-idd attaches).
- Refute: single-paper hype? Two arXiv papers (ESAA + ESAA-Conversational) **plus** independent durable-
  execution literature (B2) **plus** handoff's own pre-existing implementation all converge on append-log
  + deterministic projection → corroborated, not a lone claim. PASS.

### B2. Durable-execution / checkpoint-resume is now a named production pattern (Temporal/LangGraph/Restate/DBOS). [MED-HIGH · in-window]
- The 2026 reference architecture for durable LLM agents pairs **durable-execution primitives** (Temporal,
  AWS Step Functions Express, **Restate, DBOS**, Inngest) with the **checkpointer model** (LangGraph
  `PostgresSaver`/`RedisSaver`/`DynamoDBSaver`): every step's state is saved to a durable store so an
  interrupted run **resumes from its last checkpoint**, with **time-travel** (inspect/replay any historical
  step).
- Source: https://appscale.blog/en/blog/durable-execution-llm-agents-temporal-langgraph-checkpointing-2026 ;
  https://zylos.ai/research/2026-03-04-ai-agent-workflow-checkpointing-resumability/ (2026-03-04, in-window);
  https://docs.langchain.com/oss/python/langgraph/durable-execution (accessed 2026-06-26).
- Relevance: handoff's **resume/checkpoint/drift-reconcile** cycle is the Rust-native, no-server
  embodiment of this pattern — but where the field defaults to **external stores** (Postgres/Redis/Dynamo
  + a workflow server), handoff's differentiator is an **embedded, crash-safe, pure-Rust** store (redb,
  §A1) with a **cryptographic witness chain** (blake3 + ed25519, §A2/A3). That is a *stronger* durability/
  tamper-evidence posture than the checkpointer-on-Postgres mainstream. Best-practice handoff already
  meets: checkpoint→resume-from-last-state. Watch/borrow: explicit **time-travel replay** ergonomics.
- Refute: is "durable execution" just vendor marketing? Backed by multiple independent engines (Temporal,
  Restate, DBOS, LangGraph) + dated analyst pieces → real category. PASS (MED-HIGH; secondary sources).

---

## C. A2A standards + the handoff↔transport boundary (weave) (in-window)

### C1. A2A v1.0 (Linux Foundation) is the transport/mesh standard — and the field explicitly SEPARATES transport from durable verifiable state. [HIGH · in-window]
- A2A (Agent2Agent) is the LF cross-vendor interop standard (v1.0 current in 2026; v0.3 added gRPC +
  **signed agent/security cards** + version negotiation; 150+ orgs) — see cycle-1 §D1 (not re-derived).
  In-window field guidance (DEV, **2026-04-10**) frames the architecture as **"a mesh of internal task
  agents, company-specific domain agents, external vendor agents…"**, makes **observability a first-class
  requirement** (task traces, step spans, tool metadata, decision checkpoints), and — crucially — keeps
  **transport** (A2A: discover + exchange tasks + coordinate) **distinct** from **durable verifiable
  state**: *"every important claim should be backed by an artifact"*, treating agent outputs as **evidence
  receipts** that prove work occurred, forming an audit trail.
- Source: https://dev.to/chunxiaoxx/building-multi-agent-ai-systems-in-2026-a2a-observability-and-verifiable-execution-10gn
  (2026-04-10); cycle-1 `research/rusty-idd.trends.md` §D1 (A2A v1.0 LF, signed cards, gRPC).
- **Relevance (the handoff↔weave boundary):** the boundary is now an explicit field pattern, not just a
  local design choice — **weave = transport** (A2A-shaped: discover/route/exchange), **handoff = durable
  witnessed state** (the "evidence receipts" / audit trail). handoff's **signed witness chain** (§A3) is
  the local analogue of A2A's **signed agent cards**, and ESAA's "continuity **without** a direct A2A
  channel" (§B1) says the **log is the substrate** and A2A is the optional overlay. Best-practice for the
  union: keep handoff as the receipt/ledger plane; let weave/A2A be the transport plane; do **not** fuse
  them. Adapter direction: emit handoff witness records as A2A-compatible signed artifacts.
- Refute: does A2A subsume the need for handoff? No — A2A standardizes *transport/discovery*; it does not
  provide a durable tamper-evident ledger. The two are complementary planes (DEV article + ESAA both draw
  the line). PASS.

---

## D. Spec/intent ↔ continuity UNION patterns (handoff ⊕ rusty-idd) (in-window)

### D1. "Verifiability-first / auditable agents": intent-vs-behavior must be checked against a tamper-evident ledger — the union thesis, externally. [HIGH · mixed window]
- The 2026 reliability literature converges on **provable observability + audit**: **Verifiability-First
  Agents** (arXiv 2512.17259, Dec 2025 — baseline/older) embeds **run-time attestations** and **lightweight
  Audit Agents that continuously verify intent vs behavior** with challenge-response attestation for
  high-risk ops; **Auditable Agents** (arXiv 2604.05485, ~2026-04, in-window) and enterprise-audit guidance
  (Augment Code, in-window) require **attributability + reversibility** and **hash-chain tamper-evidence**
  (the VIL "cryptographically linked records, commitments not raw content; any post-hoc edit breaks the
  chain — tamper evidence without a blockchain").
- Source: https://arxiv.org/abs/2512.17259 (2025-12, baseline); https://arxiv.org/pdf/2604.05485
  (Auditable Agents, 2026); https://www.augmentcode.com/guides/multi-agent-outputs-n-pass-enterprise-audit
  (attributability + reversibility; accessed 2026-06-26).
- **Relevance (the union, load-bearing):** this is the precise seam between handoff and rusty-idd. handoff
  supplies the **tamper-evident witnessed ledger** (blake3-linked records + ed25519 signatures = the
  "hash chain, commitments not raw content"); rusty-idd supplies the **intent/spec** (the "what should
  happen"). An **Audit Agent that verifies intent-vs-behavior** is *exactly* a check of rusty-idd's spec
  against handoff's witnessed events — i.e. the converged control plane is **intent (rusty-idd) projected
  onto, and verified against, a witnessed event ledger (handoff)**. ESAA-Conversational (§B1) operationalizes
  this: durable **decisions/tasks recorded as explicit curation** *on the same append-only log* — the union
  is one log carrying both continuity events and curated intent. Best-practice to adopt for the union:
  model rusty-idd spec/decision records as **curated events on handoff's ledger**, with deterministic
  projection producing the spec-status view; add an intent-vs-witness verification projection.
- Refute: is "auditable/verifiable agents" a distinct, real trend vs generic observability? Two arXiv
  papers + enterprise-audit guidance + the VIL hash-chain concept, all converging on tamper-evident
  intent-vs-behavior audit → corroborated. PASS. (D1 mixes one baseline-older arXiv with in-window
  corroboration — flagged.)

---

## E. Recency ledger

| # | Finding | Best source date | Window |
|---|---------|------------------|--------|
| A1 redb 4.1.0 current/no-advisory | lib.rs / Phoronix | 2026-04-19 | in-window |
| A2 blake3 1.8.5 current | docs.rs latest | 2026-06-26 access | in-window |
| A3 ed25519/curve25519 past advisories | RustSec 2024-0344 | 2024-06-18 (status current) | older-flagged (fix confirmed) |
| A4 rusqlite/libsqlite gated+past-advisory | RustSec rusqlite/2022-0090 | 2021–2022 (pre-pin) | older-flagged |
| A5 toml 0.8.23 minor lag | cycle-1 §A (toml 0.9.6) | 2026-06-26 | in-window |
| B1 ESAA-Conversational event-sourcing | arXiv 2606.23752 | 2026-06-22 | in-window |
| B1 ESAA (predecessor) | arXiv 2602.23193 | 2026-02 | older-flagged (baseline) |
| B2 durable execution / checkpointing | Zylos / appscale | 2026-03-04 | in-window |
| C1 A2A mesh + transport/state split | DEV article | 2026-04-10 | in-window |
| C1 A2A v1.0 LF (carried) | cycle-1 §D1 | 2026-03-14 | carried (in-window) |
| D1 auditable / verifiability-first | arXiv 2604.05485 / Augment | 2026-04 | in-window |
| D1 Verifiability-First Agents | arXiv 2512.17259 | 2025-12 | older-flagged (baseline) |

Counts: **in-window: 8** · **flagged-older (still current/fix-confirmed): 4** (A3 curve25519 advisory =
2024 but fix confirmed on pin; A4 SQLite advisories pre-pin + gated; B1 ESAA predecessor baseline; D1
Verifiability-First baseline). Carried-forward from cycle 1 (not re-dated as new): A6 shared stack,
C1 A2A v1.0 LF — see `research/rusty-idd.trends.md`, no supersession.

## F. Gaps / could-not-corroborate
- redb **4.0.0** exact GA date not pinned to one primary changeset (a small-model fetch returned an
  ambiguous year). Not load-bearing: handoff is on **4.1.0** (date HIGH-confidence via lib.rs + Phoronix).
- A4 "past all advisories" is bounded by the **listed** RustSec rusqlite/libsqlite3-sys advisories, not a
  per-CVE sweep of bundled SQLite — MED. Mitigated by the crates being feature-gated + out of the default
  runtime, and by CI `cargo audit`/`cargo deny`.
- A5 toml 0.9 migration not test-validated here — flagged as optional, verify-before-bump.
- B2/D1 lean partly on secondary/analyst sources + baseline-older arXiv — corroborated by multiple
  independent sources but rated MED-HIGH / mixed-window accordingly.

---

## Sources

Machine-readable ledger: `research/sources-handoff.jsonl` (one JSON object per cited source, with
url / title / publisher / accessed_at / published_at / in_recency_window / why_used / claim_ids).
Load-bearing sources (claim ids reference §A–§D and §E):

| Claim | Source URL | Publisher | Published / accessed | In-window |
|-------|-----------|-----------|----------------------|-----------|
| A1 redb current | https://lib.rs/crates/redb | lib.rs | 2026-04-19 / accessed 2026-06-26 | yes |
| A1 redb 4.1 perf | https://www.phoronix.com/news/Redb-4.1-Released | Phoronix | 2026 | yes |
| A1 redb ACID/crash-safe | https://github.com/cberner/redb | cberner/redb | accessed 2026-06-26 | yes |
| A1 no redb advisory | https://rustsec.org/advisories/ | RustSec | accessed 2026-06-26 | yes |
| A2 blake3 latest | https://docs.rs/crate/blake3/latest | docs.rs | accessed 2026-06-26 | yes |
| A3 ed25519 advisory | https://rustsec.org/advisories/RUSTSEC-2022-0093 | RustSec | 2023 (fixed 2.0) | no (fix-confirmed) |
| A3 curve25519 advisory | https://rustsec.org/advisories/RUSTSEC-2024-0344.html | RustSec | 2024-06-18 (fixed 4.1.3) | no (fix-confirmed) |
| A4 rusqlite advisories | https://rustsec.org/packages/rusqlite.html | RustSec | accessed 2026-06-26 | no (pre-pin) |
| A4 libsqlite3-sys advisory | https://rustsec.org/advisories/RUSTSEC-2022-0090.html | RustSec | 2022 (pre-pin) | no (pre-pin) |
| B1 ESAA-Conversational | https://arxiv.org/abs/2606.23752 | arXiv | 2026-06-22 | yes |
| B1 ESAA predecessor | https://arxiv.org/abs/2602.23193 | arXiv | 2026-02 | no (baseline) |
| B1 OpenHands SDK | https://arxiv.org/pdf/2511.03690 | arXiv | 2025-11 | no (baseline) |
| B2 durable execution | https://appscale.blog/en/blog/durable-execution-llm-agents-temporal-langgraph-checkpointing-2026 | AppScale | 2026 | yes |
| B2 checkpointing/resumability | https://zylos.ai/research/2026-03-04-ai-agent-workflow-checkpointing-resumability/ | Zylos Research | 2026-03-04 | yes |
| B2 LangGraph durable exec | https://docs.langchain.com/oss/python/langgraph/durable-execution | LangChain | accessed 2026-06-26 | yes |
| C1 A2A mesh/transport-state | https://dev.to/chunxiaoxx/building-multi-agent-ai-systems-in-2026-a2a-observability-and-verifiable-execution-10gn | DEV Community | 2026-04-10 | yes |
| D1 verifiability-first | https://arxiv.org/abs/2512.17259 | arXiv | 2025-12 | no (baseline) |
| D1 auditable agents | https://arxiv.org/pdf/2604.05485 | arXiv | 2026-04 | yes |
| D1 enterprise audit | https://www.augmentcode.com/guides/multi-agent-outputs-n-pass-enterprise-audit | Augment Code | accessed 2026-06-26 | yes |
| (carried) A2A v1.0 LF; shared stack | research/rusty-idd.trends.md §A,§D1 | cycle-1 note | 2026-06-26 | carried |
