#!/usr/bin/env bash
# test-plan-weave-dispatch.sh — verifies the Codex->weave Opus lane dispatcher contract without
# launching agents or requiring an Opus provider. The real launcher is fail-closed; dry-run proves the
# five-lane plan artifact shape and that Codex does not silently run the lanes itself.
set -euo pipefail
fail() { echo "FAIL: $*" >&2; exit 1; }

root="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
resolve_script_under_test() {
  local rel="$1"
  local candidate
  for candidate in \
    "$root/scripts/$rel" \
    "$root/harness/skills/planning-engineer/scripts/$rel" \
    "$root/.claude/skills/planning-engineer/scripts/$rel" \
    "$root/.agents/skills/planning-engineer/scripts/$rel"; do
    [ -x "$candidate" ] && { printf '%s\n' "$candidate"; return 0; }
  done
  return 1
}
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

dispatcher="$(resolve_script_under_test plan-weave-dispatch.sh)" || { echo "FAIL: missing executable plan-weave-dispatch.sh" >&2; exit 1; }
out="$(bash "$dispatcher" --dry-run --target rusty-idd --root "$tmp" --run-id test-run --state-dir "$tmp/dispatch")"
[ -f "$out" ] || fail "dry-run did not produce dispatch jsonl: $out"
lines="$(wc -l < "$out" | tr -d ' ')"
[ "$lines" = "5" ] || fail "expected 5 dispatch lanes, got $lines"

for lane in code-graph web-trends governance settings-config rusty-idd-north-star; do
  grep -q "\"lane\":\"$lane\"" "$out" || fail "missing lane in dispatch jsonl: $lane"
  grep -q "\"worker_model\":\"claude-opus-4-8\"" "$out" || fail "missing Opus worker model"
  grep -q "\"mode\":\"dry-run\"" "$out" || fail "dry-run mode missing"
done

python3 - "$out" <<'PY'
import json, sys
rows = [json.loads(line) for line in open(sys.argv[1], encoding="utf-8")]
if len({row["lane"] for row in rows}) != 5:
    raise SystemExit("FAIL: lane names are not unique")
if any(row["status"] != "planned" for row in rows):
    raise SystemExit("FAIL: dry-run rows must be planned")
if any(row["target"] != "rusty-idd" for row in rows):
    raise SystemExit("FAIL: target was not preserved")
PY

# Real mode must fail closed when neither a PLAN_OPUS_PEER nor an executable PLAN_OPUS_CMD exists.
if PLAN_OPUS_CMD="$tmp/no-such-claude" bash "$dispatcher" --target rusty-idd --root "$tmp" --run-id fail-run --state-dir "$tmp/real" >/tmp/plan-weave-dispatch.out 2>/tmp/plan-weave-dispatch.err; then
  fail "real dispatch succeeded without an Opus route"
fi
grep -q 'NEEDS-HUMAN' /tmp/plan-weave-dispatch.err || fail "missing fail-closed NEEDS-HUMAN error"

echo "PASS: plan weave dispatcher emits five Opus lanes and fails closed without a weave/Opus route"
