#!/usr/bin/env bash
set -Eeuo pipefail

PKG_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET_ROOT="${1:-$(pwd)}"
CODEX_HOME_DIR="${CODEX_HOME:-$HOME/.codex}"

mkdir -p "$CODEX_HOME_DIR/agents" "$CODEX_HOME_DIR/rules"
cp "$PKG_DIR/codex/flexnetos-migration.config.toml" "$CODEX_HOME_DIR/flexnetos-migration.config.toml"
cp "$PKG_DIR"/codex/agents/*.config.toml "$CODEX_HOME_DIR/agents/"

# Project guidance: Codex reads AGENTS.md from the project tree. Preserve an existing file.
if [[ -d "$TARGET_ROOT" ]]; then
  if [[ -f "$TARGET_ROOT/AGENTS.md" ]]; then
    cp "$TARGET_ROOT/AGENTS.md" "$TARGET_ROOT/AGENTS.md.before-flexnetos-migration-package.$(date -u +%Y%m%dT%H%M%SZ).bak"
    {
      cat "$TARGET_ROOT/AGENTS.md"
      printf '\n\n---\n\n'
      cat "$PKG_DIR/codex/AGENTS.md.template"
    } > "$TARGET_ROOT/AGENTS.md.tmp"
    mv "$TARGET_ROOT/AGENTS.md.tmp" "$TARGET_ROOT/AGENTS.md"
  else
    cp "$PKG_DIR/codex/AGENTS.md.template" "$TARGET_ROOT/AGENTS.md"
  fi
else
  echo "WARN: target root does not exist, skipped AGENTS.md install: $TARGET_ROOT" >&2
fi

echo "Installed Codex profile to $CODEX_HOME_DIR/flexnetos-migration.config.toml"
echo "Installed Spark helper agent config files to $CODEX_HOME_DIR/agents/"
echo "Project guidance installed/merged at $TARGET_ROOT/AGENTS.md if target existed."
