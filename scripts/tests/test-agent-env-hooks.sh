#!/usr/bin/env bash
set -euo pipefail
root="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }

[ ! -e "$root/home/.claude/hooks" ] || fail "retired copied Claude hook directory remains"

for path in "$root/home/.claude/settings.json" "$root/home/.claude/settings.json.tmpl"; do
  python3 - "$path" <<'PY'
import json, sys
data = json.load(open(sys.argv[1]))
expected = {
    "SessionStart": ["icm hook start"],
    "PreToolUse": ["rtk hook claude", "icm hook pre"],
    "PostToolUse": ["icm hook post"],
    "UserPromptSubmit": ["icm hook prompt"],
    "SessionEnd": ["icm hook end"],
    "PreCompact": ["icm hook compact"],
}
actual = {event: [h.get("command") for rule in rules for h in rule.get("hooks", [])]
          for event, rules in data.get("hooks", {}).items()}
if actual != expected:
    raise SystemExit(f"approved RTK/ICM hook contract mismatch: {actual!r}")
if any(".claude/hooks" in command or "/nix/store/" in command
       for commands in actual.values() for command in commands):
    raise SystemExit("retired copied or store-pinned hook command remains")
PY
done
cmp -s "$root/home/.claude/settings.json" "$root/home/.claude/settings.json.tmpl" \
  || fail "settings template and source diverged"
printf 'AGENT-ENV APPROVED HOOK CONTRACT PASS\n'
