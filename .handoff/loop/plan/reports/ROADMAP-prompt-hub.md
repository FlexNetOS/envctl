# ROADMAP — prompt-hub (canonical copy)

Promoted from `reports/prompt-hub-plan.md` (cycle 6, 2026-06-27). Ordered by graph centrality +
blast-radius. Axis ∈ {quality, speed, accuracy, governance}. Only CONFIRMED/QUALIFIED + feasible rows.
This is the canonical copy; the fleet `docs/ROADMAP.md` row is a pointer to it. Never written into
prompt_hub's own tree.

| # | id | upgrade | axis | risk_tier | blast | cond | evidence |
|---|---|---------|------|-----------|-------|------|----------|
| 1 | SEC-1 | PR-diff shell-injection fix across 4 AI workflows (env: + jq --arg, no `${{ }}` in `run:`) | quality | SUPERVISED | low (CI) | — | verdicts VERDICT 4; gov-002 |
| 2 | HYG-1 | untrack `prompt-hub/prompthub.db`+`test.db`, `validation_log.txt`, retire DEPRECATED `_workspace/` | governance | APPLY | none | — | verdicts VERDICT 4; gov-001; FL-2/FL-3 |
| 3 | GA-1 | typed versioned `GoalArtifact` envelope (new feature-gated module) — step0 read rusty-idd schema, step1 serialize to it | accuracy | SUPERVISED | low (0 callers) | CROSS-REPO | verdicts UPGRADE A; architecture UPGRADE; cross-repo.md |
| 4 | PROV-facade | decompose `PromptHub` god-object via a `provenance` sub-facade (preserve get/list/audit sigs) | quality | PROPOSE | high (top centrality) | preserve sigs | verdicts UPGRADE F; architecture UPGRADE |
| 5 | GA-2 | `emit-goal` CLI + `POST /api/v1/goal/emit` (read-only on store) → RED suite GREEN | accuracy | SUPERVISED | med (routes.rs) | CROSS-REPO | verdicts UPGRADE B; architecture UPGRADE |
| 6 | LIN-1 | persist lineage table + rebuild in `PromptHub::new`; real `DateTime<Utc>` (drop `"now"` sentinel) | accuracy | PROPOSE | med (migration+hub) | — | verdicts UPGRADE C; architecture UPGRADE |
| 7 | EXPORT-1 | audit-hash head in `export` per-record provenance block | accuracy | APPLY | low (leaf) | — | verdicts UPGRADE E; architecture UPGRADE |
| 8 | DBPATH-1 | resolve store path (`--db`/`$PROMPTHUB_DB`/HubConfig/XDG) across ~15 CLI commands | governance | APPLY | low (leaf) | no-downgrade default | verdicts UPGRADE D; FL-1 |
| 9 | MEM-1 | persist learned feedback (`corrections` table) + cold-start lineage rebuild | accuracy | PROPOSE | low | — | memory-vector U1/U5 |
| 10 | POOL-1 | storage "pool" honesty (rename `SharedConnection`, drop max_connections framing) | speed | PROPOSE | med (acquire fan-in 41) | reads-only gain | verdicts UPGRADE G; architecture UPGRADE |
| 11 | TEST-1 | un-orphan root `tests/` (migrate to `prompt-hub/tests/` or add a test-owner pkg) | quality | APPLY | none | — | verdicts UPGRADE H; test-strategy |
| 12 | TOOL-1 | bump rustls floor toward 0.23.41 + `cargo deny` regression guard on the libsql/prometheus trims | quality | PROPOSE | low | — | verdicts VERDICT 7; trends |
| 13 | AR-1 | build git-kb call graph (first-party scope, `entrypoints --refresh`) + web-source ledger (90-day recency) | accuracy | PROPOSE | low | — | autoresearch U-AR-1/U-AR-3 |
| 14 | ADR-FIX | re-point ADR-0007 + record a real prompt_hub intent-store/boundary + goal-artifact ADR | governance | PROPOSE | docs | — | verdicts VERDICT 2; prompt-architecture §7 |

## Feature-Forge test-build row (the generate + run handoff)

Shaped to Feature Forge's `feature-architect` `## Verification plan` intake. RED authored; FF builds the
production code that flips it GREEN, then RED→GREEN.

| field | value |
|---|---|
| item | TB-GA — GoalArtifact emission contract (GREEN the authored RED suite) |
| red_suite | `prompt-hub/tests/goal_artifact_contract.rs` (7 tests), commit `6fa3462b1cbdc4032e090f88fabf1b27703c1d28`, branch `plan/prompt-hub-red-tests` |
| run_cmd | `cargo test -p prompt-hub --test goal_artifact_contract` (currently `0 passed; 7 failed`) |
| green_target | `PromptHub::emit_goal_artifact(prompt_id, intent) -> Result<GoalArtifact>` (or `Prompt::to_goal_artifact`); serde envelope OBJECT (not bare record) with `schema_version`, `provenance{produced_by,sources,produced_at,prompt_hub_version}`, `target:"rusty-idd"`, `artifact_kind:"goal_artifact"`, `goal`, `origin_prompt_id`; register→search→emit round-trip identical envelope |
| golden | snapshot one canonical emitted GoalArtifact JSON under `prompt-hub/tests/fixtures/`; byte-stable serialization (promote `…_stable_across_versions` to insta/golden) |
| coverage_target | 100% test-caller coverage on the emission public surface; register→emit gains ≥1 e2e caller |
| ci_gates | `Default-Features Test Compile` (ci.yml:58-59) + `Test` (ci.yml:86 nextest --workspace --all-features) |
| precondition | CROSS-REPO: reconcile the asserted field names/values with rusty-idd's real consumer schema (`rusty-idd/.handoff/loop/plan/`) BEFORE GREEN is canonical — do not hard-code a guessed wire format |
| secondary | un-orphan root `tests/` so its e2e files re-enter the gate |

## docs/ROADMAP.md pointer row (fleet)

```
| prompt-hub | cycle 6 (2026-06-27) | STORE→rusty-idd goal-artifact seam is prose-only; build GA-1/GA-2 (cross-repo gated) after SEC-1 shell-injection fix | reports/prompt-hub-plan.md | HIGH (as-built) / MED (envelope design) |
```
