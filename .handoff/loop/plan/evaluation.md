# evaluation.md — self-eval scorecard (evolution-steward)

Run: `fleet-convergence-first-run` · cycle **1** · target **rusty-idd** · 2026-06-26 (UTC).
Marker: **scorecard / self-eval / evolution**. Mode: FIRST owner-reviewed capped run
(`cycle_budget=1`, `wrap_every=1`) — this steward **evaluates + mines + routes + PROPOSES only**.
Per the first-run brief §8, **no harness edit is auto-applied or PR'd this cycle**; every routed
upgrade is recorded as a proposal for owner review in `proposed-upgrades.md`.

Evidence base: `loop_state.md`, `findings/verdicts.md`, `findings/*.md`, `reports/rusty-idd-plan.md`,
`reports/agent-run-ledger-rusty-idd.md`, `graph/target-dag.md`, `graph/rusty-idd.metrics.json`,
`dimensions.md`, `scripts/plan-artifact-gate.sh` (the gate this run answers to).

---

## Scorecard — four axes

| axis | grade | one-line |
|---|---|---|
| Friction | **B** | Clean run, but **2 within-cycle reconcile round-trips** (SendMessage), both the same CLASS — producer artifact-naming did not match gate-required names. |
| Gate quality | **A-** | Verifier caught a real fail-open defect and made 2 precise corrections that flowed into the plan; **0 false-blocks**; only soft spot is partial upgrade-row gating. |
| Coverage | **B** | Nothing silently capped — every gap (truncation lower-bounds, UNCONFIRMED cross-repo edges, ungated per-axis upgrades) is **honestly recorded**, but the debt is real. |
| Human-walls | **A** | Every NEEDS-HUMAN/`[H]` wall is **genuine** (architecture decision / deletes tracked trees / first live net dep); none avoidable. The cap itself is the brief, not a failure. |

### Friction (B)
- **2 reconcile round-trips, one CLASS.** `reports/agent-run-ledger-rusty-idd.md:32-35` records that
  agents 1 (cartographer) and 2 (trend-researcher) were **resumed via SendMessage** to emit
  **gate-named** artifacts: the split graph JSONs (`graph/rusty-idd.{symbols,callgraph,metrics}.json`)
  and the trends file's required `"Tool-currency & advisories"` / `"Sources"` headers. Both are exactly
  what `scripts/plan-artifact-gate.sh:155-157,187` asserts as REQUIRED. No re-index was needed and the
  existing graph was reused, so cost was bounded — but the round-trip was avoidable: the producers and
  the gate independently encode the same artifact names/headers and **drifted**. Two different producers
  hit the **same class** in one cycle → the class is systemic, not incidental.
- No items bounced `- [~]`→`- [ ]`; no wasted cycles; no retried agents. Friction was self-corrected
  in-cycle, not carried forward.

### Gate quality (A-) — verifier tally 22 CONFIRMED / 1 QUALIFIED / 0 REFUTED
- **Caught a real defect.** The fail-open `work-order` card load (`ts-24/25/26`, `verdicts.md:55`) —
  `serde_json::from_str::<WorkOrder>` silently accepts a foreign `schema` discriminator / bad `id` /
  drifted `intent_lock`; the verifier CONFIRMED it against source and a RED suite already fails for the
  right reason (`crates/work-order/tests/handoff_card_consumer.rs`, 3 RED + 1 GREEN). This is the plan's
  headline first-step (`reports/rusty-idd-plan.md:42-46`).
- **Not a rubber stamp despite 0 REFUTED.** It QUALIFIED C13 with two concrete corrections —
  `merge.rs` is at `crates/spec/src/model/merge.rs` (not `crates/spec/src/merge.rs`); there are **5**
  `spec_*` CLI commands, not 6 (`verdicts.md:27`) — and both corrections were carried verbatim into the
  plan (`rusty-idd-plan.md:9-12`). It also *lifted* C12 from analyst-medium after re-reading the cited
  lines (`verdicts.md:26`). That is genuine adversarial reading, so 0 REFUTED reflects strong analyst
  input, not under-skepticism.
- **0 false-blocks.** No upgrade was found infeasible; the conditions attached to U7/DC-2 (C-free trust
  path + weave-required + filesystem fallback) and U4 (measure the speed magnitude) are **strengthenings**,
  not blocks (`verdicts.md:36,39,61`).
- **Soft spot (→ Coverage, not Quality):** only the architecture set (U1–U10) + DC-2/DC-5/FL-3 were
  *feasibility-gated*; the governance/memory/autoresearch/rules-policy/filesystem **upgrade** rows reached
  the report as **candidates, explicitly not promoted** (`rusty-idd-plan.md:440-444`). The architect
  labelled them honestly — the gate held — but the upgrade set was only partially gated this cycle.

### Coverage (B) — what was left behind, all recorded
- **Truncation lower-bounds:** dead-code (≥278) and public-API (≥500) are truncated at the git-kb
  500-row cap (`graph/rusty-idd.metrics.json:100,117-118`; `rusty-idd-plan.md:432-434`). Recorded as a
  lower bound with a named "re-run explicit per-crate `code symbols`/`code dead`" follow-up before any
  DONE that needs an exhaustive count.
- **Cross-repo fabric edges UNCONFIRMED:** rusty-idd↔hf/weave, weave-as-load-bearing-transport, grit
  cross-repo adoption, hf↔icm (`findings/fleet-north-star-map.md:45-52,138`; `rusty-idd-plan.md:437-439`)
  — need cross-repo `kb_callers` before U7/DC-2 detail design.
- **Per-axis upgrade rows not feasibility-gated** (see above) — deferred to the next verifier pass.
- **Symbol-level cycles not computed** (`code flows` returned `[]`); clean-DAG verdict is authoritative
  at **crate** level only (`rusty-idd-plan.md:435-436`).
- Verdict: zero silent capping — the completeness sweep ran (`reports/codemap-rusty-idd.md`), all 11
  crates + convergence axis re-derived non-zero. Honest, but the debt is genuine, hence B not A.

### Human-walls (A) — each genuine, none avoidable
- `[H]` OWNER intent / OpenSpec goal (`rusty-idd-plan.md:137`) — the why/what binding is owner-owned by
  design (the `sr-001` north-star OPEN finding).
- `[H]` `agent-guard.toml` (`:178`) — control-plane policy, correctly human-gated.
- **U7 / DC-2 owner-gated** (`:402`, `verdicts.md:39,61`) — introduces the **first live network/IPC
  dependency** into an offline-by-construction binary; a real architecture decision (the DRAFT ADR).
- U5 (delete triple-vendored trees) / U6 (integrate-or-retire `work-order`) are PROPOSE — deletes tracked
  state / changes membership; owner-decision class.
- All walls are the structural/destructive class that **must** fail closed to a human. None is an
  avoidable stop. The whole-run cap (propose-only, 1 cycle) is the first-run brief, not a loop wall.

---

## Dimension-ledger observation (feeds lesson L2)
`dimensions.md` shows the verifier flipped 6 dims to `- [x]` (architecture, test-coverage,
governance-config, filesystem-layout, memory-vector-intelligence, distributed-compute) but left 6 at
`- [~]` (code-quality, correctness, performance, autoresearch, rules-policy-org, prompt-architecture).
Those 6 were analysed and their *claims* are CONFIRMED in `verdicts.md`, yet the ledger marks stayed
conservative — **flip authority is unstated** (who flips, on what signal), so the orchestrator had to
reconcile. Not a defect this cycle (the plan correctly treats those dims' upgrades as candidates), but a
contract gap worth closing.

---

## Headline
- 4-axis: **Friction B · Gate-quality A- · Coverage B · Human-walls A.**
- A strong, honest first cycle: the gate caught the one defect that matters and every wall was genuine.
  The only repeatable friction is a **producer↔gate artifact-naming drift** that fired twice in one cycle.
- Mined lessons: **4** (see `LESSONS.md`). Proposed upgrades: **4**, all routed, **none auto-applied**
  (first-run brief §8). No gate is weakened anywhere — two proposals *strengthen* the gate.
