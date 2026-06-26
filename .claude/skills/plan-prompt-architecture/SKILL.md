---
name: plan-prompt-architecture
description: >-
  Review prompt/tool/model/instruction surfaces as architecture. Produces prompt-architecture findings
  for each planning target: instruction sources, tool grants, model lanes, hidden couplings,
  governance controls, ADR candidates, and no-ADR rationale.
---

# plan-prompt-architecture — prompt/tool/model coupling review

AI coding agents make architecture choices through prompts, granted tools, model lanes, and runtime
instructions. This skill makes those choices visible before they become hidden infrastructure.

## Required artifact

Write `.handoff/loop/plan/findings/prompt-architecture-<T>.md` for every planned target.

Required sections:

1. **Instruction surfaces** — AGENTS.md, CLAUDE.md, `.codex/prompts/*`, `.agents/skills/*`, `.claude/*`,
   `.codex/config.toml`, hooks, policies, and any PromptHub source.
2. **Tools granted** — MCPs, local scripts, web/search, GitHub, weave, GitKB, filesystem scope,
   destructive/mutating capabilities, and read-only guarantees.
3. **Model lanes** — foreground model, background Opus/weave lanes, fallback/blocked behavior, effort
   level, and no-downgrade rule.
4. **Hidden architectural couplings** — prompt wording or tool grants that imply infrastructure,
   state storage, background execution, auth, networking, data residency, or file layout.
5. **Governance controls** — hooks, artifact gates, risk policy, human-in-the-loop boundaries,
   source ledgers, and verification evidence.
6. **ADR candidates / no-ADR rationale** — every new runtime/tool/model/backend coupling is either a
   draft ADR candidate or explicitly marked `no-ADR: <reason>`.

## Row formats

```markdown
- CLAIM: <prompt/tool/model coupling> | evidence: <path:line> | confidence: high|medium|low
- UPGRADE: <governed change> | axis: prompt-architecture | rationale: <why> | evidence: <path:line> | blast: <scope> | risk: low|med|high
- ADR-CANDIDATE: <decision> | reason: <why architectural>
- NO-ADR: <surface> | reason: <why routine/non-architectural>
```

## Gate

A plan that changes or relies on agent instructions, model routing, backend isolation, tool grants,
or prompt-derived infrastructure cannot be marked fully planned unless this artifact exists and is
lifted into `reports/<T>-plan.md`.
