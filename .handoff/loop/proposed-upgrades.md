# Proposed harness upgrades (evolution-steward escalations)

> DRAINED — empty body means no undrained proposals. The evolution-steward (Phase E) appends
> ESCALATED structural proposals here; `session-relay-wrap-up` step 3b drains every entry to a
> tracked disposition (open → `- [?]` backlog item; addressed → recorded resolved; declined →
> recorded) and resets this file. A **non-empty body at end of wrap-up means wrap-up is INCOMPLETE.**
>
> Last drained: 2026-06-18 (TASK-0042) — P1 merge-driver test RESOLVED, P2 reaper test RESOLVED,
> P3 scheduled-reaper DECLINED (accept loop-boundary-only). See backlog Epic G / TASK-0042.

---

## plan-loop / fleet-convergence — cycle 6 (prompt-hub) · 2026-06-27

Escalated by the evolution-steward (harness-evolution Phase E). Propose-by-default; gates only ever
strengthened. The plan-loop shares envctl's `.handoff/loop/` so its escalations land here; drain
these alongside the forge-loop entries.

### Apply-eligible (2nd+ recurrence — low-risk, in-scope)

- [ ] **P1 · Scope git-kb code index to member `src/`, exclude `vendor/`** — RECURRENCE (cycle 6
  again: 1.42M/1.43M symbols were `vendor/`; vendored-root index resolved ~0 call edges).
  Fix: plan-cartographer indexing step indexes each Cargo-member `src/` (or passes a `vendor/`
  exclude) rather than the repo root. In-scope, additive to the cartographer skill — apply via PR.
  Evidence: cycle-6 friction #2; `plan/LESSONS.md` row 2.

### Proposed structural / clarity (owner approval)

- [ ] **P2 · Auditor output-channel clarity [NEW]** — add a one-line directive to the
  plan-architecture-auditor (and, by pattern, every dimension auditor) brief: "WRITE your findings to
  `findings/<dimension>-<target>.md` (do not return them as text); the message you return is a short
  summary, not the deliverable." Root cause of cycle-6 friction #1 (auditor returned text; orchestrator
  had to materialize the file). Touches an agent brief → propose. Evidence: `plan/LESSONS.md` row 1;
  `findings/prompt-architecture-prompt-hub.md` header note.
- [ ] **P3 · Affirm verifier dispute-reconciliation as a named duty** — make explicit in the
  plan-verifier brief that when auditors disagree on a fact, the verifier enumerates the actual
  artifacts and reconciles (both may be right about different files) rather than picking a winner.
  Captures the cycle-6 capability win (.db-tracking reconciliation). Evidence: `plan/LESSONS.md` row 3.
- [ ] **P4 · Cross-repo-schema-binding gate language** — state in the plan-verifier feasibility gate
  + plan-analyst upgrade-authoring guidance that any upgrade encoding a cross-repo contract must be
  authored as bound (to the consumer's real schema) or explicitly conditional; a guessed wire format
  that passes a self-authored RED test still pins a fiction. STRENGTHENS the gate. Evidence:
  `verdicts.md` CONDITION on upgrades A/B/H; `plan/LESSONS.md` row 4.

### Carried forward (open from cycles 4-5 — still undrained)

- [ ] **C1 · Relax the loop slug/regex** (cycles 4-5) — the target/loop-name slug regex rejected
  valid targets; relax to admit the fleet targets without weakening any guard. (structural — propose)
- [ ] **C2 · `runs/<target>/` namespacing** (cycles 4-5) — namespace per-cycle run artifacts under
  `runs/<target>/` so parallel instances don't collide in shared `.handoff/loop/`. (structural — propose)
- [ ] **C3 · Phantom-workspace check** (cycles 4-5) — add a pre-cycle check that the claimed worktree
  actually exists / is the intended one before executing. (structural — propose)
- [ ] **C4 · Oblique sentinel-token rule** (cycles 4-5) — generalize the no-literal-sentinel rule
  (the unfinished-work and unsupported-claim marker words the gate already forbids) to
  oblique/spaced/obfuscated variants in the gate's forbidden-token scan. STRENGTHENS the gate —
  propose. (Note: cycle-6 parallel-isolation success —
  4th proof, 1st cross-loop proof — is partial evidence that C2/C3 are working in practice and could
  be promoted once formalized.)
