# prompt-hub — decision-grade plan (cycle 6)

- Target: `prompt-hub` (the Front-Door intent STORE; core lib `prompt-hub` + CLI `prompthub` + axum `prompthub-server`).
- Target root: `/home/drdave/Desktop/meta/prompt_hub` @ `f826ea33`.
- Built from CONFIRMED/QUALIFIED + feasibility-passed rows only (`findings/verdicts.md`, 2026-06-27).
- Docs-only artifact. No production code touched. Author: plan-architect (R4 + R7).
- Diagram conventions: envctl `docs/runbook/DIAGRAMS.md` box-drawing + automation legend
  `[A]` automated · `[A*]` elevated · `[P]` preview/dry-run · `[H]` human-gated · `[!!]` supervised/critical.

---

## Verdict

**prompt_hub is a genuinely capable intent STORE whose single defining job — emitting a
provenance-stamped goal artifact to rusty-idd — does not yet exist in code. Build that seam next,
but bind its wire format to rusty-idd's ACTUAL consumer schema before it lands as canonical.**

What is real and confirmed (`verdicts.md` VERDICTs 1–6):

- Real governance-of-intent: RBAC (`auth.rs` Capability::{Read,Write,Admin,SwarmOnly}, Admin-superset),
  a tamper-evident SHA-256 audit chain (`audit.rs:46-74` `SHA256(before||after||timestamp)`,
  `verify_entry_integrity`), and a lineage subsystem — all on the mutate path
  (sanitize → authorize → store → audit). CONFIRMED (`rules-policy-org` CLAIM-ORG2/ORG3).
- Real three-tier search: FTS5 (`search.rs`, BM25 + tag-boost + recency), native libsql 384-d vector
  embeddings (`migrations/0001_initial.sql:64-66` `embedding F32_BLOB(384)`), optional Qdrant
  (`qdrant.rs`, feature-gated). CONFIRMED (VERDICT 6).
- Real multi-vendor inference control plane: `multi_provider.rs` Vendor::{OpenAi,Anthropic,Google,Custom}
  with health/failover, `load_balancer.rs`, `circuit_breaker.rs`, `local_llm` (Ollama/llamafile/whisper.cpp HTTP client).
- Clean architecture: 3-crate workspace, strict 2-layer DAG (server→core 177 edges, cli→core 13;
  the server→cli 14 is a name-resolution artifact), no real multi-node cycles (effectively a DAG),
  `#![forbid(unsafe_code)]` on the core. CONFIRMED (graph §1, §4, §5).

The load-bearing gap (VERDICT 1, CONFIRMED + RED probe):

- `grep -rniE 'goal_artifact|provenance|rusty-idd' prompt-hub/src prompthub/src prompthub-server/src`
  → **zero hits**. The convergence contract lives only in `docs/plans/lifeos-meta-front-door.md` + `prompts/`.
- `hub.rs:981-999` `get()` is a "Simplified" retrieval (`results.items.into_iter().next().map(|sp| sp.prompt)`)
  — it selects a stored prompt; it does not synthesize, version, or provenance-stamp a goal.
- `models.rs:388-408` `Prompt` and `models.rs:558-566` `Intent` carry **no** `schema_version`/`provenance`;
  `Intent` is transient (no id/timestamp/author/persistence).
- The authored RED suite (`prompt-hub/tests/goal_artifact_contract.rs`, 7 tests) is GREEN's target:
  `cargo test -p prompt-hub --test goal_artifact_contract` → `0 passed; 7 failed` — all 7 panic on the
  contract assertion, none on a compile error (RED for capability-absence). CONFIRMED (VERDICT 3).

Two correction findings that the plan acts on (and that are being fed back to envctl/prompt_hub#182):

- **ADR collision** (VERDICT 2): the front-door plan's "(ADR-0007)" for "prompt_hub = intent store/boundary"
  mis-resolves — `docs/adr/0007-plugin-system.md` is the unrelated Plugin System ADR. The authoritative
  boundary ADR is not in prompt_hub. A real prompt_hub ADR for the intent-store boundary + the
  goal-artifact envelope is required (draft emitted: `reports/adr-draft-prompt-hub-goal-artifact.md`).
- **Goal-artifact is prose-only** (cross-repo map): no live cross-repo edge into rusty-idd or
  harness_hub exists in code; the seam is a **new boundary to design**, not an edge to refactor.

Live evidence noted: a weave A2A round-trip with the envctl session shipped prompt_hub#182 (the
front-door plan); this cycle found the correction TO that plan (ADR-0007 mis-citation + goal-artifact
being prose-only) and feeds it back.

Headline confidence: **HIGH** on the as-built picture and the absence finding (direct source + RED
probe + adversarial gate); **MEDIUM** on the forward design of the envelope (its field set is unbound
until rusty-idd's consumer schema is read). See Confidence.

---

## ASCII architecture

### A. Member / layer fan-in (verified from Cargo path deps; graph §1)

```
   front-ends (thin)
        ┌─────────────────────────┐                 ┌─────────────────────────────┐
        │  prompthub (CLI) [A]     │                 │  prompthub-server (HTTP)[A]  │
        │  main.rs:30              │                 │  main.rs:46                  │
        │  clap ~41 verbs          │                 │  axum Router, 111 routes     │
        │  commands/ (21) + tui    │                 │  routes.rs(194KB)+state.rs   │
        └───────────┬─────────────┘                 └──────────────┬──────────────┘
                    │ 13 edges                          177 edges   │
                    └───────────────┬──────────────────────────────┘
                                    ▼   (strict/clean; server has NO dep on cli)
                 ┌───────────────────────────────────────────────────┐
                 │  prompt-hub (core lib) — lib.rs: 70 pub mod        │
                 │  ┌───────────────────────────────────────────────┐│
                 │  │  PromptHub facade — hub.rs, 183 pub methods,   ││ ◄ God-object
                 │  │  4748 LOC (top centrality; max blast radius)   ││   (UPGRADE PROV-facade)
                 │  └───────────────────────────────────────────────┘│
                 │  Store · Search · Vibe/Intent · Governance(RBAC/   │
                 │  audit/lineage) · Providers/Cost · Ops/Sync · Plugins│
                 └───────────────────────────────┬───────────────────┘
                                                 ▼
                        ┌──────────────────────────────────────┐
                        │ external: libsql(local SQLite) · axum/│
                        │ tower · handlebars/tera · argon2 ·    │
                        │ tiktoken/tokenizers · qdrant(opt)     │
                        └──────────────────────────────────────┘
```
Source: graph/prompt-hub.graph.md §1, §5.

### B. The store internals on the mutate path (governance-of-intent)

```
  Identity ─► PromptSanitizer::sanitize ─► RbacAuthManager::authorize_action ─► Storage(libsql)
   [H]            (sanitize.rs:109)            (auth.rs:86-110, [A])              │
                                                                                 ▼
                                                            audit.log_audit  ◄── SHA-256 chain
                                                            (audit.rs:46-74)     (tamper-evident)
                                                                                 │
                                            sync (broadcast) ◄──────────────────┘
   READ path:  get()/get_by_id() ─► Action::Read gate ─► search engine (FTS5│vector│Qdrant)
```
Source: findings/rules-policy-org-prompt-hub.md CLAIM-ORG2/ORG3; graph §3 hotspots.

### C. The MISSING goal-artifact emission seam (the gap to build)

```
  UserInput (models.rs:599)                  ╔═══════════════ GAP TO BUILD ═══════════════╗
        │   POST /api/v1/input/process       ║  goal-artifact emission → rusty-idd        ║
        ▼                                     ║                                            ║
  PromptHub::process_input (hub.rs:1401)      ║  step 0 [H][!!] READ rusty-idd consumer    ║
        │                                     ║         schema  (rusty-idd/.handoff/loop/  ║
        ▼                                     ║         plan/) — DO NOT GUESS fields       ║
  Intent (models.rs:558)                      ║  step 1 [A] GoalArtifact{schema_version,   ║
   (transient: raw_text,domain,role,          ║         provenance{audit_hash,author,      ║
    task_type,complexity,urgency;             ║         sources,produced_by,produced_at},  ║
    NO id/ts/author/provenance)               ║         target:"rusty-idd",goal,           ║
        │                                     ║         origin_prompt_id}  (feature-gated) ║
        ▼                                     ║  step 2 [A] emit-goal CLI + POST           ║
  hub.rs:981-999 get() = "Simplified"  ──X──► ║         /api/v1/goal/emit (read-only store)║
   retrieval (top-ranked prompt;              ║  ⇒ RED suite goal_artifact_contract.rs     ║
   NOT provenance-stamped, NOT bound)         ║    (7 tests) flips RED → GREEN             ║
                                              ╚════════════════════════════════════════════╝
  X = the absent seam: 0 src hits for goal_artifact/provenance/rusty-idd; no handshake exists.
```
Source: graph/prompt-hub.graph.md §2; graph/prompt-hub.cross-repo.md; findings/architecture-prompt-hub.md GAPs 1–5.

### D. Fleet frame — where the seam lands (head of the intent pipeline)

```
  harness_hub          prompt_hub              rusty-idd            handoff          harness-hub
 (interpreter) ──intent──► (STORE, THIS) ──goal-artifact──► (lifecycle) ──► (witness) ──► (engine)
                              ▲  emits the              ▲ consumer schema
                              │  provenance-stamped     │ is the binding contract
                              │  envelope (to build)    │ (read it FIRST — step 0)
  weave = A2A transport plane around all of the above (not embedded in prompt_hub product code).
```
Source: loop_state.md Frame; findings/rules-policy-org-prompt-hub.md §3; cross-repo.md Verdict.

---

## Sequenced upgrade

Ordered by value/risk using graph centrality + blast-radius: highest-urgency-and-contained first;
high-blast central changes sequenced behind the work that bounds them. Axis tag per row.
Canonical roadmap rows: `reports/ROADMAP-prompt-hub.md`.

| # | Upgrade | Axis | Centrality / blast (graph) | Why this slot | Cond |
|---|---------|------|----------------------------|---------------|------|
| 1 | **SEC-1 PR-diff shell-injection fix** across 4 AI workflows (`external-ai-apis.yml:70`, `ai-safety-deployment.yml:61`, `ai-code-review.yml:58`, `multi-model-evaluation.yml:41-42`) — pass PR content via `env:` + `jq --arg`, never interpolate `${{ }}` into a `run:` body | quality/security | off the call graph (CI), but highest external attack surface; remediation already proven in `audit_sync.yml` | Highest urgency; lowest blast (AI-review path only, CI gates untouched); known-good fix exists | — |
| 2 | **HYG-1 untrack control-plane junk**: `git rm --cached prompt-hub/prompthub.db prompt-hub/test.db` (192KB each), `validation_log.txt` (0B), retire tracked DEPRECATED `_workspace/` | governance | none | Trivial, reversible, removes leaked local state from history going forward | — |
| 3 | **GA-1 typed versioned `GoalArtifact` envelope** (new `prompt-hub/src/goal_artifact.rs`, feature-gated) — **step 0**: read rusty-idd's goal-file consumer schema → **step 1**: serialize to it | accuracy | new module, **zero current callers** (graph); additive | THE strategic build target; the store's defining job. Sequenced after the cheap wins so design time isn't blocked | **CROSS-REPO** |
| 4 | **PROV-facade decompose `PromptHub`** via a `provenance` sub-facade (audit+lineage+goal-artifact) preserving public `get`/`list`/`audit` signatures | quality | **top centrality** (183 pub fns, 4748 LOC); HIGH blast | Sequenced HERE — *before* the emit surface — so it bounds GA-2's blast into the God-object, not after | preserve sigs |
| 5 | **GA-2 emit surface**: `prompthub emit-goal --role --intent --out` CLI + `POST /api/v1/goal/emit` (read-only on store) → RED suite → GREEN | accuracy | touches 194KB `routes.rs` God-file + router chain; MEDIUM blast (bounded by #4) | The actual hand-off; depends on GA-1 + bounded by PROV-facade | **CROSS-REPO** |
| 6 | **LIN-1 persist lineage**: `lineage` table + rebuild in `PromptHub::new`; replace `created_at:"now"` sentinel with real `DateTime<Utc>` | accuracy (correctness) | medium — new migration + `hub.rs` construction | Composes into GA-1's provenance; fixes the dead-code lineage + fake-timestamp defect | — |
| 7 | **EXPORT-1 audit-hash in `export`** per-record `provenance` block (reuse `audit.rs`) | accuracy | low — single leaf command (`export.rs`) | Cheap step toward provenance-stamped output; not a substitute for GA-1 | — |
| 8 | **DBPATH-1 resolve store path** (`--db` / `$PROMPTHUB_DB` / `HubConfig`, XDG data dir default) across ~15 CLI commands; only `init` honors `--path` today | governance/config | leaf call-sites, no further dependents; low blast | Removes the CONFIRMED store-fork hazard (init elsewhere then add → different DBs) | **no-downgrade default** |
| 9 | **MEM-1 persist learning + cold-start lineage**: write `UserCorrection` to a `corrections` table loaded on `Hub::new`; guarantee lineage rebuild-from-`versions` | accuracy | additive table + migration; low blast | Two recall surfaces don't survive cold start today (`learn.rs`, `lineage.rs`) | — |
| 10 | **POOL-1 storage "pool" honesty**: rename to `SharedConnection` + drop misleading `max_connections`/semaphore-as-pool framing (or real multi-conn pool) | speed | `Storage::acquire` fan-in 41; medium blast | QUALIFIED: file-DB writes serialize at the engine; a real pool buys READ concurrency ONLY — must not advertise write parallelism. Rename is the safe default | reads-only gain |
| 11 | **TEST-1 un-orphan root `tests/`** (migrate 5 files into `prompt-hub/tests/` or add a root test-owner pkg) — recovers ~40KB dead coverage | quality | none (test wiring) | Root `Cargo.toml` is virtual (`[workspace]` only) → builds 0 targets from root `tests/` | — |
| 12 | **TOOL-1 rustls floor + deny regression guard**: bump declared `rustls` floor toward 0.23.41; add a `cargo deny` guard so the libsql/prometheus feature-trims can't silently re-import the advised chain | quality | dependency manifest; low blast | Effective version is 0.23.40 (~1 patch behind), NOT 15; advisory hygiene + lock the trims | — |
| 13 | **AR-1 build the git-kb call graph + web-source ledger**: scope the index to first-party paths (exclude `vendor/`), `entrypoints --refresh` (edges currently 0); add `.handoff/loop/web-source-ledger.md` with a rolling 90-day recency gate | quality/accuracy | tooling; low blast | Graph intelligence (hotspots/blast/dead-code) is unreliable while edges=0; web claims have no dated ledger | — |
| 14 | **ADR-FIX re-point + write the real ADR**: stop citing local ADR-0007 for the boundary; record a prompt_hub ADR for the intent-store boundary + the goal-artifact envelope (draft attached) | governance/prompt-arch | docs | Resolves the number collision before any "implements ADR-0007" claim is made | — |

Items GA-1, GA-2, and the RED suite (TEST/H) carry the **non-negotiable cross-repo binding
condition**: the envelope field set MUST be derived from rusty-idd's real consumer schema
(`rusty-idd/.handoff/loop/plan/`), not invented. The authored RED assertions
(`schema_version`, `provenance.sources`, `produced_by="prompt_hub"`, `target="rusty-idd"`,
`artifact_kind="goal_artifact"`) are valid as RED-now, but the GREEN they pin must be reconciled with
that schema before it becomes the canonical gate — else GREEN hard-codes a guessed wire format. An
unbound envelope must NOT land as canonical (verdicts.md UPGRADE A/B/H conditions).

---

## Tool-evaluation

What the graph shows prompt_hub imports/links, cross-referenced with the researcher's 90-day currency
+ advisories (`research/prompt-hub.trends.md`, accessed 2026-06-27; window since ~2026-03-29).
Recommendation per tool with the cited date. Note: the verifier QUALIFIED the rustls headline — the
"15 patches behind" figure conflates the declared caret floor (`Cargo.toml:62` `0.23.26`) with the
locked/effective version (`Cargo.lock` resolves **0.23.40**, ~1 patch behind 0.23.41).

| Tool/crate (linked) | Pinned / effective | Latest (date) | Recommend | Reason (cited) |
|---|---|---|---|---|
| rustls | floor 0.23.26 / **locked 0.23.40** | 0.23.41 (2026-06-22) | **upgrade floor** | TLS patch-currency; effective gap ~1 patch (not 15). Bump declared floor for advisory hygiene (trends §tool-currency; verdicts.md VERDICT 7). |
| libsql | 0.9 → 0.9.30 (2026-03-19) | 0.9.30 stable; 0.10.0-pre.4 | **hold + guard** | Current in 0.9. `default-features=false, features=["core"]` trims the bundled hyper-rustls→rustls-0.22→rustls-webpki-0.102 chain that RUSTSEC-2026-0049/0098 hit. PRESERVE + add a deny regression guard. Do not chase 0.10 pre-release. |
| prometheus | optional, `default-features=false` (otel feature) | — | **hold + guard** | Dropping the protobuf exposition + avoiding discontinued `opentelemetry-prometheus` keeps RUSTSEC-2024-0437 (protobuf recursion) out of the graph. Guard against re-enable. |
| axum | 0.8.8 (2025-12-20) | 0.8.9 (2026-04-14) | upgrade (low-risk) | 1 patch behind; trivial bump (trends S1). |
| tower-http | 0.7.0 (2026-06-15) | 0.7.0 | hold + verify | On newest major; re-verify `cors/compression-full/trace/request-id` feature names after the 0.6→0.7 jump (trends S3). |
| tower | 0.5.2 | 0.5.3 (2026-01-12) | upgrade when convenient | 1 patch; release predates window (trends S2). |
| tower_governor | 0.7 | 0.8.0 (2025-08-14) | evaluate before bump | Rate-limit middleware; review key-extractor/governor API changes (trends S4). |
| uuid | 1.18.0 | 1.23.4 (2026-06-24) | upgrade | ~5 minors of fixes/perf; no advisory (trends S8). |
| handlebars | 6.4.0 | 6.4.2 (2026-06-24) | upgrade (low-risk) | 2 patches; template engine actually wired (trends S7). |
| argon2 / password-hash | pinned via Dependabot rationale | — | hold (curated) | Dependabot pins `password-hash`, ignores standalone semver bumps tied to argon2 0.5.x re-export (autoresearch C-WEB-2). |

Currency posture is good: **no direct RUSTSEC advisory hits the pinned set**; the risk is dominated by
transitive chains the manifest already trims, and those trims provably dodge real 2026 advisories.
Action: the only material currency bump is **rustls floor → track 0.23.41**, plus the low-risk
axum/uuid/handlebars catch-ups; the highest-leverage move is the **deny/regression guard** so the trims
can't silently regress (trends recommended actions 1, 5; verdicts.md VERDICT 7 QUALIFIED).
Coverage caveat (verifier): axum/tower/clap/argon2/tiktoken/qdrant currency was NOT independently
re-checked — `tooling-currency` is `[~]` partial.

---

## Governance

Source: findings/governance-config-prompt-hub.md (CONFIRMED items in verdicts.md VERDICT 4).

- **Shell-injection (highest urgency)** — 4 AI workflows interpolate untrusted PR diff directly into
  `run:` shell/JSON bodies (gov-002). The team already fixed this class in `audit_sync.yml` (env var,
  never interpolated) and dropped `tj-actions/changed-files` for GHSA-mrrh-fwg8-r2c3 — the AI
  workflows were not brought up to that standard. → roadmap #1 (SEC-1).
- **Tracked DB binaries** — `prompt-hub/prompthub.db` + `prompt-hub/test.db` (192KB each) are tracked
  despite `.gitignore *.db`; root `prompthub.db` is correctly ignored (gov-001, dispute reconciled in
  VERDICT 4). → roadmap #2 (HYG-1).
- **Permission drift (high)** — `settings.local.json` allows `Bash(git push *)`/`gh pr merge *`/
  `git commit *` with no deny block and no `.claude/settings.json` PreToolUse guard; `git push *`
  permits `--force` despite `rules.toml [commands.blocked]` listing it (gov-005). → PROPOSE a
  `.claude/settings.json` with denies mirroring `rules.toml` (owner-walled; additive only).
- **Guardrail/vendor protection gaps** — `[merge.protected_files]` does not cover `vendor/**` (the real
  build inputs under `.cargo/config.toml` source-replacement) nor `.claude/agents/**` + 3 of 5 skills
  incl. the self-upgrading evolution-steward (gov-003/004). → PROPOSE widen protection (strengthening only).
- **Inert label gate** — `external-ai-apis.yml:16-21` places `if:` under `on:` (GitHub ignores it);
  only the job-level `vars.ENABLE_AI_WORKFLOWS` gate is real (gov-006).
- **No-unsafe enforcement gap** — `check_safety.sh` scans only `prompt-hub/src/`; the "no unsafe
  anywhere" rule is unenforced for `prompthub` + `prompthub-server` (gov-008).
- **Vendor policy unsettled** — 31,070 tracked files; the referenced revert PR #179 is not in local
  history; `cargo audit`/`deny` inspect lock metadata, not vendored file contents (gov-007). → needs an ADR.
- Toolchain split is NOT drift: MSRV 1.91.1 vs dev pin 1.96.0 is documented and CI exercises both (gov-011).

All hooks/policy/permission/vendor changes are **PROPOSE / owner-walled** — additive (more denies,
wider protection, more crates checked); none weaken an existing guard.

## Filesystem layout

Source: findings/filesystem-layout-prompt-hub.md.

- **PASS**: idiomatic 3-crate Cargo workspace; `target/` ignored; `.handoff/ledger.db` committed per
  ADR-0004 §3; XDG config *read* path (`config.rs:77-96`).
- **FAIL FL-1** — runtime DB is written to CWD (`./prompthub.db`) via hardcoded `Path::new("prompthub.db")`
  in 7 commands + `storage.rs:30` default; correct target is `dirs::data_dir()/prompthub/`. Asymmetric
  with the XDG-aware config reader. PROPOSE/OWNER-WALL (new $HOME write). → folds into roadmap #8 (DBPATH-1).
- **FAIL FL-2** — `_workspace/` tracked but self-declared DEPRECATED (migrated to `.handoff/` 2026-06-13);
  6 files remain. → roadmap #2.
- **FLAG** — `validation_log.txt` (0B) tracked (FL-3, → #2); `vendor/` 705MiB/31070 files with no
  freshness gate (FL-4, → ADR); `.idea/`/`.junie/` IDE state tracked (FL-5); 16 loose root `.md` files
  (FL-6); 5-way name sprawl `prompt-hub`/`prompthub`/`prompthub-server`/`prompthub.db`/`prompt_hub` (FL-7,
  document via ADR); lone `scripts/update_<task-tracker>_from_audit.py` (Python) in a Rust repo (FL-8).

## Memory/vector

Source: findings/memory-vector-intelligence-prompt-hub.md (vector tier CONFIRMED in VERDICT 6).

- Durable memory: libsql `prompts`/`versions`/`metrics`/`embeddings`/`audit` tables (WAL, soft-delete,
  versioned migrations). Three-tier search confirmed (FTS5 + native 384-d vector + optional Qdrant);
  **no RAG pipeline** (`gather.rs` is filesystem context collection, not embedding-grounded retrieval).
- **Default embedder is non-semantic** — `config.rs:11-14` default `EmbedderBackend::Hash` (deterministic
  hash, ideal for tests/dev). A Hash default must not masquerade as semantic vector search.
  → U2: ship a semantic preset or clearly label Hash as lexical-only.
- **Two recall surfaces don't survive cold start**: `LearningEngine` is rebuilt throwaway per call
  (`hub.rs:1897-1907`, learned corrections lost immediately); `LineageTracker` is in-memory
  (`lineage.rs:13`). → roadmap #9 (MEM-1) + #6 (LIN-1).
- **No fleet-memory (ICM) / handoff-ledger binding in store code** — goal/intent provenance is not
  recall-informed (U4, feature-gate for portability). `.kb` embeddings disabled (`.kb/config.toml`).
- Gap to gate: no standalone `reindex`/`embed-backfill` after a backend switch (index runs only on
  create, `hub.rs:934`) → U3.

## Auto-research

Source: findings/autoresearch-prompt-hub.md.

- Real code auto-research loop exists (`.handoff/loop/research-ledger.md` D1–D6 + verdicts); real
  web/advisory cadence (Dependabot daily, `security.yml` daily `cargo audit`/`cargo deny`).
- **git-kb call graph has 0 call edges** (symbol-only index; `Call edges: 0`) and is polluted by
  `vendor/` (16,804 rust files indexed) → hotspots/blast/dead-code intelligence is unreliable; `dead`
  flags trait impls as false positives. → roadmap #13 (AR-1): scope to first-party + `entrypoints --refresh`.
- **No 90-day recency-gated web research and no web-source ledger** in any prompt_hub artifact
  (C-WEB-6); committed audit SARIF can rot silently (a 2026-06-04 instance already did). → AR-1 adds
  `web-source-ledger.md` + a stale-evidence invalidation rule.
- The 7 AI workflows are opt-in scaffolding (gate on `ENABLE_AI_WORKFLOWS`, ephemeral artifacts, no
  feedback loop), NOT a closed loop; `security_remediation.yml`'s agent step is an inert `echo` (C-WEB-4).
- `.kb` code-index cache is not gitignored — residency risk for a vendored repo (U-AR-7).

## Rules/policy

Source: findings/rules-policy-org-prompt-hub.md.

- Owner law confirmed: **Upgrade Only / No Downgrades** is binding (`CLAUDE.md:146`,
  `lifeos-meta-front-door.md:24-25,162-166`); commit/PR discipline is mechanically enforced (pre-commit
  worktree-only + lint/test gate, `CONTRIBUTING.md:61-62`); fail-closed evidence rule.
- Org chart: prompt_hub is the **Front-Door intent STORE** half (harness_hub = interpreter), feeding
  rusty-idd (owner decision D3). Two org charts coexist: the build crew
  (feature-architect→rust-implementer⇆verification-gate→docs-scribe, with backlog/continuity/evolution
  stewards) and the in-repo product swarm (Alpha…Theta).
- **A2A/weave** is the fleet transport plane *around* prompt_hub, not embedded in product code; loops
  USE it (resolved via `WEAVE_BIN`/PATH) with a degrade-visibly (never-silent) ledger-only fallback.
- Gaps: fleet role lives only in plan/skill prose, not a normative prompt_hub policy file (U1); the
  in-repo swarm-handoff helpers (`swarm.rs:179-228`) produce templates but aren't wired to a live A2A
  transport (U2); no prompt_hub-local background-agent status surface (U3); Upgrade-Only is prose, not
  a machine-checkable CI gate (U4). All additive/reversible.

## Distributed compute

Source: findings/distributed-compute-prompt-hub.md.

- Served tiers: workstation/local-server (axum, port 8080) + CLI, over a local-only libsql store.
  Offline/degraded modes are a design strength (`offline.rs` replay, `mobile.rs` bandwidth-aware).
- Coordination/control plane for inference, **not** an on-device inference runtime — model weights live
  in external servers (Ollama/llamafile/whisper.cpp over HTTP); optional ONNX (`ort`, `smart-ort`) for
  local embeddings.
- Multi-vendor mesh confirmed (`multi_provider.rs` + `load_balancer.rs` + `circuit_breaker.rs`).
- N/A tiers (explicit, 0 grep hits): AI glasses/wearables (no target); ESP32 (std+tokio-full+libsql+axum
  can't run no_std — MCU is a dumb HTTP client only); Lua/Luau (no scripting surface — met natively by
  hooks + Handlebars/Tera; recommend an explicit "no embedded interpreter" decision); Pi Zero (ARMv6/512MB
  vs heavy ort/tokenizers — a Pi 4/5 CLI/thin-client is the realistic ARM target).
- Upgrades (all additive/feature-gated): prove+pin a no-C `aarch64-*-musl` lib/CLI cross-build with a
  deny-check that no C-TLS reaches the runtime graph (UPGRADE-1); a thin UniFFI/cdylib edge fetch/cache
  client over `mobile`+`offline` (UPGRADE-2); a differential multi-model eval over the existing vendor
  mesh (UPGRADE-3); document the ESP32 ingest boundary (UPGRADE-4); record the Lua out-of-scope decision (UPGRADE-5).

---

## Test Strategy

Source: findings/test-strategy-prompt-hub.md (CONFIRMED in verdicts.md VERDICT 3).

### Current coverage (by call-graph reachability, not file presence)

- The real cargo-built integration suite is `prompt-hub/tests/` (15 files: test_hub, test_models,
  test_search, test_security, test_get_rbac, test_accessibility, test_auto_purge, test_chaos,
  test_chaos_auto, test_malware_scan, test_offline, test_qdrant, test_touch, test_voice,
  test_voice_anonymize). Hub init/config surface (`PromptHub::new`, `is_initialized`, `config`,
  `db_path`) IS reached (`test_hub.rs:5-83`). `lib.rs:145-162` is a compile-as-assertion smoke test.
- **Orphaned root `tests/`** — root `Cargo.toml` is virtual (`grep -c '[package]'` = 0), so
  `cargo test` builds **0** targets from root `tests/`; the 5 files
  (test_end_to_end/test_hub/test_models/test_search/test_security, ~40KB) are dead intended coverage.

### Coverage gaps (ranked; each cites the untested symbol/contract)

1. **The ADR-0007 convergence contract has ZERO coverage AND zero implementation** — no
   `emit_goal_artifact`/`provenance`/`schema_version`/handoff envelope (`prompt-hub/src/*.rs` grep
   empty). Highest-value untested capability.
2. Public planning models carry no provenance/schema fields, so even the closest emission (serialized
   `Prompt`/`Intent`) cannot satisfy the contract (`models.rs:387-408,557-566`).
3. The `hub.rs` hotspot's register→search→emit integration path is untested for any envelope/handoff
   behavior — only init/config is reached (`hub.rs:913,981`).
4. The orphaned root `tests/` (~40KB e2e lifecycle/search/sanitizer/lock/concurrency/pagination)
   contributes 0 to the gate.

### Designed suite (authored, built, RUN — RED)

- File: `prompt-hub/tests/goal_artifact_contract.rs` (7 tests), commit
  `6fa3462b1cbdc4032e090f88fabf1b27703c1d28` on `plan/prompt-hub-red-tests`.
- `cargo test -p prompt-hub --test goal_artifact_contract` → `0 passed; 7 failed; 0 ignored` in 0.02s;
  all 7 panic on the contract assertion (e.g. `:72` schema_version, `:98` sources, `:145` envelope),
  NONE on a compile error — RED for capability-absence. Compiles clean under both default and
  `--all-features` clippy `-D warnings`. No new deps (serde_json/uuid/chrono/semver/tempfile, all existing).

| Contract (ADR-0007) | Acceptance criterion | Test | line | status |
|---|---|---|---|---|
| Stable schema rusty-idd can consume | top-level `schema_version` string | goal_artifact_declares_stable_schema_version | 66 | RED |
| Provenance of every claim | `provenance` object present | goal_artifact_carries_provenance_block | 82 | RED |
| Source citations | `provenance.sources` non-empty array | goal_artifact_provenance_lists_source_citations | 98 | RED |
| Producer/consumer binding | `produced_by="prompt_hub"` + `target="rusty-idd"` | goal_artifact_identifies_producer_and_targets_rusty_idd | 120 | RED |
| Envelope, not bare record | `artifact_kind="goal_artifact"` + `goal` + `origin_prompt_id` | goal_artifact_envelope_wraps_the_goal_payload | 145 | RED |
| Integration path register→emit | registered prompt emits schema+provenance | registered_prompt_emits_contract_compliant_goal_artifact | 169 | RED |
| Version-pinned (golden) | two versions emit identical `schema_version` | goal_artifact_schema_is_stable_across_versions | 225 | RED |

### FF test-build spec (Feature Forge intake — the generate+run handoff)

Feature Forge implements the production capability that flips all 7 RED cases GREEN — additive, no test
edits beyond removing the "best-available emission" shim once a real emitter exists.

- `PromptHub::emit_goal_artifact(&self, prompt_id, intent) -> Result<GoalArtifact>` (or
  `Prompt::to_goal_artifact`) whose serde form is a top-level envelope OBJECT, not the bare prompt record.
- Envelope carries `schema_version: String` (e.g. `"goal-artifact/1"`), stable & identical across prompt
  versions; `provenance: { produced_by:"prompt_hub", sources:[..non-empty..], produced_at,
  prompt_hub_version }`; `target:"rusty-idd"`, `artifact_kind:"goal_artifact"`, `goal` (intent payload),
  `origin_prompt_id`. The register→search→emit round-trip produces the same contract-compliant envelope.
- Differential/golden: snapshot one canonical emitted `GoalArtifact` JSON under
  `prompt-hub/tests/fixtures/` and assert byte-stable serialization (promote `…_stable_across_versions`
  to an insta/golden snapshot).
- Coverage target: the emission API reaches 100% test-caller coverage on its public surface
  (emit fn + GoalArtifact (de)serialization + provenance population); register→emit gains ≥1 e2e caller.
- CI gates touched: `Default-Features Test Compile` (`ci.yml:58-59`) + `Test`
  (`ci.yml:86` `cargo nextest run --workspace --all-features`). Both pass for the RED suite at compile
  time; they run+assert the cases once GREEN.
- **Binding precondition (carried from the gate):** before GREEN is canonical, reconcile the asserted
  field names/values with rusty-idd's real consumer schema (`rusty-idd/.handoff/loop/plan/`). Promoted
  as a Feature-Forge test-build backlog row: `reports/ROADMAP-prompt-hub.md` (test-build row).
- Secondary FF item (architect-routed): un-orphan root `tests/` so its e2e files re-enter the gate.

## Prompt-architecture

Source: findings/prompt-architecture-prompt-hub.md.

- prompt_hub is a real prompt-architecture store (typed prompt/lineage/RBAC/audit model + CLI/HTTP/
  plugin/library tool grants), but the interpreter↔store↔rusty-idd seam exists only as prose: no typed
  goal-artifact contract in code, and the plan's "(ADR-0007)" mis-resolves (local 0007 = Plugin System).
- **Tool grants**: CLI mutating set (add/import/export/deploy/evolve/rollback/plugin/vibe/gather/
  budget/cost); a very large axum route table (routes.rs ≈194KB) — a wide mutating network surface for
  a "store"; **plugin grant = dynamic native-code loading** via libloading + inventory (catch_unwind),
  arbitrary `.so` execution beside `#![forbid(unsafe_code)]` (the guarantee does NOT extend to loaded
  objects — a real trust boundary); outbound egress (local_llm + multi_provider; CI external-API egress
  with secrets).
- **Model lanes** split three ways with no governing policy and a drifting id: task prompts pin
  `openai/gpt-4o`; fleet loops pin `anthropic/claude-opus-4-8`; CI fans out 4 models; runtime uses
  free-text `anthropic`. Model-id drift (`claude/claude-opus` CI vs `anthropic/claude-opus-4-8` vs runtime);
  no canonical registry; the no-downgrade rule is asserted at plan level, not encoded.
- ADR candidates: (1) goal-artifact emission format (typed envelope schema + version) prompt_hub →
  rusty-idd; (2) two-layer intent front-door seam (harness_hub ↔ prompt_hub); (3) model-lane routing
  policy + model-id canonicalization + no-downgrade enforcement; (4) plugin native-code trust boundary
  amending ADR-0007. No-ADR: individual prompt.yml task prompts, new CLI subcommands, sequential
  migrations, advisory instruction files.

---

## Risk policy

See `risk-policy.md` (`## prompt-hub`) for the full machine-shaped policy. Summary of the supervised
trust boundaries this plan touches:

- **Shell-injection trust boundary** [!!] — untrusted PR diff → shell in 4 AI workflows. SUPERVISED
  remediation (SEC-1); the same class the repo already fixed.
- **Secrets / provider egress** — plaintext API-key CI surface (ANTHROPIC_API_KEY, DEVIN_API_KEY over
  curl); cross-vendor egress. Keys must stay in `env:` (never inline), model ids pinned/centralized.
- **Plugin native-code trust boundary** — dynamic `.so` loading is outside the `#![forbid(unsafe_code)]`
  guarantee; documented + supervised.
- **Cross-repo schema dependency** — the GoalArtifact envelope must bind to rusty-idd's consumer schema;
  an unbound envelope landing as canonical is the headline supervised risk.
- **Destructive ops** — `git push *` permits `--force` un-gated at the Claude layer; PROPOSE a deny.

## Confidence

**Overall: HIGH on the as-built picture + the load-bearing absence; MEDIUM on the forward design of
the GoalArtifact envelope.**

- HIGH: the store's real capabilities (RBAC/audit/lineage, 3-tier search, vendor mesh), the strict
  2-layer DAG, and the goal-artifact absence — each from direct source citation, the adversarial
  verifier gate (6 CONFIRMED / 1 QUALIFIED / 0 REFUTED), and a runnable 7-RED probe.
- MEDIUM: the envelope's field set, because it is unbound until rusty-idd's consumer schema is read
  (step 0). What raises it: reading `rusty-idd/.handoff/loop/plan/` and reconciling the RED assertions.
- MEDIUM (graph fidelity): degree counts are a LOWER BOUND (the Rust resolver under-resolves
  method/trait dispatch; full-index call edges resolved only after per-member isolation). Layering +
  module structure are high-confidence (corroborated by Cargo manifests). AR-1 (build the real call
  graph) raises this.

### Named gaps / what a deeper pass should target

- **rusty-idd consumer schema** — the single highest-value unknown; gates GA-1/GA-2/RED-GREEN. Read it next.
- `tooling-currency` is `[~]` — axum/tower/clap/argon2/tiktoken/qdrant currency not independently checked.
- `perf` is `[~]` — the shared-connection write-serialization is CONFIRMED by source but no contention
  benchmark was run; POOL-1's speed framing must stay reads-only until benchmarked.
- Untouched dimensions (out of this gate's scope): `public-api-contracts`, `hotspots-coupling`,
  `dead-code` (the 416 NoCallers candidates are unconfirmed — inflated by the empty-edge index).
- Notable REFUTED/infeasible to report as findings: the "rustls 15 patches behind" sub-detail is
  REFUTED (effective ~1); the "committed binary in repo ROOT" premise was corrected (root .db is
  ignored — the real defect is the two under `prompt-hub/`); the toolchain "pin 1.91.1 drift" premise
  is not drift (it's the documented MSRV).
