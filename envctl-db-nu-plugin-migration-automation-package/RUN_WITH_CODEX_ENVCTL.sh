#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  ./RUN_WITH_CODEX_ENVCTL.sh --envctl-repo PATH --nu-plugin-repo PATH [--flexnetos-package PATH] [--codex-bin codex]

Purpose:
  Run Codex locally with the envctl database + nu_plugin migration automation master prompt.

Required:
  --envctl-repo       Path to the local envctl repository.
  --nu-plugin-repo    Path to the local nu_plugin repository.

Optional:
  --flexnetos-package Path to prior codex-flexnetos-migration-prompt-package.
  --codex-bin         Codex executable name/path. Default: codex.
USAGE
}

ENVCTL_REPO=""
NU_PLUGIN_REPO=""
FLEXNETOS_PACKAGE=""
CODEX_BIN="codex"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --envctl-repo) ENVCTL_REPO="${2:?}"; shift 2 ;;
    --nu-plugin-repo) NU_PLUGIN_REPO="${2:?}"; shift 2 ;;
    --flexnetos-package) FLEXNETOS_PACKAGE="${2:?}"; shift 2 ;;
    --codex-bin) CODEX_BIN="${2:?}"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown argument: $1" >&2; usage; exit 2 ;;
  esac
done

if [[ -z "$ENVCTL_REPO" || -z "$NU_PLUGIN_REPO" ]]; then
  usage >&2
  exit 2
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENVCTL_REPO="$(realpath "$ENVCTL_REPO")"
NU_PLUGIN_REPO="$(realpath "$NU_PLUGIN_REPO")"
if [[ -z "$FLEXNETOS_PACKAGE" ]]; then
  FLEXNETOS_PACKAGE="$SCRIPT_DIR/source/codex-flexnetos-migration-prompt-package"
fi
FLEXNETOS_PACKAGE="$(realpath "$FLEXNETOS_PACKAGE")"

[[ -d "$ENVCTL_REPO" ]] || { echo "envctl repo not found: $ENVCTL_REPO" >&2; exit 1; }
[[ -d "$NU_PLUGIN_REPO" ]] || { echo "nu_plugin repo not found: $NU_PLUGIN_REPO" >&2; exit 1; }
[[ -d "$FLEXNETOS_PACKAGE" ]] || { echo "FlexNetOS package not found: $FLEXNETOS_PACKAGE" >&2; exit 1; }
command -v "$CODEX_BIN" >/dev/null 2>&1 || { echo "Codex binary not found: $CODEX_BIN" >&2; exit 1; }

RUN_ID="$(date -u +%Y%m%dT%H%M%SZ)-envctl-nu-plugin-migration"
RUN_CONTEXT="$SCRIPT_DIR/.run-context.$RUN_ID.env"
cat > "$RUN_CONTEXT" <<EOF
RUN_ID=$RUN_ID
PROMPT_PACKAGE_DIR=$SCRIPT_DIR
ENVCTL_REPO=$ENVCTL_REPO
NU_PLUGIN_REPO=$NU_PLUGIN_REPO
FLEXNETOS_PACKAGE=$FLEXNETOS_PACKAGE
EOF

TMP_PROMPT="$(mktemp)"
{
  echo "RUN_CONTEXT_FILE=$RUN_CONTEXT"
  echo "PROMPT_PACKAGE_DIR=$SCRIPT_DIR"
  echo "ENVCTL_REPO=$ENVCTL_REPO"
  echo "NU_PLUGIN_REPO=$NU_PLUGIN_REPO"
  echo "FLEXNETOS_PACKAGE=$FLEXNETOS_PACKAGE"
  echo
  cat "$SCRIPT_DIR/PROMPT_PACKAGE_COMBINED.md"
} > "$TMP_PROMPT"

echo "Running Codex with envctl repo: $ENVCTL_REPO"
echo "Running Codex with nu_plugin repo: $NU_PLUGIN_REPO"
echo "Run context: $RUN_CONTEXT"

cd "$ENVCTL_REPO"
"$CODEX_BIN" exec \
  --config "$SCRIPT_DIR/codex/envctl-nu-plugin-migration.config.toml" \
  --sandbox workspace-write \
  --ask-for-approval on-request \
  - < "$TMP_PROMPT"

rm -f "$TMP_PROMPT"
