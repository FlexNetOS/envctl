#!/usr/bin/env bash
# ralph-plan.sh — external Ralph loop: self-restarts the planning-engineer harness with a FRESH
# context each iteration until a terminal sentinel. Read-only planning; durable ledger under
# .handoff/loop/plan/, so every restart resumes cold. SAFE by default: no permission-system bypass.
set -euo pipefail
WORKTREE="${RALPH_WORKTREE:-$(pwd)}"; BUDGET="${RALPH_BUDGET:-5}"
MAX_ITERS="${RALPH_MAX_ITERS:-50}"; SLEEP_BETWEEN="${RALPH_SLEEP:-5}"; MODEL="${RALPH_MODEL:-opus}"
WS="$WORKTREE/.handoff/loop/plan"; mkdir -p "$WS"
log(){ printf '[ralph plan %s] %s\n' "$(date -u +%H:%M:%S)" "$*" >&2; }
command -v claude >/dev/null || { log "FATAL: claude not on PATH"; exit 1; }
log "SAFE mode: read-only planning; permission prompts active; no permission-system bypass."

read -r -d '' PROMPT <<EOP || true
Resume the planning-engineer harness (external Ralph runner, fresh context): run /plan-loop resume
(if ejected here) or /harness:plan-loop resume. Worktree: $WORKTREE.
1. If .handoff/loop/plan/HANDOFF.md exists, follow session-relay-resume (authoritative): read it, run
   verify-on-resume, continue at the next target/dimension. Else have the cartographer auto-derive
   targets.md (Cargo workspace members + major modules), then start the first target.
2. Run up to $BUDGET planning cycles: per target, cartographer ‖ trend-researcher (fan-out) -> analysts
   (cited gaps + quality/speed/accuracy/governance+settings+config upgrades) -> adversarially VERIFY claims + feasibility-gate
   upgrades vs the code -> architect synthesizes the plan (ASCII diagrams + tool-eval) + ROADMAP row.
   Read-only on production code. Commit .handoff/loop/plan/ per cycle. Never let an unverified claim or
   an infeasible upgrade into a plan.
3. Then write EXACTLY ONE sentinel under .handoff/loop/plan/ and stop (no ScheduleWakeup):
   DONE (every target planned+verified + completeness sweep clean) | NEEDS-HUMAN (reason) | else HANDOFF.md.
EOP

cd "$WORKTREE"; i=0
while :; do
  i=$((i+1)); [ "$i" -gt "$MAX_ITERS" ] && { log "MAX_ITERS hit — halting."; exit 3; }
  for s in STOP DONE NEEDS-HUMAN; do [ -f "$WS/$s" ] && { log "$s — halting."; [ "$s" = DONE ] && exit 0 || exit 2; }; done
  log "iter $i/$MAX_ITERS — spawning fresh agent (budget=$BUDGET, model=$MODEL)"
  claude -p "$PROMPT" --model "$MODEL" --add-dir "$WORKTREE" \
    >>"$WS/ralph-run-$i.log" 2>&1 || log "iter $i nonzero (continuing from durable state)"
  for s in DONE NEEDS-HUMAN STOP; do [ -f "$WS/$s" ] && { log "$s."; [ "$s" = DONE ] && exit 0 || exit 2; }; done
  sleep "$SLEEP_BETWEEN"
done
