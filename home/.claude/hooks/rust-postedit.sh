#!/usr/bin/env bash
# rust-postedit.sh — async PostToolUse[Edit|Write]. rustfmt check on touched .rs
# files. Fail-open: missing toolchain must never block editing.
set -u
. "$(dirname "$0")/lib.sh"

INPUT=$(cat)
FP=""
if have_jq; then FP=$(printf '%s' "$INPUT" | jq -r '.tool_input.file_path // empty' 2>/dev/null); fi
case "${FP:-}" in *.rs) ;; *) exit 0 ;; esac

PATH="$HOME/.cargo/bin:$PATH"
command -v rustfmt >/dev/null 2>&1 || { ledger "rust.skip" "\"reason\":\"no-rustfmt\""; exit 0; }

if ! OUT=$(rustfmt --check --edition 2021 "$FP" 2>&1); then
  ledger "rust.fmt-drift" "\"file\":\"$(json_escape "$FP")\""
  echo "rustfmt drift in $FP — run cargo fmt (or rustfmt '$FP')." >&2
  exit 0
fi
ledger "rust.fmt-ok" "\"file\":\"$(json_escape "$FP")\""
exit 0
