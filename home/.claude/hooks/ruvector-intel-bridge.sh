#!/usr/bin/env bash
# ruvector-intel-bridge.sh — lifeos-wide ruvector intelligence bridge.
#
# Forwards Claude Code hook events (stdin JSON) from ANY repo under
# ~/lifeos into the shared ruvector intelligence system at
# meta-ruvector/.claude, so routing and learning accrue in ONE place
# for the whole workspace. Two stores are fed:
#
#   graph layer (ADR-050)  .claude/helpers/hook-handler.cjs
#       route context, pending insights, PageRank graph.
#       Skipped when the active project IS meta-ruvector — its own
#       project hooks already dispatch these events.
#
#   deep RL layer          .claude/intelligence/cli.js
#       Q-learning trajectories + HNSW semantic memory (post-edit /
#       post-command). Fired for EVERY project including meta-ruvector:
#       nothing else feeds this store (dormant since 2025-12-29 until
#       this bridge revived it).
#
# Runtime: bun from the nix-profile toolbin (node fallback), per the
# nix-profile-only toolchain contract.

set -u

EVENT="${1:-}"
[ -n "$EVENT" ] || exit 0

RUVECTOR_ROOT="${RUVECTOR_INTEL_ROOT:-/home/flexnetos/meta/src/meta-ruvector}"
HANDLER="$RUVECTOR_ROOT/.claude/helpers/hook-handler.cjs"
CLI="$RUVECTOR_ROOT/.claude/intelligence/cli.js"
[ -f "$HANDLER" ] || exit 0

RUNNER="$(command -v bun || true)"
[ -n "$RUNNER" ] || RUNNER="$(command -v node || true)"
[ -n "$RUNNER" ] || exit 0

IN_RUVECTOR=0
case "${CLAUDE_PROJECT_DIR:-$PWD}" in
  "$RUVECTOR_ROOT" | "$RUVECTOR_ROOT"/*) IN_RUVECTOR=1 ;;
esac

PAYLOAD="$(cat 2>/dev/null || true)"

# cd so the cwd-relative graph store (.claude-flow/data) stays unified
# in meta-ruvector rather than fragmenting one store per repo.
cd "$RUVECTOR_ROOT" 2>/dev/null || exit 0

# ── graph layer (skip inside meta-ruvector: project hooks own it) ──
if [ "$IN_RUVECTOR" -eq 0 ]; then
  printf '%s' "$PAYLOAD" | "$RUNNER" "$HANDLER" "$EVENT" || true
fi

# ── deep RL layer (always; requires jq to parse the payload) ──
command -v jq >/dev/null 2>&1 || exit 0
[ -f "$CLI" ] || exit 0
case "$EVENT" in
  post-edit)
    FILE="$(printf '%s' "$PAYLOAD" | jq -r '.tool_input.file_path // empty' 2>/dev/null || true)"
    if [ -n "$FILE" ]; then
      "$RUNNER" "$CLI" post-edit "$FILE" true >/dev/null 2>&1 || true
    fi
    ;;
  post-bash)
    CMD="$(printf '%s' "$PAYLOAD" | jq -r '.tool_input.command // empty' 2>/dev/null || true)"
    if [ -n "$CMD" ]; then
      EC="$(printf '%s' "$PAYLOAD" | jq -r '.tool_response.exit_code // .tool_response.exitCode // 0' 2>/dev/null || echo 0)"
      OK=true; [ "$EC" != "0" ] && OK=false
      ERR="$(printf '%s' "$PAYLOAD" | jq -r '.tool_response.stderr // empty' 2>/dev/null | head -c 400 || true)"
      "$RUNNER" "$CLI" post-command "$CMD" "$OK" "$ERR" >/dev/null 2>&1 || true
    fi
    ;;
esac

exit 0
