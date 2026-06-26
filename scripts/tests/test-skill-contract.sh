#!/usr/bin/env bash
# Guard hand-authored harness skill metadata/mirror contracts.
# The Claude and Codex-facing handoff-sync skills must stay byte-identical and
# short enough for skill loaders that treat frontmatter description as a routing parameter.
set -euo pipefail
root="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
fail() { echo "FAIL: $*" >&2; exit 1; }

agents_skill="$root/.agents/skills/handoff-sync/SKILL.md"
claude_skill="$root/.claude/skills/handoff-sync/SKILL.md"

cmp -s "$agents_skill" "$claude_skill" || fail "handoff-sync skill mirrors drifted (.agents != .claude)"

python3 - "$agents_skill" <<'PY'
from pathlib import Path
import sys
p = Path(sys.argv[1])
text = p.read_text()
if not text.startswith('---'):
    raise SystemExit(f'FAIL: {p} missing YAML frontmatter')
parts = text.split('---', 2)
if len(parts) < 3:
    raise SystemExit(f'FAIL: {p} malformed YAML frontmatter')
front = parts[1]
name = None
description = None
for raw in front.splitlines():
    if raw.startswith('name:'):
        name = raw.split(':', 1)[1].strip()
    if raw.startswith('description:'):
        description = raw.split(':', 1)[1].strip().strip('"')
if name != 'handoff-sync':
    raise SystemExit(f'FAIL: {p} wrong name: {name!r}')
if not description:
    raise SystemExit(f'FAIL: {p} missing description')
if len(description) > 240:
    raise SystemExit(f'FAIL: {p} description too long for routing parameter: {len(description)} > 240')
PY

# Stale doctrine that caused the failed skill must not reappear in active skill/agent config.
if grep -RInE 'redirect the shared ledger|no per-repo ledger|forbidden per-repo|There is \*\*no `hf drift`|run hf from `\$META_ROOT`|\$META_ROOT/\.handoff/ledger\.db' \
  "$root/.agents/skills/handoff-sync" \
  "$root/.claude/skills/handoff-sync" \
  "$root/.agents/agents" \
  "$root/.claude/agents" \
  "$root/.codex/agents" >/tmp/envctl-skill-contract-grep.txt 2>/dev/null; then
  cat /tmp/envctl-skill-contract-grep.txt >&2
  fail "stale handoff residency doctrine found in active skill/agent config"
fi

echo "SKILL-CONTRACT TEST PASS"
