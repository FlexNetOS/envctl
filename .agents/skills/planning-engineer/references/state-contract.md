# Planning-engineer durable-state contract (`.handoff/loop/plan/`)

The loop's truth lives on disk under `.handoff/loop/plan/` (namespaced to avoid the flat
`.handoff/loop/` forge-loop collision — mirrors the `.handoff/loop/rust-port/` precedent). A plan is
only as complete as this ledger. `targets.md`/`dimensions.md` are owned by `plan-cartographer`;
dimension status is gated by `plan-verifier`. Lay the tree down with `harness-loop-init`.

## Layout
```
.handoff/loop/plan/
  loop_state.md            # counters + planning_target/target_root/recency_window_days/graph_snapshot (see scripts/loop_state.template.md)
  targets.md               # planning-target backlog (auto-derived; owner-overridable)
  dimensions.md            # per-target dimension ledger (cartographer-owned, verifier-gated)
  graph/<T>.symbols.json   # git-kb symbols snapshot
  graph/<T>.callgraph.json # callers/callees edges + entrypoints + flows
  graph/<T>.metrics.json   # DERIVED graph intelligence (centrality/hotspots, blast-radius, dead, cycles, layering, public-api)
  graph/<T>.graph.md       # human ASCII view of the graph + metrics (diagram-ready)
  graph/<T>.diff.md        # delta vs the previous committed snapshot — how the graph "updates"
  research/<T>.trends.md   # 90-day web findings, every finding cited + dated
  findings/<dim>.md        # analyst: CLAIM + gap + UPGRADE rows
  findings/verdicts.md     # verifier verdicts (append per dimension)
  reports/codemap-<T>.md   # structural map
  reports/<T>-plan.md      # THE FINAL PLAN (diagrams + sequenced upgrades + tool-eval + gaps + confidence)
  evaluation.md            # per-cycle self-eval scorecard (superseded each cycle)
  proposed-upgrades.md     # structural harness upgrades awaiting owner (fail-closed)
  HANDOFF.md               # cold-start packet at budget
  SENTINELS: DONE · NEEDS-HUMAN · STOP · WRAP-UP-OWED
```
`<T>` = target slug (crate/subsystem). `<dim>` = dimension id.

## targets.md row format
```
- [ ] <T>: <one-line scope>            # not yet planned
```
Status: `- [ ]` pending · `- [~]` in-flight / planned-with-gaps · `- [x]` planned + verified ·
`- [!] blocked: <reason>` · `- [!!]` SUPERVISED/CRITICAL (never auto-run). Auto-derived by the
cartographer from Cargo workspace members + major modules; an explicit owner list overrides per run.

## dimensions.md row format (per target)
```
- [ ] <T>/<id> · <area> · <the specific question this dimension answers> · deps: <ids|none>
```
Status: `- [ ]` not analyzed · `- [~]` analyzed, unverified · `- [x]` verified · `- [!] blocked: <reason>`.
Dimension catalog (pick what the target needs): **architecture** (components/boundaries/layering),
**data-flow** (entrypoints + traced flows), **hotspots/coupling** (centrality, cycles), **dead-code**,
**public-API/contracts** (the surface a plan must not break), **performance** (speed upgrades),
**correctness/accuracy** (accuracy upgrades), **code-quality** (idiom, tests, lint), **tooling**
(CLIs/MCPs/crates + currency), **comparison-to-best-practice** (vs the 90-day research), and
**test-coverage** — **ALWAYS seeded** for every target (owned by `plan-test-strategist`): existing
tests by call-graph reachability + ranked coverage gaps + the designed suite, in
`findings/test-strategy-<T>.md` (which ends with a `## FF test-build spec` the architect promotes to
Feature Forge — the loop plans tests, FF builds + runs them).

## Claim format (analyst → `findings/<dim>.md`)
```
- CLAIM: <falsifiable statement> | evidence: <path:line / symbol / call-path / test> | confidence: high|medium|low
```

## Upgrade format (analyst → `findings/<dim>.md`) — the R5 deliverable
```
- UPGRADE: <the change> | axis: quality|speed|accuracy | rationale: <why> | evidence: <path:line> | blast: <impact-scope from the graph> | risk: low|med|high
```
Every upgrade names its axis (code-quality, speed, or accuracy), is grounded in graph blast-radius,
and is feasibility-gated by the verifier before it reaches the plan.

## Verdict format (verifier → `findings/verdicts.md`)
```
- <claim-or-upgrade-ref> -> CONFIRMED | REFUTED (<counter-evidence>) | QUALIFIED (<condition>) | INCONCLUSIVE (<why>)
```
Only `CONFIRMED`/`QUALIFIED` claims and feasibility-passed upgrades reach the plan. Notable `REFUTED`
overclaims (and infeasible upgrades, e.g. ones that would breach the no-C-in-trust-boundary invariant)
are reported as findings/gaps, never as recommendations.

## Discipline
- **Completeness sweep before DONE** — the cartographer re-derives the target's expected surface from
  the graph (modules / entry points / public-API) and diffs it against what was examined; any major
  unexamined area blocks DONE. A partial/zero re-derivation → INCONCLUSIVE → NEEDS-HUMAN. "Clean"
  requires a positive re-derivation that matches, not merely the absence of open `- [ ]`.
- **No unverified facts, no infeasible upgrades** — a claim/upgrade is a plan item only after surviving
  adversarial verification + (for upgrades) a feasibility gate.
- **Cite everything** — every claim, upgrade, and verdict points at real code, so any line is checkable.
- **Read-only on the target's code** — the only writes are this ledger, the graph store, and the
  architect's docs/ROADMAP + draft-ADR promotion. Never weaken a gate to force a pass.
