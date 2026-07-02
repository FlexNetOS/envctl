# ROADMAP rows — weave (cycle 4, DRAFT — owner-wall)

Promotable `docs/ROADMAP.md` rows for the weave plan. **DRAFT only — NOT written into weave's own tree**
(weave's `CLAUDE.md`/protected-files are owner-canon; these rows are proposed for the owner to land).
Canonical copy of the plan stays at `reports/weave-plan.md`. Every row traces to a CONFIRMED/QUALIFIED
verdict + a FEASIBLE upgrade.

## Upgrade roadmap rows (ordered by value/risk: centrality + blast-radius)

| # | item | axis | tier | blast | effort | traces-to | acceptance |
|---|---|---|---|---|---|---|---|
| R1 | A2A v1.0 interop adapter (additive, default-off `a2a`; `Store`/`Intent` seam; AgentCard via `sign`) | accuracy | PROPOSE-additive | 1238 (contained: additive) | L | ARCH-09, U-ARCH-2 (FEASIBLE) | RED suite `a2a_interop.rs` (3) GREEN; native Tier-2 + sign suites still GREEN; no new dep, no C |
| R2 | Dual-backend conformance harness over both `dyn Store` impls (~90 methods) | quality | APPLY | 462+488 (tests-only) | M | ARCH-11, U-ARCH-1 (FEASIBLE) | one shared suite runs on SqliteStore + LibsqlStore; locks the `send` `guard_writable` asymmetry |
| R3 | Single-source CLI↔MCP verb mirror (cross-guard test) | governance+settings+config | PROPOSE | 124+427 | M | ARCH-06, U-ARCH-4 (FEASIBLE) | test enumerates 71 CLI / 72 MCP arms / 76 catalog; RED on any orphan |
| R4 | Documented-gate 6→7 alignment + Python-out-of-CI (Rust xtask) | governance+settings+config | PROPOSE | doc + CI | S+M | GOV-003, GOV-004, U-GOV-001/002 (FEASIBLE) | 3 required-check lists agree (7 names); zero `*.py` under scripts; same `target-smoke.json` schema + `cargo deny` posture |
| R5 | Memory-organ separation ADR + ICM-blindness doc-contract (`memory.rs` = bounded send-time cache) | memory-vector-intelligence | PROPOSE | docs only | S | MEM-2, U-MEM-1 (FEASIBLE); U-MEM-2 (feasibility-QUALIFIED → option b) | ADR classifies `memory.rs`; durable recall points to ICM+handoff; provenance/opt-out + no-vector fence tests |
| R6 | `main.rs` 9631-line dispatch extraction → `dispatch/*` | quality | SUPERVISED | 427 (highest-blast bin) | L | ARCH-07, WV-FSL-1, U-ARCH-3 (FEASIBLE) | `main.rs` < cap (e.g. <2000); per-handler unit test; all existing tests pass; fenced behind R2+R3 |

## Auxiliary rows (low-priority / owner-walled, FEASIBLE)

| item | axis | tier | traces-to |
|---|---|---|---|
| `rusqlite 0.40.0 → 0.40.1` bump | tooling | APPLY | trends §D (1 patch behind, in-window) / DC-W6 |
| Repo-native git-kb freshness CI step | autoresearch | PROPOSE | autoresearch U1 |
| renovate/dependabot config (new-release recency) | autoresearch | PROPOSE | autoresearch U2 |
| Document + optionally arm PreToolUse gate | governance | SUPERVISED | GOV-001, U-GOV-009 |
| User-global residency exemption ADR (XDG/meta) | filesystem-layout | PROPOSE (owner-wall) | WV-FSL-3 |
| Move `~/.config/weave/memory` → `$XDG_DATA_HOME` | filesystem-layout | PROPOSE | WV-FSL-4 |
| `rust-toolchain.toml` pin | governance | PROPOSE | GOV-011, U-GOV-008 |
| Bring ecc instinct/identity sidecars under drift guard or delete | prompt-architecture | PROPOSE | prompt-arch UPGRADE (instruction-only) |
| Constrained-node minimal-client contract (Intent-over-HTTP) | distributed-compute | PROPOSE (doc/adapter) | DC-W1 |

## Feature-Forge test-build backlog row (the generate+run handoff)

Shaped to Feature Forge's `feature-architect ## Verification plan` intake. planning-engineer authored
+ RED-ran the additive tests; Feature Forge builds production code + GREEN-runs them.

| field | value |
|---|---|
| **test-build item** | A2A v1.0 interop adapter — take the committed RED suite GREEN, additively |
| **RED suite (committed, FAILING)** | `weave-core/tests/a2a_interop.rs` — 3 cases; `cargo test -p weave-core --test a2a_interop` → 0 passed / 3 failed / tests-ran 3; commit `b7f466f` on `plan/weave-red-tests` (unpushed) |
| **production target** | new `weave-core/src/a2a.rs` (or `to_a2a`/`from_a2a` on `model.rs`) + A2A surface on weave-mcp; default-off `a2a` feature; AgentCard signing via default-off `sign` |
| **invariants (never weaken)** | additive only — never mutate `Intent`'s serde; SQLite-mailbox transport stays the required local route; no new dep (serde_json + ed25519 only); no C in trust boundary |
| **to author alongside** | round-trip property test `from_a2a(to_a2a(i))==i`; `--features sign` AgentCard-shape test; golden A2A v1.0 `message/send` fixture |
| **CI gates touched** | `cargo test -p weave-core` (new `a2a_interop` binary); `cargo fmt --check` + `cargo clippy` (preflight subset); `sign`-feature lane if AgentCard case added |
| **coverage target** | `to_a2a`/`from_a2a` each reached by ≥1 test; A2A-1/2/3 GREEN; native Tier-2 (`integration.rs:3541/3646`) + sign (`security.rs:1388`) suites still GREEN |
| **DRAFT ADRs to land with it** | `ADR-DRAFT-weave-a2a-interop.md` (the adapter decision) |

> Promotion note: when the owner lands these, append the rows to `docs/ROADMAP.md`; the test-build row
> is the Feature-Forge "generate + run" handoff (it is NOT a code change to weave by the planner).
