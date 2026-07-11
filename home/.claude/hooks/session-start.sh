#!/usr/bin/env bash
# session-start.sh — SessionStart. Ledger + weave handshake (skip-if-absent) +
# re-injection of the operating LAWS as session context (stdout → context).
set -u
. "$(dirname "$0")/lib.sh"

cat >/dev/null

if command -v weave >/dev/null 2>&1; then
  weave hook session 2>/dev/null || true
  WEAVE="present"
else
  WEAVE="absent"
fi
ledger "session.start" "\"weave\":\"$WEAVE\""

cat <<'EOF'
FlexNetOS operating LAWS (harness-enforced, non-negotiable):
1. NEVER DELETE — ALWAYS ARCHIVE (~/.claude/hooks/harness-archive.sh; archives in ~/.claude/archive/).
2. UPGRADE ONLY — no config/feature/capability regressions; superset merges only.
3. HEAL, DO NOT HARM — stop and ask before risking a working system.
4. REAL EXECUTION ONLY — "done" requires observed command output.
5. NO NEW documents/reports — terminal reporting only; operational config files are fine.
6. Git topology: only main/master and develop are long-lived; PRs target develop.
7. CONTAINMENT: no nested Claude sessions; subagents never spawn agents; max 6 active agents; budget ceiling 80% rate-limit.
8. STOP MEANS STOP — decisions go through AskUserQuestion and block until answered.
Harness state: ledger /home/flexnetos/meta/var/log/claude-harness/ledger.jsonl (append-only), kill switch: /home/flexnetos/meta/src/envctl/home/bin/harness-halt.sh (full path; not on PATH).
EOF
exit 0
