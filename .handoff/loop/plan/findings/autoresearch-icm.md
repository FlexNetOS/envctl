# autoresearch — icm (cycle 7)

Target: **icm** (persistent-memory organ). Repo: `/home/drdave/Desktop/meta/icm`.
Axis: `autoresearch`. Frame: meta = one converging system; icm is the substrate an
auto-research loop would WRITE findings to (`store`) and READ from (`recall` / `wake_up`).
Read-only audit; only this findings file written.

Verdict up front: icm is a memory **store**, not a researcher. It has **no code
auto-research (no git-kb)** and **no web auto-research (no fetch, no recency window)**.
It DOES have an automatic time-decay freshness signal (weight), but **no content/source
invalidation** and **no automatic expiry** — so stale findings persist and can be pinned
fresh by mere re-access. Three freshness gaps detailed below.

Markers present: `code auto-research` / `git-kb`; `web auto-research` / `90-day` / `recency`; `stale` / `invalidate`.

---

## 1. Code auto-research (git-kb relationship)

ICM does not integrate with git-kb code intelligence and does not link stored memory to
code symbols/files, so memory cannot be invalidated when code changes.

| # | CLAIM | Evidence (file:line) | Verdict |
|---|-------|----------------------|---------|
| C1 | **Zero git-kb / code-intelligence integration.** No callers/callees/symbols/call-graph linkage anywhere in icm. | grep across `crates/**/*.rs` for `git.kb`/`gitkb`/`call.graph`/`kb_symbols`/`kb_callers`/`code.intelligen` → **0 hits**. | CONFIRMED |
| C2 | icm has its OWN code scanner (`learn`), but it is a **file/dir scan, not an AST/call-graph**: identity, dependencies, modules, entrypoints, configs, scripts. | `crates/icm-core/src/learn.rs:27-53` (`learn_project` → `scan_dependencies`/`scan_modules`/`scan_entrypoints`/`scan_configs`/`scan_scripts`). | CONFIRMED |
| C3 | `learn` is a **static snapshot, delete-and-recreate** on re-run — never auto-invalidated when code changes; goes stale until a human re-runs `icm learn`. | `crates/icm-core/src/learn.rs:37-45` (`if let Ok(Some(existing)) … delete_memoir` then recreate). | CONFIRMED |
| C4 | The ONLY code linkage on a memory is `MemorySource::ClaudeCode { file_path: Option<String> }` — a plain string label, never re-checked against the file. There is no symbol→memory edge. | `crates/icm-core/src/memory.rs:130-141`. | CONFIRMED |
| C5 | Auto-linking links **memory↔memory** by embedding cosine similarity (`related_ids`), NOT memory↔code. | `crates/icm-core/src/auto_link.rs:1-91` (`auto_link_memory`, `add_backrefs`). | CONFIRMED |
| C6 | Consequence for the loop: a finding stored in icm that cites `foo.rs::bar` cannot auto-invalidate when `bar` changes — no edge exists from the code graph to the memory row. icm and git-kb are orthogonal substrates with no bridge. | Synthesis of C1–C5. | CONFIRMED |

## 2. Web auto-research (90-day / recency window)

ICM cannot fetch the web, stores no dated/sourced web provenance, and applies no recency
window on recall.

| # | CLAIM | Evidence (file:line) | Verdict |
|---|-------|----------------------|---------|
| W1 | **`web.rs` is NOT web research** — it is the Axum HTTP dashboard serving an embedded SvelteKit SPA over local memories. | `crates/icm-cli/src/web.rs:1-29` (module doc "Web dashboard … Axum HTTP server with embedded SvelteKit SPA"; `#[derive(Embed)] WebAssets`). | CONFIRMED |
| W2 | The only outbound HTTP (`ureq`) is for LLM summarizer (Ollama), cloud sync, Anthropic token-count bench, and self-upgrade download — **never doc/source fetching for research**. | `summarizer.rs:283`, `cloud.rs:235/326/367`, `bench_format.rs:180`, `upgrade.rs:41/57`. | CONFIRMED |
| W3 | No `MemorySource` variant for Web/URL; no `source_url` / `fetched_at` / `published_at` field. A web finding's provenance/date can only live as free text inside `summary`. | `crates/icm-core/src/memory.rs:130-141` (only `ClaudeCode`, `Conversation`, `Manual`). | CONFIRMED |
| W4 | **No 90-day window / recency cutoff on recall.** Hybrid recall applies project/topic/keyword filters + graph expansion only — no date filter. | `crates/icm-mcp/src/tools.rs:1149-1235` (`tool_recall`); `crates/icm-store/src/store.rs:1153-1225` (`search_hybrid` = 0.3·FTS + 0.7·vector, no time term). | CONFIRMED |
| W5 | Keyword/FTS/topic recall order by `weight DESC` only — weight is a *soft* decayed signal (see §3), not a recency window. | `store.rs:1045` (keywords), `store.rs:1079` (fts), `store.rs:1340` (by_topic). | CONFIRMED |
| W6 | **Correction to trend-researcher** ("no recency filter on recall"): TRUE for `recall`, but `icm_wake_up` DOES rank by `importance × recency × weight`, recency = `1/(1+days/30)` (~0.5 @30d, ~0.25 @90d). Soft decay, still **not** a hard window. | `crates/icm-core/src/wake_up.rs:93,198-213` (`compute_score`). | CONFIRMED |

## 3. Stale / invalidate (decay / forgetting / consolidation)

Importance is static; weight decays automatically; but expiry and content/source
invalidation are absent — the freshness gap.

| # | CLAIM | Evidence (file:line) | Verdict |
|---|-------|----------------------|---------|
| S1 | **Importance is STATIC** — a 4-level enum never auto-mutated after store (dedup only takes the max on re-store). | `crates/icm-core/src/memory.rs:96-103`; re-store upgrade-only at `store.rs:842-845` (`max_importance`). | CONFIRMED |
| S2 | **Weight IS dynamic.** `apply_decay` multiplies weight by an importance- and access-weighted factor (critical never decays; high 0.5×, medium 1×, low 2×; access slowdown capped at 5). | `crates/icm-store/src/store.rs:1267-1311`. | CONFIRMED |
| S3 | Decay is **automatic on recall**: `maybe_auto_decay` applies 0.95 if >24h since last, called in both recall paths. | `store.rs:130-151` (`maybe_auto_decay`); callers `crates/icm-cli/src/main.rs:1680`, `crates/icm-mcp/src/tools.rs:1156`. | CONFIRMED |
| S4 | **Expiry (`prune`) is NOT automatic** — only manual (`icm prune`, dashboard, TUI). Stale low-weight memories accumulate until a human acts. | `prune` def `store.rs:1313-1334`; only manual callers `main.rs:3233`, `web.rs:580`, `tui.rs:626` (rest are tests). | CONFIRMED |
| S5 | Staleness is *defined and surfaced but not acted on*: `topic_health` flags `weight<0.5 AND last_accessed>14d` as `stale_count`; `icm health` just reports it. | `store.rs:1446-1510` (`topic_health`); `TopicHealth::status` `crates/icm-core/src/memory.rs:188-200`. | CONFIRMED |
| S6 | `consolidate_topic` merges a topic into one row with **weight RESET to 1.0** — it re-freshens (can resurrect a decayed memory), it does NOT expire stale content. | `store.rs:1378-1426`; reset at `web.rs:477-479` (`consolidated.weight = 1.0`). | CONFIRMED |
| S7 | `extract_patterns` is read-only keyword clustering (no expiry); `forget`/`forget_topic` are manual deletes by id/topic. | `PatternCluster` `memory.rs:162-173`; `tools.rs:1293` (`tool_forget`), `tools.rs:1305` (`tool_forget_topic`), `tools.rs:1525` (`tool_extract_patterns`). | CONFIRMED |
| S8 | **No content/source invalidation.** A stored finding that becomes factually wrong is never flagged; weight only decays slowly, and any recall RESETS recency (`last_accessed`, `access_count++`), so a stale-but-frequently-recalled fact stays pinned near the top. | recall bumps access `tools.rs:1224-1226` (`batch_update_access`); decay uses access count `store.rs:1300`; recency uses `last_accessed` `wake_up.rs:205-210`. | CONFIRMED |

---

## Upgrade rows (axis: autoresearch)

| # | UPGRADE | Axis | Rationale (gap) | Acceptance | Risk | Reversibility |
|---|---------|------|-----------------|------------|------|---------------|
| U1 | Add a `MemorySource::Web { url, fetched_at, published_at }` variant (and optionally `MemorySource::CodeRef { repo, symbol, file, rev }`) so web/code findings carry dated, sourced provenance instead of free-text. | autoresearch | W3, C4 — no provenance field; loop cannot record source/date structurally. | New variant round-trips through `store_inner`/`row_to_memory`; recall renders source+date; existing 3 variants unaffected. | Med (schema/serde + dedup hash interplay) | High (additive enum variant + migration; old rows default to `Manual`). |
| U2 | Add an optional recency window to recall (`max_age_days` / `since`) and/or fold a recency term into `search_hybrid`'s combined score (mirror `wake_up::compute_score`). | autoresearch | W4, W5, S8 — semantic recall ignores time entirely; stale facts rank by similarity alone. | `recall(query, max_age_days=90)` excludes older rows; default behavior unchanged when arg absent. | Low | High (additive arg; default off). |
| U3 | Make expiry a scheduled/auto step (run `prune` on the same >24h cadence as `maybe_auto_decay`, behind a config flag), with a dry-run report. | autoresearch | S4, S5 — decay auto-runs but expiry never does; stale accumulates. | After N idle days a low-importance weight<θ memory is pruned automatically; critical/high never pruned; report emitted. | Med (data loss if mis-tuned) | High (config-gated, off by default; dry-run first). |
| U4 | Add a code↔memory invalidation hook: when icm stores a memory tagged with `file_path`/`CodeRef`, record the file rev; a re-`learn`/git-kb sync flags or down-weights memories whose cited file changed. | autoresearch | C2, C3, C6 — no code-change invalidation; `learn` snapshots are static. | Editing a cited file marks linked memories `needs-review` (weight penalty or flag) on next sync. | High (cross-substrate bridge to git-kb; new coupling) | Med (bridge is additive but introduces a dependency on git-kb presence — keep graceful-degrade). |
| U5 | Distinguish "freshness via re-access" from "freshness via re-verification": stop letting plain recall reset recency for `Web`/`CodeRef` memories (only an explicit re-store/verify should refresh `published_at`/rev). | autoresearch | S8 — recall pins stale facts by bumping `last_accessed`. | A web/code memory recalled but not re-verified continues to decay; only verify refreshes its dated provenance. | Med (changes access semantics for a subset) | High (scoped to new source variants from U1). |

## Gate handoff (tests that fail closed on missing stale-evidence checks)

| Gate | Asserts | Status |
|------|---------|--------|
| G1 | A `recall(max_age_days=90)` over a fixture with one 200-day-old and one 1-day-old memory returns ONLY the fresh one. | RED until U2 (no recency arg exists today — `tools.rs:1149-1235`). |
| G2 | Storing a web finding preserves a structured source URL + fetched date retrievable after round-trip (not buried in summary text). | RED until U1 (no `Web` source variant — `memory.rs:130-141`). |
| G3 | After simulated idle (>θ days, weight<θ, importance=low), an auto-expiry pass removes the row without manual `icm prune`. | RED until U3 (prune is manual only — `store.rs:1313-1334`, callers `main.rs:3233`/`web.rs:580`/`tui.rs:626`). |
| G4 | Editing a file cited by a `CodeRef` memory marks that memory `needs-review` on next sync. | RED until U4 (no code↔memory edge — C1/C4). |

## N/A items
- "git-kb command set used by icm" — N/A — icm invokes no git-kb commands (C1); the loop's code-graph refresh runs in git-kb, then findings are *stored into* icm with no live link.
- "icm web-source ledger rows" — N/A — icm has no web-fetch/source-ledger concept (W1–W3); the source ledger lives in the planning loop, not in icm.
- "vendor advisory/CVE recall" — N/A — icm stores no dated advisories (W3); no recency to gate them (W4).

## Confidence
High. All claims read directly from source at cited lines; cross-checked grep for absence
(git-kb, web-fetch, recency filter) and presence (decay/prune/wake-up recency). The one
nuance vs prior trend-researcher input (recall vs wake_up recency) is resolved in W6.
