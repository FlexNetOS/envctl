#!/usr/bin/env bash
set -Eeuo pipefail

PKG_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PRIMARY_INPUT="${1:-/home/flexnetos/FlexNetOS}"
COMPARE_INPUT="${2:-/home/flexnetos/lifeos}"

expand_path() {
  local p="$1"
  if [[ "$p" == ~/* ]]; then
    printf '%s/%s' "$HOME" "${p#~/}"
  else
    printf '%s' "$p"
  fi
}

PRIMARY_ROOT="$(expand_path "$PRIMARY_INPUT")"
COMPARE_ROOT="$(expand_path "$COMPARE_INPUT")"

if ! command -v codex >/dev/null 2>&1; then
  echo "ERROR: codex CLI not found in PATH." >&2
  echo "Install Codex CLI first, then rerun this script." >&2
  exit 127
fi

mkdir -p "$PKG_DIR/.run"
RUN_CONTEXT="$PKG_DIR/.run/run-context.env"
cat > "$RUN_CONTEXT" <<EOF
PROMPT_PACKAGE_DIR=$PKG_DIR
PRIMARY_ROOT=$PRIMARY_ROOT
COMPARE_ROOT=$COMPARE_ROOT
EXPECTED_ORCHESTRATOR_MODEL=codex-5.4
EXPECTED_HELPER_MODEL=spark-5.3
EOF

if [[ ! -d "$PRIMARY_ROOT" ]]; then
  echo "ERROR: primary root does not exist: $PRIMARY_ROOT" >&2
  echo "Try: ./RUN_WITH_CODEX.sh \"$HOME/home/flexnetos/FlexNetOS\" \"$HOME/home/flexnetos/lifeos\"" >&2
  exit 2
fi

cd "$PRIMARY_ROOT"

# Feed the master prompt through stdin so the prompt can remain version-controlled as a file.
# The prompt itself instructs Codex to verify model availability and stop if Codex 5.4 / Spark 5.3 cannot be resolved.
{
  printf 'RUN_CONTEXT_FILE=%s\n' "$RUN_CONTEXT"
  printf 'PRIMARY_ROOT=%s\n' "$PRIMARY_ROOT"
  printf 'COMPARE_ROOT=%s\n' "$COMPARE_ROOT"
  printf 'PROMPT_PACKAGE_DIR=%s\n\n' "$PKG_DIR"
  cat "$PKG_DIR/prompts/MASTER_PROMPT.md"
} | codex exec - \
    --model codex-5.4 \
    --sandbox workspace-write \
    --ask-for-approval on-request \
    --cd "$PRIMARY_ROOT" \
    --profile flexnetos-migration
