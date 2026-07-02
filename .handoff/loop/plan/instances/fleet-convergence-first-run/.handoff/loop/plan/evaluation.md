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

---

## Cycle 2 (handoff / union) — self-eval scorecard (evolution-steward)

Run: `fleet-convergence-first-run` · cycle **2** · target **handoff** (continuity kernel) planned as the
**UNION with rusty-idd** · 2026-06-26 (UTC). Marker: **scorecard / self-eval / evolution**.
Mode: owner approved continuation but **PROPOSE-only for harness edits this cycle** (record
APPLY-when-approved); the owner reviews before unattended free-running. This steward
**evaluates + mines + routes + PROPOSES only** — no harness edit auto-applied or PR'd.

Evidence base: `loop_state.md`, `findings/verdicts.md` (## handoff, cycle 2 — lines 79–214),
`findings/union-handoff-rusty-idd.md`, `reports/handoff-plan.md`, `reports/union-plan-handoff-rusty-idd.md`,
`reports/agent-run-ledger-handoff.md`, `dimensions.md:20–32`, `HANDOFF.md`.

### Scorecard — four axes

| axis | grade | one-line |
|---|---|---|
| Friction | **A** | Reconcile load **LOW** — every lane wrote **gate-named artifacts directly** (cycle-1 lesson **L1 applied**); `reports/agent-run-ledger-handoff.md` records **0 SendMessage reconcile round-trips** (cycle 1 had 2 of the same class). L2 flip-rule also held: `dimensions.md` flips each handoff dim with a per-dim verdict citation. |
| Gate quality | **A** | Verifier tally **57 CONFIRMED / 3 QUALIFIED / 0 REFUTED** and it ran **3 empirical experiments** (`cargo build`), incl. correcting the inherited **blake3+ed25519 → SHAKE-256-unsigned** witness-chain framing. Adversarial, not a rubber stamp; 0 false-blocks; every QUALIFIED is a strengthening condition. |
| Coverage | **B+** | handoff/`performance` left `[~]` (fail-closed — no measured delta); below-leaf tests **genuinely blocked** by the RuVector path-dep wall (ts-U2/ts-U3 can't author a COMPILING RED in-tree); ledger read-API (Seam 2 / union-3) **unbuilt**. All recorded, none silent — but the debt is real and partly external. |
| Human-walls | **A** | The RuVector resolution (A-U1) and the MERGE (union steps 1+2) are **genuine SUPERVISED owner walls** — large blast, `rusty-idd-*` pkg-name collision, witness-crypto move. None avoidable; correctly fail-closed. |

### Friction (A) — L1 validated in the field
- `reports/agent-run-ledger-handoff.md:9-23` lists 14 lanes, each emitting the **exact gate-required
  names** (`graph/handoff.{symbols,callgraph,metrics}.json`, `research/handoff.trends.md` with its
  required headers). **No lane was resumed via SendMessage to rename an artifact** — the cycle-1
  friction CLASS (L1) did **not** recur. The behavioral fix worked even though the durable skill edit
  (P1) is still APPLY-when-approved. This is the strongest signal of the cycle.
- L2 (flip authority) also held behaviorally: `dimensions.md:20-32` flips 11/12 handoff dims (+ union)
  to `[x]`, each citing its verifier verdict; the single `[~]` (`performance`, :23) carries a one-line
  fail-closed reason. Cycle-1's "6 dims left `[~]` despite CONFIRMED claims" did not repeat.

### Gate quality (A) — 57 CONFIRMED / 3 QUALIFIED / 0 REFUTED + 3 empirical experiments
- **Empirical, not static.** `verdicts.md:89-93` records three `cargo`-level experiments: EXP-1 proved
  the RuVector path-dep fails the workspace at **manifest-load** (`cargo build -p ledger` AND
  `--no-default-features --features redb-store` both fail) — the union is provably non-standalone;
  EXP-2 proved exactly one public `Ledger` per feature set + SHA3-256 pseudo-embeddings; EXP-3 is the
  **KEY CORRECTION** — the witness chain is `shake256_256` (`RuVector/.../witness.rs:74`), UNSIGNED
  (`ledger/src/v1.rs:20` imports no `sign`), NOT blake3+ed25519. blake3 is used only for
  `work-order::compute_intent_lock`. Any seed/doc/trends text saying "blake3+ed25519 witness chain" is
  REFUTED and the correction is propagated (mem-U3).
- **Not a rubber stamp despite 0 REFUTED.** The 3 QUALIFIED are precise: A-C7 corrects "rvf-crypto
  default-features=false" (ledger's `rvf-crypto` ships `ed25519` ON; the no-C conclusion still holds);
  A-C9/union-1 downgrade the tool-derived "95%" aggregate to QUALIFIED while CONFIRMING the
  fork/superset/byte-identical facts. The RED suite was **re-run standalone** (`ts-RED`,
  `verdicts.md:163`): 1 passed / 3 failed — a true RED, not an exit-0 fail-open.
- **0 false-blocks; gate only strengthens.** Every QUALIFIED-feasible upgrade (A-U2, A-U5, mem-U1/2/6,
  ts-U2/U3, UP-2/4, DC-3, ar-U5, union-2) carries a *condition* (RuVector-resolve / no-C boundary /
  default-warn-first / witnessed no-downgrade), never a relaxation. `infeasible = 0`.

### Coverage (B+) — what was left, all recorded
- **handoff/performance `[~]`** (`dimensions.md:23`) — fail-closed: only perf-adjacent verdicts
  (mem-U2/RVF write-amp) are QUALIFIED with magnitude unmeasured; no measured build-time/binary-size/
  runtime delta. Correct posture, real gap.
- **Below-leaf tests blocked by the RuVector wall** (`verdicts.md:165-166`, `union-plan…:156`): ts-U2
  (handoff-intake refusal) and ts-U3 (ledger read-API contract) cannot produce a COMPILING RED in-tree
  because the whole workspace fails manifest-load until A-U1. This is an **external** blocker, not a
  harness miss — but it caps what the test lane could verify this cycle.
- **Ledger read API (Seam 2 / union-3) is unbuilt** (`verdicts.md:161`, union-3) — a CONFIRMED design
  gap the union must close; recorded, not silently dropped.
- 11/12 handoff dims + the union dim verified; the completeness sweep ran (`reports/codemap-handoff.md`).
  Honest, but the blocked-test debt is genuine → B+ not A.

### Human-walls (A) — each genuine SUPERVISED
- **A-U1 (resolve RuVector path-dep)** — `handoff-plan.md:165` tier **SUPERVISED**, blast = entire kernel
  (`Ledger.open` 120); moves witness-crypto vendoring. The #1 action of the whole plan; correctly walled.
- **MERGE steps 1+2** — `union-plan…:66,80,150` both **SUPERVISED**: large blast, `rusty-idd-*`
  pkg-name collision (A-U4), witness-crypto move. `[H]` owner-walled merge in Diagram 2
  (`handoff-plan.md:101,107`).
- The 3 decision-findings (north-star home, run-from/residency/transport, harness_hub audience —
  `HANDOFF.md`) remain owner verdicts by design, not pre-answered. All walls are the structural/
  destructive class that **must** fail closed to a human. None avoidable.

### Headline (cycle 2)
- 4-axis: **Friction A · Gate-quality A · Coverage B+ · Human-walls A** — a markedly stronger cycle than
  cycle 1, driven by cycle-1 lessons **L1 + L2 paying off in the field** (0 reconciles; clean per-dim
  flips) and a verifier that went **empirical** (3 cargo experiments, the SHAKE-256 correction).
- Mined lessons this cycle: **3** (L5–L7; see `LESSONS.md`). The framing-vs-source correction CLASS
  **recurred** (cycle-1 verdict corrections → cycle-2 SHAKE-256) → L5 escalates to upgrade-now.
- Proposed upgrades: cycle-1 **P1–P4 carried** (P1 now field-validated) + **3 new** (P5 clean-clone
  standalone-build gate, P6 cross-repo schema-drift gate, P7 source-derive security-critical framing).
  **None auto-applied** (PROPOSE-only this cycle). **No gate weakened anywhere** — every new proposal
  strengthens or clarifies a gate, fail-closed.
