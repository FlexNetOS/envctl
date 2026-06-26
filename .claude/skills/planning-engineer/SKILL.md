---
name: planning-engineer
description: >-
  Packaged planning/architecture harness (invoked as /harness:planning-engineer). Runs ONE deep
  planning cycle on a target subsystem/crate/feature-area: deep web research (best-practices + latest
  trends, last 3 months) + a repo scan with symbol & data-flow mapping + a persistent CODE GRAPH that
  yields graph-based intelligence → adversarially-verified gap analysis → a decision-grade plan with
  ASCII architecture diagrams, code-quality/speed/accuracy/governance+settings+config upgrades, TDD RED-suite evidence, and tool-evaluation. ALWAYS use
  for: "plan <subsystem>", "architecture plan", "deep planning", "design the architecture of <X>",
  "what should we upgrade in <X>", AND follow-ups — "re-run", "run it again", "update the plan",
  "revise", "redo only the <dimension>", "dig deeper on <X>", "based on the previous plan". For a
  CONTINUOUS loop over many targets use `plan-loop`; for cross-session handoff use session-relay-*.
  Read-only on the target's code; writes only plans/graph under .handoff/loop/plan/ + docs.
---

# planning-engineer — deep planning & architecture cycle  (`/harness:planning-engineer`)

Leader skill of the **planning-engineer** packaged harness. It turns one *planning target* (a crate /
subsystem / feature-area) into a **decision-grade plan**: research the field → map the code into a
graph → analyze gaps & design upgrades → **try to refute every claim and feasibility-gate every
upgrade against the source** → synthesize a plan with ASCII diagrams, a sequenced
quality/speed/accuracy/governance+settings+config upgrade roadmap, and a tool-evaluation. The defining property (inherited from
`code-research`): conclusions and recommendations are *earned by evidence that survived refutation* —
so the plan is safe to act on. **Read-only** on the target's production code; the only writes are the
plan/graph artifacts under `.handoff/loop/plan/`, additive RED test suites (the one permitted mutation), and (architect only) the docs/ROADMAP+ADR promotion.

For a CONTINUOUS run over a backlog of targets, the `plan-loop` skill wraps this one cycle in the
Ralph loop. This skill IS one cycle.

## Execution mode — Hybrid (background fan-out sub-agents + file-based), and why

Single-orchestrator with specialist sub-agents over a durable ledger — **not** a live `TeamCreate`
team: team state dies at the loop's self-restart boundary, so the truth must live in the on-disk
ledger under `.handoff/loop/plan/` (the loop resumes cold from files, never from conversation memory).
Per phase:

| Phase | Mode | Shape |
|-------|------|-------|
| 1 Map + Research | Sub-agent, **parallel** (`run_in_background:true`) | cartographer ‖ trend-researcher |
| 2 Analyze | Sub-agent, **parallel** | one analyst per dimension + governance/config auditor + test strategist → cited gaps + upgrades |
| 3 Verify (gate) | Sub-agent, **parallel** | one verifier per dimension → refute claims + feasibility-gate upgrades |
| 4 Synthesize | Sub-agent, **sequential** | architect → plan + ASCII diagrams + tool-eval |
| 5 Self-eval | Sub-agent, **sequential** | evolution-steward → evaluate this cycle, queue upgrades |

All `Agent` calls use `model: "opus"`. Data transfer is **file-based** (pass artifact PATHS, never
contents) + **return-value** (each agent returns a one-line verdict the orchestrator reduces).

## Agents (in the plugin's shared `harness/agents/` pool)

| Agent | Owns | Requirement | Shared? |
|-------|------|-------------|---------|
| `plan-cartographer` | map the target + build/diff the CODE GRAPH from `git-kb code` + derive graph intelligence (centrality/hotspots, blast-radius, dead, cycles, layering, public-API) + symbol & data-flow maps + seed dimensions + the pre-DONE completeness sweep | **R3b + R8** | specialist |
| `plan-trend-researcher` | deep WEB research — best-practices + latest trends over a rolling 90-day window, every finding cited + dated | **R3a** | specialist |
| `plan-analyst` | per-dimension analysis → cited claims, named gaps, and upgrade options each tagged **quality / speed / accuracy** | **R5** | specialist |
| `plan-governance-config-auditor` | control-plane + settings/config scan: rules/instructions/hooks/policy/CLAUDE.md/AGENTS.md, `.claude`/`.codex`, MCP rot, skill overload, token burn, permission/config drift | **prompt P2/P5/P6** | specialist |
| `plan-test-strategist` | the always-on **`test-coverage`** dimension: map existing tests, author additive RED tests for each accepted plan item, count-verify tests-ran > 0, emit traceability, and hand GREEN implementation to Feature Forge | **prompt P8** | specialist |
| `plan-verifier` | adversarially **refute** each claim against the source AND **feasibility-gate** each upgrade (the gate) | gate for R3/R5/R8 | specialist |
| `plan-architect` | synthesize → plan with **ASCII diagrams** + **tool-evaluation** + sequenced upgrade roadmap; promote to docs/ROADMAP + draft ADR | **R4 + R7** | specialist |
| `continuity-steward` | cold-start HANDOFF.md at budget | continuity | shared |
| `evolution-steward` | Phase 5 self-eval + fail-closed harness self-upgrade | **R6** | shared |

Skills used: `plan-cartography`, `plan-trend-research`, `plan-governance-config`,
`plan-test-strategy`, `plan-synthesis`, the reused `code-research-verify` (the verifier's refute discipline), `session-relay-wrap-up`,
`session-relay-resume`, `harness-evolution`, `icm-memory`. (The 90-day field research applies the
deep-research *method* — fan-out search → deep-read → adversarial verify → cited synthesis —
implemented inline by `plan-trend-research`; there is no separate `deep-research` skill to load.)

## Phase 0: Context check + inputs

Decide mode (initial / **resume** via `session-relay-resume` / **partial-redo-one-dimension** /
**new** — archive prior artifacts to `.handoff/loop/plan/_done/` on a new unrelated target). The
orchestrator needs the **planning target** `T` and its **target_root** (abs path of the subsystem) —
if not supplied, the `plan-cartographer` auto-derives `targets.md` from the repo's Cargo workspace
members + major modules and the loop picks the next `- [ ]`. Record `T`, `target_root`,
`recency_window_days: 90` in `loop_state.md`. (Read-only target: no build required to start, but the
verifier may run it — note the toolchain.)

## Phase 1: MAP + RESEARCH (fan-out, parallel) — `run_in_background:true`

Spawn **both** concurrently:
- `plan-cartographer` → `.handoff/loop/plan/graph/<T>.{symbols,callgraph,metrics}.json` +
  `<T>.graph.md` + `<T>.diff.md` (delta vs the previous committed snapshot — this is the graph
  *update*) + `reports/codemap-<T>.md`; seeds `dimensions.md` with the dimensions this target needs
  (architecture, data-flow, hotspots/coupling, dead-code, public-API/contracts, perf, correctness/
  accuracy, tooling, governance+settings+config, test-coverage, …). Built **only** from `git-kb code` JSON (no C dep, no graph DB).
- `plan-trend-researcher` → `.handoff/loop/plan/research/<T>.trends.md` — best-practices + latest
  trends in a **rolling 90-day window** (compute from today's date; prefer in-window sources, flag
  older), every finding cited + dated.

Await both, commit `dimensions.md` + the graph + research.

## Phase 2: ANALYZE (fan-out, parallel)

For each `- [ ]` code dimension, spawn `plan-analyst` (parallel) → `.handoff/loop/plan/findings/<dim>.md`. Also spawn `plan-governance-config-auditor` for the governance+settings+config axis and `plan-test-strategist` for the always-on test-coverage/P8 axis:
falsifiable **CLAIM** rows (each citing `file:line` / symbol / call-path / test) + named **gaps** +
**UPGRADE** rows each tagged `axis: quality|speed|accuracy|governance+settings+config` with rationale, evidence, blast-radius
(from the graph) and risk. Analysts query the graph — blast-radius to scope each upgrade's risk,
centrality to prioritize. Mark dimensions `- [~]` (analyzed, unverified).

## Phase 3: VERIFY (adversarial, parallel) — the gate

For each analyzed dimension, spawn `plan-verifier` → it tries to **refute** each material claim
against the code (read the cited source; run it where ambiguous) **and feasibility-gates each
upgrade** (is it actually buildable here, within the invariants? does it really serve its axis?).
Verdicts → `.handoff/loop/plan/findings/verdicts.md`. Only `CONFIRMED`/`QUALIFIED` claims and
feasibility-passed upgrades reach the plan; `REFUTED`/`INCONCLUSIVE` do not (notable refuted
overclaims are still reported as findings). Mark verified dimensions `- [x]`. **Never let an
unverified claim or an infeasible upgrade into the plan** — that is the whole point.

## Phase 3.5: TDD RED-suite authoring (prompt P8, permitted mutation)

`plan-test-strategist` turns every accepted gap/upgrade acceptance criterion into additive tests before implementation: unit/integration/e2e/golden/property where appropriate plus a differential-drive live case when a CLI/binary behavior is involved. It writes tests only in additive test locations (`tests/`, `#[cfg(test)]`, `scripts/differential-drive.cases.sh`, or an equivalent clearly test-only path), runs them, records the RED failure, and count-verifies `tests-ran > 0`. A test that passes before implementation is invalid and must be rewritten. Emit `findings/test-strategy-<T>.md` with the plan-item ↔ acceptance criterion ↔ test(s) ↔ RED|GREEN traceability matrix and the Feature-Forge GREEN handoff. No production code, gate relaxation, or destructive change is permitted.

## Phase 4: SYNTHESIZE

`plan-architect` → `.handoff/loop/plan/reports/<T>-plan.md`: **verdict first** (the headline
recommendation), then **ASCII architecture diagrams** (envctl `DIAGRAMS.md` conventions — box-drawing,
`Source: file:section`, the `[A]/[A*]/[P]/[H]/[!!]` legend), the **sequenced upgrade roadmap** (each
item tagged quality/speed/accuracy, ordered by value/risk using graph centrality + blast-radius), a
**governance/settings/config findings**, dedicated **tool-evaluation** section (tools/CLIs/MCPs/crates the target uses, their 90-day currency /
advisories from the researcher, recommend upgrade/hold), named **gaps**, and a stated **confidence**.
Then promote: append a `docs/ROADMAP.md` row (canonical copy stays under `reports/`); emit a **draft**
ADR at `.handoff/decisions/ADR-####-<slug>.md` **only** for a genuine architecture decision. Docs
only — never touch production code. Commit.

## Output contract

End every cycle with paths to: plan file, ASCII diagrams including the control-plane diagram, graph snapshot + diff, gap→upgrade table across quality/speed/accuracy/governance+settings+config, tool-eval table, governance findings, settings/config hygiene findings (MCP rot / skill overload / token burn / permission/config drift), TDD RED-suite evidence with tests-ran count and traceability matrix, evolution scorecard/LESSONS/proposed-upgrades, and the resume pointer.

## Phase 5: SELF-EVAL (every cycle) — `evolution-steward`, lightweight

Skill `harness-evolution`. Evaluate THIS cycle (friction, **gate quality** — did a wrong claim slip
past verify? did verify false-refute a sound upgrade?, coverage, human-walls) → append `LESSONS.md`;
**QUEUE** upgrades (low-risk in-scope → a `- [?]` item; structural → `proposed-upgrades.md`). **Do
NOT apply mid-cycle** — applying low-risk queued upgrades happens only at the batch boundary
(`wrap_every`) / HAND OFF via feature-branch → PR → auto-merge with a CLAUDE.md change-history row;
structural changes stay PROPOSED for the owner. **Never weaken the verify/completeness/DONE gate —
only strengthen it.** This per-cycle evaluate + boundary-apply cadence is how "self-eval + self-upgrade
on every run" reconciles with the fail-closed, never-mid-cycle rule.

## DONE gate (evidence-backed, fail-closed)

Write `.handoff/loop/plan/DONE` only when: every target in `targets.md` is `- [x]` (planned) or an
explicit `- [!]`, AND every dimension of the target is `- [x]` (verified) or `- [!]`; AND the
`plan-cartographer`'s **completeness sweep** re-derives the target's expected surface from the graph
(modules / entry points / public-API) and finds nothing major unexamined; AND the plan answers from
CONFIRMED evidence with a stated confidence + named gaps. A partial/zero re-derivation →
**INCONCLUSIVE → write `.handoff/loop/plan/NEEDS-HUMAN`**, not DONE. Record the sweep result inside DONE.

## Continuity & error handling

- **HAND OFF** at budget via `session-relay-wrap-up` (Phase 5 retro → ICM store → continuity-steward
  HANDOFF + commit → heartbeat → stop). **RESUME** via `session-relay-resume` (ICM recall → weave
  inbox → committed HANDOFF authoritative → verify-on-resume → continue at the next target/dimension).
- **Retry once; never fabricate evidence.** An analyst/researcher error → mark that dimension `- [!]`
  and continue others. A claim that can't be verified stays unconfirmed (out of the plan). A behavioral
  claim that can't be run in this env → INCONCLUSIVE, surfaced under gaps — never asserted. The
  verifier wins any conflict (it checks real source). **Never weaken a gate to force a pass.**
- Agents recall/store durable memory via `icm-memory` as needed (graceful no-op if ICM absent).

## Eject

`bash scripts/eject.sh <target-repo>` copies the harness (5 plan skills + shared skills + 5 plan
agents + shared agents) into the target's `.claude/` and scaffolds `.handoff/loop/plan/`. See
`references/state-contract.md`. Invoke as `/planning-engineer` (loop via `/plan-loop`).

## Test Scenarios

**Happy path:** Target = `crates/secrets-proto`. Phase 1: cartographer builds the graph (symbols +
call edges + flows), derives metrics (a hotspot `MintReq`, zero dead code, public-API = the tonic
service), seeds dimensions {architecture, data-flow, public-API/contracts, tooling}; trend-researcher
reports current tonic/prost best-practices dated within 90 days. Phase 2: analysts produce cited
claims + upgrades (e.g. `UPGRADE: derive prost validation | axis: accuracy | …`). Phase 3: verifier
CONFIRMS the wired claims, QUALIFIES one upgrade ("only if feature X"), REFUTES an overclaim. Phase 4:
architect emits an ASCII service diagram + a quality/speed/accuracy/governance+settings+config roadmap + a tool-eval (prost
version currency) + a ROADMAP row, confidence stated. Phase 5: evolution-steward logs one lesson, no
structural change. Production code untouched (`git status` shows only `.handoff/` + `docs/ROADMAP.md`).

**Error path (unverifiable / infeasible):** an analyst proposes "switch the store to an in-process C
cache for speed" → the verifier feasibility-gates it against the NON-NEGOTIABLE no-C-in-trust-boundary
invariant → REFUTED (infeasible), excluded from the roadmap and listed under gaps with the reason,
never recommended. A behavioral perf claim that can't be benchmarked in this env → INCONCLUSIVE →
listed under gaps ("needs a perf harness"), not stated as fact.

## References
- `references/state-contract.md` — the `.handoff/loop/plan/` layout + dimension/claim/verdict/upgrade schema.
- `references/eject.md` — install into a target repo.
- `scripts/loop_state.template.md` · `scripts/eject.sh` · `scripts/ralph-plan.sh` (SAFE, read-only).
