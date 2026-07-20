---
description: Run one evidence-backed Planning Engineer architecture cycle.
argument-hint: "<planning-target>"
---

You are executing the Codex-native prompt mirror for the envctl Planning Engineer harness.

Use `.agents/skills/planning-engineer/SKILL.md` as the authoritative workflow and align it with
`/home/flexnetos/FlexNetOS/prompt_hub/prompts/planning-engineer-loop.prompt.yml`. Also read
`AGENTS.md`, `.codex/AGENTS.md`, `/home/flexnetos/FlexNetOS/.meta.yaml`, `.agents/skills/plan-cartography/SKILL.md`,
`.agents/skills/plan-trend-research/SKILL.md`, `.agents/skills/plan-governance-config/SKILL.md`,
`.agents/skills/plan-test-strategy/SKILL.md`, and `.agents/skills/plan-synthesis/SKILL.md` before
acting.

Arguments supplied to this prompt: $ARGUMENTS

Required behavior:
- Run exactly one planning cycle on the supplied target; if no target is supplied, inspect
  `.handoff/loop/plan/targets.md` and choose the first safe pending target.
- Keep production code read-only. The permitted writes are planning artifacts under
  `.handoff/loop/plan/`, docs/ROADMAP or draft ADR promotion, and additive RED test evidence only
  when the planning skill explicitly requires it.
- Launch the 5× Opus 4.8 max-effort background-agent lanes via weave before reducing findings; keep the
  foreground interactive and use weave to launch/route the actual Opus workers; fail closed only if weave cannot produce an Opus-capable worker.
- Include the governance/settings/config/filesystem-layout axis, rusty-idd north-star verdict, tool-evaluation, ASCII
  diagrams, graph snapshot/diff, and TDD RED-suite evidence/counts in the final plan contract.
- Verify claims adversarially before they enter the plan; unverified or infeasible upgrades must stay
  out of the roadmap.
- Commit, push, open a PR, and arm auto-merge for any coherent committed repo change.

Filesystem-layout requirement: every planning cycle must map file/folder organization against FHS/XDG, repo-native Cargo layout, and envctl/meta placement invariants, emitting `findings/filesystem-layout-<T>.md`.

P0-P2 upgrade contract: before DONE, the loop must produce and gate `graph/target-dag.{json,md}`, `findings/prompt-architecture-<T>.md`, `reports/agent-run-ledger-<T>.md`, `risk-policy.md`, `agent-backend-matrix.md`, `agent-interop.md`, `research/sources-<T>.jsonl`, and must run `scripts/plan-artifact-gate.sh .handoff/loop/plan`. Use TDP topological ready-set scheduling and SELF-REVISION for localized replans; keep weave as the current Opus transport while recording ACP/A2A/MCP/GitHub-cloud interop as strict-upgrade options only.

Critical architecture-loop contract: every plan must cover persistent memory/vector intelligence, aggressive code+web auto-research, Upgrade Only/No Downgrades policy, automation-first agent org chart, weave/A2A/MCP communication, background-agent execution, Rust+Lua runtime strategy, distributed compute across workstation/mobile/AI glasses/Pi Zero/ESP32, and a multi-vendor local+cloud mesh. Required artifacts: `findings/memory-vector-intelligence-<T>.md`, `findings/autoresearch-<T>.md`, `findings/rules-policy-org-<T>.md`, and `findings/distributed-compute-<T>.md`.
