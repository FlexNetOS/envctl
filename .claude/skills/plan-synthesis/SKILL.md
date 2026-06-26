---
name: plan-synthesis
description: >-
  Turn verified findings into a decision-grade PLAN — verdict-first, ASCII architecture diagrams, a
  sequenced quality/speed/accuracy/governance+settings+config upgrade roadmap, a tool-evaluation, named gaps, and a stated
  confidence — then promote it (ROADMAP row + draft ADR). ALWAYS use to "write the plan", "synthesize
  the findings", "draw the architecture", "what should we upgrade", "tool evaluation", "promote to the
  roadmap", "draft an ADR", AND follow-ups — "revise the plan", "redo the synthesis", "re-render the
  diagrams", "update the roadmap", "based on the previous plan". Uses ONLY CONFIRMED/QUALIFIED
  evidence; docs only, never touches production code. Used by `plan-architect`.
---

# plan-synthesis — synthesize the plan + diagrams + tool-eval, then promote (R4 + R7)

Integrate the verified findings into **the plan** for one target: a decision-grade document that leads
with the headline recommendation, shows the architecture in ASCII, sequences the upgrades by value and
risk, evaluates the tooling, names what's still uncertain, and states a confidence. Then **promote**
it into the durable record (a `docs/ROADMAP.md` row, and a draft ADR only for a genuine architecture
decision). Output: `.handoff/loop/plan/reports/<T>-plan.md`. Used by `plan-architect`.

**Two hard rules:**
- **Evidence only** — use **only `CONFIRMED`/`QUALIFIED`** claims and **feasibility-passed** upgrades
  from `findings/verdicts.md`. `REFUTED`/`INCONCLUSIVE` items and infeasible upgrades **do not enter
  the plan** — notable refuted overclaims and infeasible-but-tempting ideas are recorded under **gaps**
  with the reason. This evidence-earned property is the whole point; never re-admit a dropped claim.
- **Docs only** — synthesis writes plans/diagrams/ROADMAP/ADR. It **never edits production code**.
  `git status` after a cycle should show only `.handoff/` + `docs/ROADMAP.md` (+ a draft ADR).

> **Templates live in `references/diagram-and-adr.md`** — the full ASCII diagram legend, the ADR
> template, and the ROADMAP-row format. This file is the imperative method.

## The plan document shape (`reports/<T>-plan.md`)

Write the plan in this order — **verdict first**, evidence after:

1. **Verdict** — the headline recommendation in 1–3 sentences (what to do about this target now), with
   a stated **confidence: High/Medium/Low** and the basis.
2. **ASCII architecture diagrams** — rendered with box-drawing characters; **cite each diagram
   `Source: file:section`** (the graph snapshot, the codemap, or a `docs/secrets/*`/`docs/ARCHITECTURE.md`
   section). Annotate hotspots, cycles, and layering with the automation legend where a step is
   automated/gated (see below). Produce **all of these that apply to the run**:
   - a **current-architecture** diagram of the target (module / data-flow structure from the graph);
   - a **target-state** diagram for the proposed plan (P7) — what the architecture looks like *after*
     the roadmap's upgrades land;
   - a **control-plane** diagram (REQUIRED — from the governance + settings/config scans, not vibes)
     showing the governance flow and the runtime/build bind, with edges labelled by relationship
     (`governs` / `enforces` / `configures` / `gates`):
     `hooks (lifecycle) ▶ policy/guard (rules.toml + cognitum gate) ▶ claim ▶ scope ▶ handoff`,
     `settings (perms/env/model/effort/MCP) ▶ runtime`,
     `toolchain/Cargo ▶ build ▶ CI (.github/workflows)`,
     `.handoff/policy.toml ▶ loop cadence + merge`,
     plus where `rules` / `instructions` / `CLAUDE.md` / `AGENTS.md` bind in;
   - for a **fleet / multi-repo** run, a **fleet-level** diagram (repos, shared substrates, cross-repo
     edges from `cross-repo-reference`). A diagram that doesn't apply (e.g. fleet diagram on a
     single-crate run) is skipped with a one-line note, never silently omitted.
3. **Sequenced upgrade roadmap** — the ordered list of upgrades, each tagged
   `axis: quality|speed|accuracy|governance+settings+config|filesystem-layout|prompt-architecture|memory-vector-intelligence|autoresearch|rules-policy-org|distributed-compute`, ordered by value/risk (recipe below). Only CONFIRMED/QUALIFIED +
   feasible items.
4. **Tool-evaluation** — the tool/CLI/MCP/crate inventory with currency/advisories → upgrade/hold/pin
   (recipe below).
5. **Governance, settings & config** — control-plane findings (rules/instructions/hooks/policy/CLAUDE.md/AGENTS.md), settings hygiene (MCP rot / skill overload / token burn / permission drift), config drift, and APPLY/PROPOSE/REGENERATE routing.
6. **Filesystem layout** — file/folder organization map, path inventory, FHS/XDG/repo-native placement verdicts, boundary map (repo/meta/user/system), and enforcement-test handoff from `findings/filesystem-layout-<T>.md`.
7. **Memory/vector intelligence** — ICM, `.handoff`, source ledgers, GitKB/vector/RAG freshness, and cold-start recall proof from `findings/memory-vector-intelligence-<T>.md`.
8. **Auto-research cadence** — code graph/web/vendor refresh, stale-evidence invalidation, and recurrence cadence from `findings/autoresearch-<T>.md`.
9. **Rules/policy/agent org** — Upgrade Only/No Downgrades, automation-first routing, org chart, A2A/weave comms, and human-bottleneck replacement from `findings/rules-policy-org-<T>.md`.
10. **Distributed compute fabric** — Rust+Lua hardware matrix, local+cloud vendor mesh, control/data plane, mobile/AI glasses/Pi/Pi Zero/ESP32 support from `findings/distributed-compute-<T>.md`.
11. **Prompt-architecture** — prompt/tool/model/runtime couplings, ADR candidates/no-ADR rationale, and governance controls from `findings/prompt-architecture-<T>.md`.
12. **Risk policy / backend / interop** — HITL/SUPERVISED routing, lane isolation decisions, and weave/MCP/ACP/A2A/GitHub-cloud interop registry.
13. **Test Strategy & Coverage** — current coverage (by call-graph reachability), the ranked coverage
   gaps (untested public-API / hotspots / data-flows / error-paths, each citing the symbol), and the
   designed suite (cases, types, golden fixtures) that closes them and covers the roadmap's upgrades —
   from `findings/test-strategy-<T>.md`. Ends by promoting the **FF test-build spec** (recipe below).
14. **Gaps** — what is unverified, infeasible-here, or needs a harness (e.g. a perf claim that couldn't
   be benchmarked → "needs a perf harness"); plus notable REFUTED overclaims. Honesty over completeness.
15. **Confidence** — restate the overall confidence and what would raise it.

## ASCII diagram conventions (R4)

From envctl `docs/runbook/DIAGRAMS.md` (full legend + worked example in `references/diagram-and-adr.md`):
- **Box-drawing characters** only: `┌ ─ └ ┐ │ ┘ ┬ ┤ ├ ┴ ┼` and arrows `▶ ◀ ▲ ▼`. No ASCII-art `+--+`.
- **Cite every diagram** `Source: file:section` immediately above or below it (graph snapshot, codemap,
  or a docs section) — a diagram with no source is a guess.
- **Automation legend** where a flow has automated/gated steps:
  `[A]` automated · `[A*]` automated-elevated (sudo) · `[P]` preview/dry-run · `[H]` human-gated ·
  `[!!]` supervised/critical (the loop refuses to auto-run). Tag the relevant boxes/arrows.
- Render *behavior and connection*, not just names; lift the graph's structure from
  `graph/<T>.graph.md` and add the layering/hotspot/cycle annotations from `metrics.json`.

## Gap → upgrade rubric (the roadmap ordering)

For each surviving upgrade (CONFIRMED/QUALIFIED + feasibility-passed in `verdicts.md`):
1. **Tag the axis** — exactly one of `quality` (correctness/maintainability/safety), `speed` (latency/throughput/build-time), `accuracy` (result correctness/precision), `governance+settings+config` (control-plane/settings/config coherence), or `filesystem-layout` (standard OS/repo file-folder organization). Carry the
   axis from the analyst's UPGRADE row.
2. **Order by value/risk using the graph** — rank by **graph centrality** of the touched symbols
   (`metrics.hotspots`) and **blast-radius** (`metrics.blast_radius`): high-centrality + bounded-blast,
   low-risk upgrades come first; high-blast or high-risk ones are sequenced later with the dependency
   noted. A low-centrality cleanup is low priority even if easy.
3. **Admit only feasible + verified items** — anything `REFUTED`/`INCONCLUSIVE` or feasibility-gated
   out (e.g. an upgrade that would violate the no-C-in-trust-boundary invariant) is **excluded from the
   roadmap** and listed under gaps with the reason. Never recommend an infeasible upgrade.

Each roadmap entry uses the full UPGRADE row format (`plan-analyst` schema): `axis` ·
`target-surface` · `evidence` · `blast` · `effort` · `risk-tier` (APPLY/PROPOSE/REGENERATE) ·
`acceptance` (the falsifiable condition its P8 RED test encodes — **1:1 with the test**) ·
`reversibility` (NORTH-STAR triad Integrity · Reversibility · Capability-Gain) · the verdict ref that
cleared it. An upgrade whose `acceptance` has no matching RED test (P8) is flagged **unverifiable** —
a fail-closed finding against the plan itself, not a silent roadmap row.

## Tool-evaluation rubric (R7)

1. **Inventory** the tools/CLIs/MCPs/crates the target actually uses — from the graph
   (`metrics.public_api` + the codemap's external deps), not from a guess. For a fleet/meta run,
   always evaluate the **standing fleet toolset** by name: `rtk`, `hf`, `git kb` / code-intelligence,
   the `meta` CLI, the connected MCP servers (from the settings/config scan's `[mcp_servers.*]`
   inventory), the harness skills (the `.claude/skills/*` count → skill-overload), JS tooling
   (**bun**, never pnpm/node), and the **shimmy + ruvllm** migration (the official ollama replacement —
   evaluate readiness but **do NOT recommend removing ollama until swap-out is parity-proven**). Honor
   the latest-toolchain standing rule (clang/llvm-21 is load-bearing).
2. **Currency + advisories** — pull each tool's latest version, release date, breaking-change flag, and
   any CVE/advisory from the researcher's `research/<T>.trends.md` **Tool-currency & advisories**
   subsection (the R7 input).
3. **Recommend** `upgrade` / `hold` / `pin` per tool, with a one-line rationale grounded in (a) the
   target's real usage (graph) and (b) the currency/advisory evidence (research). A tool with a fresh
   CVE → `upgrade` (or `pin` to the patched version); a heavily-depended-on tool with a breaking major
   release → `hold` + note the migration cost from blast-radius. Cite the evidence for each call.

## Test Strategy & the Feature-Forge handoff (the testing component)

The `plan-governance-config-auditor` produces `findings/governance-config-<T>.md`; lift its verified detector table and APPLY/PROPOSE/REGENERATE routing into the plan and tool-eval.

The `plan-filesystem-layout-auditor` produces `findings/filesystem-layout-<T>.md`; lift its verified path inventory, placement verdicts, boundary map, and layout enforcement test handoff into the plan. A target with material file/folder drift cannot be marked fully planned without either an accepted filesystem-layout upgrade or an explicit qualified gap.

The `plan-test-strategist` produces `findings/test-strategy-<T>.md` — current coverage (by call-graph
reachability), the ranked coverage gaps, the designed suite, and a `## FF test-build spec`. Lift its
**verified** rows into the plan's *Test Strategy & Coverage* section, then **promote the suite to
Feature Forge** — the planning loop authors and RED-runs additive tests; Feature Forge writes the production code and turns them GREEN. The handoff:
- Append a **test-build ROADMAP row** (format in `references/diagram-and-adr.md`) shaped to Feature
  Forge's `feature-architect` `## Verification plan` intake: the test surface, the concrete cases
  (symbol/flow + assertion + type), the golden fixtures to capture, the coverage target, and the CI
  gate(s) the new tests touch.
- **Only verified, feasible items** enter the section/handoff — a "this is untested" claim the verifier
  REFUTED (it *is* tested), and any infeasible test design, are dropped, never handed to FF.
- This is the "creation + implementation" path: the plan specifies the suite precisely; **Feature Forge
  generates and runs it.** Do not write production code here; test code must remain additive and RED before implementation.

## Promotion (durable record)

After writing `reports/<T>-plan.md`:
- **Append a `docs/ROADMAP.md` row** — one row pointing at the canonical plan under `reports/` (row
  format in `references/diagram-and-adr.md`). The plan stays the canonical copy; ROADMAP is the index.
- **Append a Feature-Forge test-build row** — the test-suite handoff (format in
  `references/diagram-and-adr.md`), shaped to FF's `feature-architect` `## Verification plan` intake so
  Feature Forge builds + runs the designed suite. Skip only if the target genuinely needs no new tests
  (record that under *Test Strategy & Coverage*).
- **Draft an ADR — only for a genuine architecture decision.** If the plan settles a real
  architectural choice (a new boundary, a backend swap, a contract change), emit a **draft** ADR at
  `.handoff/decisions/ADR-####-<slug>.md` (template in `references/diagram-and-adr.md`; pick the next
  free `####`). A routine upgrade list is **not** an ADR — do not manufacture one. Status `draft`;
  the owner accepts it.
- **Docs only** — ROADMAP + ADR + the plan are the only writes. Commit them with the `.handoff/` state.

## Discipline

- **Verdict first, evidence after** — lead with the recommendation; a reader gets the decision in the
  first paragraph and the proof below.
- **Only what survived the gate** — CONFIRMED/QUALIFIED + feasible. Re-admitting a refuted/infeasible
  item defeats the harness; list it under gaps instead.
- **Cite every diagram and every recommendation** — `Source: file:section` on diagrams; an evidence
  pointer (verdict ref / `path:line` / research source) on every upgrade and tool call.
- **Name the gaps honestly** — an unbenchmarkable perf claim is a gap ("needs a perf harness"), never
  asserted as fact; a tempting-but-infeasible idea is a gap, never a recommendation.
- **No production-code edits** — synthesis is the plan, not the build. The implementer (a different
  harness) acts on the plan later.

## References
- `references/diagram-and-adr.md` — full ASCII diagram legend + worked example, the ADR template, and
  the `docs/ROADMAP.md` row format + the UPGRADE row format reused in the roadmap.


## P0-P2 synthesis inputs

Lift these runtime-gated artifacts into the plan before promotion:

- `findings/memory-vector-intelligence-<T>.md` — persistent memory/vector/code intelligence guarantees.
- `findings/autoresearch-<T>.md` — constant code+web research cadence and stale-evidence invalidation.
- `findings/rules-policy-org-<T>.md` — owner rules, policy, agent org chart, A2A/weave comms.
- `findings/distributed-compute-<T>.md` — Rust+Lua distributed compute across all hardware/vendors.
- `graph/target-dag.md` — TDP scheduling and SELF-REVISION implications.
- `findings/prompt-architecture-<T>.md` — prompt/tool/model coupling review and ADR/no-ADR routing.
- `reports/agent-run-ledger-<T>.md` — background-lane observability and failures/retries.
- `risk-policy.md` — HITL/SUPERVISED routing for high-risk plan items.
- `agent-backend-matrix.md` — lane isolation/backend choice.
- `agent-interop.md` — weave/MCP/ACP/A2A/GitHub-cloud routing decision.
- `research/sources-<T>.jsonl` — source provenance supporting research claims.

A plan missing any applicable P0-P2 section cannot be marked fully planned; `plan-artifact-gate.sh`
will reject DONE.
