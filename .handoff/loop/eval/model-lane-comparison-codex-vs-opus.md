# Model-Lane Comparison — Codex sub-agents vs Opus sub-agents (planning loop)

> Author: planning-loop orchestrator (Opus foreground). Scope: the **same** fleet-convergence
> planning crew run two ways — **cycle 7 (icm) on Opus sub-agents** vs **cycle 8 (harness-hub) on
> Codex sub-agents** — under the owner directive "strictly swap codex for opus" (Claude tokens
> ~exhausted). The orchestrator role, the artifact contract, and the gate (`plan-artifact-gate.sh`)
> were held **identical** across both; only the worker model lane changed. Cycle-8 quality numbers
> are PRELIMINARY (the Codex wave was still running when this was written).

## 1. The process (unchanged across both lanes)
One capped planning cycle per target: **cartographer ‖ trend-researcher → 7 axis auditors + analyst
+ test-strategist (RED) → verifier gate → architect synthesis → evolution-steward → gate → ship
(envctl plan PR + target RED PR) → release lease → ICM store → notify**. Durable state in
`.handoff/loop/plan/`. The gate is the invariant that makes the lane swappable: a plan only ships if
the gate-named artifacts exist with the required markers, regardless of which model produced them.

## 2. The two mechanisms

| | **Opus sub-agent (cycle 7)** | **Codex sub-agent (cycle 8)** |
|---|---|---|
| Launch | `Agent` tool, `subagent_type: harness:plan-*`, `run_in_background:true` | `codex exec -s workspace-write -C <root> -o <file> "PROMPT"` as background Bash |
| Role priming | **Auto-loaded** from `harness/agents/<name>.md` (agentType resolves the def) | **Manual** — full role+method embedded in each prompt (no agentType) |
| Result | `task-notification` with a structured `<result>` + `usage{subagent_tokens,tool_uses,duration_ms}` | raw `-o` last-message file + verbose stdout log; `tokens used N` printed in stdout |
| Resumability | `SendMessage` to the agentId (context intact) | `codex exec resume --last` / session id (separate mechanism) |
| Sandbox | full tool access (`*`) | filesystem-confined `workspace-write`; network off by default |
| Token pool | **Claude/Opus budget** (the constraint) | **Codex's own budget** (the reason for the swap) |
| Re-invoke-on-exit | yes (harness-native) | yes (background Bash re-invokes the orchestrator) |

## 3. Measured cost — Opus run (cycle 7, icm), actual telemetry
12 agents, run largely in parallel; wall-clock wave ≈ 10 min (slowest: cartographer 611s, trend 511s).

| agent | tokens | tool_uses | ms |
|---|---|---|---|
| cartographer | 124,991 | 36 | 610,936 |
| trend-researcher | 111,789 | 34 | 510,552 |
| memory-vector | 106,413 | 21 | 244,974 |
| autoresearch | 126,173 | 19 | 229,575 |
| convergence analyst | 102,522 | 27 | 312,976 |
| verifier | 101,434 | 19 | 269,231 |
| test-strategist | 100,591 | 27 | 431,468 |
| prompt-architecture | 99,949 | 20 | 226,503 |
| filesystem | 85,376 | 10 | 222,712 |
| rules-policy | 81,966 | 15 | 203,148 |
| governance | 80,397 | 19 | 214,040 |
| distributed-compute | 78,915 | 14 | 178,727 |
| **total** | **≈1.20M** | **261** | wave ≈10 min |

That ~1.2M is **Opus budget**, drawn down per cycle — exactly why a 7th–8th cycle hit the ceiling.
Codex smoke test: a trivial file-write cost 32,655 Codex tokens (Codex has notable per-task overhead),
but on **Codex's pool**, not Claude's. Full Codex-wave totals to be appended from cycle-8 stdout.

## 4. What was GOOD
- **The gate made the swap safe.** Because `plan-artifact-gate.sh` + the marker contract are
  lane-agnostic, swapping the worker model required **zero** change to the definition of "done." This
  is the single most important design property the run validated.
- **The background-and-notify pattern is lane-portable.** `run_in_background` Bash re-invokes the
  orchestrator on `codex exec` exit just like `task-notification` does for Opus agents — the
  orchestration loop didn't change.
- **Opus quality was excellent** (cycle 7): deep, code-cited findings; genuine cross-agent
  reconciliation (the 768-vs-384 embedding-dim drift was independently flagged by 3 agents and
  adjudicated by the verifier's empirical probe); a real `cargo build` reality check; fail-closed gate.
- **Codex came up first try** for file I/O (smoke test green) and `workspace-write` is a **stronger
  safety posture** than the Opus agents' full tool access — writes were filesystem-confined.
- **Token-frugal foreground close worked** (cycle 7): architect+steward synthesis done in-context
  with no extra sub-agents when budget ran low — a reusable "low-token close" mode.

## 5. What was BAD / friction
- **Codex full-access is blocked by Claude Code's auto-mode classifier.** `codex exec -s
  danger-full-access` was denied ("Create Unsafe Agents"); had to drop to `workspace-write`. A real
  operational wall — the most capable Codex mode is unavailable without an explicit Bash permission rule.
- **No role pre-loading for Codex.** Opus agents inherit `harness/agents/<name>.md` automatically;
  Codex needs the entire role+method+markers embedded in each prompt. This **raises orchestrator token
  cost** (the one budget still being spent) and risks method drift from the canonical agent def.
- **No structured telemetry from Codex.** Opus returns clean `usage{tokens,tool_uses,duration}`;
  Codex prints `tokens used N` in verbose stdout, so cost-parity accounting needs scraping.
- **Sandbox papercuts.** `workspace-write` disables network (had to add
  `-c sandbox_workspace_write.network_access=true` for the researcher); `--add-dir`/`-C`/
  `--skip-git-repo-check` must be set deliberately per lane.
- **Result ergonomics.** The Opus `<result>` is a curated summary; the Codex `-o` file is whatever the
  model last said, and the real transcript is a long stdout log you must NOT read wholesale.
- **Recurring harness friction (lane-independent):** the gate's placeholder-token regex keeps
  tripping on self-referential text (cycle 6 `TODO.md`; cycle 7 a "no TODO used" meta-note); the
  targets.md parser rejects non-`#` prose lines. Both bit again this session.

## 6. RECOMMENDATIONS
1. **Adopt the inverted dual-model lane as standard under token pressure:** Codex `workspace-write`
   workers for the parallel READ-heavy auditor lanes (separate pool, sandboxed, strong at file work) +
   Opus for the **verifier gate** and **synthesis** where reasoning quality is decisive. This is the
   loop's existing "dual-model" law, inverted: Codex bg workers + Opus fg orchestrator/gate.
2. **Build `envctl/scripts/plan-codex-dispatch.sh`** (sibling to `plan-weave-dispatch.sh`): wraps
   `codex exec` with (a) the canonical `harness/agents/<name>.md` injected as the system/first prompt
   so Codex workers follow the **exact same method** as Opus agents (recovers agentType priming),
   (b) standard sandbox flags + a `--net` toggle, (c) `-o` capture + a **normalized result envelope**,
   (d) scrape `tokens used N` into the agent-run-ledger for cost parity.
3. **Keep the gate the sole arbiter of "done"** — never special-case a lane. Lane-agnostic gating is
   what made this swap safe; protect it.
4. **Fix the recurring gate papercuts now** (2nd+ recurrence ⇒ apply-eligible): (a) make the
   placeholder-token check ignore fenced/quoted self-references or require the tokens as standalone
   list-cells, not substrings; (b) have the cartographer always seed `targets.md` with `#`-prefixed
   prose. Only ever *strengthen* the gate.
5. **Default sandbox policy:** `workspace-write` + network-on for research lanes; document that
   `danger-full-access` trips the classifier and needs an explicit owner-approved Bash rule.
6. **Quality A/B:** once cycle 8 completes, diff Codex vs Opus findings depth on the SAME repo class
   (cite-density, cross-agent reconciliation, verifier refutation rate) and record it in `evaluation.md`
   — decide lane assignment per dimension from evidence, not assumption.

## 7. Verdict (preliminary)
- **Best integration / richest output / cleanest telemetry:** Opus sub-agents (harness-native).
- **Best token economy / safest sandbox / sustainable at scale:** Codex sub-agents (separate pool,
  `workspace-write`).
- **The right architecture is HYBRID, gated identically:** route by dimension — Codex for breadth,
  Opus for the gate and synthesis — with a dispatch wrapper that gives Codex the same role priming and
  normalized results the Agent tool gives Opus. The cycle-7→cycle-8 swap proved the orchestration and
  the gate are model-portable; the remaining work is an adapter, not a redesign.
