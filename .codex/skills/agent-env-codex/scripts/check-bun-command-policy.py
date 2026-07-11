#!/usr/bin/env python3
"""Reject executable npm/npx recipes in Markdown skill files."""

from __future__ import annotations

import re
import sys
from pathlib import Path

NPM_COMMAND = re.compile(
    r"\bnpm\s+(?:install|i|ci|run|test|update|audit|publish|version|pkg|view|exec|init|uninstall|pack)\b"
)
NPX_COMMAND = re.compile(r'\bnpx(?:\s+|(?=["\']))')
NEGATIVE_POLICY = re.compile(
    r"\b(?:avoid|forbid|forbidden|legacy|never|no longer|not use|retired)\b",
    re.IGNORECASE,
)
SKIP_PARTS = {".git", "node_modules", "target", "var"}


def skill_files(root: Path):
    for path in root.rglob("*.md"):
        if any(part in SKIP_PARTS for part in path.parts):
            continue
        if "skills" not in path.parts:
            continue
        yield path


def executable_fragments(line: str, in_fence: bool) -> list[str]:
    fragments: list[str] = []
    if in_fence or "Bash(" in line or re.search(r"\b(?:run|command)\s*:", line):
        fragments.append(line)
    fragments.extend(re.findall(r"`([^`]+)`", line))
    return fragments


def main() -> int:
    root = Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
    findings: list[str] = []
    for path in skill_files(root):
        in_fence = False
        for number, line in enumerate(path.read_text(errors="replace").splitlines(), 1):
            if line.lstrip().startswith("```"):
                in_fence = not in_fence
                continue
            if line.lstrip().startswith("|") and "bun" in line:
                # The explicit legacy-to-Bun mapping table is documentation,
                # not an executable recipe.
                continue
            for fragment in executable_fragments(line, in_fence):
                if NEGATIVE_POLICY.search(line) and not in_fence:
                    continue
                if NPM_COMMAND.search(fragment) or NPX_COMMAND.search(fragment):
                    findings.append(f"{path.relative_to(root)}:{number}:{line.strip()}")
                    break
    if findings:
        print("executable npm/npx skill recipes found; use bun/bunx:", file=sys.stderr)
        print("\n".join(findings), file=sys.stderr)
        return 1
    print("Bun/Bunx skill command policy: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
