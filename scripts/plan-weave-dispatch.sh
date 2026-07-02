#!/usr/bin/env bash
# plan-weave-dispatch.sh — launch/route the Planning Engineer's five Opus lanes through weave.
#
# Codex cannot assume Anthropic model slugs are supported by its ChatGPT provider. This helper keeps
# Codex as the foreground orchestrator and uses weave as the transport for the actual Opus worker:
#   * PLAN_OPUS_CMD=<claude-compatible-cli>  -> weave spawn one peer per lane
#   * PLAN_OPUS_PEER=<existing-peer/session> -> weave ask that peer once per lane
# If neither route is available, it fails closed before any weaker-model work can run.
set -euo pipefail

TARGET=""
TARGET_ROOT=""
RUN_ID=""
STATE_DIR=".handoff/loop/plan/weave-dispatch"
DRY_RUN=0
ORCH=""
OPUS_MODEL="${PLAN_OPUS_MODEL:-claude-opus-4-8}"

usage() {
  cat >&2 <<'USAGE'
usage: bash scripts/plan-weave-dispatch.sh --target <slug> --root <target-root> [--run-id <id>] [--state-dir <dir>] [--dry-run]

Routes the five PromptHub Planning Engineer lanes through weave. In real mode, set either:
  PLAN_OPUS_CMD=/path/to/claude-compatible-cli   # uses weave spawn
  PLAN_OPUS_PEER=<registered-opus-peer-or-session-id>  # uses weave ask

Dry-run writes the dispatch plan without requiring weave or an Opus runtime.
USAGE
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --target) TARGET="${2:-}"; shift 2 ;;
    --root) TARGET_ROOT="${2:-}"; shift 2 ;;
    --run-id) RUN_ID="${2:-}"; shift 2 ;;
    --state-dir) STATE_DIR="${2:-}"; shift 2 ;;
    --dry-run) DRY_RUN=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "error: unknown arg: $1" >&2; usage; exit 2 ;;
  esac
done

[ -n "$TARGET" ] || { echo "error: --target required" >&2; usage; exit 2; }
[ -n "$TARGET_ROOT" ] || { echo "error: --root required" >&2; usage; exit 2; }
RUN_ID="${RUN_ID:-run-$$}"
ORCH="${PLAN_WEAVE_ORCH:-plan-orchestrator-$RUN_ID}"
mkdir -p "$STATE_DIR"
OUT="$STATE_DIR/$RUN_ID.jsonl"
: > "$OUT"

json_escape() {
  python3 -c 'import json,sys; print(json.dumps(sys.stdin.read())[1:-1])'
}
write_record() {
  local lane="$1" mode="$2" peer="$3" ref="$4" status="$5" detail="$6"
  printf '{"run_id":"%s","target":"%s","target_root":"%s","lane":"%s","mode":"%s","worker_model":"%s","peer":"%s","ref":"%s","status":"%s","detail":"%s"}\n' \
    "$(printf '%s' "$RUN_ID" | json_escape)" \
    "$(printf '%s' "$TARGET" | json_escape)" \
    "$(printf '%s' "$TARGET_ROOT" | json_escape)" \
    "$(printf '%s' "$lane" | json_escape)" \
    "$(printf '%s' "$mode" | json_escape)" \
    "$(printf '%s' "$OPUS_MODEL" | json_escape)" \
    "$(printf '%s' "$peer" | json_escape)" \
    "$(printf '%s' "$ref" | json_escape)" \
    "$(printf '%s' "$status" | json_escape)" \
    "$(printf '%s' "$detail" | json_escape)" >> "$OUT"
}

resolve_weave() {
  if [ -n "${WEAVE_BIN:-}" ]; then printf '%s\n' "$WEAVE_BIN"; return 0; fi
  if command -v weave >/dev/null 2>&1; then command -v weave; return 0; fi
  local meta_root="${META_ROOT:-}"
  if [ -z "$meta_root" ] && [ -n "${TARGET_ROOT:-}" ] && [ -d "$TARGET_ROOT/.." ]; then
    meta_root="$(cd "$TARGET_ROOT/.." && pwd -P)"
  fi
  if [ -n "$meta_root" ]; then
    if [ -x "$meta_root/weave/target/release/weave" ]; then printf '%s\n' "$meta_root/weave/target/release/weave"; return 0; fi
    if [ -x "$meta_root/weave/target/debug/weave" ]; then printf '%s\n' "$meta_root/weave/target/debug/weave"; return 0; fi
  fi
  return 1
}

LANES=(
  code-graph
  web-trends
  governance
  settings-config
  rusty-idd-north-star
)

prompt_for_lane() {
  local lane="$1"
  cat <<PROMPT
Planning Engineer lane: $lane
Target: $TARGET
Target root: $TARGET_ROOT
Model contract: $OPUS_MODEL max effort. Use PromptHub intent, graph-first evidence, standard OS file/folder layout (FHS/XDG), and write only .handoff/loop/plan artifacts. Return artifact paths + verdict.
PROMPT
}

if [ "$DRY_RUN" -eq 1 ]; then
  for lane in "${LANES[@]}"; do
    write_record "$lane" "dry-run" "plan-opus-bg-$lane" "dry-run" "planned" "weave transport plan only; no worker launched"
  done
  echo "$OUT"
  exit 0
fi

WEAVE="$(resolve_weave)" || {
  echo "NEEDS-HUMAN: weave binary not found; cannot route Opus plan lanes" >&2
  exit 10
}
"$WEAVE" attach --name "$ORCH" >/dev/null

if [ -n "${PLAN_OPUS_PEER:-}" ]; then
  for lane in "${LANES[@]}"; do
    body="$(prompt_for_lane "$lane")"
    out="$($WEAVE ask --from "$ORCH" --to "$PLAN_OPUS_PEER" --subject "plan-loop/$RUN_ID/$lane" --body "$body" 2>&1)"
    write_record "$lane" "weave-ask" "$PLAN_OPUS_PEER" "$out" "queued" "ask routed to existing Opus-capable peer"
  done
  echo "$OUT"
  exit 0
fi

OPUS_CMD="${PLAN_OPUS_CMD:-claude}"
if ! command -v "$OPUS_CMD" >/dev/null 2>&1 && [ ! -x "$OPUS_CMD" ]; then
  echo "NEEDS-HUMAN: PLAN_OPUS_CMD/claude not executable and PLAN_OPUS_PEER not set; cannot spawn Opus workers" >&2
  exit 11
fi

for lane in "${LANES[@]}"; do
  peer="plan-opus-$RUN_ID-$lane"
  body="$(prompt_for_lane "$lane")"
  out="$($WEAVE spawn "$peer" --cwd "$TARGET_ROOT" --cmd "$OPUS_CMD" --cmd --model --cmd "$OPUS_MODEL" --cmd -p --cmd "$body" 2>&1)"
  write_record "$lane" "weave-spawn" "$peer" "$out" "spawned" "spawned Opus lane worker via weave"
done

echo "$OUT"
