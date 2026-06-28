# Plan-loop evaluation — fleet-convergence planning loop (per-cycle self-eval / scorecard)

Plan-loop (planning-engineer harness) evolution scorecard. One `## <target>` section per cycle,
appended at each cycle boundary by the evolution-steward (harness-evolution method). This is the
plan-loop's per-cycle scratch; the durable cross-run memory is the root `LESSONS.md`.

## grit

Cycle 5 · TARGET = grit (symbol-level merge/lock substrate for the union) · dated 2026-06-27.
Run: plan-grit-20260627, isolated worktree `plan/loop-grit` off `origin/master`. Crew: 12 agents
(cartographer, trend-researcher, dependency-graph-auditor, 8 axis auditors + analyst, test-strategist)
→ verifier (GATE) → architect. Self-eval grades below.

### Scorecard

| axis | grade | basis |
|------|-------|-------|
| Friction | LOW-MED | 3 frictions, all caught & worked-around in-cycle; none bounced an item backward or corrupted the plan. |
| Gate quality | HIGH | Verifier: 12 CONFIRMED / 1 QUALIFIED / 1 INCONCLUSIVE / 0 REFUTED material claims; PASS. No wrong claim slipped past; no sound upgrade false-refuted. |
| Coverage | QUALIFIED-COMPLETE | grit marked `[~]` planned-with-gaps: 7/10 dims verified; 3 left `[~]` for a stated, non-silent reason (unverifiable without cloud creds / not line-exhaustive). |
| Human walls | NONE | No owner-authz wall hit this cycle; the 3 unverifiable dims are credential/scope gaps, not approval gates. |

### Gate quality — detail (did anything slip / false-block?)

- **No wrong claim slipped.** The 12 CONFIRMED material claims each cite source (e.g. line-level
  `git merge --no-ff` at `src/git/mod.rs:221-253`; the per-symbol `Symbol.hash` computed at
  `parser/mod.rs:329` but NEVER read for merge/dedup — both adversarially confirmed).
- **No sound upgrade false-refuted — and a correct REFUTE landed.** The verifier REFUTED exactly one
  framing on feasibility: "grit as-is = the in-boundary (no-C) union engine" — REFUTED because grit's
  C substrate (rusqlite-bundled + tree-sitter) violates the envctl no-C trust-boundary invariant. The
  underlying capability (dedup/reconcile) was still passed through as feasible *outside* the boundary.
  That is the gate doing its job: rejecting a wrong premise while preserving the sound core. Net
  union-fitness verdict: grit is **UNFIT as-is** for union step 2 (advisory symbol-lock + line-level
  git coordinator; reconcile is RED-proven absent) but usable as the coordination substrate around a
  pure-Rust dedup engine that must be built.
- **Gate strengthened, never weakened.** Every governance upgrade row STRENGTHENS a gate (AGENTS.md
  hard-rules, destructive-guard rules, MSRV/toolchain pin, clippy `--all-targets` + audit/deny,
  deny-unknown `enum Backend`, 0600 secret perms); none weakens one. The phantom-workspace
  remediation was itself verified feasible (a prerequisite for grit standalone build/CI).

### Friction — detail (3 items, all in-cycle-recovered)

1. **Graph-before-analysis ordering.** The analyst/axis auditors ran before the cartographer's graph
   JSON was materialized; the analyst correctly *declared* the gap rather than fabricating a query
   (`findings/architecture-grit.md:10` — "noted gap, not a fabricated graph query"). No false data
   entered the plan, but it cost analytic depth on graph-derived metrics. → ordering lesson.
2. **Phantom-workspace wall.** A stray `meta/.worktrees/Cargo.toml` made Cargo walk up and pulled
   grit into a foreign virtual workspace, blocking standalone `cargo build`. Recurs across the fleet
   (same sibling-escape class as handoff A-U1). → environment-hazard lesson (2nd recurrence).
3. **Self-referential sentinel-token trap.** Several auditors had to be explicitly warned that
   writing the completeness-gate's own rejection sentinel words verbatim (the placeholder/uncertainty
   vocabulary the gate rejects case-insensitively) would trip the gate on their own findings — recur
   from cycle 4. → gate-trap lesson (2nd recurrence). Mitigation: reference the class obliquely.

### Scorecard win — live cross-loop convergence over weave (NEW capability)

A weave A2A round-trip happened **mid-cycle**: this envctl session asked rusty-idd (over weave) to
verify the lifeos-meta-front-door plan; rusty-idd replied with 3 corrections (D3 two-layer front
door, D1 union binding, the grit-unfit caveat — consistent with this cycle's own UNFIT verdict);
envctl folded all 3 and shipped prompt_hub PR #182. This is the cycle-4 "transport plane" finding
working **end-to-end**: the planning loops are now talking to each other via weave, not just leaving
async handoffs. First demonstrated occurrence of in-cycle A2A coordination → capability lesson.

### Parallel-isolation proof (3rd consecutive)

The whole cycle ran via `prompt_hub/prompts/plan-loop-parallel-run.md` in an isolated worktree off
`origin/master` with ZERO edits to the union loop branch or the weave branch — parallel isolation
proven a 3rd time. No cross-branch contamination; the convergence is additive and reviewable.

### Net

A clean, gate-honest cycle: a correct REFUTE on a wrong union-engine framing, a transparent `[~]`
coverage stance (gaps named, not capped), and the first live weave A2A round-trip between loops. Two
classes hit their 2nd recurrence (phantom-workspace, sentinel-token trap) → flagged apply-eligible.
No gate weakened; all structural items routed to `proposed-upgrades.md` for owner approval.
