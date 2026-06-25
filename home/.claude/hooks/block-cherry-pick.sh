#!/usr/bin/env bash
# PreToolUse(Bash) guard — HARD-BLOCK every `git cherry-pick` INVOCATION.
#
# The owner has said "DO NOT CHERRY PICK" repeatedly. This hook makes it
# impossible at the tool layer (not a memory/instruction the model can forget).
#
# Precision matters: it must block RUNNING `git cherry-pick`, NOT merely
# MENTIONING the phrase (e.g. a commit message, PR body, or branch name that
# contains "cherry-pick"). So it splits the command on shell separators, strips
# quoted strings, and only fires when `cherry-pick` is the actual git subcommand.
#
# Sanctioned alternatives when a fast squash-auto-merge orphans a branch:
#   - branch off the updated base and RE-DO the edits, or
#   - `git rebase` onto the updated base, or `git merge`.  NEVER cherry-pick.
set -euo pipefail

input="$(cat)"

python3 - "$input" <<'PY'
import json, re, sys

raw = sys.argv[1] if len(sys.argv) > 1 else ""
try:
    cmd = json.loads(raw).get("tool_input", {}).get("command", "")
except Exception:
    cmd = ""

# Split into command segments on shell separators so a `git commit -m "..."`
# can't shield a later `&& git cherry-pick`.
segments = re.split(r'&&|\|\||;|\n|\|', cmd)

def strip_quotes(s):
    s = re.sub(r"'[^']*'", " ", s)   # single-quoted strings
    s = re.sub(r'"[^"]*"', " ", s)   # double-quoted strings
    return s

# A real invocation: optional env assignments, then `git`, optional global
# options (-C path / -c k=v / --git-dir=...), then `cherry-pick` as the subcommand.
INVOCATION = re.compile(r'(?:^|\s)git\b(?:\s+-{1,2}[^\s]+(?:\s+[^\s]+)?)*\s+cherry-pick\b')

hit = any(INVOCATION.search(strip_quotes(seg)) for seg in segments)

if hit:
    reason = ("\U0001F6AB\U0001F94A BLOCKED: `git cherry-pick` is FORBIDDEN here "
              "— the owner has said DO NOT CHERRY PICK, repeatedly. Use "
              "`git rebase` onto the updated base, `git merge`, or re-do the edits "
              "on a fresh branch. Do NOT cherry-pick.")
    print(json.dumps({"hookSpecificOutput": {
        "hookEventName": "PreToolUse",
        "permissionDecision": "deny",
        "permissionDecisionReason": reason,
    }}))
    print(reason, file=sys.stderr)
    sys.exit(2)

sys.exit(0)
PY
