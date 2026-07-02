# DRAFT ROADMAP rows — handoff (cycle 2, union with rusty-idd)

DRAFT `docs/ROADMAP.md` rows. The canonical plan is `reports/handoff-plan.md`. Promotion INTO
`handoff/docs/ROADMAP.md` is a **PROPOSED owner action** — handoff is read-only this run (owner-wall),
so these are authored here, not written into handoff's tree. Every row traces to a CONFIRMED/QUALIFIED
+ feasible verdict in `findings/verdicts.md`. Order = the sequenced roadmap (value/risk by graph
centrality + blast-radius). The 5 union-specific steps are in `reports/union-plan-handoff-rusty-idd.md`.

## Upgrade rows

| seq | id | upgrade | axis | tier | blast | target-surface | acceptance gate | trace |
|---|---|---|---|---|---|---|---|---|
| 1 | A-U1 | Resolve RuVector `../../` path deps (vendor/publish/git-pin) — standalone blocker | governance | **SUPERVISED** | KERNEL (Ledger.open 120) | `hf/Cargo.toml:48,52,59`, `ledger/Cargo.toml:16-20,31-35`, CI | `cargo build --workspace` green w/ no sibling RuVector (E1, RED today) | EXP-1, A-U1, gov-003, DC-1, mem-U4, fs U1 |
| 2 | A-U4 | Dedup `crates/{cli,core,runner,spec,tui}` → rusty-idd superset; re-apply HFTASK-0082 | quality | **SUPERVISED** | 5 crates (~0 KERNEL) | handoff `crates/*` vs rusty-idd | one `rusty-idd-*` pkg per name (E3); spec/tui golden parity | A-C9, A-U4, fs V3 |
| 3 | A-U3 | rusty-idd deps handoff `work-order`+`validate_card` (kill mirror) | accuracy | PROPOSE | rusty-idd wo consumers | rusty-idd `crates/work-order` → handoff | cards pass `validate_card`; schemas byte-identical | A-C13, A-U3 |
| 4 | ts-U1 | Fail-closed work-order LOADER tests (AUTHORED, RED) → FF GREEN | accuracy | APPLY | wo loaders | `work-order/tests/union_failclosed.rs` (`d74ad4b`) | 3 RED flip GREEN via `from_card_json` | ts-2/ts-RED |
| 5 | gov-U1 | Bridge `hooks.toml` block gates → Claude PreToolUse → `hf hook run` | governance | PROPOSE | every Claude edit | `.claude/settings.json` | out-of-scope edit DENIED (live) | gov-001 |
| 6 | A-U2 | Ledger feature graph: `default=[redb-store]`, `v2` opt-in | quality | PROPOSE | ledger + v2 readers | `ledger/Cargo.toml:29-37` | default tree excludes rvf-runtime/index/types (gated on A-U1) | A-U2, mem-U2 |
| 7 | mem-U3 | Correct witness provenance → SHAKE-256 (optionally wire ed25519) | accuracy | APPLY/PROPOSE | doc / signing | `rvf-crypto/src/witness.rs:4`, trends §A3 | claim==impl; tampered-sig fails verify (if signed) | EXP-3, mem-3 |
| 8 | UP-1 | Fold rusty-idd CLI UNDER handoff policy gates | rules-policy | PROPOSE | rusty-idd CLI (LOW) | rusty-idd cmds → `hf policy check-edit`/gatekeeper | out-of-scope rusty-idd write REFUSED (exit 1) | rp-teeth, UP-1 |
| 9 | A-U5 | One-`Ledger`-per-feature compile test + ADR note | quality | APPLY (gated) | ledger (test) | `ledger/tests/`, `lib.rs:36-40` | single resolvable `ledger::Ledger` per feature set (post A-U1) | EXP-2, A-U5 |
| 10 | mem-U1 | Wire `query_by_intent`→`hf recall` w/ REAL embeddings OR delete v2-default | memory-vector | **SUPERVISED** | ledger overlay | `ledger/src/v2.rs:42-56,344-346` | `hf recall` semantic hits OR ADR delegation; embedder C-free | mem-1/2/6 |
| 11 | gov-U2/UP-5 | Self-enforce agent-guard via handoff PreToolUse (repo-portable) | governance | PROPOSE | destructive cmds | `.claude/settings.json` | reset/force-push DENIED in a fresh clone | gov-002 |
| 12 | gov-U4 | Guard `rusty-idd next` SessionStart hook with `command -v` | governance | PROPOSE | SessionStart | `.claude/settings.json:23-32` | clone w/o rusty-idd exits 0 gracefully | gov-004 |
| 13 | gov-U5 | Add `rust-toolchain.toml` pinned to CI 1.96.0 | governance | PROPOSE | local/preflight builds | new `rust-toolchain.toml` | `rustup show` == 1.96.0 | gov-005 |
| 14 | gov-U6 | Commit `.mcp.json` registering `hf-mcp` | governance | PROPOSE | MCP discovery | new `.mcp.json` | MCP client lists `hf-mcp` | gov-006 |
| 15 | gov-U7 | Tighten `Bash(git -C * push:*)` to repo-scoped | governance | PROPOSE | push grant | `.claude/settings.json:36-39` | unrelated-repo push prompts | gov-007 |
| 16 | gov-U9 | Doc-sync destructive-cmd rule to all 8 guard patterns | governance | APPLY | doc | `.claude/rules/meta-destructive-commands.md` | rule enumerates 8 pattern ids | gov-009 |
| 17 | ts-U4 | Golden `task_schema_json()` parity across the mirror | quality | APPLY | schema contract | `work-order/src/lib.rs:257` | mirror reproduces byte-for-byte | ts-U4 |
| 18 | pa-U2 | Pin/version-stamp `hf`↔`hf-mcp` | prompt-arch | PROPOSE | 35 MCP tools | `hf/src/bin/hf-mcp.rs:228-262` | skew assertion fails on mismatch | pa-hf-mcp |
| 19 | pa-U3 | Trim 1541-skill `.agent/skills-catalog.md` surface | prompt-arch | APPLY | context budget | `.agent/skills-catalog.md` | catalog scoped/regenerated | pa-U3 |
| 20 | pa-U4 | Make opus model-lane explicit policy | prompt-arch | APPLY | loop invocations | `.claude/skills/handoff-loop/SKILL.md:37` | lane stated as policy | pa-single-opus-lane |
| 21 | ar-U1 | git-kb index-staleness gate (`hf doctor`/CI) | autoresearch | APPLY | index freshness | `hf doctor` | DEGRADED+exit 1 on partial index | ar C16/C17 |
| 22 | ar-U2 | Symmetric `cargo audit` per-PR (not only promotion) | autoresearch | APPLY | per-PR CI | `.github/workflows/ci.yml` | new advisory fails per-PR | ar C9 |
| 23 | ar-U4 | One fleet currency bot (Renovate vs Dependabot) | autoresearch | APPLY | currency PRs | `renovate.json` | one bot governs both repos | ar C8 |
| 24 | fs-U3 | Un-track `.idea/` user-IDE state | filesystem | APPLY | root | `.idea/` (13 files) | `git ls-files .idea` empty | fs V7 |
| 25 | fs-U4 | Route generated `.agent/skills-catalog.md` off committed root | filesystem | REGENERATE | root | `.agent/skills-catalog.md` | root no 313K blob; doctor regenerates | fs V8 |
| 26 | fs-U6 | Mark `schemas/*.schema.json` provenance (generated vs authored) | filesystem | REGENERATE | schemas | `schemas/{task,packet,session}.schema.json` | regen task.schema.json byte-identical | fs V12 |
| 27 | DC-4 | Enforce `allows_network`/`path_scope` cross-node egress | distributed | PROPOSE | gatekeeper/route | `work-order/src/lib.rs:80`, `handoff-gatekeeper/src/lib.rs:204-258` | egress-forbidden order refused | DC-4 |
| 28 | DC-2 | Leaf-node proxy contract (mobile/Pi-Zero/ESP32) | distributed | PROPOSE | MCP/work-order seam | `hf/src/bin/hf-mcp.rs`, `work-order` | off-host result witnessed w/ correct correlation_id | DC-2 |
| 29 | UP-3 | Add `evolution-steward` to handoff's org | rules-policy | PROPOSE | org chart | `.claude/agents/` | witnessed retro; cannot auto-edit protected file | UP-3 |
| 30 | mem-U5 | Decision/"why" memory (ICM or ledger-curated events) | memory | PROPOSE | recall path | 0 icm refs | curated decision event recorded+recalled | mem-U5 |
| 31 | UP-2 | Enforce declared network/dep policies (default-warn→block) | rules-policy | PROPOSE | check-edit | `.handoff/policies/rules.toml:10-11` | un-audited dep add exits 1 (RED) | rp-declared-unenforced |
| 32 | UP-4 | Witnessed dual-model background lane (No-Downgrades guard) | rules-policy | PROPOSE | loop invocations | `.claude/skills/handoff-loop/SKILL.md:37` | gates stay opus; silent gate-tier downgrade BLOCKED | rp-org-chart |
| 33 | pa-U1 | Reconcile dual front door to ONE canonical entry | prompt-arch | PROPOSE | SessionStart | `.claude/settings.json:14-33` | one Front Door; fork-drift resolved | pa-dual/fork-drift |
| 34 | mem-U6 | Unified fleet recall facade (provenance-tagged) | memory | PROPOSE | recall facade | 5-store split | one `recall` returns ≥2 organ-tagged hits | mem-U6 |
| 35 | fs-U5 | Home/remove root orphans (intent-driven-template/, spike/) | filesystem | PROPOSE | root | `intent-driven-template/`, `spike/` | root only owned/routed surfaces | fs V9/V10 |
| 36 | ar-U3 | Scheduled research cadence (trend note + graph snapshot) | autoresearch | PROPOSE | CI schedule | `.github/workflows/` | trend Date advances; out-of-window auto-flagged | ar C18 |
| 37 | A-U6 | Manifest-cross-checked graph-integrity gate (planning-only) | governance | APPLY | planning artifacts | `graph/` cartography step | SCC/dead-code flagged vs Cargo DAG | A-C15 |
| — | DC-3 | Native weave mesh binding (feature-gated) | distributed | **SUPERVISED** | first live net dep | `handoff-lease/src/lib.rs:148-181` | round-trips w/o spawning weave; offline byte-identical | DC-3 |
| — | DC-5 | Guardrail ADR: no embedded/Lua/in-kernel network stack | distributed (guardrail) | PROPOSE | ADR + CI grep | repo-wide | no no_std/mlua/HTTP-client crate enters Cargo.toml | DC-5 |
| — | ar-U5 | Delete last C dep (`legacy-sqlite`) after fleet migration | autoresearch | **SUPERVISED** | ledger feature | `ledger/Cargo.toml:23,37` | `cargo tree -i rusqlite` empty all feature sets | ar C11 |

ADR-candidates recorded (not all emitted): canonical Front Door & interpreter boundary; `hf-mcp` as
the union T11 control seam; deterministic-classifier-vs-LLM-interpreter split (pa §8). The genuine
architecture decision (the MERGE union + RuVector resolution) IS emitted as a DRAFT ADR —
`reports/ADR-DRAFT-handoff-rusty-idd-union.md`.

## Feature-Forge test-build row (the generate + run handoff)

Shaped to Feature Forge's `feature-architect` `## Verification plan` intake. The RED suite is AUTHORED
and RED-run by the planning-engineer (test-strategist); Feature Forge builds the production code that
flips it GREEN. Do NOT rewrite the tests. Full spec in `reports/handoff-plan.md` (`## Test Strategy` →
`### FF test-build spec`), carried from `findings/test-strategy-handoff.md`.

| field | value |
|---|---|
| backlog id | FF-handoff-001 |
| title | Fail-closed `handoff.task.v1` work-order LOADER (+ intake refusal + ledger read-API once unblocked) |
| kind | test-build (RED → GREEN); engine-first, additive-only, no-downgrade |
| RED suite (authored, do not rewrite) | `work-order/tests/union_failclosed.rs` (commit `d74ad4b`) |
| RED tests | `workorder_load_rejects_foreign_schema_card`, `workorder_load_rejects_malformed_id_card`, `workorder_load_rejects_card_with_drifted_intent_lock` |
| GREEN fence (must stay GREEN) | `fixture_is_a_clean_valid_card` |
| production change to flip GREEN | add `WorkOrder::from_card_json(&str) -> Result<WorkOrder, LoadError>` chaining serde + `handoff_schema::validate_card` + `intent_unchanged` (`work-order/src/lib.rs`) |
| blocked-until-A-U1 | `handoff-intake/tests/intake_failclosed.rs` (front-door refusal); `ledger/tests/read_api.rs` (read-API contract, design-only) |
| differential/golden | golden `work_order::task_schema_json()` == rusty-idd mirror byte-for-byte |
| CI gate(s) | `cargo test`/`cargo nextest` once RuVector resolves in CI (`hf/Cargo.toml:46`); Format/clippy unaffected (additive) |
| trace | ts-2/ts-RED (CONFIRMED empirical, `1 passed / 3 failed`); roadmap rows ts-U1/A-U3 |
