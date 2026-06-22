#!/usr/bin/env bash
# Stop / PreCompact hook (envctl) — auto-checkpoint this session into the FLEET ledger (ADR-0004 §3)
# AND mirror it one-way into GitKB (`hf sync`, ADR-0003 HFTASK-0011 — TASK-0024 GO-LIVE).
#
# LIVE: the kernel's `hf checkpoint --auto --quiet` + `hf sync` verbs landed (Epic A TASK-0001/0002,
# meta/handoff #17). Both calls are fail-soft — if `hf` is absent OR rejects a flag we swallow it and
# exit 0, so the session is NEVER blocked. `hf sync` is one-way (ledger truth → .kb context/overridable),
# so a checkpoint also lands in code intelligence; this makes "auto-sync to .handoff and .kb" TRUE.
#
# Ledger-residency (the kernel invariant): the shipped `hf` resolves a CWD-relative `.handoff/`
# (`const HF=".handoff"`, no --ledger flag), so we run it from $META_ROOT — the witnessed FLEET
# ledger is $META_ROOT/.handoff/ledger.db ONLY. NEVER a per-repo ledger (that would violate ADR-0004).
# $META_ROOT is resolved by walking up to the .meta.yaml marker, so this works from envctl or any
# of its worktrees (meta/.worktrees/<slug>/envctl) without a hardcoded path.
set -u

# --- batch wrap-up cadence enforcement (cheap, fail-soft, no git) ---------------------------------
# The forge-loop runs tasks back-to-back with no per-task pause; the heavy continuity work (reaper +
# wrap-up reconcile + evolution-steward retro) is batched to a boundary every `wrap_every` completed
# cycles. That boundary is an agentic step and is easy to SKIP — and skipping it is exactly what let
# 46 worktrees pile up. So this hook — which fires every Stop/PreCompact — drops a durable
# `WRAP-UP-OWED` marker the moment a boundary comes due. session-relay-resume is FAIL-CLOSED on that
# marker (must run the owed wrap-up before picking new work), so a missed boundary is caught at the
# next resume rather than silently lost. Pure file I/O (read counters, maybe touch one file): safe to
# run on every turn. The marker is cleared by the wrap-up that satisfies it (sets last_wrapup_total).
PROJ="${CLAUDE_PROJECT_DIR:-$PWD}"
LS="$PROJ/.handoff/loop/loop_state.md"
if [ -f "$LS" ]; then
  ct="$(awk '/^cycles_total:/{print $2; exit}' "$LS" 2>/dev/null)"
  we="$(awk '/^wrap_every:/{print $2; exit}' "$LS" 2>/dev/null)"
  lw="$(awk '/^last_wrapup_total:/{print $2; exit}' "$LS" 2>/dev/null)"
  we="${we:-5}"; lw="${lw:-0}"
  case "$ct$we$lw" in *[!0-9]*|'') ct="" ;; esac   # only act when all three parsed as integers
  if [ -n "$ct" ] && [ "$((ct - lw))" -ge "$we" ]; then
    marker="$PROJ/.handoff/loop/WRAP-UP-OWED"
    [ -f "$marker" ] || printf 'boundary due: cycles_total=%s last_wrapup_total=%s wrap_every=%s\nrun /session-relay-wrap-up (reaper + reconcile + evolution-steward) before picking new work.\n' \
      "$ct" "$lw" "$we" > "$marker" 2>/dev/null || true
  fi
fi

d="${CLAUDE_PROJECT_DIR:-$PWD}"
META_ROOT=""
while [ "$d" != "/" ] && [ -n "$d" ]; do
  [ -f "$d/.meta.yaml" ] && META_ROOT="$d" && break
  d="$(dirname "$d")"
done
[ -n "$META_ROOT" ] || exit 0

# find hf: prefer PATH (post-relocation), else the kernel build under meta/handoff.
HF="$(command -v hf 2>/dev/null || true)"
if [ -z "$HF" ]; then
  for c in "$META_ROOT/handoff/target/release/hf" "$META_ROOT/handoff/target/debug/hf"; do
    [ -x "$c" ] && HF="$c" && break
  done
fi
[ -n "$HF" ] || exit 0

# fail-closed residency: refuse to let a per-repo ledger be created. Only the fleet ledger at
# $META_ROOT/.handoff/ledger.db is permitted; run hf from there so its CWD-relative .handoff resolves to it.
cd "$META_ROOT" 2>/dev/null || exit 0
"$HF" checkpoint --auto --quiet >/dev/null 2>&1 || true
# TASK-0024 GO-LIVE: one-way mirror the witnessed FLEET ledger → GitKB (.kb context/overridable/
# {active,progress}). Same $META_ROOT residency as the checkpoint (never a per-repo ledger). Fail-soft.
"$HF" sync --auto >/dev/null 2>&1 || true
exit 0
