---
name: planning-engineer
description: >-
  Packaged planning/architecture harness (invoked as /harness:planning-engineer). Runs ONE deep
  planning cycle on a target subsystem/crate/feature-area: deep web research (best-practices + latest
  trends, last 3 months) + a repo scan with symbol & data-flow mapping + a persistent CODE GRAPH that
  yields graph-based intelligence → adversarially-verified gap analysis → a decision-grade plan with
  ASCII architecture diagrams, code-quality/speed/accuracy/governance+settings+config upgrades, TDD RED-suite evidence, and tool-evaluation. ALWAYS use
  for: "plan <subsystem>", "architecture plan", "deep planning", "design the architecture of <X>",
  "what should we upgrade in <X>", AND follow-ups — "re-run", "run it again", "update the plan",
  "revise", "redo only the <dimension>", "dig deeper on <X>", "based on the previous plan". For a
  CONTINUOUS loop over many targets use `plan-loop`; for cross-session handoff use session-relay-*.
  Read-only on the target's code; writes only plans/graph under .handoff/loop/plan/ + docs.
---

# planning-engineer — deep planning & architecture cycle  (`/harness:planning-engineer`)

Leader skill of the **planning-engineer** packaged harness. It turns one *planning target* (a crate /
subsystem / feature-area) into a **decision-grade plan**: research the field → map the code into a
graph → analyze gaps & design upgrades → **try to refute every claim and feasibility-gate every
upgrade against the source** → synthesize a plan with ASCII diagrams, a sequenced
quality/speed/accuracy/governance+settings+config upgrade roadmap, and a tool-evaluation. The defining property (inherited from
`code-research`): conclusions and recommendations are *earned by evidence that survived refutation* —
so the plan is safe to act on. **Read-only** on the target's production code; the only writes are the
plan/graph artifacts under `.handoff/loop/plan/`, additive RED test suites (the one permitted mutation), and (architect only) the docs/ROADMAP+ADR promotion.

For a CONTINUOUS run over a backlog of targets, the `plan-loop` skill wraps this one cycle in the
Ralph loop. This skill IS one cycle.

## Standing laws (non-negotiable, every cycle)

These bind every phase and every agent in this harness; a plan or upgrade that violates one is a
fail-closed finding, never a recommendation:
- **Fail-closed.** Absence of failure is NOT proof of success — a green exit / empty result / missing
  file is a finding to investigate, never a pass. Every claim cites positive evidence
  (`file:line` / a graph-query row / a dated URL).
- **Owner walls → NEEDS-HUMAN.** Physical / account / irreversible / scope-expanding actions are
  surfaced, never silently performed. Research + planning are read-only and autonomous; mutating the
  fleet is not part of this loop. The **one permitted mutation** is authoring **additive RED test
  suites** (P8) — tests only ADD verification; they never change product code or weaken a gate.
- **TDD-native / falsifiable.** Every plan item is expressed as a *failing test* before it counts; an
  item with no test that can fail is itself a finding. "Done" = a GREEN suite (`tests-ran > 0`), never
  prose.
- **Latest-toolchain standing rule + owner corrections.** JS tooling is **bun**, never pnpm/node;
  **shimmy + ruvllm** are the official ollama replacement but **do NOT remove ollama until swap-out is
  parity-proven**; **clang/llvm-21** is load-bearing. The tool-eval (R7) and the config-drift detector
  enforce currency against this rule.
- **Evidence over vibes.** Diagrams and gaps derive from the code graph and cited sources, not memory.

## Source-of-truth intent and meta/envctl relationship

This ejected harness must stay aligned with the owner prompt in
`/home/drdave/Desktop/meta/prompt_hub/prompts/planning-engineer-loop.prompt.yml`. Treat that
PromptHub file as the upstream **north-star intent** and `harness_hub` as the reusable packaged
source-of-truth for this planning-engineer harness. The envctl `.claude/`/`.agents/` trees are
ejected, runnable, tested mirrors of that package. When they differ, preserve the stricter
requirement, patch the `harness_hub` package first, then re-eject/sync envctl; never silently
downgrade the prompt's intent.

Current relationship truth:
- `meta` is the fleet/workspace mission-control root (`/home/drdave/Desktop/meta`) and `.meta.yaml`
  is the fleet index the loop reads before deriving targets.
- `envctl` is a first-class meta peer: the pure-Rust agentic environment manager that installs tools
  into meta (`meta/.toolchains/`, `$META_ROOT`) and governs reproducible agent/runtime config.
- `prompt_hub` carries owner-authored intent prompts; this planning-engineer package is the
  operational harness that turns that intent into durable `.handoff/loop/plan/` evidence.
- The first fleet loop run is expected to surface **`rusty-idd`** as the priority planning target and
  the real upgrade path into the Forge/IDD loop; if auto-derivation misses it, record that as a
  governance/config finding and seed `rusty-idd` explicitly.

## Execution mode — Hybrid (background fan-out sub-agents + file-based), and why

Single-orchestrator with specialist sub-agents over a durable ledger — **not** a live `TeamCreate`
team: team state dies at the loop's self-restart boundary, so the truth must live in the on-disk
ledger under `.handoff/loop/plan/` (the loop resumes cold from files, never from conversation memory).
Per phase:

| Phase | Mode | Shape |
|-------|------|-------|
| 1 Five-lane fleet fan-out | Sub-agent, **parallel background** (`run_in_background:true`) | 5 Opus 4.8 max-effort lanes (below) |
| 2 Analyze | Sub-agent, **parallel** | one analyst per dimension + governance/config auditor + test strategist → cited gaps + upgrades |
| 3 Verify (gate) | Sub-agent, **parallel** | one verifier per dimension → refute claims + feasibility-gate upgrades |
| 4 Synthesize | Sub-agent, **sequential** | architect → plan + ASCII diagrams + tool-eval |
| 5 Self-eval | Sub-agent, **sequential** | evolution-steward → evaluate this cycle, queue upgrades |

**Foreground-interactive law:** the orchestrator/main chat stays available to the owner. Heavy web,
repo, governance, settings/config, and `rusty-idd`/north-star work runs only in background agents; the
foreground reduces structured artifact paths and verdicts. Do not block the owner chat with inline
fleet scans.

**Five required background-agent lanes (launch together, max effort):**
1. `plan-opus-bg-code-graph` — per-repo `git-kb code` index/symbols/callers/callees/impact/flows,
   code-graph snapshot+diff, cross-repo references.
2. `plan-opus-bg-web-trends` — rolling 90-day web/context7/Hugging Face research plus skeptic check.
3. `plan-opus-bg-governance` — rules, instructions, hooks, policy, CLAUDE.md/AGENTS.md drift.
4. `plan-opus-bg-settings-config` — `.codex`/`.claude` settings, MCP rot, skill overload, token burn,
   permissions, `.meta.yaml`, Cargo/toolchain/CI/bun/config drift, and filesystem-layout triage.
5. `plan-opus-bg-rusty-idd-north-star` — first-run `rusty-idd` surfacing, meta↔envctl↔prompt_hub
   relationship truth, owner north-star capture, Forge/IDD-loop upgrade path.

All actual lane work uses Opus-class max effort. In Claude-compatible runtimes use explicit
`claude-opus-4-8`/`anthropic/claude-opus-4-8` with `run_in_background:true`.

**Codex transport law: use weave for Opus.** Codex's ChatGPT/OpenAI provider may not resolve Anthropic
model slugs directly, so Codex must **not** try to run `model = "claude-opus-4-8"` subagents. Instead:
1. `weave attach --name envctl-plan-orchestrator` to register the foreground.
2. For each lane, `weave spawn <lane-peer> --cmd claude --cmd --model --cmd claude-opus-4-8 --cmd -p
   --cmd <lane prompt>` when a Claude CLI surface is available, or create a fenced `weave job`/`weave
   ask` routed to an already registered Opus-capable peer.
3. Prefer the packaged helper `scripts/plan-weave-dispatch.sh` (root copy) or
   `agent-skills/capability-packs/planning-engineer/scripts/plan-weave-dispatch.sh` (ejected copy) so dispatch is
   repeatable and test-gated. It writes `.handoff/loop/plan/weave-dispatch/<run-id>.jsonl` with one
   row per lane.
4. Record the weave peer/session id plus message/job id in `.handoff/loop/plan/loop_state.md` and the
   lane artifact.
5. If weave cannot produce or reach an Opus-capable worker, fail closed with a provider/transport gap;
   do not silently perform the heavy lane in Codex or any weaker model.

Data transfer is **file-based** (pass artifact PATHS, never contents) + **return-value** (each worker
returns a one-line verdict the orchestrator reduces through weave receipts/inbox/job results).



## Owner north-star architecture-loop upgrade (critical)

The planning loop is not just a repo planner; it is the architecture loop for a Rust/Lua distributed
agent fabric. Every target plan must explicitly cover these critical components:

1. **Persistent memory + vector intelligence** — ICM recall/store, `.handoff` ledger, source ledgers,
   GitKB code graph, vector/RAG indexes when present, freshness, and cold-start recall proof.
2. **Constant auto-research** — deliberately over-emphasize continuous code and web research. Refresh
   GitKB graph intelligence and current web/vendor/tool docs every cycle; stale evidence invalidates
   recommendations.
3. **Rules, policy, and agent org chart** — Upgrade Only, No Downgrades, automate everything
   researchable, replace human bottlenecks with agents where safe, explicit SUPERVISED/owner-only
   boundaries, real agent org chart, and agent-to-agent communication via weave/A2A/MCP surfaces.
4. **Always background agents** — heavy research/code mapping/policy scans run in background so the
   foreground chat remains responsive and weave messages are not missed.
5. **Rust + Lua north star** — Rust owns safe control/data planes and embedded/no_std paths; Lua/Luau
   is evaluated as the small script/policy/runtime layer where it improves deployability without
   weakening trust-boundary invariants.
6. **Distributed compute across owner hardware** — plan for workstation/GPU, local servers, mobile,
   AI glasses/wearables, Raspberry Pi/Pi Zero class Linux, ESP32/ESP32-S3 class MCU, and offline/
   degraded operation.
7. **Multi-vendor local+cloud mesh** — plans must evaluate local models plus multiple vendors/clouds
   working together (OpenAI, Anthropic/Claude via weave where used, Cloudflare, Hugging Face,
   GitHub/Copilot cloud agent, and project-local providers), with failover and data-residency policy.

Required artifacts for each target:
- `findings/memory-vector-intelligence-<T>.md`
- `findings/autoresearch-<T>.md`
- `findings/rules-policy-org-<T>.md`
- `findings/distributed-compute-<T>.md`

## June 2026 P0-P2 planning-harness upgrades (mandatory)

The research report at `.handoff/loop/plan/research/agentic-planning-trends-2026-06.md` is now part
of this harness contract. Every planning run must include these additions:

1. **P0 runtime artifact gate** — before `DONE`, run `scripts/plan-artifact-gate.sh` against the real
   `.handoff/loop/plan/` directory. The gate validates required artifacts, schema markers, verified
   verdicts, TDP state, source ledger, prompt-architecture review, risk policy, backend matrix,
   interop registry, agent-run ledger, and terminal target/dimension rows.
2. **P0 TDP target DAG** — spawn/use `plan-dependency-graph-auditor` to produce
   `graph/target-dag.json` and `graph/target-dag.md`. Pick targets from the topological ready set;
   when verification changes an upstream assumption, append `SELF-REVISION` and replan only affected
   downstream nodes.
3. **P0 prompt-architecture review** — spawn/use `plan-prompt-architecture-auditor` to produce
   `findings/prompt-architecture-<T>.md`. Treat prompt/tool/model/backend changes as architectural
   couplings until the artifact records an ADR candidate or explicit no-ADR rationale.
4. **P1 background observability** — write `reports/agent-run-ledger-<T>.md` with lane, peer/session,
   model, effort, start/end, artifact hashes/paths, retries, tool failures, and verdicts. The
   foreground orchestrator may answer owner progress questions from this ledger without interrupting
   background workers.
5. **P1 backend isolation matrix** — write `agent-backend-matrix.md`; classify each lane as
   `read-only-local`, `isolated-worktree`, `container`, `remote-vm`, `cloud-agent`, or `ACP/A2A`, and
   fail closed when a lane requiring isolation runs in the foreground checkout.
6. **P1 risk/HITL policy** — write `risk-policy.md`; mark destructive ops, trust-boundary dependency
   changes, secrets/credential surfaces, filesystem migrations outside repo scope, and provider/model
   changes as `SUPERVISED` unless an explicit verified policy says otherwise.
7. **P1 eval fixtures** — keep `scripts/tests/test-plan-evals.sh` green; these fixtures prove the loop
   rejects unverified claims, bad source ledgers, missing TDP self-revision, and missing
   prompt-architecture findings.
8. **P2 source ledger** — `plan-trend-researcher` must write `research/sources-<T>.jsonl` with URL,
   title, publisher, accessed_at, published_at, in_recency_window, why_used, and claim_ids for every
   source used by claim-bearing findings.
9. **P2 interop registry** — write `agent-interop.md` recording weave, MCP, ACP, A2A, and GitHub cloud
   agent availability/routing. Weave remains the required current Opus transport; any new adapter is a
   strict upgrade, never a downgrade or silent replacement.

## Agents (in the plugin's shared `harness/agents/` pool)

| Agent | Owns | Requirement | Shared? |
|-------|------|-------------|---------|
| `plan-cartographer` | map the target + build/diff the CODE GRAPH from `git-kb code` + derive graph intelligence (centrality/hotspots, blast-radius, dead, cycles, layering, public-API) + symbol & data-flow maps + seed dimensions + the pre-DONE completeness sweep | **R3b + R8** | specialist |
| `plan-trend-researcher` | deep WEB research — best-practices + latest trends over a rolling 90-day window, every finding cited + dated | **R3a** | specialist |
| `plan-analyst` | per-dimension analysis → cited claims, named gaps, and upgrade options each tagged **quality / speed / accuracy** | **R5** | specialist |
| `plan-governance-config-auditor` | control-plane + settings/config scan: rules/instructions/hooks/policy/CLAUDE.md/AGENTS.md, `.claude`/`.codex`, MCP rot, skill overload, token burn, permission/config drift | **prompt P2/P5/P6** | specialist |
| `plan-memory-vector-intelligence-auditor` | persistent memory + vector intelligence: ICM, `.handoff`, source ledgers, GitKB/vector/RAG freshness, cold-start recall proof | **memory-vector-intelligence** | specialist |
| `plan-autoresearch-loop-auditor` | constant code+web auto-research cadence, stale-evidence invalidation, graph/web recency refresh | **autoresearch** | specialist |
| `plan-rules-policy-org-auditor` | Upgrade Only/No Downgrades policy, automation-first rules, real agent org chart, A2A/weave communication, human-bottleneck replacement | **rules-policy-org** | specialist |
| `plan-distributed-compute-auditor` | Rust+Lua distributed compute across workstation/mobile/glasses/Pi/Pi Zero/ESP32/local+cloud vendors | **distributed-compute** | specialist |
| `plan-dependency-graph-auditor` | TDP target/dimension dependency DAG, topological ready-set scheduling, node-scoped context, localized SELF-REVISION | **P0 TDP** | specialist |
| `plan-filesystem-layout-auditor` | standard OS file/folder organization: FHS/XDG, repo-native Cargo layout, envctl/meta placement boundaries, root clutter, generated/cache/state/log/runtime placement, and enforcement-test handoff | **filesystem-layout** | specialist |
| `plan-prompt-architecture-auditor` | prompt/tool/model/runtime coupling review; ADR/no-ADR routing for prompt-induced architecture | **P0 prompt-architecture** | specialist |
| `plan-test-strategist` | the always-on **`test-coverage`** dimension: map existing tests, author additive RED tests for each accepted plan item, count-verify tests-ran > 0, emit traceability, and hand GREEN implementation to Feature Forge | **prompt P8** | specialist |
| `plan-verifier` | adversarially **refute** each claim against the source AND **feasibility-gate** each upgrade (the gate) | gate for R3/R5/R8 | specialist |
| `plan-architect` | synthesize → plan with **ASCII diagrams** + **tool-evaluation** + sequenced upgrade roadmap; promote to docs/ROADMAP + draft ADR | **R4 + R7** | specialist |
| `continuity-steward` | cold-start HANDOFF.md at budget | continuity | shared |
| `evolution-steward` | Phase 5 self-eval + fail-closed harness self-upgrade | **R6** | shared |

Skills used: `plan-cartography`, `plan-trend-research`, `plan-governance-config`,
`plan-memory-vector-intelligence`, `plan-autoresearch-loop`, `plan-rules-policy-org`, `plan-distributed-compute`, `plan-dependency-graph`, `plan-filesystem-layout`, `plan-prompt-architecture`, `plan-test-strategy`, `plan-synthesis`, the reused `code-research-verify` (the verifier's refute discipline), `session-relay-wrap-up`,
`session-relay-resume`, `harness-evolution`, `icm-memory`. (The 90-day field research applies the
deep-research *method* — fan-out search → deep-read → adversarial verify → cited synthesis —
implemented inline by `plan-trend-research`; there is no separate `deep-research` skill to load.)

## Phase 0: Context check + inputs

Decide mode (initial / **resume** via `session-relay-resume` / **partial-redo-one-dimension** /
**new** — archive prior artifacts to `.handoff/loop/plan/_done/` on a new unrelated target). The
orchestrator needs the **planning target** `T` and its **target_root** (abs path of the subsystem) —
if not supplied, the `plan-cartographer` auto-derives `targets.md` from the repo's Cargo workspace
members + major modules and the loop picks the next `- [ ]`. Record `T`, `target_root`,
`recency_window_days: 90` in `loop_state.md`. (Read-only target: no build required to start, but the
verifier may run it — note the toolchain.)

## Phase 1: MAP + RESEARCH (five-lane fan-out, parallel background) — `run_in_background:true`

Launch the **five Opus 4.8 background lanes** above in parallel; at minimum the existing packaged
specialists map onto them as: cartographer/code-graph, trend-researcher/web-trends,
governance-config-auditor/governance+settings, settings/config hygiene, and the dedicated rusty-idd
north-star lane.

Required artifacts:
- `plan-cartographer` / code-graph lane → `.handoff/loop/plan/graph/<T>.{symbols,callgraph,metrics}.json` +
  `<T>.graph.md` + `<T>.diff.md` (delta vs the previous committed snapshot — this is the graph
  *update*) + `reports/codemap-<T>.md`; seeds `dimensions.md` with the dimensions this target needs.
  Built **only** from `git-kb code` JSON (no C dep, no graph DB). Required command families:
  `git-kb code doctor`, `index`, `symbols`, `callers`, `callees`, `impact`, `flows`, `query hotspots`,
  `query public-api`, `query entrypoints`, `query cross-service-impact`, `query dead-code-explain`,
  and `dead`; text grep is not a substitute for graph intelligence.
- `plan-trend-researcher` / web-trends lane → `.handoff/loop/plan/research/<T>.trends.md` —
  best-practices + latest trends in a **rolling 90-day window** (compute from today's date; prefer
  in-window sources, flag older), every finding cited + dated.
- governance/settings/config lanes → `findings/governance-config-<T>.md` with control-plane drift,
  settings/config hygiene, and routing (`APPLY|PROPOSE|REGENERATE`).
- memory-vector auditor → `findings/memory-vector-intelligence-<T>.md` with memory inventory, vector intelligence map, recall guarantees, and upgrade rows.
- autoresearch auditor → `findings/autoresearch-<T>.md` with code graph refresh, web/vendor recency, cadence, and stale-evidence rules.
- rules-policy-org auditor → `findings/rules-policy-org-<T>.md` with policy table, agent org chart, A2A/weave map, and automation-first routing.
- distributed-compute auditor → `findings/distributed-compute-<T>.md` with Rust/Lua hardware matrix, vendor mesh, control/data plane, and failover.
- dependency-graph auditor → `graph/target-dag.json` + `graph/target-dag.md` with ready-set scheduling,
  node-scoped context, and SELF-REVISION rows.
- filesystem-layout auditor → `findings/filesystem-layout-<T>.md` with path inventory, FHS/XDG +
  repo-native placement verdicts, boundary map, exact migration/enforcement upgrade rows, and CI/test
  handoff. This is mandatory because file/folder organization drift is a recurring planning blind spot.
- prompt-architecture auditor → `findings/prompt-architecture-<T>.md` with instruction surfaces,
  tools granted, model lanes, hidden architectural couplings, governance controls, and ADR/no-ADR routing.
- trend researcher also writes `research/sources-<T>.jsonl`; orchestrator writes `reports/agent-run-ledger-<T>.md`,
  `risk-policy.md`, `agent-backend-matrix.md`, and `agent-interop.md`.
- rusty-idd north-star lane → `findings/rusty-idd-north-star-<T>.md` capturing whether `rusty-idd` is
  the correct first surfaced target, how it binds to the Forge/IDD loop, and any meta/envctl/prompt_hub
  relationship gaps.

Await all lanes, then commit `dimensions.md` + graph + research + governance/settings/config +
filesystem-layout + prompt-architecture + TDP DAG + north-star findings.

## Phase 2: ANALYZE (fan-out, parallel)

For each `- [ ]` code dimension, spawn `plan-analyst` (parallel) → `.handoff/loop/plan/findings/<dim>.md`. Also spawn `plan-governance-config-auditor` for the governance+settings+config axis, `plan-filesystem-layout-auditor` for the standard OS/repo layout axis, and `plan-test-strategist` for the always-on test-coverage/P8 axis:
falsifiable **CLAIM** rows (each citing `file:line` / symbol / call-path / test) + named **gaps** +
**UPGRADE** rows each tagged `axis: quality|speed|accuracy|governance+settings+config|filesystem-layout|prompt-architecture|memory-vector-intelligence|autoresearch|rules-policy-org|distributed-compute` with rationale, evidence, blast-radius
(from the graph) and risk. Analysts query the graph — blast-radius to scope each upgrade's risk,
centrality to prioritize. Mark dimensions `- [~]` (analyzed, unverified).

## Phase 3: VERIFY (adversarial, parallel) — the gate

For each analyzed dimension, spawn `plan-verifier` → it tries to **refute** each material claim
against the code (read the cited source; run it where ambiguous) **and feasibility-gates each
upgrade** (is it actually buildable here, within the invariants? does it really serve its axis?).
Verdicts → `.handoff/loop/plan/findings/verdicts.md`. Only `CONFIRMED`/`QUALIFIED` claims and
feasibility-passed upgrades reach the plan; `REFUTED`/`INCONCLUSIVE` do not (notable refuted
overclaims are still reported as findings). Mark verified dimensions `- [x]`. **Never let an
unverified claim or an infeasible upgrade into the plan** — that is the whole point.

## Phase 3.5: TDD RED-suite authoring (prompt P8, permitted mutation)

`plan-test-strategist` turns every accepted gap/upgrade acceptance criterion into additive tests before implementation: unit/integration/e2e/golden/property where appropriate plus a differential-drive live case when a CLI/binary behavior is involved. It writes tests only in additive test locations (`tests/`, `#[cfg(test)]`, `scripts/differential-drive.cases.sh`, or an equivalent clearly test-only path), runs them, records the RED failure, and count-verifies `tests-ran > 0`. A test that passes before implementation is invalid and must be rewritten. Emit `findings/test-strategy-<T>.md` with the plan-item ↔ acceptance criterion ↔ test(s) ↔ RED|GREEN traceability matrix and the Feature-Forge GREEN handoff. No production code, gate relaxation, or destructive change is permitted.

## Phase 4: SYNTHESIZE

`plan-architect` → `.handoff/loop/plan/reports/<T>-plan.md`: **verdict first** (the headline
recommendation), then **ASCII architecture diagrams** (envctl `DIAGRAMS.md` conventions — box-drawing,
`Source: file:section`, the `[A]/[A*]/[P]/[H]/[!!]` legend), including a file/folder organization map
when layout is material to the target, the **sequenced upgrade roadmap** (each
item tagged quality/speed/accuracy, ordered by value/risk using graph centrality + blast-radius), a
**governance/settings/config findings**, **filesystem-layout findings** (path inventory, placement verdicts, boundary map, enforcement tests), dedicated **tool-evaluation** section (tools/CLIs/MCPs/crates the target uses, their 90-day currency /
advisories from the researcher, recommend upgrade/hold), named **gaps**, and a stated **confidence**.
Then promote: append a `docs/ROADMAP.md` row (canonical copy stays under `reports/`); emit a **draft**
ADR at `.handoff/decisions/ADR-####-<slug>.md` **only** for a genuine architecture decision. Docs
only — never touch production code. Commit.

## Output contract

End every cycle with paths to: plan file, ASCII diagrams including the control-plane diagram and any file/folder organization map, graph snapshot + diff, gap→upgrade table across quality/speed/accuracy/governance+settings+config/filesystem-layout, tool-eval table, governance findings, settings/config hygiene findings (MCP rot / skill overload / token burn / permission/config drift), memory/vector findings, autoresearch findings, rules/policy/org findings, distributed-compute findings, prompt-architecture findings, filesystem-layout findings (path inventory / placement verdicts / boundary map / enforcement tests), TDD RED-suite evidence with tests-ran count and traceability matrix, evolution scorecard/LESSONS/proposed-upgrades, and the resume pointer.

## Phase 5: SELF-EVAL (every cycle) — `evolution-steward`, lightweight

Skill `harness-evolution`. Evaluate THIS cycle (friction, **gate quality** — did a wrong claim slip
past verify? did verify false-refute a sound upgrade?, coverage, human-walls) → append `LESSONS.md`;
**QUEUE** upgrades (low-risk in-scope → a `- [?]` item; structural → `proposed-upgrades.md`). **Do
NOT apply mid-cycle** — applying low-risk queued upgrades happens only at the batch boundary
(`wrap_every`) / HAND OFF via feature-branch → PR → auto-merge with a CLAUDE.md change-history row;
structural changes stay PROPOSED for the owner. **Never weaken the verify/completeness/DONE gate —
only strengthen it.** This per-cycle evaluate + boundary-apply cadence is how "self-eval + self-upgrade
on every run" reconciles with the fail-closed, never-mid-cycle rule.

## DONE gate (evidence-backed, fail-closed)

Write `.handoff/loop/plan/DONE` only after `scripts/plan-artifact-gate.sh .handoff/loop/plan` passes and when: every target in `targets.md` is `- [x]` (planned) or an
explicit `- [!]`, AND every dimension of the target is `- [x]` (verified) or `- [!]`; AND the
`plan-cartographer`'s **completeness sweep** re-derives the target's expected surface from the graph
(modules / entry points / public-API) and finds nothing major unexamined; AND the plan answers from
CONFIRMED evidence with a stated confidence + named gaps. A partial/zero re-derivation →
**INCONCLUSIVE → write `.handoff/loop/plan/NEEDS-HUMAN`**, not DONE. Record the sweep result inside DONE.

## Continuity & error handling

- **HAND OFF** at budget via `session-relay-wrap-up` (Phase 5 retro → ICM store → continuity-steward
  HANDOFF + commit → heartbeat → stop). **RESUME** via `session-relay-resume` (ICM recall → weave
  inbox → committed HANDOFF authoritative → verify-on-resume → continue at the next target/dimension).
- **Retry once; never fabricate evidence.** An analyst/researcher error → mark that dimension `- [!]`
  and continue others. A claim that can't be verified stays unconfirmed (out of the plan). A behavioral
  claim that can't be run in this env → INCONCLUSIVE, surfaced under gaps — never asserted. The
  verifier wins any conflict (it checks real source). **Never weaken a gate to force a pass.**
- Agents recall/store durable memory via `icm-memory` as needed (graceful no-op if ICM absent).

## Eject

`bash scripts/eject.sh <target-repo>` copies the harness (5 plan skills + shared skills + 5 plan
agents + shared agents) into the target's `.claude/` and scaffolds `.handoff/loop/plan/`. See
`references/state-contract.md`. Invoke as `/planning-engineer` (loop via `/plan-loop`).

## Test Scenarios

**Happy path:** Target = `crates/secrets-proto`. Phase 1: cartographer builds the graph (symbols +
call edges + flows), derives metrics (a hotspot `MintReq`, zero dead code, public-API = the tonic
service), seeds dimensions {architecture, data-flow, public-API/contracts, tooling}; trend-researcher
reports current tonic/prost best-practices dated within 90 days. Phase 2: analysts produce cited
claims + upgrades (e.g. `UPGRADE: derive prost validation | axis: accuracy | …`). Phase 3: verifier
CONFIRMS the wired claims, QUALIFIES one upgrade ("only if feature X"), REFUTES an overclaim. Phase 4:
architect emits an ASCII service diagram + a quality/speed/accuracy/governance+settings+config roadmap + a tool-eval (prost
version currency) + a ROADMAP row, confidence stated. Phase 5: evolution-steward logs one lesson, no
structural change. Production code untouched (`git status` shows only `.handoff/` + `docs/ROADMAP.md`).

**Error path (unverifiable / infeasible):** an analyst proposes "switch the store to an in-process C
cache for speed" → the verifier feasibility-gates it against the NON-NEGOTIABLE no-C-in-trust-boundary
invariant → REFUTED (infeasible), excluded from the roadmap and listed under gaps with the reason,
never recommended. A behavioral perf claim that can't be benchmarked in this env → INCONCLUSIVE →
listed under gaps ("needs a perf harness"), not stated as fact.

## References
- `references/state-contract.md` — the `.handoff/loop/plan/` layout + dimension/claim/verdict/upgrade schema.
- `references/eject.md` — install into a target repo.
- `scripts/loop_state.template.md` · `scripts/eject.sh` · `scripts/ralph-plan.sh` (SAFE, read-only).
