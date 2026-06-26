# Agentic planning / architecture / engineering research — June 2026

Date: 2026-06-26  
Scope: envctl `planning-engineer` / `plan-loop` harness, with emphasis on autonomous planning, architecture governance, background agents, subagents, graph intelligence, and strict-upgrade-only execution.

## Current envctl baseline verified from source

The recovered planning loop already has several strong elements:

- Five required background-agent lanes: code graph, web trends, governance, settings/config+filesystem-layout, and rusty-idd north-star.
- Codex-to-Opus routing is fail-closed through weave rather than pinning unsupported Anthropic model slugs in Codex agent TOML.
- GitKB is the code-graph substrate. Current `git-kb code doctor --json` reports 3,843 Rust symbols, 8,830 call edges, 144 Rust files, and 24,023 unresolved calls.
- The loop now includes a first-class filesystem-layout axis using FHS/XDG, repo-native Cargo layout, and envctl/meta placement boundaries.
- Contract tests cover prompt front doors, ejected skill/agent mirrors, weave dispatch shape, and the new filesystem-layout skill/agent presence.

## External references reviewed

| Reference | What matters for envctl |
|---|---|
| OpenAI Codex subagents docs | Subagents are explicitly for parallel codebase exploration and multi-step feature planning; they are opt-in and cost more tokens, so envctl's lane launch should be deliberate and observable. |
| GitHub Copilot cloud agent docs | Background agents research, plan, branch, test, and expose logs/PRs. GitHub emphasizes tracked branches/logs over ephemeral IDE decisions. |
| OpenHands | Agent Canvas is a control center that can run OpenHands, Claude Code, Codex, Gemini, or ACP-compatible agents across local/remote/cloud backends. This validates envctl's need for a provider-neutral control plane rather than one CLI-specific loop. |
| Open SWE | Modern async coding agents converge on isolated sandboxes, subagents, middleware, Slack/Linear/GitHub invocation, automatic PR creation, and customizable orchestration over an upstream harness. |
| mini-swe-agent | Strong benchmark results can come from a very small harness; envctl should avoid complexity without gates/evals proving the added structure is paying for itself. |
| Microsoft Agent Framework | Production agent frameworks expose graph-based orchestration, sequential/concurrent/handoff/group-collab patterns, durability, restartability, observability, governance, and human-in-the-loop control. |
| Task-Decoupled Planning (TDP), Jan 2026 | Long-horizon planning should decompose work into a dependency graph, use node-scoped context, schedule ready nodes topologically, and self-revise only affected downstream specs. |
| Code as Agent Harness, May 2026 | The harness itself should be executable, verifiable, stateful, and regression-tested; evaluation should go beyond final task success and include shared-state consistency and safe multi-agent coordination. |
| Agentic AI in SDLC, Apr 2026 | The field is moving from code completion to delegated execution under human supervision; open problems are evaluation, governance, technical debt, skill redistribution, and attention economics. |
| Architecture Without Architects, Apr 2026 | Agent prompts cause architectural choices. The planning loop must surface prompt-architecture coupling as an explicit review artifact, not bury it in prose. |
| FHS 3.0 / XDG Base Directory | File/folder placement should be a standards-checked planning axis, now partially addressed by `plan-filesystem-layout`. |

## Gap hunt: what envctl still misses

### P0 — Missing runtime artifact validator for plans

The contract is documented and some prompt/eject invariants are tested, but there is no standalone gate that validates a real planning run's output directory before a target can be marked done.

Required upgrade:
- Add `scripts/plan-artifact-gate.sh`.
- Validate required artifacts exist for each target: graph JSON, graph markdown, diff, codemap, trends, governance findings, filesystem-layout findings, test strategy, verdicts, plan, tool-eval, diagrams, evolution scorecard, and resume pointer.
- Validate schema markers: `CLAIM`, `UPGRADE`, `VERDICT`, `CONFIRMED|QUALIFIED|REFUTED|INCONCLUSIVE`, axis tags, cited evidence, and no `TODO`/placeholder evidence in promoted recommendations.
- Validate `DONE` cannot exist unless every target/dimension row is terminal and the pre-DONE completeness sweep is recorded.

Why latest research supports it: Code-as-harness work emphasizes verifiable/stateful harnesses; GitHub/Open SWE workflows rely on tracked logs/branches rather than invisible decisions.

### P0 — No TDP-style dependency graph for targets/dimensions

The loop has `targets.md` and `dimensions.md`, but scheduling is still mostly linear. It does not model dependencies between planning targets/dimensions, nor does it confine recovery/replanning to the affected subgraph.

Required upgrade:
- Add `plan-dependency-graph` skill/agent.
- Produce `.handoff/loop/plan/graph/target-dag.json` and `target-dag.md`.
- Pick by topological ready-set, not first unchecked line.
- Add `SELF-REVISION` rows when a verifier refutes a claim or when a target changes upstream assumptions.
- Replan only downstream nodes impacted by the changed finding.

### P0 — Prompt-architecture coupling is not explicit enough

The harness reviews architecture decisions, but not every prompt/tool/model/instruction change is treated as an architectural decision. June 2026 research warns that prompt wording and tool access can select infrastructure and architecture.

Required upgrade:
- Add `prompt-architecture-review` axis.
- For each plan, output `.handoff/loop/plan/findings/prompt-architecture-<T>.md` with: instruction surfaces, tools granted, model lanes, hidden architectural couplings, governance controls, and ADR candidates.
- Gate any new subagent/tool/model/runtime capability with an ADR or explicit no-ADR rationale.

### P1 — Background-agent observability is too thin

`weave-dispatch/<run-id>.jsonl` proves lane dispatch shape, but not enough to debug or evaluate agent performance.

Required upgrade:
- Add agent run spans: lane, peer/session/job id, model, effort, start/end, input artifact hashes, output artifact paths, token/cost if available, retries, tool failures, verdict, and upstream/downstream dependency ids.
- Add a consolidated `reports/agent-run-ledger-<T>.md`.
- Add a status reporter that lets foreground chat query background progress without interrupting workers.

### P1 — No sandbox/backend abstraction for planning/forge workers

OpenHands/Open SWE/GitHub cloud agent converge on isolated execution environments and backend-neutral agent runners. envctl has worktrees and weave, but no explicit backend interface for local shell vs container vs remote VM vs cloud/ACP agent.

Required upgrade:
- Add `agent-backend-matrix.md` and `plan-agent-backend` config.
- Classify each lane as `read-only-local`, `isolated-worktree`, `container`, `remote-vm`, `cloud-agent`, or `ACP/A2A`.
- Fail closed if a lane requiring isolation is run in the foreground checkout.

### P1 — No golden eval suite for planning quality

The contract tests check files/instructions, not whether the loop actually produces better plans. Modern agent repos benchmark harnesses; mini-swe-agent is a warning that added complexity must be earned.

Required upgrade:
- Add `scripts/tests/test-plan-evals.sh` with small frozen fixtures:
  - missing filesystem-layout artifact should fail;
  - unverified claim promoted to roadmap should fail;
  - empty GitKB entrypoints must be marked `INCONCLUSIVE`, not pass;
  - prompt-architecture coupling without review should fail;
  - TDP downstream self-revision fixture should pass.
- Store fixture outputs under `.handoff/loop/plan/evals/fixtures/` or `scripts/tests/fixtures/plan/`.

### P1 — Human-in-the-loop policy is implicit

Planning is read-only, but the plan can hand high-risk work to Feature Forge. Microsoft/GitHub production patterns put HITL approval at risk boundaries.

Required upgrade:
- Add a `risk_policy` table to plan output.
- Force `SUPERVISED` markers for destructive ops, trust-boundary dependency changes, credential/secrets surfaces, filesystem migrations outside repo scope, and provider/model changes.
- Validate risk-to-approval routing in `plan-artifact-gate.sh`.

### P2 — Standards/source ledger needs reproducibility

The researcher cites sources, but there is no machine-readable source ledger with dates, recency status, and confidence.

Required upgrade:
- Add `.handoff/loop/plan/research/sources-<T>.jsonl` with URL, title, publisher, accessed_at, published_at, in_recency_window, why_used, and claim ids.
- Have `plan-trend-researcher` write this before synthesis.

### P2 — Interop should recognize ACP/A2A/MCP, not only weave

OpenHands advertises ACP-compatible agents and Microsoft is moving toward interop patterns around hosted agents/tool protocols. envctl's local substrate is weave, but the planning loop should name interop boundaries.

Required upgrade:
- Add `agent_interop` section to loop state: `weave`, `mcp`, `ACP`, `A2A`, `GitHub cloud agent` availability and routing decision.
- Keep weave as the current required route for Opus lanes; add adapters as strict upgrades only.

## Recommended upgrade queue

1. **P0: plan-artifact-gate** — runtime gate for actual output completeness and schema validity.
2. **P0: plan-dependency-graph/TDP** — target/dimension DAG, ready-set scheduling, localized self-revision.
3. **P0: prompt-architecture-review** — make prompt/tool/model induced architecture explicit and ADR-governed.
4. **P1: agent-run-ledger + progress reporter** — observable background lane spans and foreground status query.
5. **P1: backend isolation matrix** — codify when worktree/container/remote/cloud isolation is required.
6. **P1: planning eval fixtures** — prove harness quality, not just file presence.
7. **P1: risk-policy/HITL gate** — map upgrades to owner approval surfaces before Feature Forge executes.
8. **P2: source-ledger JSONL** — reproducible web research inputs.
9. **P2: ACP/A2A/MCP interop registry** — strict-upgrade route beyond weave without weakening current Opus routing.

## Source links

- OpenAI Codex subagents: https://developers.openai.com/codex/subagents
- GitHub Copilot cloud agent: https://docs.github.com/en/copilot/concepts/agents/cloud-agent/about-cloud-agent
- OpenHands: https://github.com/OpenHands/openhands
- Open SWE: https://github.com/langchain-ai/open-swe
- mini-swe-agent: https://github.com/SWE-agent/mini-swe-agent
- Microsoft Agent Framework: https://github.com/microsoft/agent-framework
- TDP: https://arxiv.org/abs/2601.07577
- Code as Agent Harness: https://arxiv.org/abs/2605.18747
- Agentic AI in SDLC: https://arxiv.org/abs/2604.26275
- Architecture Without Architects: https://arxiv.org/abs/2604.04990
- FHS 3.0: https://specifications.freedesktop.org/fhs/latest
- XDG Base Directory: https://specifications.freedesktop.org/basedir/
