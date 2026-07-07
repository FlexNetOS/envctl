#!/usr/bin/env bash
# config-audit.sh — ConfigChange. Every config mutation lands in the ledger.
set -u
. "$(dirname "$0")/lib.sh"

INPUT=$(cat)
SRC=""; FP=""
if have_jq; then
  SRC=$(printf '%s' "$INPUT" | jq -r '.source // empty' 2>/dev/null)
  FP=$(printf '%s' "$INPUT" | jq -r '.file_path // empty' 2>/dev/null)
fi
ledger "config.change" "\"source\":\"$(json_escape "${SRC:-?}")\",\"file\":\"$(json_escape "${FP:-?}")\""
exit 0
