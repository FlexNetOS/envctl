#!/usr/bin/env python3
"""Reject executable npm/npx recipes in every text surface of a skill."""

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
TEXT_SUFFIXES = {
    ".bash",
    ".cjs",
    ".js",
    ".json",
    ".jsonc",
    ".jsx",
    ".md",
    ".mjs",
    ".nu",
    ".py",
    ".rs",
    ".sh",
    ".toml",
    ".ts",
    ".tsx",
    ".txt",
    ".yaml",
    ".yml",
    ".zsh",
}
SHELL_SUFFIXES = {".bash", ".nu", ".sh", ".zsh"}
CONFIG_SUFFIXES = {".json", ".jsonc", ".toml", ".yaml", ".yml"}
CODE_SUFFIXES = {".cjs", ".js", ".jsx", ".mjs", ".py", ".rs", ".ts", ".tsx"}
SHELL_COMMAND = re.compile(
    r"(?:^|&&|\|\||[;|]|\$\(|\bthen\b|\bdo\b)\s*"
    r"(?:command\s+|exec\s+|sudo\s+|env(?:\s+\w+=[^\s]+)*\s+)?"
    r"(?:npm\s+(?:install|i|ci|run|test|update|audit|publish|version|pkg|view|"
    r"exec|init|uninstall|pack)\b|npx(?:\s+|$))"
)
CONFIG_COMMAND_KEY = re.compile(
    r"(?i)(?:command|cmd|run|script|args|entrypoint)\s*[=:]\s*.*"
    r"(?:\bnpm\s+(?:install|i|ci|run|test|update|audit|publish|version|pkg|view|"
    r"exec|init|uninstall|pack)\b|\bnpx(?:\s+|[\"']))"
)
CODE_PROCESS_CALL = re.compile(
    r"(?i)(?:subprocess|Command|spawn|exec|system|shell|process|run)"
    r".{0,160}(?:[\"']npm[\"']|[\"']npx[\"']|[\"']npm\s|[\"']npx\s)"
)


def skill_files(root: Path):
    for path in root.rglob("*"):
        if not path.is_file():
            continue
        if any(part in SKIP_PARTS for part in path.parts):
            continue
        if not any(part in {"agent-skills", "skills"} for part in path.parts):
            continue
        if path.suffix.lower() not in TEXT_SUFFIXES:
            continue
        if path.resolve() == Path(__file__).resolve():
            # This validator necessarily contains the forbidden token patterns.
            continue
        yield path


def executable_fragments(line: str, in_fence: bool) -> list[str]:
    fragments: list[str] = []
    if in_fence or "Bash(" in line or re.search(r"\b(?:run|command)\s*:", line):
        fragments.append(line)
    fragments.extend(re.findall(r"`([^`]+)`", line))
    return fragments


def non_markdown_command(line: str, suffix: str) -> bool:
    stripped = line.strip()
    if not stripped or stripped.startswith(("#", "//", "/*", "*")):
        return False
    if suffix in SHELL_SUFFIXES:
        return bool(SHELL_COMMAND.search(line))
    if suffix in CONFIG_SUFFIXES:
        return bool(CONFIG_COMMAND_KEY.search(line))
    if suffix in CODE_SUFFIXES:
        return bool(SHELL_COMMAND.search(line) or CODE_PROCESS_CALL.search(line))
    return False


def main() -> int:
    root = Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
    findings: list[str] = []
    for path in skill_files(root):
        suffix = path.suffix.lower()
        in_fence = False
        for number, line in enumerate(path.read_text(errors="replace").splitlines(), 1):
            if suffix != ".md":
                if non_markdown_command(line, suffix):
                    findings.append(
                        f"{path.relative_to(root)}:{number}:{line.strip()}"
                    )
                continue
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
