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

# All active skill descriptions must stay within Codex's loader limit.
# rust-port previously exceeded 1024 chars; keep the compact metadata as a
# no-downgrade routing surface by requiring the old trigger phrases too.
python3 - "$root" <<'PY'
from pathlib import Path
import sys

root = Path(sys.argv[1])


def frontmatter(path: Path) -> str:
    text = path.read_text()
    if not text.startswith("---"):
        raise SystemExit(f"FAIL: {path} missing YAML frontmatter")
    parts = text.split("---", 2)
    if len(parts) < 3:
        raise SystemExit(f"FAIL: {path} malformed YAML frontmatter")
    return parts[1]


def description(path: Path) -> str:
    lines = frontmatter(path).splitlines()
    for idx, raw in enumerate(lines):
        if raw.startswith("description:"):
            val = raw.split(":", 1)[1].strip()
            if val in (">-", ">", "|", "|-") or not val:
                chunks = []
                for nxt in lines[idx + 1 :]:
                    if nxt and not nxt.startswith(" "):
                        break
                    chunks.append(nxt.strip())
                return " ".join(chunks).strip()
            return val.strip().strip('"')
    raise SystemExit(f"FAIL: {path} missing description")


skill_paths = sorted(root.glob(".agents/skills/*/SKILL.md")) + sorted(
    root.glob(".claude/skills/*/SKILL.md")
)
for path in skill_paths:
    desc = description(path)
    if len(desc) > 1024:
        raise SystemExit(
            f"FAIL: {path} description exceeds Codex loader limit: {len(desc)} > 1024"
        )

agents_rust = root / ".agents/skills/rust-port/SKILL.md"
claude_rust = root / ".claude/skills/rust-port/SKILL.md"
if description(agents_rust) != description(claude_rust):
    raise SystemExit("FAIL: rust-port frontmatter descriptions drifted (.agents != .claude)")

rust_desc = description(agents_rust).lower()
required = [
    "port <project> to rust",
    "rust port",
    "rewrite in rust",
    "full-parity rust port",
    "port meta/archon to rust",
    "resume",
    "continue the port",
    "run it again",
    "re-run",
    "redo only the <unit/phase>",
    "based on the previous result",
    "what's left to port",
    "install/eject the rust-port harness into <repo>",
    "port <x> to rust and merge into <y>",
    "merge the rust code into <repo>",
    "reconcile the port with <repo>",
    "opus",
    "sonnet",
    "haiku",
    "differential parity test",
    "100% parity",
]
missing = [item for item in required if item not in rust_desc]
if missing:
    raise SystemExit(
        "FAIL: rust-port compact description lost trigger(s): " + ", ".join(missing)
    )
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
