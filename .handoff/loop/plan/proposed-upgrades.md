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
