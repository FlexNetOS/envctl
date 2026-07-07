#!/usr/bin/env bash
# harness-archive.sh — the sanctioned replacement for rm on user data (LAW 1).
# Moves each given path into ~/.claude/archive/<UTC>/rm-redirect/<absolute-path>.
set -eu
ARCHIVE_ROOT="$HOME/.claude/archive"
TS=$(date -u +%Y%m%dT%H%M%SZ)
[ $# -ge 1 ] || { echo "usage: harness-archive.sh <path>..." >&2; exit 2; }
for p in "$@"; do
  [ -e "$p" ] || [ -L "$p" ] || { echo "skip (absent): $p" >&2; continue; }
  abs=$(readlink -f "$p" 2>/dev/null || printf '%s' "$p")
  dest="$ARCHIVE_ROOT/$TS/rm-redirect$abs"
  mkdir -p "$(dirname "$dest")"
  mv "$p" "$dest"
  echo "archived: $p -> $dest"
done
