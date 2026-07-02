# proposed-upgrades.md — routed harness upgrades (evolution-steward)

Run: `fleet-convergence-first-run` · cycle 1 · target rusty-idd · 2026-06-26.

**FIRST-RUN BRIEF §8 IN FORCE: every item below is PROPOSED for owner review. Nothing is auto-applied
or PR'd this cycle.** Each row records what this steward *would* route, its risk tier, and the
apply-vs-propose policy that will govern it once the loop runs free. Laws honored: **only ever
strengthen/clarify the verify/completeness/DONE gate, never weaken it; fail-closed; when unsure,
treat as structural and propose.**

Risk tiers:
- **APPLY-deferred** — low-risk, in-scope (tighten a skill instruction, add an example/checklist/trigger,
  bundle a repeated helper, add a producer-side self-check). Auto-appliable via PR **once the owner
  approves the first-run free-running mode** — never mid-cycle.
- **PROPOSE** — structural / touches a gate↔state relationship / changes a contract → owner approval
  required before any edit, even after free-running mode begins.
- **REGENERATE** — re-run a producer to refresh an artifact (no harness-code change).

---

## P1 — Single-source artifact-name contract (lesson L1) — **APPLY-deferred**
**Problem (CLASS):** producers (`plan-cartography`, `plan-trend-research`) and the completeness gate
(`scripts/plan-artifact-gate.sh`) independently hard-code the same required artifact filenames + section
headers; they drift → mid-cycle SendMessage reconciles. Fired twice in one cycle (graph JSON splits +
trends headers), so it is systemic.
**Route:** `plan-cartography` SKILL + `plan-trend-research` SKILL bodies — add an explicit "emit these
EXACT gate-required names/headers" checklist that mirrors `plan-artifact-gate.sh:155-157,187`; optionally
bundle a tiny `scripts/check-artifact-names.sh` the producer runs before it reports DONE, reading the
same name list the gate asserts (single source of truth).
**Risk:** low — additive checklist / self-check in producer skills; does not touch the gate or any
verdict logic. Strictly reduces friction.
**Apply-vs-propose:** APPLY-deferred — tighten skill instruction + bundle helper, exactly the low-risk
in-scope class. Land via feature branch → PR → auto-merge **after owner enables free-running**, with a
CLAUDE.md change-history row.
**Acceptance:** a fresh cycle produces all gate-named graph splits + trends headers with **zero**
post-hoc SendMessage reconciles.

## P2 — Strengthen the gate: promoted upgrades must carry a feasibility verdict (lesson L4) — **PROPOSE**
**Problem (CLASS):** the plan can carry upgrade rows that were never feasibility-gated. This cycle the
architect handled it correctly (labelled them *candidates*, not *promoted* — `rusty-idd-plan.md:440-444`),
but nothing in the gate **enforces** that distinction; a future cycle could promote an ungated upgrade.
**Route:** `scripts/plan-artifact-gate.sh` — add an assertion that any upgrade row presented as a
**promoted/plan** row (vs an explicitly-labelled *candidate*) must reference a CONFIRMED/QUALIFIED +
feasibility-passed verdict id in `findings/verdicts.md`; + `plan-synthesis` SKILL — require the
promoted-vs-candidate label on every upgrade row.
**Risk:** touches a **gate** → PROPOSE by law. This is a pure **strengthening** (it can only add a
required check; it never lets anything through that passes today). Never weakens.
**Apply-vs-propose:** PROPOSE — gate change, owner approval required even after free-running.
**Acceptance:** the gate fails closed if a plan promotes an upgrade row with no feasibility verdict;
candidate-labelled rows pass.

## P3 — Assign dimension-ledger flip authority (lesson L2) — **PROPOSE**
**Problem (CLASS):** `dimensions.md` marks (`[~]`→`[x]`) have no stated owner/flip-rule, so the verifier
stays conservative (left 6 dims `[~]` though their claims are CONFIRMED) and the orchestrator reconciles.
**Route:** `plan-loop` orchestrator SKILL — state the flip-rule ("verifier flips a dim to `[x]` when it
records a CONFIRMED/QUALIFIED verdict for that dim; orchestrator flips only the documented residual and
records why"); mirror in `plan-verifier` agent def.
**Risk:** clarifies the gate↔state relationship (who may change verified-state) → treat as structural →
PROPOSE. It only *adds* clarity on when a dim may be marked verified; it does not let an unverified dim
be marked `[x]` — fail-closed preserved.
**Apply-vs-propose:** PROPOSE — touches verified-state authority.
**Acceptance:** next cycle, every dim with a CONFIRMED verdict is `[x]` with no orchestrator reconcile
step; any residual `[~]` carries a one-line reason.

## P4 — Scale the DAG artifact, not the gate (lesson L3) — **PROPOSE**
**Problem (CLASS):** node-per-slug completeness is correct, but a 63-node/30-edge DAG md is heavy and
grows with the backlog.
**Route:** `plan-dependency-graph` SKILL — formalize that `graph/target-dag.md` MAY summarize (rollup of
ready-set + topo layers) while `graph/target-dag.json` stays the **exhaustive, gated** companion. The
one-node-per-slug rule in `plan-artifact-gate.sh:121-142` stays **unchanged**.
**Risk:** representation-only, but it sits next to a gate, so PROPOSE to be safe. **Explicitly does not
weaken** the gate (JSON remains exhaustive + gated).
**Apply-vs-propose:** PROPOSE — near-gate, fail-closed default.
**Acceptance:** md stays readable as the backlog grows; the JSON still fails closed on any missing slug.

---

## Coverage follow-ups (REGENERATE — not harness changes; recorded so they are not lost)
These are data-refresh actions the next cycle should run; no harness edit:
- **R1** — re-run explicit per-crate `git-kb code symbols` + `code dead` to lift the 500-row truncation
  before any DONE that needs an exhaustive count (esp. U4/U10 magnitudes). Ref `rusty-idd-plan.md:432-434`.
- **R2** — cross-repo `kb_callers` to confirm/refute the UNCONFIRMED fabric edges (rusty-idd↔hf/weave,
  weave-as-transport, grit adoption, hf↔icm) before U7/DC-2 detail design. Ref `rusty-idd-plan.md:437-439`.
- **R3** — next verifier pass: feasibility-gate the per-axis upgrade candidates (governance/memory/
  autoresearch/rules-policy/filesystem) so they can be promoted from candidate to plan rows.

## Not done (and why)
- **No auto-apply, no PR this cycle** — first-run brief §8 (owner reviews before the loop runs free).
- **No gate weakening anywhere** — P2 and P4 only strengthen/clarify; refused-by-default would be any
  "loosen the gate so more passes" framing (none proposed).
- **No cross-harness force-apply** — L1–L4 are scoped to the planning-engineer harness this run stewarded
  (`harness_hub/harness/skills/plan-*` + `envctl/.claude/skills/planning-engineer`). Any value to other
  harnesses is theirs to adopt, not forced here (scope law).

---

# Cycle 2 (handoff / union) — routed harness upgrades

Run: `fleet-convergence-first-run` · cycle 2 · target handoff (UNION with rusty-idd) · 2026-06-26.

**OWNER CONTINUATION APPROVED, but harness edits remain PROPOSE-only this cycle** (owner reviews before
unattended free-running). Every row below is recorded as a proposal with its risk tier and the
apply-vs-propose policy that will govern it once free-running begins. Laws honored: **only ever
strengthen/clarify a gate, never weaken one; fail-closed; when unsure, treat as structural and propose.**

## Carry-forward (cycle 1 P1–P4)
- **P1 — single-source artifact-name contract — APPLY-when-approved (now FIELD-VALIDATED).** Cycle 2
  produced ALL gate-named graph splits + trends headers with **0 SendMessage reconciles**
  (`reports/agent-run-ledger-handoff.md:9-23`) — the lesson works behaviorally; the durable skill-edit
  (+ optional bundled `scripts/check-artifact-names.sh`) is still un-landed. **Recommend: APPLY first
  thing once the owner enables free-running** (it has now demonstrated zero-regression value). Low-risk,
  in-scope (skill checklist + helper); land via feature branch → PR → auto-merge + CLAUDE.md row.
- **P2 — gate strengthen: promoted upgrades must carry a feasibility verdict — PROPOSE (still open).**
  Cycle 2 *behaviorally* improved (all 39 upgrade rows carry feasible/infeasible verdicts,
  `verdicts.md` tallies) but the **gate still doesn't enforce** it. Keep proposed; strengthen-only.
- **P3 — dimension-flip authority — PROPOSE (behaviorally validated, still un-encoded).** Cycle 2
  flipped 11/12 dims with per-dim verdict citations (`dimensions.md:20-32`); encode the flip-rule in
  `plan-loop` + `plan-verifier` to lock it in. Clarifies verified-state authority; fail-closed.
- **P4 — scale the DAG artifact, not the gate — PROPOSE (unchanged).** Representation-only; JSON stays
  exhaustive + gated.

## P5 — Clean-clone / standalone-build gate (lesson L6) — **PROPOSE**
**Problem (CLASS):** a "standalone / portable-root residency" claim passes a crate-graph read but a
`../../sibling` path-dep fails the instant the repo is cloned alone — EXP-1 proved the union is
non-standalone (`verdicts.md:91`), the plan's #1 blocker (`handoff-plan.md:18,208`). The harness had no
*standing* empirical residency check; this cycle the verifier ran it ad-hoc.
**Route:** (a) `plan-verifier` agent def — make a clean-clone / standalone `cargo build` a STANDING
experiment for any target that claims standalone residency (resolve path-deps in a sibling-free scratch
clone; record PASS/FAIL empirically); (b) a CI + `plan-artifact-gate.sh` check that fails closed when a
target asserting standalone residency carries an unresolved cross-repo path-dep.
**Risk:** touches a **gate/method** → PROPOSE by law. Pure **strengthening** — it only ADDS an empirical
residency assertion; it can never let a today-passing target through. Never weakens.
**Apply-vs-propose:** PROPOSE — gate/verifier-method change, owner approval required even after free-running.
**Acceptance:** the gate fails closed when a "standalone" target has an unresolved `../../` path-dep; a
genuinely standalone target (clean-clone `cargo build --workspace` green) passes.

## P6 — Cross-repo schema-drift gate for mirrored contracts (lesson L7) — **PROPOSE**
**Problem (CLASS):** when a port-and-merge plan finds a **contract mirror** (a file-copied schema, not a
dependency), the two repos can silently diverge and the consumer inherits any fail-open through the
mirror — `rusty-idd`'s `work-order` mirrors handoff's `task.schema.json` (`verdicts.md:110`, A-C13), and
the union inherits the fail-open loader through it (`verdicts.md:159`, ts-3). ts-U4 (golden
`task_schema_json` parity) is the fix.
**Route:** `rust-port-merge` / `cross-repo-referencer` SKILL — when a mirrored cross-repo contract is
detected, REQUIRE a differential-golden drift artifact (capture both sides' schema, diff, fail on
mismatch) before the merge is planned as feasible.
**Risk:** strengthens the merge-planning method (adds a required drift artifact) → PROPOSE. Never
weakens — it only adds a required check; absent the artifact the merge stays NOT-feasible (fail-closed).
**Cross-harness note:** `rust-port-merge`/`cross-repo-referencer` are NOT the planning-engineer harness
this run stewarded — this is **proposed TO that harness**, never force-applied here (scope law).
**Acceptance:** a planned union/merge that finds a mirrored contract carries a golden parity artifact;
the merge is not feasible-rated without it.

## P7 — Source-derive security-/architecture-critical framing (lesson L5 — recurred, upgrade-now) — **PROPOSE**
**Problem (CLASS):** inherited framing (seed / doc / trends / prior-cycle) about a crypto/safety/contract
mechanism is an unverified claim until read at the source line. This cycle the inbound "blake3+ed25519
witness chain" framing was REFUTED to SHAKE-256-unsigned (`verdicts.md:93`). The *verifier-corrects-
inherited-framing* class also fired in cycle 1 (merge.rs path + spec_* count), so it **recurs → upgrade-
now** per the escalation rule.
**Route:** (a) `plan-verifier` agent def — a security-/architecture-critical claim (crypto, unsafe,
trust-boundary, signing, schema-contract) must be CONFIRMED from a cited source line, never from a seed/
doc/trends restatement; mark INHERITED-UNVERIFIED until source-derived; (b) `plan-trend-research` SKILL —
do not propagate seed/doc crypto/safety framing into findings without a source check.
**Risk:** strengthens the verifier method (adds a required source-derivation for a claim class) →
PROPOSE. Never weakens — it raises the bar for a sensitive claim class only.
**Apply-vs-propose:** PROPOSE — verifier-method change. (The `plan-trend-research` half is the low-risk
in-scope skill-tightening class and could be APPLY-when-approved; the verifier-method half is PROPOSE.)
**Acceptance:** next cycle, any crypto/safety/trust-boundary claim in a finding cites a source line or is
marked INHERITED-UNVERIFIED; no inherited framing reaches the plan unverified.

## Coverage follow-ups (REGENERATE / data-refresh — not harness changes)
- **R4** — author ts-U2 (handoff-intake refusal) + ts-U3 (ledger read-API contract) as COMPILING RED
  **after A-U1 resolves the RuVector wall** — both are blocked in-tree today (`verdicts.md:165-166`).
- **R5** — design the missing ledger read API (Seam 2 / union-3) before the union DONE
  (`verdicts.md:161`).
- **R6** — measure the deferred `performance` dimension deltas (build-time/binary-size/runtime) so
  `handoff/performance` can flip from `[~]` (`dimensions.md:23`).

## Not done (and why)
- **No auto-apply, no PR this cycle** — owner reviews before unattended free-running (continuation brief).
- **No gate weakening anywhere** — P2/P5/P6/P7 only strengthen/clarify; any "loosen the gate so cycles
  pass" framing is refused by default (none proposed).
- **No cross-harness force-apply** — P6 is proposed TO `rust-port-merge`/`cross-repo-referencer`, not
  applied here; L5–L7 routing stays scoped to the harness this run stewarded (scope law).
