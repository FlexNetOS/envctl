# Plan-loop run evaluation — fleet-convergence planning loop

Per-cycle scorecard written by the evolution-steward (harness-evolution Phase E) at each cycle
boundary. Append one `## <target>` section per cycle; this file is per-run scratch (the durable
memory is `plan/LESSONS.md`). Earlier cycles (4-5: grit, weave, union) recorded in their own
worktrees; this instance opens at cycle 6.

## prompt-hub

Cycle 6 · TARGET=prompt-hub · 2026-06-27 · run `plan-prompt-hub-20260627` (parallel instance,
loop_branch `plan/loop-prompt-hub`, isolated worktree off `origin/master`). 12-agent crew →
plan-verifier gate. Gate counts: **6 CONFIRMED / 1 QUALIFIED / 0 REFUTED / 0 INCONCLUSIVE** → PASS.
Target outcome: prompt-hub `[~]` planned-with-gaps (5/10 dims verified `[x]`, 2 `[~]`, 3 `[ ]`
out-of-scope).

### Scorecard

| Axis | Grade | Basis |
|---|---|---|
| Friction | **MEDIUM-LOW** | One new friction class (auditor returned text not a file) + one recurrence (vendor-polluted code index) + one benign cross-auditor disagreement that the gate resolved. All recovered in-cycle; nothing bounced `[~]`→`[ ]`. |
| Gate quality | **HIGH** | Gate held both ways: load-bearing absence CONFIRMED by grep+source+RED dump; over-claimed trend correctly QUALIFIED (the "15 patches behind" sub-detail refuted at lock level). Cross-repo binding condition ADDED to upgrades A/B/H (strengthening, not weakening). Verifier reconciled the .db-tracking dispute. |
| Coverage | **GOOD (honest gaps)** | 5/10 dims flipped `[x]`; perf + tooling-currency left `[~]` fail-closed (no benchmark / partial currency sweep); 3 dims `[ ]` declared out-of-gate-scope. No silent capping. |
| Human walls | **NONE** | No owner authorization required; no irreversible external action. Read-only on target code; zero edits to any sibling branch. |

### Wins (evidence)

- **4th parallel-isolation proof — AND first cross-loop isolation proof.** Ran through
  `plan-loop-parallel-run.md` in an isolated worktree with ZERO edits to the union loop branch, the
  grit/weave branches, OR prompt_hub's OWN concurrent `plan/meta-arch-integration-loop` (envctl's
  separate loop running in the same target repo). First demonstration that two loops can run
  concurrently in one target repo without interference.
- **Weave A2A round-trip correction.** The cycle found a correction to the merged prompt_hub#182
  front-door plan — its "(ADR-0007)" citation mis-resolves (local ADR-0007 is "Plugin System"; the
  goal-artifact is prose-not-code) — to be fed back to envctl via weave, continuing the cross-loop
  A2A round-trip opened in cycle 5. Evidence: `findings/prompt-architecture-prompt-hub.md` §4;
  `findings/verdicts.md` VERDICT 2.
- **No over-claim under pressure.** Headline load-bearing claim (goal-artifact contract is doc-only)
  CONFIRMED by zero-hit grep across all 3 members' `src/` + a 7-test RED dump of the actual emitted
  keys; the staleness trend correctly down-graded to QUALIFIED rather than asserted.
- **Cross-repo contract not guessed.** The `GoalArtifact` envelope (upgrades A/B/H) was gated on
  rusty-idd's ACTUAL consumer schema, not the analyst's plausible-but-unbound field guess
  (`verdicts.md` upgrade-feasibility gate, CONDITION on A).

### Frictions (evidence → routing)

1. **Sub-agent output-channel ambiguity [NEW class].** The prompt-architecture auditor RETURNED its
   findings as text instead of writing `findings/prompt-architecture-prompt-hub.md` (misread a "return
   directly" directive); the orchestrator had to materialize the file to satisfy the gate. See the
   file's own header note (lines 5-6). → one-line clarity fix to the auditor brief (PROPOSED).
2. **Vendor-pollutes-code-index [RECURRENCE].** git-kb code index resolved 0 call edges on the
   vendored tree (1.42M of 1.43M symbols were `vendor/`); had to re-index each member `src/` in
   isolation. Recurrence of a prior-cycle hazard → apply-eligible note to the cartographer indexing
   step.
3. **Cross-auditor .db-tracking disagreement [benign].** Governance + filesystem auditors disagreed on
   `.db` tracking — both correct about different files (`prompt-hub/{prompthub.db,test.db}` tracked;
   root `prompthub.db` gitignored). The verifier reconciled it (`verdicts.md` VERDICT 4). This is the
   gate working as designed (capability win), not a defect.

### Net

A high-quality cycle: the gate did its job in both directions, isolation held against a concurrent
loop in the same repo, and the one genuinely-new friction (auditor output channel) plus one recurrence
(vendor index pollution) route to small, in-scope harness fixes. No gate weakened; one apply-eligible
item (vendor index), the rest proposed. No work lost; no human wall.
