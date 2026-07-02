# Autoresearch findings — grit (cycle 5)

| field | value |
|---|---|
| target | **grit** — symbol-level merge/lock substrate (tree-sitter AST indexer + SQLite/S3/Azure lock store) for parallel AI agents |
| code root (READ-ONLY) | /home/drdave/Desktop/meta/grit |
| snapshot | `@57b6084` · branch `master` · Cargo `grit 0.4.0` · cycle 5 |
| audited | 2026-06-27 |
| recency window (90d) | 2026-03-29 → 2026-06-27 (from today; no grit-specific trends note exists yet — see W4) |
| inputs | `Cargo.toml`, `.github/workflows/ci.yml`, `src/parser/mod.rs`, `src/db/mod.rs`, `src/db/lock_store.rs`, `src/git/mod.rs`, `src/cli/mod.rs`, `CHANGELOG.md`; live `git kb code stats`/`doctor` over grit; crates.io / azure.github.io / devops.com (web) |
| verdict | grit has **NO repo-native code auto-research** (no `.kb/` in tree, no git-kb config, no CI re-index/index step) and **NO web auto-research** of any kind (no dependabot/renovate, no deny.toml/cargo-audit, no advisory gate, no source ledger, no recency window). Its dependency surface is materially **stale** (rusqlite 0.31 vs ~0.40.1; azure_* 0.21 on the now-deprecated pre-GA Azure-SDK-for-Rust crate line). It DOES carry a strong **runtime lock-staleness** model (lock TTL + `gc_expired_locks` + PID-liveness merge-lock reaping) that is the runtime analogue of — not a substitute for — research-staleness invalidation. Frame this as a convergence upgrade: plug grit into the fleet autoresearch loop. |

---

## 1. Code auto-research

**CLAIM C1 — grit has NO repo-native code-intelligence index in-tree: no `.kb/`, no git-kb config, no CI index/re-index step. [CONFIRMED]**
- `ls .kb` → `No such file or directory`; `git status --short` clean after running `git kb code stats` (the index is NOT stored in grit's working tree); no `config.toml` under any `kb` path; no `.gitkb`.
- `.github/workflows/ci.yml` has exactly one job (`test`, macOS + Linux matrix) running `cargo build` → `cargo test` → `cargo fmt -- --check` → `cargo clippy -- -D warnings` (ci.yml:11-58). There is **no** indexing / kb / re-index / graph-snapshot step.
- Evidence: directory listing of `/home/drdave/Desktop/meta/grit`; `.github/workflows/ci.yml:11-58`.

**CLAIM C2 — code auto-research for grit is satisfied HARNESS-SIDE: a `git-kb` (v0.2.10) branch-aware index over grit already exists and is fresh, built by the plan loop, not by grit. [CONFIRMED]**
- `git kb code stats` (run 2026-06-27): **511 symbols / 995 call edges / 3,094 unresolved calls / 93 files**; `Last indexed: 2026-06-27T05:11:49Z`; `Stale files: 0`. Languages: rust 385, typescript 106, python 20 (the ts/py come from grit's `test-projects/` + `examples/` fixtures).
- `git kb code doctor` (branch `master`, source-scoped): 511 symbols / **38 files** / 995 edges; unresolved by reason `no_match` 1,977 (63.9%), `skip_list` 1,008 (32.6%), `ambiguous` 101 (3.3%), `stdlib_allowlist` 8. Top files by unresolved calls: `src/cli/mod.rs` 488, `src/parser/mod.rs` 474, `src/db/mod.rs` 332, `src/git/mod.rs` 267, `src/db/sqlite_store.rs` 230, `src/db/s3_store.rs` 182.
- So the per-cycle snapshot/diff, entrypoint/public-API, hotspots, dead-code, and unresolved-call facts are produced by the **plan-cartographer** via `git kb code` (commands: `git kb code stats|doctor|symbols --json|callers|callees|impact|dead|entrypoints`), external to grit's tree. This finding records WHERE code auto-research happens; the cartographer artifact re-derives the graph metrics.
- The 3,094 unresolved calls (63.9% `no_match`) are expected for a tree-sitter call-resolver over a CLI-heavy crate, but they are the concrete "unresolved calls" surface the verifier/cartographer must track cycle-over-cycle.

**CLAIM C3 — grit is ITSELF a tree-sitter symbol/dep extractor, but it indexes OTHER repos for lock coordination, never itself for self-research. [CONFIRMED]**
- `src/parser/mod.rs` defines `SymbolIndex` with `scan_all()` (parser/mod.rs:250) and `scan_with_deps()` (parser/mod.rs:101) — full symbol + call-dep extraction across **14 languages** (`lang_configs()`, parser/mod.rs:426-571: ts/js/rust/python/c#/go/java/c/cpp/ruby/php/swift/kotlin). `grit symbols` (CLI Symbols, cli/mod.rs:84-89) surfaces these.
- This engine exists to populate the **lock table** (symbol-level claims), i.e. grit consumes a target repo's AST so agents can lock symbols — it is not turned on grit's own knowledge currency. grit therefore has the *capability* to self-index but no wiring/cadence to do so.

**Assessment.** grit is read-only-correct from the harness's view: the loop's code auto-research is the harness `git kb code` snapshot/diff (C2). The **gap** is twofold: (a) no *repo-native* freshness signal — a contributor working in grit alone gets no guarantee the committed graph/symbol view matches HEAD and no CI guard; (b) grit's own indexer (C3) is never pointed at grit, so there is no self-hosted code-intelligence artifact. Both are addressable without touching grit's trust boundary (see U1, U4).

## 2. Web auto-research

**CLAIM W1 — grit has NO automated dependency-update / advisory bot: no `.github/dependabot.yml`, no `renovate.json`/`.renovaterc`, no `deny.toml`, no `cargo-audit`/`.cargo/audit.toml`. [CONFIRMED]**
- `ls` for `.github/dependabot.{yml,yaml}`, `renovate.json`, `.renovaterc*`, `deny.toml`, `audit.toml`, `.cargo/audit.toml` → all `No such file or directory`.
- `ci.yml` (the only CI besides release tooling) runs **no** `cargo audit` / `cargo deny` / advisory job. There is therefore **zero** time-windowed *or* event-driven web pull of new crate releases or RUSTSEC advisories anywhere in the repo. This is a strictly weaker posture than the fleet sibling weave, which at least has an event-driven `deny.toml` + supply-chain audit CI gate.

**CLAIM W2 — grit's dependency surface is materially STALE against current upstream (web-verified). [CONFIRMED — dated]**
- `rusqlite = "0.31"` (Cargo.toml:14). Current upstream is the **0.38 / 0.40.1** line (docs.rs `rusqlite 0.38.0` latest; crates.io notes bundled SQLite 3.53.2 at rusqlite 0.40.1 / libsqlite3-sys 0.38.1) — grit is ~7-9 minor releases behind on its primary storage crate. Source: crates.io / docs.rs `rusqlite`, accessed 2026-06-27.
- `azure_core = "0.21"`, `azure_storage = "0.21"`, `azure_storage_blobs = "0.21"` (Cargo.toml:62-64). These `0.21` crates are the **pre-GA / predecessor** Azure-SDK-for-Rust line. Microsoft brought the **official Azure SDK for Rust to General Availability** (DevOps.com; Azure SDK Releases Feb/Mar 2026 at azure.github.io) on a *new* crate lineage (`azure_core` 1.x) — the old `azure_storage*` 0.21 crates are effectively superseded/deprecated. Source: devops.com "Microsoft Brings the Azure SDK for Rust to General Availability"; azure.github.io/azure-sdk/releases/2026-02|2025-03; accessed 2026-06-27.
- These are exactly the "is dep X current" facts that a **90-day recency** web pass (harness trend-researcher) or a renovate bot would surface automatically; today grit surfaces none of them.

**CLAIM W3 — there is no official-docs-first source ledger and no contradiction-check surface for grit. [CONFIRMED]**
- grit's tree carries no `research/`, no source ledger, no SECURITY/advisory doc, no "deps verified on <date>" record. CHANGELOG.md (release-please generated) tracks features/fixes, not dependency currency or advisory state (CHANGELOG.md:1-25).

**CLAIM W4 — no grit-specific harness trends note / source ledger exists yet for this cycle; the 90-day window is computed but unfilled. [CONFIRMED]**
- The loop's `research/` dir holds `agentic-planning-trends-2026-06.md` and `plan-architecture-loop-distributed-compute-2026-06.md` only — neither is grit-scoped; `graph/` holds only `.gitkeep` (no committed grit graph snapshot). So unlike weave (which has `research/weave.trends.md` + `graph/weave.metrics.json`), grit has **no** filled source-ledger/recency artifact this cycle. The recency window in the header is derived from today's date, not from an existing ledger. This is itself an autoresearch gap the loop must close (the trend-researcher must emit `research/grit.trends.md`).

## 3. Cadence + stale-evidence invalidation

**CLAIM S1 — grit has NO research-staleness cadence or invalidation of ANY kind (code or web). [CONFIRMED]**
- No CI cron / `schedule:` trigger (ci.yml triggers are `push`/`pull_request` on master|develop + `workflow_dispatch` only, ci.yml:3-7) — so even the build/test gate is event-driven, and there is no periodic re-index, re-audit, or recency re-check. Nothing in grit detects or invalidates stale dependency pins, stale advisory state, or a stale code graph. Per-cycle / batch / resume cadence for grit is therefore **entirely harness-side** (plan-loop cartographer + trend-researcher), with no repo-side counterpart to anchor it.

**CLAIM S2 — grit DOES carry a strong runtime LOCK-staleness model: lock TTL + `gc_expired_locks` + availability-excludes-expired. This is the runtime analogue of research-staleness invalidation. [CONFIRMED — core in-repo precedent]**
- `LockStore` trait declares `ttl_seconds` on acquire (lock_store.rs:11,34) plus `gc_expired_locks()` (lock_store.rs:41) and `refresh_ttl()` (lock_store.rs:42).
- The locks table defaults `ttl_seconds INTEGER DEFAULT 600` (db/mod.rs:98) with a forward migration for pre-TTL DBs (db/mod.rs:133-146).
- Availability **excludes expired locks**: the join keeps a lock only while `(julianday('now') - julianday(l.locked_at)) * 86400 <= COALESCE(l.ttl_seconds, 600)` (db/mod.rs:188-193) — i.e. an expired lock is silently **invalidated** and the symbol is reported free again. Proven by `test_availability_ignores_expired_locks` (db/mod.rs:794-817): a 1s-TTL lock locked in the past must NOT occupy the symbol.
- `grit gc` (CLI `Gc` — "Garbage-collect expired locks", cli/mod.rs) is the operator-facing reap. `grit heartbeat` (cli/mod.rs:166) + `refresh_ttl` keep a live holder's lock from expiring.

**CLAIM S3 — grit's merge-lock reaper is a second TTL→invalidate→reclaim precedent (PID liveness, with a 30s time fallback). [CONFIRMED]**
- `src/git/mod.rs:296-337`: on an existing merge-lock, grit decides staleness by a definitive PID liveness check (`kill -0`); a **dead** holder marks the lock stale and it is removed/reclaimed (git/mod.rs:316-336). When liveness is unknowable it falls back to a time heuristic — a lock older than **30s** is treated stale (git/mod.rs:331). A *live* holder is never stolen regardless of age (git/mod.rs:299-304).

**CLAIM S4 — the lock-staleness model (S2/S3) is N/A as code/web auto-research, but is the precedent the research-staleness TTL should mirror. [CONFIRMED]**
- "Constant runtime auto-research" for grit maps onto its **lock liveness** subsystem, not onto evidence research: TTL → expire → invalidate → reclaim is exactly the shape of "evidence past its recency window → flag stale → re-fetch." It neither re-indexes grit's graph nor re-pulls advisories/releases, so for the autoresearch axis it is **N/A — runtime lock-liveness cadence, not code/web evidence refresh**. It is, however, the strongest in-repo argument that grit's authors already understand TTL-based invalidation and would accept a research-staleness TTL of the same form (U3).

## 4. Upgrade rows (axis: autoresearch)

| id | upgrade | axis | evidence | acceptance | risk | reversibility |
|---|---|---|---|---|---|---|
| U1 | Add a repo-native code-graph freshness gate: a CI step (or pre-commit) that runs `git kb code index` + `stats` over `src/` and fails if the committed graph snapshot drifts from HEAD (mirrors the harness cartographer, but in-grit). | autoresearch | C1 (no `.kb/`, no CI index step), C2 (graph is harness-side only; 511 sym / 995 edge / 0 stale baseline) | CI job exists; editing a public symbol without refreshing the snapshot fails the check; `cargo build`/`test` untouched. | low — additive CI only; git-kb has no C dep; no trust-boundary code. | high — delete the job + snapshot. |
| U2 | Stand up web auto-research: add a scheduled `renovate.json` (or `dependabot.yml`) **and** a `deny.toml` + `cargo deny check advisories` CI job, so new releases (rusqlite 0.31→0.40.x) and RUSTSEC advisories are surfaced/gated automatically instead of never. | autoresearch | W1 (no bot/audit/deny), W2 (rusqlite + azure_* materially stale, web-dated 2026-06-27) | a scheduled PR-raising bot config lands; a stale pin raises an automated PR; an un-waived advisory fails CI. | low-med — bot PRs add noise; azure_* major-line bump (0.21→GA 1.x) is a real migration to scope, not auto-merge. | high — remove config files. |
| U3 | Add an explicit research-staleness TTL to grit's harness artifacts (`research/grit.trends.md` recency window) modeled on the lock TTL, so out-of-window evidence auto-flags the same way an expired lock is dropped from availability. | autoresearch | S2 (`ttl_seconds` default 600, `gc_expired_locks`, availability-excludes-expired db/mod.rs:188-193), S3 (merge-lock 30s/PID reaper), W4 (no ledger yet) | a stated TTL on the trends note; an out-of-window source is flagged exactly as an expired lock is invalidated. | low — docs/convention only. | high — revert doc. |
| U4 | Convergence: point grit's own 14-language `SymbolIndex` (C3) at grit and at the union targets (handoff/rusty-idd shared crates) so grit's self-extracted symbol graph feeds the union step-2 dedup — closing the loop where the merge/lock substrate also self-hosts its code-intelligence. | autoresearch | C3 (`scan_with_deps` parser/mod.rs:101; 14 langs:426-571), loop_state Frame (grit powers union step 2) | grit can emit its own symbol/dep JSON for grit + union crates; output cross-checks the harness git-kb graph (C2). | med — new wiring; must not alter lock semantics. | high — feature-gated; remove the self-index path. |

## 5. Gate handoff — tests proving stale-evidence checks fail closed

grit already proves the **runtime** staleness path fails closed (the in-repo precedent for the autoresearch TTL):
- **`test_availability_ignores_expired_locks`** (db/mod.rs:794-817) — asserts an expired (past-`locked_at`, low-TTL) lock does NOT occupy its symbol; i.e. stale lock evidence is invalidated, not honored. The dual case (refresh restores occupancy) is exercised at db/mod.rs:817.
- **PID-liveness merge-lock reaper** (git/mod.rs:296-337) — a dead holder's lock is provably reclaimed; a live holder's is provably never stolen. Both directions fail closed.

**Gap for the loop to add (RED handoff):** there is currently **no** test that grit's *code graph snapshot* is fresh (U1) and **no** test that crate pins are not stale beyond the recency window / carry an un-waived advisory (U2), and **no** `research/grit.trends.md` ledger to invalidate against (U3/W4). RED tests to author (additive, read-only on production code):
1. **U1 graph-freshness** — assert `git kb code stats` over `src/` yields a symbol/edge count matching the committed snapshot (fail on drift); baseline today = 511 symbols / 995 edges / 0 stale files (C2).
2. **U2 currency/advisory** — assert no direct dependency is >1 minor behind its crates.io latest without a documented waiver (would fire today on rusqlite 0.31 and azure_* 0.21, W2), and that `cargo deny check advisories` passes.
3. **U3 ledger-staleness** — assert `research/grit.trends.md` exists and every source row is within the 90-day window or explicitly flagged, mirroring the lock TTL invalidation shape (S2).

---

### Required markers (gate)
- **code auto-research** / **git-kb**: §1 — grit has no repo-native git-kb index in-tree (C1); code auto-research is the harness's per-cycle `git kb code` snapshot (511 sym / 995 edge, C2); grit's own tree-sitter indexer is never turned on grit itself (C3).
- **web auto-research** / **90-day** / **recency**: §2 — grit has NO web auto-research at all (no dependabot/renovate/deny/audit, W1); deps are stale and web-dated (rusqlite 0.31, azure_* 0.21 pre-GA, W2); no source ledger and the 90-day recency window is unfilled this cycle (W3/W4).
- **stale** / **invalidate**: §3 — no research-staleness cadence (S1); the lock TTL `gc_expired_locks` + availability-excludes-expired path is the runtime analogue that invalidates stale locks (S2), as is the 30s/PID merge-lock reaper (S3); both are precedents, not code/web evidence refresh (S4).

### Confidence
HIGH on all in-repo facts (CI shape, absence of .kb/dependabot/renovate/deny, lock TTL + reaper mechanics, parser capability — all from cited source + live `git kb code` over grit). HIGH on the dependency-staleness web facts (rusqlite line, Azure SDK GA), dated 2026-06-27 — version-exactness on rusqlite latest (0.38 vs 0.40.1) is from docs.rs/crates.io summaries and should be re-pinned by the trend-researcher's `research/grit.trends.md`. MEDIUM on U2/U4 scope (azure 0.21→GA-1.x is a real migration; the self-index convergence is new wiring) — appetite-dependent, not correctness-dependent.
