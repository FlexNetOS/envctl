---
name: plan-synthesis
description: >-
  Turn verified findings into a decision-grade PLAN — verdict-first, ASCII architecture diagrams, a
  sequenced quality/speed/accuracy upgrade roadmap, a tool-evaluation, named gaps, and a stated
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
2. **ASCII architecture diagrams** — the target's structure + the graph's intelligence, rendered with
   box-drawing characters; **cite each diagram `Source: file:section`** (the graph snapshot, the
   codemap, or a `docs/secrets/*`/`docs/ARCHITECTURE.md` section). Annotate hotspots, cycles, and
   layering with the automation legend where a step is automated/gated (see below).
3. **Sequenced upgrade roadmap** — the ordered list of upgrades, each tagged
   `axis: quality|speed|accuracy`, ordered by value/risk (recipe below). Only CONFIRMED/QUALIFIED +
   feasible items.
4. **Tool-evaluation** — the tool/CLI/MCP/crate inventory with currency/advisories → upgrade/hold/pin
   (recipe below).
5. **Test Strategy & Coverage** — current coverage (by call-graph reachability), the ranked coverage
   gaps (untested public-API / hotspots / data-flows / error-paths, each citing the symbol), and the
   designed suite (cases, types, golden fixtures) that closes them and covers the roadmap's upgrades —
   from `findings/test-strategy-<T>.md`. Ends by promoting the **FF test-build spec** (recipe below).
6. **Gaps** — what is unverified, infeasible-here, or needs a harness (e.g. a perf claim that couldn't
   be benchmarked → "needs a perf harness"); plus notable REFUTED overclaims. Honesty over completeness.
7. **Confidence** — restate the overall confidence and what would raise it.

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
1. **Tag the axis** — exactly one of `quality` (correctness/maintainability/safety),
   `speed` (latency/throughput/build-time), or `accuracy` (result correctness/precision). Carry the
   axis from the analyst's UPGRADE row.
2. **Order by value/risk using the graph** — rank by **graph centrality** of the touched symbols
   (`metrics.hotspots`) and **blast-radius** (`metrics.blast_radius`): high-centrality + bounded-blast,
   low-risk upgrades come first; high-blast or high-risk ones are sequenced later with the dependency
   noted. A low-centrality cleanup is low priority even if easy.
3. **Admit only feasible + verified items** — anything `REFUTED`/`INCONCLUSIVE` or feasibility-gated
   out (e.g. an upgrade that would violate the no-C-in-trust-boundary invariant) is **excluded from the
   roadmap** and listed under gaps with the reason. Never recommend an infeasible upgrade.

Each roadmap entry uses the UPGRADE row format (`references/diagram-and-adr.md`) with its evidence,
blast-scope, risk, and the verdict ref that cleared it.

## Tool-evaluation rubric (R7)

1. **Inventory** the tools/CLIs/MCPs/crates the target actually uses — from the graph
   (`metrics.public_api` + the codemap's external deps), not from a guess.
2. **Currency + advisories** — pull each tool's latest version, release date, breaking-change flag, and
   any CVE/advisory from the researcher's `research/<T>.trends.md` **Tool-currency & advisories**
   subsection (the R7 input).
3. **Recommend** `upgrade` / `hold` / `pin` per tool, with a one-line rationale grounded in (a) the
   target's real usage (graph) and (b) the currency/advisory evidence (research). A tool with a fresh
   CVE → `upgrade` (or `pin` to the patched version); a heavily-depended-on tool with a breaking major
   release → `hold` + note the migration cost from blast-radius. Cite the evidence for each call.

## Test Strategy & the Feature-Forge handoff (the testing component)

The `plan-test-strategist` produces `findings/test-strategy-<T>.md` — current coverage (by call-graph
reachability), the ranked coverage gaps, the designed suite, and a `## FF test-build spec`. Lift its
**verified** rows into the plan's *Test Strategy & Coverage* section, then **promote the suite to
Feature Forge** — the planning loop *designs* tests; it never writes or runs them (read-only). The handoff:
- Append a **test-build ROADMAP row** (format in `references/diagram-and-adr.md`) shaped to Feature
  Forge's `feature-architect` `## Verification plan` intake: the test surface, the concrete cases
  (symbol/flow + assertion + type), the golden fixtures to capture, the coverage target, and the CI
  gate(s) the new tests touch.
- **Only verified, feasible items** enter the section/handoff — a "this is untested" claim the verifier
  REFUTED (it *is* tested), and any infeasible test design, are dropped, never handed to FF.
- This is the "creation + implementation" path: the plan specifies the suite precisely; **Feature Forge
  generates and runs it.** Do not write test code here.

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
