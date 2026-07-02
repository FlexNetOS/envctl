# Plan verdicts (verifier gate)

## prompt-hub

Date: 2026-06-27. Verifier: plan-verifier (cycle 6, TARGET=prompt-hub).
Method: adversarial — each claim assumed false, refuted against cited source; each upgrade assumed
infeasible, gated against prompt_hub invariants (additive / no-downgrade / `#![forbid(unsafe_code)]`
/ no-C trust boundary / goal-artifact envelope must bind to rusty-idd's ACTUAL consumer schema).
Empirical probes run in the RED worktree `/home/drdave/Desktop/meta/.worktrees/plan-prompt-hub-red/prompt_hub`
and the live target `/home/drdave/Desktop/meta/prompt_hub`.

### Material claims

VERDICT 1 — goal-artifact emission contract absent (LOAD-BEARING) -> CONFIRMED
- `grep -rniE 'goal_artifact|goal-artifact|provenance|rusty-idd|rusty_idd' prompt-hub/src prompthub/src prompthub-server/src` → **zero hits** across all 3 members' src/. The seam exists only in `docs/` + `prompts/` markdown.
- `hub.rs:981-999` `get()` is a "Simplified" relevance retrieval (`results.items.into_iter().next().map(|sp| sp.prompt)`) — a retrieval, not a provenance-stamped goal producer. No provenance, no rusty-idd handshake.
- `models.rs:388-408` `Prompt` keys = id,name,version,status,system_prompt,…author — **no** `schema_version`/`provenance`. `models.rs:558-566` `Intent` = raw_text,domain,role,task_type,complexity,urgency,extracted_entities — transient, no id/timestamp/author/persistence.
- Cross-confirmed empirically: the RED suite's panics dump the actual emitted keys (Prompt + Intent above), proving the contract object does not exist. The cycle's headline verdict — **real front door is `process_input`/`get`; the rusty-idd goal-artifact contract is ASPIRATIONAL / doc-only** — is CONFIRMED exactly as the analyst stated.

VERDICT 2 — ADR-0007 collision -> CONFIRMED
- `docs/adr/0007-plugin-system.md:1` = `# ADR-0007: Plugin System` (Status: Accepted). The plan's "(ADR-0007)" citation for prompt_hub=intent-store/goal-artifact-boundary mis-resolves; the local 0007 is the unrelated Plugin System ADR. The authoritative boundary ADR is NOT in prompt_hub — any "implements ADR-0007" claim for the convergence seam is a number collision and must be re-pointed (rusty-idd / META_ROOT) before use.

VERDICT 3 — test-strategist: orphaned root tests + 7 RED for contract-absence -> CONFIRMED
- Root `Cargo.toml` has `grep -c '[package]'` = **0** (virtual `[workspace]` only) → cargo builds 0 targets from root `tests/`. The 5 root files (`test_end_to_end, test_hub, test_models, test_search, test_security`) are ORPHANED (≈40KB dead intended coverage). Real tests live in `prompt-hub/tests/`.
- Probe: `cargo test -p prompt-hub --test goal_artifact_contract` (RED worktree) → **`test result: FAILED. 0 passed; 7 failed; 0 ignored`**, finished in 0.02s. All 7 panic on the contract assertion (e.g. line 72 schema_version, 159 envelope, 108 sources), **NONE** on a compile error — RED for capability-absence, not toolchain failure. Exactly 7 tests ran (no exit-0-zero-tests artifact). The strategist's count and RED-reason are CONFIRMED.

VERDICT 4 — governance/security: PR-diff shell injection + .db tracking -> CONFIRMED (with reconciliation)
- Script-injection class CONFIRMED across the 4 AI workflows — untrusted PR-controlled content interpolated via `${{ }}` directly into model payloads/`$GITHUB_OUTPUT`:
  - `external-ai-apis.yml:70` — `"content": "...review this Rust code change:\n\n${{ steps.diff.outputs.content }}"`
  - `ai-safety-deployment.yml:61` — `"...Analyze this Rust code...${{ steps.changes.outputs.diff }}"`
  - `ai-code-review.yml:58` — `"...Review these Rust code changes:\n\n${{ steps.changes.outputs.diff_output }}"`
  - `multi-model-evaluation.yml:41-42` — `echo "title=${{ github.event.pull_request.title }}"` / `body=...` into `$GITHUB_OUTPUT` (classic title/body injection sink).
- **.db-tracking dispute RESOLVED — both auditors right about different files.** `git ls-files | grep -E '\.db$'`:
  - `.handoff/ledger.db` (intentional — `.gitignore:19` `!.handoff/ledger.db`)
  - `prompt-hub/prompthub.db` — **TRACKED** (governance auditor correct)
  - `prompt-hub/test.db` — **TRACKED** (governance auditor correct)
  - `vendor/libsql/tests/{template,test}.db` (vendored fixtures)
  - Root `prompthub.db` is **GITIGNORED** (`.gitignore:8`) and NOT tracked (filesystem auditor correct).
  Net defect that stands: **two committed DB binaries under `prompt-hub/`** (`prompthub.db`, `test.db`) — a real control-plane hygiene gap; the root .db is clean.

VERDICT 5 — store caveats (pool + db-path) -> CONFIRMED
- libsql "pool" = single shared `Connection` + `Semaphore`, NOT a real pool: `storage.rs:44-57` `Storage { db, conn: Connection, config, semaphore }`, doc comment "Shared connection opened (and migrated) at construction. Reused for every pooled `acquire()`" — every acquire hands out a clone of one handle (writes serialize). `storage.rs:31-33` `max_connections` from `available_parallelism()*2+1` buys no parallel DB I/O. CONFIRMED.
- DB path hardcoded `"prompthub.db"`: `grep -rln 'prompthub.db' prompthub/src/commands/` → **15 files**; only `init.rs:10` honors `--path` (`path.unwrap_or_else(|| Path::new("prompthub.db"))`). So `init --path /elsewhere.db` then `add` operate on different (CWD-relative) databases. CONFIRMED. (Note: `StorageConfig.db_path` field exists at `storage.rs:18` but the CLI commands bypass it.)
- Not independently re-run but consistent with source: SHA-256 audit chain log-only (`audit.rs`), lineage in-memory/test-only with hardcoded `created_at: "now"` (`lineage.rs:86`, CONFIRMED by direct read). These corroborate the provenance-fragmentation gap.

VERDICT 6 — memory: real 3-tier search + non-semantic default embedder -> CONFIRMED
- FTS5: `search.rs` "FAST engine — SQLite FTS5", `prompts_fts` virtual table over name/system_prompt/tags (BM25 + tag-boost + recency). CONFIRMED.
- Native vector: `migrations/0001_initial.sql:64-66` `CREATE TABLE ... embeddings (... embedding F32_BLOB(384))` — libsql native 384-d vector column. CONFIRMED.
- Default embedder non-semantic: `config.rs:11-14` `enum EmbedderBackend { #[default] Hash, ... }` — "Deterministic hash-based embedding ... ideal for tests/dev". Default is Hash (non-semantic). CONFIRMED. (Optional Qdrant path consistent with the dimension; not re-exercised.)

VERDICT 7 — trend advisory (rustls staleness + libsql/prometheus trims dodge RUSTSEC) -> QUALIFIED
- **Sub-detail REFUTED:** "rustls 0.23.26→0.23.41 (15 patches behind)" overstates. `Cargo.toml:62` *declares the floor* `rustls = "0.23.26"`, but `Cargo.lock` *resolves to* **0.23.40** (single rustls entry; no 0.22 chain) — i.e. ~1 patch behind 0.23.41 at the effective/built version, not 15. The "15 patches behind" figure conflates the declared caret floor with the locked version.
- **Trim claim QUALIFIED-CONFIRMED (structure) / web-sourced (advisory IDs):** `Cargo.toml:30-31` documents trimming libsql's bundled hyper-rustls→rustls-0.22→rustls-webpki-0.102 chain; lock shows exactly **one** rustls (0.23.40), corroborating the trim. `prometheus` is optional behind the `otel` feature (`prompt-hub/Cargo.toml:21,111`). The specific RUSTSEC-2026-0049/0098 + RUSTSEC-2024-0437 IDs are web-sourced and not independently re-validated here → QUALIFIED per the dimension's recency caveat. Actionable currency item: bump the declared `rustls` floor to track 0.23.41 (advisory hygiene), but the effective exposure is far smaller than the headline number implies.

### Counts
CONFIRMED: 6 (claims 1,2,3,4,5,6) · QUALIFIED: 1 (claim 7) · REFUTED: 0 · INCONCLUSIVE: 0.
(≥1 CONFIRMED satisfied. Claim 7 carries one REFUTED sub-detail — the "15 patches behind" figure.)

### Upgrade feasibility gate
Invariant baseline VERIFIED present: `prompt-hub/src/storage.rs:1` `#![forbid(unsafe_code)]`; libsql/axum
are existing pure-Rust deps; no upgrade below introduces C into the trust boundary.

- UPGRADE A — versioned `GoalArtifact` envelope (new feature-gated module) -> feasible (QUALIFIED).
  Additive, zero current callers (graph), within invariants. **CONDITION (non-negotiable):** the envelope
  field set MUST be derived from rusty-idd's ACTUAL consumer schema (cross-repo dependency), NOT invented
  here. The analyst's proposed fields (`schema_version`,`provenance.{audit_hash,author,lineage_path}`,
  `selected_prompt`) are a plausible-but-unbound guess — feasibility holds only once the rusty-idd
  consumer contract is read (`rusty-idd/.handoff/loop/plan/`) and the shape reconciled. Gate: do not
  land the type as canonical until that binding exists.

- UPGRADE B — `emit-goal` CLI + `POST /api/v1/goal/emit` route -> feasible (QUALIFIED, depends on A).
  Additive command + route; read-only on the store. Same cross-repo-schema condition as A. Blast: touches
  the 193KB `routes.rs` God-file + router chain (medium) — accept with the decomposition in F to bound it.

- UPGRADE C — persist lineage to a `lineage` table + rebuild in `PromptHub::new` + replace `created_at:"now"` with real `DateTime<Utc>` -> feasible.
  Forward-additive migration, droppable table, fixes the CONFIRMED `lineage.rs:86` fake-timestamp defect.
  Serves correctness axis. No invariant conflict.

- UPGRADE D — resolve DB path (`--db`/`$PROMPTHUB_DB`/`HubConfig`, XDG default) across the 15 CLI commands -> feasible.
  Additive global flag, leaf call-sites (no further dependents). **No-downgrade condition:** default must
  preserve today's `prompthub.db` resolution so existing invocations don't silently re-target. Serves the
  governance/config axis; removes the CONFIRMED store-fork hazard. APPLY-tier is appropriate.

- UPGRADE E — include audit-hash head in `export` per-record `provenance` block -> feasible.
  Single leaf command (`export.rs`), additive field, reuses the existing `audit.rs` primitive. Low blast.
  Serves accuracy axis. Note this is a step toward — not a substitute for — the rusty-idd envelope (A).

- UPGRADE F — decompose `PromptHub` God-object via a `provenance` sub-facade -> feasible (care).
  Pure internal refactor; **no-downgrade condition:** existing public `get`/`list`/`audit` signatures must
  be preserved (the acceptance test already asserts this). High blast (central type) — sequence it to
  bound A/B, not after. Serves quality axis.

- UPGRADE G — storage "pool" honesty (rename `SharedConnection` OR real multi-conn pool) -> feasible (QUALIFIED on the speed axis).
  Rename/doc path is trivially feasible and additive. A real multi-connection pool is buildable in pure
  Rust, BUT the tagged **speed** win is only partial: a libsql/SQLite file DB serializes WRITES at the
  engine level regardless of pool size, so a real pool buys READ concurrency only — not the write
  parallelism the `max_connections` framing implies. Honest-rename variant is the safe default; the
  "real pool for speed" variant is QUALIFIED (reads-only gain) and must not advertise write parallelism.

- UPGRADE H (test-strategy) — RED suite `goal_artifact_contract.rs` + un-orphan root `tests/` -> feasible (QUALIFIED).
  RED suite builds + runs (7 RED, verified). Un-orphaning (migrate the 5 files into `prompt-hub/tests/`,
  or add a root test-owner package) is additive and recovers ~40KB coverage — feasible. **CAVEAT:** the
  authored RED assertions encode an ASSUMED envelope (`schema_version`, `provenance.sources`,
  `produced_by="prompt_hub"`, `target="rusty-idd"`, `artifact_kind="goal_artifact"`). They are valid as
  RED-now, but the GREEN they pin is the same unbound contract as A — the field names/values must be
  reconciled with rusty-idd's real consumer schema before they become the canonical gate, else GREEN
  hard-codes a guessed wire format.

Feasibility tally: feasible = 8/8 — but **5 carry a binding condition** (A,B,H gated on the rusty-idd
cross-repo schema; D on no-downgrade default; G QUALIFIED to reads-only on the speed axis). None are
infeasible; none violate the no-C / no-unsafe / additive invariants. The gate was not weakened: the
cross-repo-schema condition on A/B/H is a STRENGTHENING — those upgrades may not land a unilaterally
invented envelope as canonical.

### Dimension ledger reconciliation (fail-closed)
Flipped to [x] (a verdict substantially covers the dimension's core question):
- architecture — VERDICTs 1,5 (god-object/get-simplified/storage/provenance-fragmentation CONFIRMED).
- front-door-and-goal-artifact-seam — VERDICT 1 (the load-bearing question, CONFIRMED + RED probe).
- data-flow — VERDICTs 1,5 (Intent→get path, lineage dead-code, store-path, provenance attach-point).
- governance-security — VERDICT 4 (PR-diff injection + .db tracking CONFIRMED+reconciled).
- test-coverage — VERDICT 3 (was [~]; 7-RED probe + orphaned-root CONFIRMED).
Marked [~] (partially covered — fail-closed, not fully verified):
- perf — storage shared-conn/serialization CONFIRMED (VERDICT 5) but no contention benchmark run.
- tooling-currency — rustls + libsql/prometheus verified (VERDICT 7) but axum/tower/clap/argon2/
  tiktoken/qdrant currency NOT independently checked.
Left untouched [ ] (out of this gate's claim scope):
- public-api-contracts, hotspots-coupling, dead-code.
</content>
</invoke>
