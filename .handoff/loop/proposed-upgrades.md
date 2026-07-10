# Proposed harness upgrades (evolution-steward escalations)

> DRAINED — empty body means no undrained proposals. The evolution-steward (Phase E) appends
> ESCALATED structural proposals here; `session-relay-wrap-up` step 3b drains every entry to a
> tracked disposition (open → `- [?]` backlog item; addressed → recorded resolved; declined →
> recorded) and resets this file. A **non-empty body at end of wrap-up means wrap-up is INCOMPLETE.**
>
> Last drained: 2026-06-28 (batch boundary 54) — plan-loop / fleet-convergence cycle 6 proposals
> drained to backlog TASK-0079 through TASK-0086.

---

## 2026-07-09 — batch boundary 65 (Epic I blueprint R2–R10) — ESCALATED, undrained

**P1 — Queue-aware merge shepherd for the limited-runner fleet (forge-loop; STRUCTURAL).**
The "BEHIND treadmill" (LESSONS 2026-07-09) is mitigated today by hand-serialized update-branch
chains. Propose a queue-aware shepherd in `forge-loop` that, at each merge event, updates exactly
ONE armed PR's branch → waits for its required check → merges → advances, instead of re-queueing
all armed PRs (which re-BEHINDs them under strict up-to-date protection and saturates the 2
runners). Orthogonal capacity lever: raise runner slots (>2) — an infra decision for the owner.
Do NOT weaken the strict up-to-date gate. Evidence: Epic I R2–R10, 2-runner fleet serialized all
org CI. Risk: touches merge orchestration → owner approval before apply.

**P2 — Mechanical post-arm worktree-edit guard (forge-loop; STRUCTURAL).**
The 2026-06-23 armed-PR-worktree hazard (LESSONS row 27) recurred as a near-miss (LESSONS
2026-07-09, recurrence 2). The documented note held but is not mechanically enforced. Propose a
fail-closed pre-edit guard that REFUSES edits in a worktree whose branch has an armed OPEN PR
(e.g. check `gh pr view <branch> --json state,autoMergeRequest`; if armed+open, block and direct
the edit to a fresh worktree off develop). STRENGTHENS the existing discipline — never weakens a
gate. Evidence: R10 semantic.rs first written into the armed R8 worktree, caught manually
pre-commit. Risk: adds a guard to the loop's edit path → owner approval before apply.
