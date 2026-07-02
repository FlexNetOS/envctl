# icm — Plan (cycle 7, fleet-convergence planning loop)

> Synthesized in foreground-Opus (token-frugal close; ~5% Opus budget) from the cycle-7 crew:
> cartographer + trend-researcher + 7 axis auditors + convergence analyst + test-strategist +
> verifier gate. Uses ONLY verifier-CONFIRMED/QUALIFIED + feasible facts. The analyst's "default
> 384d" figure was REFUTED by the verifier and is corrected here to **768d default (e5-base); 384d
> is the no-embedder fallback** — cosmetic const/comment drift, NOT a data bug (storage self-heals
> at runtime dim, schema.rs:17-37,416-464).

## Verdict
**icm is the canonical persistent-memory / semantic-vector organ of the fabric — a genuine PEER (not
parent or child, not redundant) of handoff's witnessed-continuity ledger and git-kb's code graph:
three disjoint corpora (agent knowledge / continuity events / code structure), overlapping only in
the vector-recall *mechanism*, which is corpus-separated so no plane collapses.** It is real and
buildable (4-crate workspace, `cargo build -p icm-store` EXIT 0; clean layering, 0 violations; 294
tests). BUT for the handoff+rusty-idd union it has two structural gaps: (1) it is **NOT bound as
data** — it binds only by CLAUDE.md/AGENTS.md convention + a connected MCP server + ad-hoc CLI calls
("graceful no-op if ICM absent"; zero code coupling to handoff), so it is an optional sidecar, never
authoritative continuity data; and (2) it carries an **unconditional C floor** (rusqlite{bundled} +
sqlite-vec always linked; ONNX/fastembed optional) which cannot live inside handoff's **no-C** (redb)
trust boundary. Convergence verdict = **SIDECAR**: icm stays a memory *service* the no-C kernel talks
to over its existing MCP/CLI seam (mirrors cycle-5's grit verdict), with a typed `memory` pointer
contract added to `handoff.context_capsule.v1` to make the binding DATA, and a fail-closed CI dep-gate
to make the no-C boundary mechanical.

## ASCII architecture
```
                 ┌──────────────────────────── meta fabric (north-star @ $META_ROOT + handoff) ───────────────────────────┐
                 │                                                                                                          │
   intent/why ──▶│  rusty-idd (intent control plane) ──┐                                                                   │
                 │                                       │   bind-as-data: capsule.memory{endpoint,scope,recall_contract}   │
 continuity ────▶│  handoff (no-C kernel: redb ledger) ─┼───────────────► [ icm memory pointer ] ◀── NEW typed contract    │
                 │        ▲  no-C trust boundary          │                        │                                        │
                 │        │  (CI dep-gate: deny rusqlite/ │                        │  MCP/CLI seam (existing)                │
                 │        │   sqlite-vec/onnx in kernel)  │                        ▼                                        │
                 │        └───────────────────────────────┘     ┌────────────── icm (SIDECAR memory service) ───────────┐ │
                 │                                               │ icm-cli (40 verbs, axum web) ─┐                       │ │
                 │   git-kb (code graph) — peer corpus           │ icm-mcp (31 tools, UNGATED) ──┤                       │ │
                 │                                               │ icm-store (SqliteStore) ──────┤  C FLOOR (unconditional)│
                 │   prompt_hub (intent store) — distinct plane, │   SQLite(rusqlite bundled) +  │  rusqlite+sqlite-vec   │ │
                 │   duplicate C-bearing FTS5+vector substrate   │   vec0 float[768] cosine +FTS5 │  +ONNX(optional)       │ │
                 │   (384 dims documented vs 768 real)           │ icm-core (Embedder: e5-base768)│                       │ │
                 │                                               └────────────────────────────────────────────────────────┘ │
                 │   data residency TODAY: user-global XDG (~/.local/share/icm, ~/.config/icm[creds], ~/.cache/icm)          │
                 │   → owner-wall migration to meta-owned root via ICM_CONFIG/[store]path/--db/XDG_CACHE_HOME (envctl-owned) │
                 └──────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

## Sequenced upgrade roadmap (ordered by convergence leverage × blast-radius; all 9 verifier-feasible)
1. **Bind-as-data: `memory` pointer in `handoff.context_capsule.v1`** (axis: accuracy/convergence) — typed {endpoint, scope, recall_contract} block so the union references icm as DATA, not convention. Blast: handoff capsule schema + icm export contract; cross-repo. **This is the headline + the RED contract's sibling.** risk: med.
2. **No-C CI dep-gate** (axis: governance) — fail-closed workflow denying rusqlite/sqlite-vec/onnx in the handoff kernel crates, making the SIDECAR boundary mechanical (mirrors ADR-0001/0017). risk: low.
3. **Recency/decay = the RED GREEN target** (axis: accuracy) — `apply_decay` (store.rs:1267, SQL :1290-1305) ignores `last_accessed`/`created_at`; make decay time-aware (Ebbinghaus/recency) so capsule-surfaced memories are recency-weighted. 5 RED tests already pin this (branch `plan/icm-red-tests`). risk: med.
4. **Bundled-SQLite currency check** (axis: governance/security) — CI is `cargo audit`-only and BLIND to bundled C (libsqlite3-sys 0.32 → SQLite **3.49.1**); add a manual/scripted bundled-SQLite version gate + bump rusqlite 0.34→0.40.x (SQLite 3.53.2). risk: low.
5. **Write-side governance on the 31 MCP tools** (axis: rules/policy) — destructive mutators (`forget`/`forget_topic`/`consolidate`/`update`) are ungated and injected identically into ~15 hosts; add capability gating (least-privilege; cf. prompt_hub RBAC). risk: med.
6. **Meta-owned data residency** (axis: filesystem) — OWNER-WALL: envctl redirects icm's XDG data/config/cache into a meta-owned root (lever already exists: ICM_CONFIG/[store]path/--db/XDG_CACHE_HOME); envctl owns preview/apply/lock/rollback/parity. risk: med (owner-wall).
7. **Provenance-aware recall** (axis: accuracy/security) — `MemorySource` is recorded but NOT trust-weighted in recall (memory-poisoning surface, 2026); add admission + trust-weighting. risk: med.
8. **Stale-doc + toolchain hygiene** (axis: governance) — replace the 26.2K French CLAUDE.md (describes an abandoned Turso/libsql+1536d stack) with the real rusqlite+fastembed truth; pin the rust-toolchain; gitignore the committed `.claude/scheduled_tasks.lock`; untrack `web/dist`. risk: low.
9. **sqlite-vec / fastembed currency** (axis: speed/accuracy) — bump sqlite-vec 0.1.6→0.1.9 (fixes DELETE on long-metadata vec0 = icm's shape) and fastembed 4.9.1→5.x; consider ANN over brute-force KNN and RRF over the fixed 0.3/0.7 blend. risk: low.

## Tool-evaluation (currency + advisories; from trend-researcher, verifier-checked version facts)
- **rusqlite 0.34 / libsqlite3-sys 0.32 → bundled SQLite 3.49.1 (CONFIRMED from sqlite3.h).** Upstream shipped CVE fixes since; `cargo audit` cannot see bundled C (no RustSec mapping) — a real supply-chain blind spot. The specific CVE-2026-11822 (FTS5) is research-supplied, not repo-adjudicable, but the *mechanism* (audit-blind bundled C) is CONFIRMED.
- **sqlite-vec 0.1.6** → 0.1.9 (DELETE bug on long-metadata vec0, exactly icm's shape). **fastembed 4.9.1** → 5.x. **ureq 2.x** → 3.x. clap/serde/serde_json/ulid current, advisory-free.
- Embedding lane: default `intfloat/multilingual-e5-base` = **768d** (real), 384 = no-embedder fallback; `DEFAULT_EMBEDDING_DIMS=384` const is a naming/comment defect, storage is dim-consistent at runtime.

## Governance
26.2K stale French CLAUDE.md (abandoned Turso/libsql+1536d) — wrong info + per-session token tax; `.claude/` is a rusty-idd thin adapter but there is no `.idd/` plane; adapter claims a `render --check` CI gate no workflow runs; committed runtime lock `.claude/scheduled_tasks.lock`; no toolchain pin (rides `@stable` with `-D warnings`); no deny.toml/dependabot/SECURITY.md; `cargo audit`-only gate is bundled-C-blind. (CONFIRMED.)

## Filesystem layout
Repo-native Cargo layout is clean (4 crates; sound .gitignore for target/*.db). Runtime data plane is **user-global XDG** (`~/.local/share/icm/memories.db`, `~/.config/icm/credentials`, `~/.cache/icm`) — none meta-owned, none symlinked into meta → diverges from handoff's `$META_ROOT` residency. Cheap redirect lever exists. Two tracked-file drifts: live `.claude/scheduled_tasks.lock`, generated `web/dist`. (CONFIRMED.)

## Memory/vector
Canonical memory + semantic-vector plane. Model REAL but crude vs 2026: static importance (MAX-merged on dedup), recall-event-quantized 0.95 decay gated to 24h, no episodic/semantic/procedural tiering, no A-MAC. Hybrid FTS5 ⊕ vec0 cosine with a fixed linear 0.3/0.7 blend (not RRF) over brute-force KNN (no ANN). vec0 float[**768**] cosine at runtime. Provenance recorded but not trust-weighted. (CONFIRMED; 768 corrected from analyst's 384.)

## Auto-research
Zero git-kb/code-intelligence integration (`learn` = static file scan, no symbol→memory edges → findings never auto-invalidate on code change). Cannot fetch web (no URL `MemorySource`, no dated provenance). No recency window on `recall`; `icm_wake_up` does soft `importance×recency×weight` ranking (not a hard window). (Analysed; verifier left `[~]`.)

## Rules/policy
Upgrade-Only/No-Downgrades PASS (additive migrations w/ brick-on-upgrade test; decay weight-only, skips `critical`; `prune` opt-in, spares critical+high). icm = shared memory bus but UNGOVERNED (no write-side RBAC; any caller can `forget` any topic; agent/role are metadata not principals). weave/A2A absent (local single-process; only RTK cloud-sync egress). Background writes correctness-safe (WAL + 30s busy_timeout + atomic upserts) but unpooled → possible hard BUSY under wide fan-out. (Analysed; verifier left `[~]`.)

## Distributed compute
Pure-Rust source, unconditional C floor (rusqlite bundled + sqlite-vec); ONNX optional/flaggable. Ships 5 desktop/server 64-bit targets; aarch64-Linux covers 64-bit Pi; no 32-bit Pi Zero / mobile / iOS-Android; ESP32 impossible (std + SQLite-C). Lua/Luau = N/A (hard-coded Rust policy). Local-first; one optional single-vendor RTK cloud-sync; local fastembed embedding vendor with an `Embedder` trait offload seam. C-floor vs handoff no-C (redb) = the SIDECAR crux. (CONFIRMED; verifier left dim `[~]`.)

## Test Strategy
RED contract = **dynamic, time-aware (recency/Ebbinghaus) importance & decay** (most evidence-grounded + behavioral against the existing public API). **tests-ran: 5** (`cargo test -p icm-store --test recency_decay_red` → 0 passed / 5 failed / 0 ignored), clippy-clean under CI flags, additive-only, commit `258667e` on `plan/icm-red-tests`. RED for the right reason (`apply_decay` never reads `last_accessed`/`created_at`). FF test-build spec: make decay time-aware → tests go GREEN; this is the Feature-Forge handoff.

## Prompt-architecture
recall→prompt-injection is ad-hoc markdown assembly (no versioned envelope; only `EMPTY_PACK_HEADER` stable). 31 MCP tools dispatched ungated, destructive mutators equal to reads, injected identically into ~15 hosts (auditable/reversible via versioned install-manifest). Two LLM-adjacent lanes: embedding (fastembed e5-base 768) + a summarization lane shelling to host CLIs (claude-haiku-4-5/gpt-5-mini/claude-sonnet-4-5). No ADR set in-tree; 5 ADR-candidates emitted (recall envelope; tool-grant least-privilege; embedding-model lane; summarization delegate; icm↔handoff↔rusty-idd memory-ownership boundary). (Analysed; verifier left `[~]`.)

## Risk policy
See `risk-policy.md ## icm`. SUPERVISED items: data-residency migration (owner-wall; envctl owns apply/lock/rollback/parity), any cloud-sync credential handling, destructive memory mutators. Trust-boundary: C-floor must stay OUTSIDE handoff's no-C kernel (SIDECAR + CI dep-gate).

## Confidence
**HIGH** on plane-identity / peer-status / C-floor / SIDECAR / not-bound-as-data / residency / build-probe (all code-cited, verifier-CONFIRMED). **MEDIUM** on the long-horizon RVF single-no-C-vector-core consolidation and on the specific bundled-SQLite CVE id (research-supplied). 4 dimensions (autoresearch, rules-policy-org, distributed-compute, prompt-architecture) analysed-not-adjudicated → icm stays **`[~]` planned-with-gaps**, not `[x]`.
