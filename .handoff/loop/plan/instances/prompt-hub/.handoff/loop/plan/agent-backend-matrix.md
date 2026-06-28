# Agent backend matrix

Which execution backend each agent lane should run on, by task class. Backends:
read-only-local · isolated-worktree · container · remote-vm · cloud-agent. Comms: ACP / A2A.
Grounded in the cycle-6 frame (weave = A2A transport; loops run in isolated worktrees;
prompt_hub product code is never mutated by planning).

## prompt-hub

| Lane / task class | Backend | Mutates code? | Comms | Rationale (cited) |
|---|---|---|---|---|
| plan-loop analysts (architecture, governance, filesystem, memory, autoresearch, rules-policy, distributed-compute, prompt-architecture) | read-only-local | no | A2A (weave heartbeat) | read-only audits over the target; findings are the only writes (`findings/*.md`). Distributed-compute/rules-policy findings confirm read-only mode. |
| plan-cartographer / dependency-graph / trend-researcher | read-only-local | no | A2A | graph built from `git-kb code` JSON; trends from web fetch; both read-only on target code (graph/, research/). |
| plan-test-strategist (authoring RED tests) | isolated-worktree | additive tests only | A2A | RED suite authored + RUN in `/home/drdave/Desktop/meta/.worktrees/plan-prompt-hub-red/prompt_hub` on `plan/prompt-hub-red-tests` (test-strategy §RED suite). |
| plan-verifier (refute + run probes) | isolated-worktree | no | A2A | empirical probes run in the RED worktree + live target; default-skeptical gate (verdicts.md header). |
| Feature Forge GREEN build (GA-1/GA-2 emit + lineage persist) | isolated-worktree | yes (production code) | A2A + ACP | the construction crew mutates code in a dedicated worktree/branch; worktree-per-task is repo law (rules-policy P8). |
| heavy background research / code-mapping / governance scans | remote-vm / cloud-agent (Opus) | no | A2A (weave→Opus) | dual-model background lanes run on Opus workers via weave; fail-closed if Opus unobtainable (rules-policy §3, P3). |
| GitHub CI gates (cargo fmt/clippy/test/audit/deny) + AI workflows | container | no | — | CI runs in GitHub-hosted containers; AI workflows gate on `ENABLE_AI_WORKFLOWS` (autoresearch C-WEB-7). |
| advisory remediation loop (`security_remediation.yml`) | cloud-agent | proposes PRs | ACP | currently an inert `echo`; when wired, opens one verified PR or escalates (autoresearch U-AR-4). |
| edge fetch/cache client (mobile/wearables, future) | container / remote-vm | no | A2A | thin UniFFI/cdylib over `mobile`+`offline`; read-mostly to bound trust surface (distributed-compute UPGRADE-2). |
| ARM/Pi cross-build smoke | container (aarch64) | no | — | `aarch64-*-musl` lib/CLI-only cross-build; deny-check no C-TLS in runtime graph (distributed-compute UPGRADE-1). |

Backend selection rules:
- read-only audit → **read-only-local** (no mutation possible).
- any code mutation (tests or production) → **isolated-worktree** (worktree-per-task law, P8).
- heavy/parallel research → **remote-vm / cloud-agent** Opus lanes via weave, fail-closed.
- untrusted/external surfaces (CI, AI workflows) → **container** isolation.
- cross-session PR-opening automation → **cloud-agent** with ACP handoff, owner-gated merge.
