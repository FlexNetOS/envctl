#!/usr/bin/env bash
set -u

root="${FLEXNETOS_ROOT:-/home/flexnetos/FlexNetOS}"
gate="$root/src/envctl/.codex/hooks/flexnetos-runtime-gate.sh"

tmp_state="$(mktemp -d)"
trap 'rm -rf "$tmp_state"' EXIT

pass_count=0

run_gate() {
  local name="$1"
  local event="$2"
  local payload="$3"
  printf '%s' "$payload" | env \
    FLEXNETOS_GATE_STATE="$tmp_state/$name" \
    FLEXNETOS_GATE_LOG_DIR="$tmp_state/$name/log" \
    /bin/bash "$gate" "$event" >/dev/null 2>&1
}

run_expect_pass() {
  local name="$1"
  local event="$2"
  local payload="$3"
  if run_gate "$name" "$event" "$payload"; then
    pass_count=$((pass_count + 1))
  else
    printf 'not ok - %s\n' "$name" >&2
    exit 1
  fi
}

run_expect_fail() {
  local name="$1"
  local event="$2"
  local payload="$3"
  if run_gate "$name" "$event" "$payload"; then
    printf 'not ok - %s\n' "$name" >&2
    exit 1
  else
    pass_count=$((pass_count + 1))
  fi
}

run_expect_pass "meta-additive-status" \
  pre-tool-use \
  "{\"cwd\":\"$root/src/meta\",\"hook_event_name\":\"PreToolUse\",\"tool_input\":{\"command\":\"/home/flexnetos/FlexNetOS/usr/bin/meta project list --json\"}}"

run_expect_fail "meta-core-delete-without-archive" \
  pre-tool-use \
  "{\"cwd\":\"$root\",\"hook_event_name\":\"PreToolUse\",\"tool_input\":{\"command\":\"rm -rf $root/src/meta/meta_core/target\"}}"

snapshot_state="$tmp_state/meta-core-delete-with-archive"
FLEXNETOS_GATE_STATE="$snapshot_state" \
  FLEXNETOS_GATE_LOG_DIR="$snapshot_state/log" \
  /bin/bash "$gate" snapshot "$root/src/meta" >/dev/null 2>&1 || {
    printf 'not ok - meta snapshot setup\n' >&2
    exit 1
  }

if printf '%s' "{\"cwd\":\"$root\",\"hook_event_name\":\"PreToolUse\",\"tool_input\":{\"command\":\"mkdir -p $root/var/lib/codex-runtime-gate/archives && tar -czf $root/var/lib/codex-runtime-gate/archives/meta_core-target-before-delete.tgz -C $root/src/meta/meta_core target && rm -rf $root/src/meta/meta_core/target\"}}" \
    | env \
      FLEXNETOS_GATE_STATE="$snapshot_state" \
      FLEXNETOS_GATE_LOG_DIR="$snapshot_state/log" \
      /bin/bash "$gate" pre-tool-use >/dev/null 2>&1; then
  pass_count=$((pass_count + 1))
else
  printf 'not ok - meta-core-delete-with-archive\n' >&2
  exit 1
fi

printf 'ok - %s runtime gate policy checks passed\n' "$pass_count"
