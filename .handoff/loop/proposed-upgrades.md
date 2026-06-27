# Proposed harness upgrades (evolution-steward escalations)

> DRAINED — empty body means no undrained proposals. The evolution-steward (Phase E) appends
> ESCALATED structural proposals here; `session-relay-wrap-up` step 3b drains every entry to a
> tracked disposition (open → `- [?]` backlog item; addressed → recorded resolved; declined →
> recorded) and resets this file. A **non-empty body at end of wrap-up means wrap-up is INCOMPLETE.**
>
> Last drained: 2026-06-18 (TASK-0042) — P1 merge-driver test RESOLVED, P2 reaper test RESOLVED,
> P3 scheduled-reaper DECLINED (accept loop-boundary-only). See backlog Epic G / TASK-0042.

---

# Plan-loop (fleet-convergence) — structural proposals (PROPOSE-only)

> Separate ledger from the forge-loop block above (this worktree hosts the plan-loop too). These are
> STRUCTURAL items the evolution-steward will not auto-apply (orchestrator ordering, harness docs,
> cross-skill edits) — owner approval required. None weakens a gate; all strengthen or are additive.
> `[FLAG-APPLY]` = a class at its 2nd recurrence (apply-eligible now per once→noted / 2nd→upgrade-now).

## Carried forward from cycle 4 (still open)

- **P-C4-1 — Relax the snake_case slug regex.** The plan-loop's target/finding slug validation
  rejects legitimate target names that aren't strict snake_case (hyphenated repo names, dotted
  crate paths). Relax the regex to accept the real fleet naming surface without weakening any other
  validation. STRUCTURAL (touches a validation gate's input grammar — widening, so PROPOSE + verify
  it never admits a path-traversal/empty slug). Status: OPEN (carried from cycle 4).
- **P-C4-2 — Namespace durable state under `runs/<target>/`.** Per-cycle plan artifacts currently
  collide on shared filenames across targets when not isolated by worktree; namespace them under
  `runs/<target>/` so cycles are independently addressable and a multi-target batch can't overwrite a
  prior target's findings. STRUCTURAL (changes the state-contract layout other skills read). Status:
  OPEN (carried from cycle 4).

## New this cycle (cycle 5 / grit)

- **P-C5-1 — Pre-cycle environment check for stray phantom-workspace manifests. [FLAG-APPLY — 2nd
  recurrence]** Before any build/verify gate, scan ancestor directories for a stray `Cargo.toml`
  that would make Cargo walk up and absorb the target into a foreign virtual workspace; either fail
  closed with a clear remediation (remove/relocate the stray manifest) or pin the target with an
  empty `[workspace]`. Recurs across the fleet (handoff A-U1 sibling-escape was the 1st occurrence;
  grit cycle 5 the 2nd) → eligible to apply now. STRUCTURAL (adds an orchestrator pre-cycle step).
  Evidence: verdicts.md FEASIBILITY phantom-workspace remediation row (verified feasible).
- **P-C5-2 — Document weave as an in-cycle cross-loop coordination primitive.** Add a harness note
  that a loop MAY use a weave A2A round-trip DURING a cycle (ask another loop/session to verify a
  plan, fold corrections, ship) — not only for async handoff heartbeats. Keep propose-by-default: the
  committed artifact stays authoritative; weave is the observable coordination channel. STRUCTURAL
  (orchestrator/harness-doc capability statement). Evidence: cycle-5 envctl↔rusty-idd round-trip →
  prompt_hub PR #182 (3 corrections folded). First demonstrated occurrence → noted, propose.
- **P-C5-3 — Graph-before-analysis ordering in the plan-loop schedule.** Make the cartographer's
  graph JSON a topological upstream dependency of the axis-auditor/analyst ready-set (or have those
  agents explicitly declare a graph-absent gap, never proceed graph-blind). Removes the depth loss
  from the cycle-5 race where analysis ran before the graph existed. STRUCTURAL (orchestrator
  scheduling / dependency DAG). Evidence: `findings/architecture-grit.md:10` (declared-gap, correct
  fail-closed behavior — fix the race so depth isn't lost). First occurrence → noted, propose.

## Cross-skill (route to owner; scope law — propose, never force-apply)

- **P-C5-4 — Oblique-reference rule for the sentinel-token gate trap. [FLAG-APPLY — 2nd
  recurrence]** Add to the plan-* auditor skill bodies a one-line rule + example: when writing a
  finding ABOUT the completeness gate's rejection vocabulary, reference the class obliquely (name the
  sentinel category; never quote the literal tokens) so an honest gap-finding still passes the gate.
  STRENGTHENS reliability, weakens nothing. 2nd recurrence (1st = cycle 4) → apply-eligible; queued
  for the batch boundary because the skill sources live in harness_hub, outside this isolated
  worktree (record + route, never a live mutation here).
