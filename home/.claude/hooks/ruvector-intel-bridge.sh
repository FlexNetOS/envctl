#!/usr/bin/env bash
# ruvector-intel-bridge.sh — lifeos-wide ruvector intelligence bridge.
#
# Forwards Claude Code hook events (stdin JSON) from ANY repo under
# ~/lifeos into the shared ruvector intelligence layer at
# meta-ruvector/.claude (helpers/hook-handler.cjs), so task routing and
# edit/command learning accrue in ONE store for the whole workspace
# instead of only inside meta-ruvector sessions.
#
# Skips when the active project IS meta-ruvector — its own project hooks
# already dispatch the same events, and double-dispatch would
# double-count learning signals.
#
# Runtime: bun from the nix-profile toolbin (node fallback), per the
# nix-profile-only toolchain contract.

set -u

EVENT="${1:-}"
[ -n "$EVENT" ] || exit 0

RUVECTOR_ROOT="${RUVECTOR_INTEL_ROOT:-/home/flexnetos/lifeos/src/meta-ruvector}"
HANDLER="$RUVECTOR_ROOT/.claude/helpers/hook-handler.cjs"
[ -f "$HANDLER" ] || exit 0

case "${CLAUDE_PROJECT_DIR:-$PWD}" in
  "$RUVECTOR_ROOT" | "$RUVECTOR_ROOT"/*) exit 0 ;;
esac

RUNNER="$(command -v bun || true)"
[ -n "$RUNNER" ] || RUNNER="$(command -v node || true)"
[ -n "$RUNNER" ] || exit 0

# cd so the cwd-relative graph store (.claude-flow/data) stays unified
# in meta-ruvector rather than fragmenting one store per repo.
cd "$RUVECTOR_ROOT" 2>/dev/null || exit 0
exec "$RUNNER" "$HANDLER" "$EVENT"
