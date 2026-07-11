#!/usr/bin/env bash
# test-agent-env-hooks.sh — hermetic contract tests for the agent-env harness hooks.
# Adopted from the codex sibling's tested-code discipline: the claude half's routing and
# capture logic must be regression-protected, not manually-verified-once.
# Network-free. Uses a temp HOME/HARNESS_VAR; never touches the live box state.
set -euo pipefail
root="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
HOOK="$root/home/.claude/hooks/bash-to-nu.py"
STOP="$root/home/.claude/hooks/ccbrain-session-stop.sh"
T=$(mktemp -d)
trap 'rm -rf "$T"' EXIT
export HARNESS_VAR="$T/var"
fail() { echo "FAIL: $1" >&2; exit 1; }

payload() { python3 -c "import json,sys; print(json.dumps({'tool_name':'Bash','tool_input':{'command':sys.argv[1]}}))" "$1"; }

# ── bash-to-nu.py ───────────────────────────────────────────────────────────
# 1. passthrough: nu-prefixed
out=$(payload "nu -l -c 'ls'" | python3 "$HOOK")
echo "$out" | grep -q updatedInput && fail "nu-prefixed command was wrapped"
# 2. passthrough: backslash escape hatch
out=$(payload "\\git status" | python3 "$HOOK")
echo "$out" | grep -q updatedInput && fail "backslash escape was wrapped"
# 3. passthrough: BASH_NU_ROUTE=0
out=$(payload "echo hi" | BASH_NU_ROUTE=0 python3 "$HOOK")
echo "$out" | grep -q updatedInput && fail "BASH_NU_ROUTE=0 did not disable routing"
# 4. wrap: plain command → nu -l -c "^bash <scratch>"; scratch holds the command
mkdir -p "$T/bin"
cat > "$T/bin/rtk" <<'STUB'
#!/usr/bin/env bash
exit 1
STUB
chmod +x "$T/bin/rtk"
out=$(payload "echo hello-wrap" | PATH="$T/bin:$PATH" python3 "$HOOK")
echo "$out" | grep -q '"command": "nu -l -c' || fail "plain command not nu-wrapped"
f=$(echo "$out" | python3 -c "import json,sys,re; c=json.load(sys.stdin)['hookSpecificOutput']['updatedInput']['command']; print(re.search(r'\\^bash ([^\"]+)',c).group(1))")
[ "$(cat "$f")" = "echo hello-wrap" ] || fail "scratch file does not hold the original command (rtk-unavailable path)"
# 5. rtk compose: stub rtk rewrites the command; scratch must hold the rewrite
cat > "$T/bin/rtk" <<'STUB'
#!/usr/bin/env bash
if [ "${1:-}" = "hook" ]; then
  cat >/dev/null
  printf '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"allow","updatedInput":{"command":"REWRITTEN-BY-RTK"}}}\n'
fi
STUB
out=$(payload "git status" | PATH="$T/bin:$PATH" python3 "$HOOK")
f=$(echo "$out" | python3 -c "import json,sys,re; c=json.load(sys.stdin)['hookSpecificOutput']['updatedInput']['command']; print(re.search(r'\\^bash ([^\"]+)',c).group(1))")
[ "$(cat "$f")" = "REWRITTEN-BY-RTK" ] || fail "rtk compose result not written to scratch"
# 6. fail-open: garbage stdin → plain allow, exit 0
out=$(echo "not-json" | python3 "$HOOK") || fail "hook non-zero on garbage stdin"
echo "$out" | grep -q '"permissionDecision": "allow"' || fail "garbage stdin did not fail open"
echo "ok: bash-to-nu.py (6 contracts)"

# ── ccbrain-session-stop.sh second pass ─────────────────────────────────────
if command -v jq >/dev/null 2>&1 && command -v sqlite3 >/dev/null 2>&1; then
  export HOME="$T/home"
  SID="test-sess-0001"
  mkdir -p "$HOME/.ccboard" "$HOME/.claude/projects/x"
  # fake session JSONL whose last assistant text carries all three prefixes
  python3 - "$HOME/.claude/projects/x/$SID.jsonl" <<'PY'
import json, sys
msg = {"type": "assistant", "message": {"content": [{"type": "text",
       "text": "PROGRESS: tested capture\nDECISION: hermetic tests adopted\nBLOCKED: nothing"}]}}
open(sys.argv[1], "w").write(json.dumps(msg) + "\n")
PY
  touch "$HOME/.ccboard/.summary_guard_${SID}"
  printf '{"session_id":"%s","cwd":"/tmp/proj"}' "$SID" | bash "$STOP" >/dev/null
  n=$(sqlite3 "$HOME/.ccboard/insights.db" "SELECT COUNT(*) FROM insights WHERE session_id='$SID';")
  [ "$n" = "3" ] || fail "expected 3 captured insights, got $n"
  # regression (the pipefail bug): a response with ONLY a BLOCKED line must not abort
  SID2="test-sess-0002"
  python3 - "$HOME/.claude/projects/x/$SID2.jsonl" <<'PY'
import json, sys
msg = {"type": "assistant", "message": {"content": [{"type": "text", "text": "BLOCKED: only this line"}]}}
open(sys.argv[1], "w").write(json.dumps(msg) + "\n")
PY
  touch "$HOME/.ccboard/.summary_guard_${SID2}"
  printf '{"session_id":"%s","cwd":"/tmp/proj"}' "$SID2" | bash "$STOP" >/dev/null || fail "stop hook aborted on missing PROGRESS field (pipefail regression)"
  n=$(sqlite3 "$HOME/.ccboard/insights.db" "SELECT COUNT(*) FROM insights WHERE session_id='$SID2' AND type='blocked';")
  [ "$n" = "1" ] || fail "BLOCKED-only response not captured (got $n)"
  echo "ok: ccbrain-session-stop.sh (capture + pipefail regression)"
else
  echo "SKIP: ccbrain capture tests (jq/sqlite3 not on PATH)"
fi

# ── syntax floors ───────────────────────────────────────────────────────────
bash -n "$root/.claude/skills/agent-env-claude/phase0.sh" || fail "phase0.sh syntax"
bash -n "$root/home/.claude/hooks/ccbrain-session-start.sh" || fail "ccbrain-session-start.sh syntax"
echo "ok: syntax floors"
echo "AGENT-ENV-HOOKS TESTS PASS"
