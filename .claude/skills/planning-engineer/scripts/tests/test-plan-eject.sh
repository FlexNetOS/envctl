#!/usr/bin/env bash
# test-plan-eject.sh — proves the packaged planning-engineer eject.sh upholds its no-downgrade
# semantics when installing into a target repo's .claude/:
#   * OWN skills+agents are ALWAYS (re)copied (a re-eject refreshes them — sentinel is wiped)
#   * SHARED infra present in the target is NEVER clobbered (a target's canonical version survives)
#   * SHARED infra ABSENT in the target IS copied in (only if the hub plugin actually carries it)
#   * the .handoff/loop/plan/{graph,research,findings,reports} scaffold is created
#   * a nonexistent target dir is rejected (non-zero exit)
#
# Hermetic: runs the REAL eject.sh IN PLACE (it derives its plugin root from its own path) against a
# fresh tmpdir TARGET, then asserts the on-disk outcome. No network, no real workspace touched.
set -euo pipefail

fail() { echo "FAIL: $*" >&2; exit 1; }
# Locate the packaged planning-engineer eject.sh regardless of which of the two byte-identical copies
# is running (mirrored into envctl/scripts/tests/ and the harness_hub plugin). Walk up from this
# script to the meta-worktree root (holding both envctl/ and harness_hub/) and descend to the plugin.
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; root="$here"
REL="harness_hub/harness/skills/planning-engineer/scripts/eject.sh"
while [ "$root" != "/" ] && [ ! -f "$root/$REL" ]; do root="$(dirname "$root")"; done
EJECT="$root/$REL"
# Fallback for envctl standalone CI (no meta-worktree root; only the ejected .claude copy is present):
[ -f "$EJECT" ] || EJECT="$(git -C "$here" rev-parse --show-toplevel 2>/dev/null)/.claude/skills/planning-engineer/scripts/eject.sh"
[ -f "$EJECT" ] || { echo "FAIL: planning-engineer eject.sh not found from $here" >&2; exit 1; }
PLUGIN="$(cd "$(dirname "$EJECT")/../../.." && pwd)"   # harness/  — same root eject.sh computes

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
TARGET="$tmp/target"
mkdir -p "$TARGET"

run_eject() { bash "$EJECT" "$TARGET" >/dev/null 2>&1; }

# 1. first eject — OWN skills + agents must land
run_eject || fail "first eject exited non-zero"
for s in planning-engineer plan-loop plan-cartography plan-trend-research plan-synthesis code-research-verify; do
  [ -d "$TARGET/.claude/skills/$s" ] || fail "OWN skill not copied: $s"
done
for a in plan-cartographer plan-trend-researcher plan-analyst plan-verifier plan-architect; do
  [ -f "$TARGET/.claude/agents/$a.md" ] || fail "OWN agent not copied: $a"
done

# 2. scaffold — the plan state tree must exist
for d in graph research findings reports; do
  [ -d "$TARGET/.handoff/loop/plan/$d" ] || fail "scaffold dir missing: .handoff/loop/plan/$d"
done

# 3. OWN overwrites on re-eject — drop a sentinel into an OWN skill, re-eject, sentinel must vanish
SENTINEL="__SENTINEL_$(date +%s)_$$__"
echo "$SENTINEL" >> "$TARGET/.claude/skills/planning-engineer/SKILL.md"
grep -q "$SENTINEL" "$TARGET/.claude/skills/planning-engineer/SKILL.md" || fail "could not seed OWN sentinel"
run_eject || fail "re-eject exited non-zero"
grep -q "$SENTINEL" "$TARGET/.claude/skills/planning-engineer/SKILL.md" \
  && fail "OWN skill was NOT refreshed on re-eject (sentinel survived)" || true

# 4. SHARED present is never clobbered — pre-seed a shared skill + a shared agent with sentinels,
#    then re-eject and assert BOTH sentinels SURVIVE (the existing canonical version is kept).
SH_SK_SENT="__SHARED_SKILL_KEEP_$$__"
SH_AG_SENT="__SHARED_AGENT_KEEP_$$__"
echo "$SH_SK_SENT" > "$TARGET/.claude/skills/session-relay-wrap-up/SKILL.md"   # dir already exists (eject'd it)
echo "$SH_AG_SENT" > "$TARGET/.claude/agents/evolution-steward.md"
run_eject || fail "shared-keep eject exited non-zero"
grep -q "$SH_SK_SENT" "$TARGET/.claude/skills/session-relay-wrap-up/SKILL.md" \
  || fail "SHARED skill was clobbered (sentinel lost — no-downgrade violated)"
grep -q "$SH_AG_SENT" "$TARGET/.claude/agents/evolution-steward.md" \
  || fail "SHARED agent was clobbered (sentinel lost — no-downgrade violated)"

# 5. SHARED absent gets copied — into a FRESH target so the shared item is genuinely absent at eject.
#    Pick a shared SKILL that EXISTS in the hub plugin (check the hub first); SKIP with a note if none.
SHARED_CAND=""
for cand in session-relay-wrap-up session-relay-resume harness-loop-init harness-evolution icm-memory; do
  [ -d "$PLUGIN/skills/$cand" ] && { SHARED_CAND="$cand"; break; }
done
if [ -n "$SHARED_CAND" ]; then
  TARGET2="$tmp/target2"; mkdir -p "$TARGET2"
  [ ! -d "$TARGET2/.claude/skills/$SHARED_CAND" ] || fail "fresh target unexpectedly already has $SHARED_CAND"
  bash "$EJECT" "$TARGET2" >/dev/null 2>&1 || fail "fresh-target eject exited non-zero"
  [ -d "$TARGET2/.claude/skills/$SHARED_CAND" ] \
    || fail "SHARED skill absent-in-target was NOT copied on eject: $SHARED_CAND"
  echo "  shared absent->copied verified via: $SHARED_CAND"
else
  echo "  NOTE: hub carries no shared skill — SHARED-when-absent sub-check SKIPPED"
fi

# 6. error path — nonexistent target dir must be rejected
if bash "$EJECT" "$tmp/does-not-exist-$$" >/dev/null 2>&1; then
  fail "eject accepted a nonexistent target dir (should exit non-zero)"
fi

echo "PASS: eject (re)copies OWN, refreshes OWN on re-eject, preserves present SHARED, copies absent SHARED, scaffolds plan tree, rejects bad target"
