#!/usr/bin/env bash
# canonical: scripts/tests/blueprint/install.sh
# Installs the R1 swarm-immune fix into the live ruvector runtime and stages the
# runnable bun/psql blueprint tests under var/lib/ruvector/tests/.
#
#   * Law-1: the live wrapper is ARCHIVED (never deleted) before it is replaced.
#   * Idempotent: re-running with the fix already in place is a no-op archive
#     (nothing is being replaced, so nothing new is archived) + identical copy.
#   * Targets var/lib/ruvector/ and ~/.claude/archive/ ONLY — never usr/bin, so it
#     stays clear of the meta-local-policy gate (which flags usr/bin frontdoor
#     writes).
#
# This script is the RED->GREEN transition mechanism for T1: before it runs, the
# live wrapper is the broken (string-name) version and T1 is RED; after it runs,
# the live wrapper is the fixed version and T1 is GREEN.
set -euo pipefail

SELF_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUV="${RUVECTOR_DIR:-/home/flexnetos/lifeos/var/lib/ruvector}"
LIVE="$RUV/swarm-immune.mjs"
CANON="$SELF_DIR/runtime/swarm-immune.mjs"
TESTS_DST="$RUV/tests"
ARCHIVE_ROOT="${HARNESS_ARCHIVE_ROOT:-$HOME/.claude/archive}"

[ -f "$CANON" ] || { echo "install: canonical wrapper missing: $CANON" >&2; exit 1; }

echo "== install.sh: R1 swarm-immune fix + blueprint test staging =="

# --- Law-1 archive, then install the canonical fix over the live path ---
if [ -f "$LIVE" ] && ! cmp -s "$LIVE" "$CANON"; then
  ts="$(date -u +%Y%m%dT%H%M%SZ)"
  ad="$ARCHIVE_ROOT/$ts/var-lib-ruvector"
  mkdir -p "$ad"
  cp -a "$LIVE" "$ad/swarm-immune.mjs"
  echo "archived live wrapper -> $ad/swarm-immune.mjs (Law-1)"
elif [ -f "$LIVE" ]; then
  echo "live wrapper already matches the fix — no archive needed (idempotent)"
else
  echo "no live wrapper present — fresh install"
fi

install -m 0644 "$CANON" "$LIVE"
echo "installed R1 fix -> $LIVE"

# --- stage the runnable bun/psql blueprint tests into the runtime test home ---
mkdir -p "$TESTS_DST" "$TESTS_DST/fixtures"
install -m 0644 "$SELF_DIR/t1_swarm_immune.mjs"        "$TESTS_DST/t1_swarm_immune.mjs"
install -m 0644 "$SELF_DIR/t4_router_discrimination.mjs" "$TESTS_DST/t4_router_discrimination.mjs"
install -m 0755 "$SELF_DIR/t3_embedder_wiring.sh"      "$TESTS_DST/t3_embedder_wiring.sh"
install -m 0644 "$SELF_DIR/fixtures/router_prompts.json" "$TESTS_DST/fixtures/router_prompts.json"
echo "staged bun/psql tests -> $TESTS_DST/ (t1, t4, t3 + fixtures/router_prompts.json)"

echo "install.sh complete"
