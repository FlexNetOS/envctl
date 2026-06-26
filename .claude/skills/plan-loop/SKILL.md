---
name: plan-loop
description: >-
  Run the planning-engineer crew CONTINUOUSLY over a backlog of planning targets — the Ralph loop.
  ALWAYS use when asked to: plan a whole repo subsystem-by-subsystem, "keep planning", "loop on the
  architecture", run the planning engineer "until done"/"on repeat"/"unattended", "plan the backlog",
  or "resume the planning loop" from a handoff. Each iteration plans the next undone target via the
  full cartographer→researcher→analyst→verifier→architect cycle, then **self-evaluates AND
  self-upgrades the harness after every cycle** (the `evolution-steward` runs the `harness-evolution`
  method each run — evaluate → mine lessons → route → queue/apply fail-closed), checkpoints, and
  self-paces. Follow-ups — "resume the loop", "re-run", "continue", "keep planning", "run it again",
  "redo only the <target>". At the per-session cycle budget it hands off to a fresh session via
  session-relay. Do NOT use for a single one-off plan (use `planning-engineer` directly) or for
  building/implementing code (use feature-forge/forge-loop). Read-only on the target's code.
---

# Planning Engineer Loop (Ralph)

You run the `planning-engineer` crew as a **self-perpetuating loop** over a durable backlog of
planning targets, instead of one target at a time. The owner intent is captured upstream in
`/home/drdave/Desktop/meta/prompt_hub/prompts/planning-engineer-loop.prompt.yml`; this runnable loop
must preserve that prompt's fleet-wide, self-evolving, background-agent, graph-first contract. The *Ralph* pattern: durable state on disk, each
iteration reads it, plans the next undone target, writes the result back, and re-fires. The loop's
intelligence lives in the **backlog + checkpoints**, not in conversation memory — that is exactly
what lets a fresh session pick it up with zero loss (see `session-relay-resume`).

## Why this shape
Conversation context rots and token cost climbs the longer a single session runs. A loop that keeps
all its truth in durable files (`targets.md` + `dimensions.md` + checkpoints) can be carried across
many short, cheap sessions instead of one long, degrading one. **Never hold loop state only in your
head — write it down every iteration.** State precedence: **Git > `.handoff/loop/plan/` markdown
views**; when a `hf` witnessed ledger is present, it outranks the markdown and the markdown is
corrected to it.



## Owner critical architecture-loop axes (mandatory)

Every planning iteration must cover persistent memory/vector intelligence, constant code+web
auto-research, rules/policy/agent-org/A2A communication, Rust+Lua runtime strategy, distributed compute
across owner hardware (mobile, AI glasses/wearables, Pi/Pi Zero, ESP32), and multi-vendor local+cloud
mesh. Missing any of `findings/memory-vector-intelligence-<T>.md`, `findings/autoresearch-<T>.md`,
`findings/rules-policy-org-<T>.md`, or `findings/distributed-compute-<T>.md` blocks DONE.

## June 2026 P0-P2 loop upgrades (mandatory)

The loop now schedules and gates planning work with the full P0-P2 upgrade set:

- Use `graph/target-dag.json` / `graph/target-dag.md` as the Task-Decoupled Planning scheduler; pick
  from the topological ready-set, not only the first unchecked row in `targets.md`.
- Write `SELF-REVISION` rows and re-open only affected downstream nodes when verifier evidence changes
  an upstream assumption.
- Require `findings/prompt-architecture-<T>.md` for prompt/tool/model/runtime coupling review.
- Maintain `reports/agent-run-ledger-<T>.md` so foreground chat can inspect background lane progress
  without interrupting workers.
- Maintain `agent-backend-matrix.md`, `risk-policy.md`, and `agent-interop.md` for isolation,
  human-in-the-loop routing, and weave/MCP/ACP/A2A/GitHub-cloud interop decisions.
- Require `research/sources-<T>.jsonl` alongside `research/<T>.trends.md` for reproducible web research.
- Run `scripts/plan-artifact-gate.sh .handoff/loop/plan` before writing or trusting `DONE`.

## Durable state (the loop's memory) — all under `.handoff/loop/plan/`
Namespaced under `.handoff/loop/plan/` (NOT the flat `.handoff/loop/` — that is forge-loop's; this
mirrors the `.handoff/loop/rust-port/` namespacing precedent). Lay it down with `harness-loop-init`.
- **`targets.md`** — the planning backlog VIEW: `- [ ] <T>: <one-line>` per target (→ `- [x]` when
  its plan is complete & verified; `- [!]` blocked; `- [~]` in-flight; `- [!!]` SUPERVISED — never
  auto-run). Auto-derived (see below).
- **`dimensions.md`** — per-target dimension ledger (cartographer-owned, verifier-gated).
- **`graph/target-dag.json` + `graph/target-dag.md`** — P0 TDP scheduler: nodes, edges, ready-set, and SELF-REVISION rows.
- **`loop_state.md`** — counters: `cycles_this_session`, `cycles_total`, `cycle_budget`, `wrap_every`,
  `last_wrapup_total`, `session_started` (UTC, passed in — never call Date.now), `planning_target`,
  `target_root`, `recency_window_days`, `graph_snapshot`, `last_item`, `status`.
- **Per-cycle artifacts** — `graph/<T>.*`, `research/<T>.trends.md`, `findings/<dim>.md`,
  `findings/filesystem-layout-<T>.md` + `verdicts.md`, `reports/codemap-<T>.md` + `<T>-plan.md`,
  `evaluation.md`.
- **Sentinels**: `DONE`, `NEEDS-HUMAN`, `STOP`, `WRAP-UP-OWED`.

## Target backlog (`targets.md`) — auto-derived, owner-overridable
If `targets.md` does not exist, the first cycle's `plan-cartographer` **auto-derives** it by
enumerating the repo's Cargo workspace members + major modules, one `- [ ] <T>: <one-line>` each
(keep targets small & independent — one crate/subsystem per item). It must also read
`/home/drdave/Desktop/meta/.meta.yaml`, `AGENTS.md`, `.codex/config.toml`, and the PromptHub planning
prompt so the backlog reflects the **meta↔envctl relationship** and owner north star, not only Cargo
members. First-run seed rule: include **`rusty-idd`** as the first surfaced fleet planning target when
it exists under meta; that lane is the real upgrade path into the Forge/IDD loop. An **explicit
owner-supplied target list** in the invocation OVERRIDES the auto-derived list for that run. DONE
scope = every target in `targets.md` planned + verified.

> **hf-aware (optional):** if `hf` is on PATH and the ledger-residency guard holds, you MAY mint
> planning targets as cards and pick via `hf resume --json` (as forge-loop does). Otherwise use the
> markdown `targets.md` path. The planning loop is correct on the markdown path alone.

## One iteration (the loop body)
1. **Read state.** `targets.md` + `loop_state.md`. Confirm the worktree is clean (`git status`) and on
   the loop branch.
2. **Phase-0 stop checks (read ALL sentinels first, in order):**
   - `.handoff/loop/plan/STOP` → **halt immediately**, no re-fire (human kill switch; top priority).
   - `.handoff/loop/plan/NEEDS-HUMAN` → stop and surface for a human; do not auto-pick around it.
   - `.handoff/loop/plan/DONE` **OR** completion confirmed (every target `- [x]`/`- [!]`) → **DONE**:
     report, no re-fire.
   - `.handoff/loop/plan/WRAP-UP-OWED` → a batch boundary came due → **run the batch boundary now**
     (below) BEFORE picking another target.
   - **Batch boundary due** — `cycles_total - last_wrapup_total >= wrap_every` → run the boundary,
     then continue if still under `cycle_budget`.
   - `cycles_this_session >= cycle_budget` → **HAND OFF**: invoke `session-relay-wrap-up`, then stop.
3. **Pick** the next target: refresh `graph/target-dag.json` and pick the topological ready-set node. Fallback to the top unchecked unblocked `- [ ] <T>` in `targets.md` only when the DAG is absent during the first derivation cycle. A `- [!!]`
   SUPERVISED target → **REFUSE to auto-run**: write `.handoff/loop/plan/NEEDS-HUMAN` (target id +
   why), do not pick it, stop.
4. **Run ONE planning cycle** on `T` via the `planning-engineer` orchestrator using the required
   **5× Opus 4.8 max-effort background-agent fan-out** while the foreground chat remains interactive:
   code-graph ‖ web-trends ‖ governance ‖ settings/config+filesystem-layout ‖ rusty-idd+north-star → analysts (incl.
   `plan-filesystem-layout-auditor` and `plan-test-strategist` on the always-on dimensions) → verifiers (gate) → architect
   → **`evolution-steward` self-eval + self-upgrade (every cycle — see "Self-eval + self-upgrade
   after every cycle" below)**. This produces
   `reports/<T>-plan.md` (with a *Test Strategy & Coverage* section) + the graph + the ROADMAP/ADR
   promotion + a **Feature-Forge test-build handoff** (the loop plans the suite; Feature Forge builds +
   runs it) + the cycle's `evaluation.md` + a `LESSONS.md` row. On an unresolvable verifier
   gate or a `- [!!]`/blocked condition, write `NEEDS-HUMAN`, mark the target `- [!]` with a one-line
   reason, and move on (don't thrash).
5. **Write state back:** tick `targets.md` (`- [x]` only when the plan is complete AND its dimensions
   are verified `- [x]`; `- [~]` if in-flight; `- [!]` blocked). Increment `cycles_this_session` and
   `cycles_total`, update `planning_target`/`last_item`/`status` in `loop_state.md`, append a one-line
   progress note. Commit the `.handoff/loop/plan/` update (text only).
6. **Re-fire silently** (see Self-pacing). **No per-target pause/summary** — write to
   `.handoff/loop/plan/` and go straight to the next target. A consolidated user-facing summary is
   produced ONLY at the batch boundary (`wrap_every`) and at HAND OFF.

## Self-eval + self-upgrade after every cycle (mandatory — leverages `harness-evolution`)
Every completed cycle ends by running the `evolution-steward` over THIS cycle via the
`harness-evolution` method — the loop is self-improving by construction, not only at the batch
boundary. This is **at the cycle boundary, never mid-cycle** (the rules can't change under a running
cycle), so "after every run" and the fail-closed "never mid-cycle" rule reconcile cleanly — a *run*
is one completed cycle. Per cycle the steward:
1. **Evaluates** the cycle → writes/supersedes `.handoff/loop/plan/evaluation.md` (friction · gate
   quality: did a wrong claim slip past verify / did verify false-refute a sound upgrade? · coverage ·
   human-walls).
2. **Mines lessons** (the *class* of problem, generalized — not the one instance) and appends one
   append-only row per lesson to `LESSONS.md`, bumping the recurrence counter.
3. **Routes** each lesson (skill body / agent def / orchestrator / description / bundled script).
4. **Queues or applies, fail-closed:** a genuinely *low-risk in-scope* edit (tighten an instruction,
   add an example/trigger/checklist item, bundle a repeated helper, fix a stale reference) is applied
   via the standard **feature-branch → PR → auto-merge** flow with a CLAUDE.md change-history row;
   anything structural (add/remove/merge an agent, reorder phases, touch a gate/guard) is **PROPOSED**
   in `.handoff/loop/proposed-upgrades.md`, never applied. **Only ever strengthen the
   verify/completeness/DONE gate — never weaken it.** A lesson seen once is *noted*; the **second**
   recurrence of its class is applied now.

Keep it lightweight per cycle (the steward reads the cycle's artifacts, not the whole run) so it does
not dominate cycle cost; the apply happens between cycles, never inside one.

## Batch wrap-up cadence (the periodic boundary — consolidates the per-cycle evolution)
Every `wrap_every` completed cycles (default 5; `cycles_total - last_wrapup_total >= wrap_every`, or a
`WRAP-UP-OWED` marker), run in order: (1) **reaper** — `bash scripts/reap-worktrees.sh --apply` (clears
merged per-cycle worktrees if any were spawned); (2) **wrap-up reconcile** — `session-relay-wrap-up`
status-truth over `targets.md` + the ICM store + the consolidated batch summary; (3) **evolution-steward
consolidation** — NOT a fresh retro (each cycle already self-evaluated + self-upgraded above): dedupe
the cycle-level `LESSONS.md` rows, roll up recurrence counts across the batch, flush any still-queued
low-risk `- [?]` upgrades the per-cycle apply deferred, and surface the standing `proposed-upgrades.md`
for the owner. Then **clear the marker, set `last_wrapup_total = cycles_total`**, and continue if under
`cycle_budget`. The boundary is *in-session* — NOT a hand-off.

## Background-agent launch contract (Codex + Claude compatible)

The loop must never do fleet scans or web sweeps inline in the foreground. Launch five background
lanes immediately and reduce only their structured artifact paths/verdicts:

| Lane | Codex dispatcher shim | Actual worker transport/model | Required output |
|------|-----------------------|-------------------------------|-----------------|
| Code graph + cross-repo refs | `plan-opus-bg-code-graph` | weave → `claude-opus-4-8`, max/xhigh | graph snapshots/diffs + `git-kb code` query JSON |
| Web/trends + skeptic | `plan-opus-bg-web-trends` | weave → `claude-opus-4-8`, max/xhigh | dated 90-day research + refutation notes |
| Governance/control plane | `plan-opus-bg-governance` | weave → `claude-opus-4-8`, max/xhigh | rules/instructions/hooks/policy/CLAUDE/AGENTS drift |
| Settings/config + filesystem layout | `plan-opus-bg-settings-config` + `plan-filesystem-layout-auditor` | weave → `claude-opus-4-8`, max/xhigh | MCP rot, skill overload, token burn, permissions/config drift, FHS/XDG/repo layout verdicts |
| Rusty-IDD north star | `plan-opus-bg-rusty-idd-north-star` | weave → `claude-opus-4-8`, max/xhigh | first-target verdict + Forge/IDD loop upgrade path |

**Use weave when running from Codex.** Codex dispatchers stay on the supported Codex model; they are
not the heavy workers. The orchestrator registers with `weave attach --name envctl-plan-orchestrator`,
then launches or routes each lane via `weave spawn`, `weave job create`, or `weave ask` to an
Opus-capable Claude peer. Prefer `scripts/plan-weave-dispatch.sh` so the five lane records are
materialized under `.handoff/loop/plan/weave-dispatch/<run-id>.jsonl`. Record weave
peer/session/message/job ids in `loop_state.md` and route by unique peer/session id, not a reused alias
that could hit weave's ambiguous-target guard. If weave cannot produce an Opus worker, **fail closed**
with a provider/transport gap. Do not silently downgrade to Sonnet, Codex, or a lower-effort model for
the actual lane work. The owner must be able to keep talking to the foreground orchestrator while these
lanes run.

## Self-pacing (how the loop re-fires)
- Default: **dynamic `/loop` / runtime scheduler** — re-enter this skill for the next iteration by using the active runtime's supported loop/scheduling surface (for Claude Code, `/loop`/`CronCreate`; do not name or call an unavailable tool). Pass the same `/plan-loop …` prompt verbatim. A planning cycle is deliberative (more reasoning,
  more web/graph work than a code cycle); a warm-cache short delay (≤270s) is fine for back-to-back
  cycles. When you HAND OFF or finish, **do not schedule another re-entry**; ending the scheduling chain is what stops the loop.
- A cycle counts only when a planning cycle **completes** (plan written + verified, or target blocked).

## Cycle budget (the handoff trigger)
Per-session budget is cycles-only: default **3** completed cycles per session unless the user sets
another (`/plan-loop budget=N …`). Record in `loop_state.md`. At the budget you invoke
`session-relay-wrap-up` (checkpoint + announce + arm successor), then stop; the successor resets
`cycles_this_session` to 0 and continues at the next target. `wrap_every` (in-session boundary) and
`cycle_budget` (hand-off) are independent knobs.

> **Truly-unattended runs:** the in-session successor cron is session-only in this runtime, so for
> set-and-forget operation launch via the external `scripts/ralph-plan.sh` runner (SAFE, read-only —
> spawns a fresh `claude -p` per cycle) with `cycle_budget` set to the 5–8 the context holds.

## Resume (entering mid-loop from a handoff)
If invoked to **resume** (a `.handoff/loop/plan/HANDOFF.md` exists, or weave inbox / successor cron
says so): follow `session-relay-resume` first (read HANDOFF → verify-on-resume baseline, fail →
NEEDS-HUMAN → reset `cycles_this_session`), then run the iteration body normally.

## Stop conditions & sentinel write semantics (end the loop — no re-fire)
- **DONE** — only when completion is confirmed AND `scripts/plan-artifact-gate.sh .handoff/loop/plan` passes AND the cartographer's **pre-DONE completeness sweep**
  re-derives each planned target's expected surface from its graph and finds nothing major unexamined.
  A partial/zero re-derivation → INCONCLUSIVE → write `NEEDS-HUMAN`, not DONE.
- **NEEDS-HUMAN** — an unresolvable verifier gate, any `- [!!]` SUPERVISED target, or a
  progress-blocking structural harness upgrade. Stop and surface.
- **STOP** — the human kill switch: halt re-fire immediately, ahead of all checks.
- **Cycle budget reached** → hand off (session-relay), then stop.

## Test Scenarios
**Happy path:** `/plan-loop budget=3` on envctl with no `targets.md`. Cycle 1's cartographer
auto-derives `targets.md` from the 8 workspace crates, then plans `engine`; cycles 2–3 plan `cli` and
`secrets-engine`. Each writes `reports/<T>-plan.md` (diagrams + quality/speed/accuracy/governance+settings+config roadmap +
tool-eval) + a `docs/ROADMAP.md` row, ticks the target `- [x]`, increments the ledger. After cycle 3
the budget trips → `session-relay-wrap-up` writes HANDOFF, the successor resumes at `secretd`.

**Error path:** a target's verifier returns INCONCLUSIVE on a behavioral perf claim it can't
benchmark in this env → that dimension is `- [!]`, the gap is recorded in the plan, the loop marks the
target `- [~]` (planned-with-gaps) and proceeds to the next target rather than thrashing; the gap
surfaces in the batch summary for a human decision.
