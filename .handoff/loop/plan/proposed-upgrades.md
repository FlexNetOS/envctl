# proposed-upgrades.md — routed harness upgrades (evolution-steward)

Run: `plan-loop-parallel / weave` · cycle **4** · target **weave** · 2026-06-26.

**PROPOSE-ONLY THIS CYCLE.** Every item below is recorded for owner review; nothing is auto-applied or
PR'd. Laws honored: **only ever strengthen/clarify a gate, never weaken one; fail-closed; when unsure,
treat as structural and propose; no cross-harness force-apply (scope law).**

Risk tiers:
- **APPLY-when-approved** — low-risk, in-scope (tighten a skill instruction, add an example/checklist/
  trigger, bundle a repeated helper, add a producer-side self-check). Auto-appliable via feature branch →
  PR → auto-merge **once the owner enables free-running** — never mid-cycle.
- **PROPOSE** — structural / touches a gate↔state relationship / changes a contract → owner approval
  required even after free-running begins.
- **REGENERATE** — re-run a producer to refresh an artifact (no harness-code change).

---

## NEW this cycle (P8, P9)

## P8 — Namespace per-instance loop state under `runs/<target>/` (lesson L8) — **PROPOSE**
**Problem (CLASS):** the parallel-run design (L8) is validated — own-worktree + branch + lease gave true
isolation with zero union-branch edits. But each parallel instance still writes its loop artifacts into
the SAME relative `.handoff/loop/plan/` tree (just in a different worktree). When two instances later need
to be compared, rolled up, or run from one checkout, their `evaluation.md` / `LESSONS.md` /
`proposed-upgrades.md` / `dimensions.md` collide by path. A `runs/<target>/` (e.g.
`.handoff/loop/plan/runs/weave/…`) namespace would let parallel instances coexist and roll up cleanly
without per-instance worktree juggling.
**Route:** `plan-loop` orchestrator SKILL (parallel-run mode) — define a per-target state namespace
`runs/<target>/` for instance-scoped artifacts; keep the shared durable ledger append-only at the loop
root. Carry of the parallel prompt's known rough edge (`prompt_hub/prompts/plan-loop-parallel-run.md`).
**Risk:** representation/path-layout only; does not touch any gate or verdict logic. Sits near the
completeness gate's artifact-name assertions, so PROPOSE to be safe — but it cannot weaken the gate (the
gate's required names move with the namespace, still asserted).
**Apply-vs-propose:** PROPOSE — changes the state-layout contract the gate reads. Once approved it
decomposes into an APPLY-when-approved skill edit + a gate path update reviewed together.
**Acceptance:** two parallel instances (e.g. weave + a sibling target) each write a full
`runs/<target>/` plan tree with no path collisions; the completeness gate still fails closed on a missing
gate-named artifact within each namespace; the durable ledger stays single + append-only.

## P9 — "Peer-artifact pending-vs-missing" rule for axis-auditor prompts (lesson L9) — **APPLY-when-approved**
**Problem (CLASS):** under parallel fan-out, a concurrent axis lane can read a peer's sibling artifact
before that peer has finished writing it and momentarily see it absent. Treating expected-but-absent as a
fail-closed "missing artifact" finding would ship a false negative. This cycle it stayed harmless — no
false finding reached `verdicts.md` because the orchestrator already gates analysis on orientation
completion — but the rule should be explicit so the defense doesn't rely on timing.
**Route:** `plan-analyst` / the axis-auditor agent prompts (`plan-governance-config-auditor`,
`plan-memory-vector-intelligence-auditor`, `plan-prompt-architecture-auditor`, etc.) — add a one-line
rule: *an expected peer artifact absent during concurrent execution is PENDING (re-check after the
producing lane is known complete), NOT a fail-closed missing-artifact finding; reserve a hard finding for
an artifact still absent AFTER its producer lane reports done.* Note in the same place that the
orchestrator's orientation-completion gate is the standing structural defense.
**Risk:** low — additive clarification to agent prompts; touches no gate, no verdict logic. It does NOT
relax fail-closed behavior — it narrows it to "absent after producer done", which is strictly more
correct, never more permissive for a genuinely-missing artifact.
**Apply-vs-propose:** APPLY-when-approved — agent-prompt clarification, the low-risk in-scope class. Land
via feature branch → PR → auto-merge after the owner enables free-running, with a CLAUDE.md change row.
**Acceptance:** an axis lane that reads a not-yet-written peer artifact records it as PENDING and
re-checks; a hard "missing artifact" finding fires only after the producing lane is complete.

---

## CARRY-FORWARD (cycle 1–2 P1–P7) — status this cycle

- **P1 — single-source artifact-name contract — APPLY-when-approved (re-validated, still un-landed).**
  Cycle 4 (13 parallel lanes) emitted all gate-named artifacts with **0 SendMessage reconciles** — the
  behavioral fix holds even under parallelism (L8/L1-recurrence). The durable skill-edit (+ optional
  bundled `scripts/check-artifact-names.sh`) remains un-landed. Recommend APPLY first once free-running is
  enabled. Low-risk, in-scope.
- **P2 — gate strengthen: promoted upgrades must carry a feasibility verdict — PROPOSE (still open).**
  Cycle 4 behaviorally compliant (all 10 UPGRADE rows carry a feasibility verdict: 9 FEASIBLE / 1
  FEASIBILITY-QUALIFIED / 0 INFEASIBLE, `verdicts.md:82`), but the **gate still does not enforce** the
  promoted-vs-candidate distinction. Keep proposed; strengthen-only.
- **P3 — dimension-flip authority — PROPOSE (partially exercised, still un-encoded).** Cycle 4 flipped
  5/12 dims `[x]` with per-dim verdict citations and left 7 `[~]` each carrying a one-line missing-artifact
  reason (`dimensions.md:4-15`) — the desired behavior, but the flip-rule is still not encoded in
  `plan-loop` + `plan-verifier`. Encode it to lock in. Clarifies verified-state authority; fail-closed.
- **P4 — scale the DAG artifact, not the gate — PROPOSE (unchanged).** Representation-only; JSON stays
  exhaustive + gated. No change this cycle.
- **P5 — clean-clone / standalone-build standing experiment + gate — PROPOSE (not exercised this cycle).**
  weave was analysed in-tree; no standalone-residency claim was tested this pass. Keep proposed;
  strengthen-only.
- **P6 — cross-repo schema-drift gate for mirrored contracts — PROPOSE (cross-harness, not exercised).**
  No port-and-merge in this cycle. Proposed TO `rust-port-merge`/`cross-repo-referencer`, never
  force-applied here (scope law). Keep open.
- **P7 — source-derive security-/architecture-/contract-critical framing — PROPOSE (RECURRED again →
  stays upgrade-now; claim class EXTENDED).** Cycle 4 added a third instance of the L5 class:
  GOV-003 — the enforced CI gate is **7** required checks (incl. supply-chain `audit`) while `CLAUDE.md`
  and `policy.toml` document **6** (`verdicts.md:31,56`); companion GOV-004 — `ci.yml` runs Python while
  the no-Python invariant claims otherwise (`verdicts.md:57`). **Extend P7's claim class** to explicitly
  include *enforced-gate counts and CI/build-plane invariants* (documented-vs-enforced drift), alongside
  crypto/safety/signing/schema. Route unchanged: `plan-verifier` agent def (source-derive the claim) +
  `plan-trend-research` SKILL (do not propagate inherited framing). Strengthen-only.

---

## Coverage follow-ups (REGENERATE / data-refresh — not harness changes; recorded so they are not lost)
- **R7** — run the verifier over the **4 present-but-not-gated** axis findings before any weave DONE that
  needs them: `filesystem-layout-weave.md`, `autoresearch-weave.md`, `rules-policy-org-weave.md`,
  `distributed-compute-weave.md` (`dimensions.md:10,12,13,14` — each `[~]` "present but not gated").
- **R8** — author dedicated `code-quality` / `correctness` / `performance` findings for weave (currently
  folded into architecture slices only, `dimensions.md:5-7`) so those three axes can flip from `[~]`.
- **R9** — feed the 3 verifier corrections back to the analyst before they become plan facts: `Store`
  count 29→~90, MCP tools 78→72 arms/76 catalog, ARCH-12 `call_tool↔tool_meta` = real bounded recursion
  (`verdicts.md:84-88`); and re-scope U-ARCH-1 acceptance to the real ~90-method surface.
- **R10** — trace the inject 5-cluster SCC (`spawn/kill/run_bounded*`) left **INCONCLUSIVE** this pass
  (`verdicts.md:51`) to confirm/refute it as a resolver artifact vs real recursion.

---

## Not done (and why)
- **No auto-apply, no PR this cycle** — PROPOSE-only run brief; owner reviews before unattended
  free-running.
- **No gate weakening anywhere** — P2/P5/P6/P7/P8 only strengthen/clarify; P9 narrows fail-closed to the
  strictly-more-correct "absent after producer done". Any "loosen the gate so cycles pass" framing is
  refused by default (none proposed). The verifier feasibility-gate (no-C / strict-upgrade) was preserved
  on all 10 UPGRADE rows.
- **No cross-harness force-apply** — P6 is proposed TO `rust-port-merge`/`cross-repo-referencer`; L8–L10
  routing stays scoped to the planning-engineer harness this run stewarded
  (`harness_hub/harness/skills/plan-*` + `envctl/.claude/skills/planning-engineer`). Scope law.
