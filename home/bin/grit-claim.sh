#!/usr/bin/env bash
# grit-claim.sh — harness entrypoint for parallel CODE coordination via grit 0.5.0.
#
# grit locks AST `file::symbol` scopes (NOT config files like settings.json — those
# are coordinated by worktree isolation, not grit). Use this before editing code that
# a parallel agent/session might also touch, so two agents don't clobber the same
# function in a shared checkout.
#
# Agent identity defaults to the Claude session id so locks are attributable in
# `grit status`. FAIL-OPEN: if grit is not installed, print a notice and exit 0 —
# never block work on a missing coordinator.
#
# Locks expire at their TTL (default 600s) unless refreshed: run `grit-claim.sh
# heartbeat` (or pass a longer GRIT_TTL) for long edits.
set -u

AGENT="${GRIT_AGENT:-${CLAUDE_SESSION_ID:-claude-$$}}"
TTL="${GRIT_TTL:-600}"

if ! command -v grit >/dev/null 2>&1; then
  echo "grit not installed — parallel-code coordination skipped." >&2
  echo "install: nix profile install path:/home/flexnetos/lifeos/src/grit" >&2
  exit 0
fi

usage() {
  cat >&2 <<EOF
grit-claim.sh — parallel code coordination (agent=$AGENT, ttl=${TTL}s)
usage:
  grit-claim.sh claim  <intent> <file::symbol>...   claim symbols (auto-inits grit)
  grit-claim.sh assign <intent> <file>              auto-pick + claim a free symbol in <file>
  grit-claim.sh release <file::symbol>...           release specific symbols
  grit-claim.sh done                                release ALL this agent's locks (+ merge worktree)
  grit-claim.sh status                              show current locks
  grit-claim.sh heartbeat                           refresh this agent's lock TTL
env: GRIT_AGENT (identity), GRIT_TTL (seconds)
EOF
  exit 2
}

cmd="${1:-status}"; shift 2>/dev/null || true
case "$cmd" in
  claim)     [ $# -ge 2 ] || usage; intent="$1"; shift
             [ -d .grit ] || grit init >/dev/null 2>&1 || true
             exec grit claim --agent "$AGENT" --intent "$intent" --ttl "$TTL" "$@" ;;
  assign)    [ $# -ge 2 ] || usage; intent="$1"; shift
             [ -d .grit ] || grit init >/dev/null 2>&1 || true
             exec grit assign --agent "$AGENT" --intent "$intent" --file "$1" --ttl "$TTL" ;;
  release)   [ $# -ge 1 ] || usage; exec grit release --agent "$AGENT" "$@" ;;
  done)      exec grit done --agent "$AGENT" ;;
  status)    exec grit status ;;
  heartbeat) exec grit heartbeat --agent "$AGENT" --ttl "$TTL" ;;
  *)         usage ;;
esac
