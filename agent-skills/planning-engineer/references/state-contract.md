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
  graph/target-dag.json      # TDP target/dimension DAG: nodes, edges, ready_set, self_revision
  graph/target-dag.md        # human DAG + ready-set + SELF-REVISION rows
  research/<T>.trends.md   # 90-day web findings, every finding cited + dated
  research/sources-<T>.jsonl # machine-readable source ledger (url/title/publisher/dates/claim_ids)
  findings/<dim>.md        # analyst: CLAIM + gap + UPGRADE rows
  findings/memory-vector-intelligence-<T>.md # persistent memory + vector/code intelligence guarantees
  findings/autoresearch-<T>.md # constant code+web auto-research cadence and stale evidence invalidation
  findings/rules-policy-org-<T>.md # owner policy, agent org chart, A2A/weave communication
  findings/distributed-compute-<T>.md # Rust+Lua multi-vendor edge/cloud hardware fabric
  findings/prompt-architecture-<T>.md # prompt/tool/model/runtime coupling review + ADR/no-ADR rationale
  findings/verdicts.md     # verifier verdicts (append per dimension)
  reports/codemap-<T>.md   # structural map
  reports/<T>-plan.md      # THE FINAL PLAN (diagrams + sequenced upgrades + tool-eval + gaps + confidence)
  reports/agent-run-ledger-<T>.md # background-lane observability: spans, model, effort, artifacts, verdicts
  risk-policy.md             # HITL/SUPERVISED routing for high-risk upgrades
  agent-backend-matrix.md    # lane isolation/backend decision table
  agent-interop.md           # weave/MCP/ACP/A2A/GitHub cloud-agent registry and routing decision
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
**governance+settings+config** — **ALWAYS seeded** for every target (owned by `plan-governance-config-auditor`): control-plane/settings/config coherence, MCP rot, skill overload, token burn, permission/config drift.

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


## Runtime artifact gate

Before a target is marked DONE, run `bash scripts/plan-artifact-gate.sh .handoff/loop/plan` (or the
ejected copy under `.claude/skills/planning-engineer/scripts/`). The gate validates the actual runtime
artifacts, not just prompt prose:

- required graph/research/findings/report artifacts exist for every `- [x]` target;
- `target-dag.json/md` records TDP ready-set scheduling and SELF-REVISION capability;
- `sources-<T>.jsonl` is valid JSONL with URL/title/publisher/accessed/published/recency/claim ids;
- memory/vector, autoresearch, rules/policy/org, distributed-compute, prompt-architecture, filesystem-layout, governance/config, test strategy, verdicts, risk policy,
  backend matrix, interop registry, and agent-run ledger are present;
- `DONE` is rejected unless all target/dimension rows are terminal and the completeness sweep is
  recorded with confirmed/qualified evidence.

## P0-P2 upgrade axes now required

- **P0 artifact validation** — `plan-artifact-gate.sh` is the runtime DONE/completeness validator.
- **P0 TDP scheduling** — `plan-dependency-graph` owns `graph/target-dag.{json,md}` and localized
  self-revision.
- **P0 prompt-architecture review** — `plan-prompt-architecture` owns
  `findings/prompt-architecture-<T>.md` and ADR/no-ADR routing for prompt/tool/model couplings.
- **P1 observability/backend/risk/evals** — `reports/agent-run-ledger-<T>.md`,
  `agent-backend-matrix.md`, `risk-policy.md`, and `scripts/tests/test-plan-evals.sh` make background
  work inspectable and gated.
- **P2 reproducible research/interoperability** — `sources-<T>.jsonl` and `agent-interop.md` preserve
  source provenance and future weave/MCP/ACP/A2A/GitHub-cloud routing decisions without weakening the
  current weave→Opus law.


## Owner critical architecture-loop artifacts

For each completed target, these artifacts are mandatory in addition to the P0-P2 gate artifacts:

- `findings/memory-vector-intelligence-<T>.md` — ICM/`.handoff`/source ledger/GitKB/vector/RAG
  inventory, freshness, recall/store guarantees, and cold-start proof.
- `findings/autoresearch-<T>.md` — constant code graph + web/vendor research cadence, contradiction
  checks, and stale-evidence invalidation.
- `findings/rules-policy-org-<T>.md` — Upgrade Only, No Downgrades, automation-first policy, real
  agent org chart, background-agent law, A2A/weave message map, and human-bottleneck replacement.
- `findings/distributed-compute-<T>.md` — Rust+Lua runtime strategy, workstation/local/mobile/AI
  glasses/Pi/Pi Zero/ESP32 hardware matrix, multi-vendor local+cloud mesh, failover, telemetry,
  secrets, and data-residency policy.
