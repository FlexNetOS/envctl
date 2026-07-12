#!/usr/bin/env bash
# test-agent-env-hooks.sh — hermetic contract tests for the agent-env harness hooks.
# Adopted from the codex sibling's tested-code discipline: the claude half's routing and
# capture logic must be regression-protected, not manually-verified-once.
# Network-free. Uses a temp HOME/HARNESS_VAR; never touches the live box state.
set -euo pipefail
root="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
STOP="$root/home/.claude/hooks/ccbrain-session-stop.sh"
T=$(mktemp -d)
trap 'rm -rf "$T"' EXIT
export HARNESS_VAR="$T/var"
fail() { echo "FAIL: $1" >&2; exit 1; }

payload() { python3 -c "import json,sys; print(json.dumps({'tool_name':'Bash','tool_input':{'command':sys.argv[1]}}))" "$1"; }

# ── shell ownership regression ──────────────────────────────────────────────
# Yazelix is Nushell-owned and Bash is available inside that configured runtime.
# Do not reintroduce Claude-side bash-to-nu wrappers, scratch-file dispatchers,
# or parallel shell launchers.
[ ! -e "$root/home/.claude/hooks/bash-to-nu.py" ] || fail "bash-to-nu wrapper still exists"
! grep -R "bash-to-nu.py\|yazelix_nu.sh" "$root/home/.claude/settings.json" "$root/home/.claude/settings.json.tmpl" "$root/home/.claude/rules" >/tmp/envctl-bash-to-nu-grep.txt 2>/dev/null || {
  cat /tmp/envctl-bash-to-nu-grep.txt >&2
  fail "bash-to-nu wrapper still configured or documented"
}
echo "ok: no bash-to-nu wrapper"

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


# ── substrate hook parity: WL-084 weave (session/prompt/wake) + icm wired in the SOURCE ──
# The live ~/.claude/settings.json gained these from `weave setup` / icm install, but the
# ADR-0006 source (home/.claude/settings.json) must carry them too — PATH-resolved, never
# /nix/store/<hash>-pinned (stale-hash rot: a profile rebuild leaves hooks firing old builds).
for pat in "weave hook session" "weave hook prompt" "weave hook wake" \
           "icm hook start" "icm hook pre" "icm hook post" "icm hook prompt" "icm hook end" "icm hook compact"; do
  grep -qF "$pat" "$root/home/.claude/settings.json" || fail "repo settings.json missing substrate hook: $pat"
done
! grep -q "/nix/store/" "$root/home/.claude/settings.json" "$root/home/.claude/settings.json.tmpl" || fail "store-hash path pinned in settings (stale-hash rot class)"
echo "ok: substrate hook parity (weave WL-084 + icm) PATH-resolved in source settings"

# ── settings residuals: live-only keys adopted; owner ruling on root-var wiring honored ──
for f in "$root/home/.claude/settings.json" "$root/home/.claude/settings.json.tmpl"; do
  grep -q '"effortLevel"' "$f" || fail "$(basename "$f") missing effortLevel (live-only key never adopted into source)"
  # Owner ruling 2026-07-07 (env_cmd_tests::settings_json_matches_rendered_tmpl_no_drift):
  # no META_ROOT/LIFEOS_ROOT/placeholder wiring in session config — explicit real paths only.
  for banned in 'META_ROOT' 'LIFEOS_ROOT' 'META_FILE' '${'; do
    ! grep -qF "$banned" "$f" || fail "$(basename "$f") contains banned root-var wiring: $banned"
  done
done
# Operator ratification 2026-07-12 (decision marker settings-defaultmode-auto-persistence.answered):
# permissions.defaultMode:"auto" is DECLARED state — a Tier-B toggle an operator may ratify into
# durable config; it must survive re-renders in both files.
for f in "$root/home/.claude/settings.json" "$root/home/.claude/settings.json.tmpl"; do
  python3 -c "import json,sys; d=json.load(open(sys.argv[1])); sys.exit(0 if d.get('permissions',{}).get('defaultMode')=='auto' else 1)" "$f" \
    || fail "$(basename "$f") missing ratified permissions.defaultMode=auto"
done
diff -q "$root/home/.claude/settings.json.tmpl" "$root/home/.claude/settings.json" >/dev/null \
  || fail "settings.json.tmpl != settings.json (identity-render drift; the Rust gate requires byte-parity)"
echo "ok: settings residuals (effortLevel adopted; no root-var wiring; tmpl==settings byte-parity)"

# ── syntax floors ───────────────────────────────────────────────────────────
bash -n "$root/.claude/skills/agent-env-claude/phase0.sh" || fail "phase0.sh syntax"
bash -n "$root/home/.claude/hooks/ccbrain-session-start.sh" || fail "ccbrain-session-start.sh syntax"
echo "ok: syntax floors"
echo "AGENT-ENV-HOOKS TESTS PASS"
