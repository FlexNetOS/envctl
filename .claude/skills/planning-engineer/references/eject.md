# Eject the planning-engineer harness into a target repo

`bash scripts/eject.sh <target-repo-dir>` installs the harness into a target repo's `.claude/` and
scaffolds the durable-state tree. It is **copy + scaffold only** — and the harness is read-only on the
target's production code, so ejecting cannot change the target's behavior.

## What it copies
- **Skills** → `<target>/.claude/skills/`: the 5 planning skills (`planning-engineer`, `plan-loop`,
  `plan-cartography`, `plan-trend-research`, `plan-synthesis`) + the reused `code-research-verify`
  (the plan-verifier's refute discipline) + the shared loop infra (`session-relay-wrap-up`,
  `session-relay-resume`, `harness-loop-init`, `harness-evolution`).
- **Agents** → `<target>/.claude/agents/`: the 5 specialists (`plan-cartographer`,
  `plan-trend-researcher`, `plan-analyst`, `plan-verifier`, `plan-architect`) + the shared
  `continuity-steward`, `evolution-steward`.
- **State** → scaffolds `<target>/.handoff/loop/plan/{graph,research,findings,reports}/`.

## Apply yourself (repo-specific — not edited for you)
```gitignore
# .gitignore
.claude/*
!.claude/agents/
!.claude/skills/
.handoff/loop/plan/*.log
.handoff/loop/plan/ralph-run-*.log
```
```markdown
# CLAUDE.md pointer
## Harness: Planning Engineer
**Goal:** continuous, evidence-backed planning/architecture — code graph + 90-day research →
adversarially-verified gaps → a plan with ASCII diagrams, quality/speed/accuracy/governance+settings+config upgrades, tool-eval.
**Trigger:** for "plan <subsystem>", "architecture plan", "deep planning", "loop on the architecture",
or "resume the planning loop", use the `plan-loop` (continuous) / `planning-engineer` (single-cycle)
skill. Read-only on production code; writes plans/graph under `.handoff/loop/plan/` + docs.
```
Invoke as `/planning-engineer` (single cycle) or `/plan-loop` (continuous). The external read-only
runner is `scripts/ralph-plan.sh`.
