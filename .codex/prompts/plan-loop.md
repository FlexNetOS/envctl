---
description: Run or resume the continuous Planning Engineer Ralph loop.
argument-hint: "[resume|budget=N|target=<slug>]"
---

You are executing the Codex-native prompt mirror for the envctl Planning Engineer loop.

Use `.agents/skills/plan-loop/SKILL.md` as the authoritative workflow and align it with
`/home/drdave/Desktop/meta/prompt_hub/prompts/planning-engineer-loop.prompt.yml`. Also read
`.agents/skills/planning-engineer/SKILL.md`, `AGENTS.md`, `.codex/AGENTS.md`, and
`/home/drdave/Desktop/meta/.meta.yaml` before acting.

Arguments supplied to this prompt: $ARGUMENTS

Required behavior:
- Run the continuous planning loop over `.handoff/loop/plan/targets.md`; do not treat this as a
  one-off chat proposal.
- On `resume`, reconstruct state from `.handoff/loop/plan/loop_state.md`, the latest plan reports,
  ICM recall, and any committed handoff packet before choosing work.
- Each iteration launches the 5× Opus 4.8 max-effort background-agent lanes via weave (code graph, web
  trends, governance, settings/config, rusty-idd north-star) so the foreground remains interactive,
  then runs analysis, test strategy, verification, synthesis, and self-eval.
- If no backlog exists, seed the first fleet target as `rusty-idd` when present under meta and record
  the meta↔envctl↔prompt_hub relationship in the planning artifacts.
- Keep production code read-only except for explicitly additive RED test evidence required by the
  planning contract.
- At budget or batch boundary, run the session-relay/wrap-up path instead of leaving state in chat.
- Commit, push, open a PR, and arm auto-merge for any coherent committed repo change.

Filesystem-layout requirement: every planning cycle must map file/folder organization against FHS/XDG, repo-native Cargo layout, and envctl/meta placement invariants, emitting `findings/filesystem-layout-<T>.md`.

P0-P2 upgrade contract: before DONE, the loop must produce and gate `graph/target-dag.{json,md}`, `findings/prompt-architecture-<T>.md`, `reports/agent-run-ledger-<T>.md`, `risk-policy.md`, `agent-backend-matrix.md`, `agent-interop.md`, `research/sources-<T>.jsonl`, and must run `scripts/plan-artifact-gate.sh .handoff/loop/plan`. Use TDP topological ready-set scheduling and SELF-REVISION for localized replans; keep weave as the current Opus transport while recording ACP/A2A/MCP/GitHub-cloud interop as strict-upgrade options only.

Critical architecture-loop contract: every plan must cover persistent memory/vector intelligence, aggressive code+web auto-research, Upgrade Only/No Downgrades policy, automation-first agent org chart, weave/A2A/MCP communication, background-agent execution, Rust+Lua runtime strategy, distributed compute across workstation/mobile/AI glasses/Pi Zero/ESP32, and a multi-vendor local+cloud mesh. Required artifacts: `findings/memory-vector-intelligence-<T>.md`, `findings/autoresearch-<T>.md`, `findings/rules-policy-org-<T>.md`, and `findings/distributed-compute-<T>.md`.
