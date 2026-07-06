#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  ./INSTALL_IN_REPOS.sh --envctl-repo PATH --nu-plugin-repo PATH

Copies additive Codex guidance into local repo prompt directories without overwriting existing files unless the destination file is this package's namespaced file.
USAGE
}

ENVCTL_REPO=""
NU_PLUGIN_REPO=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --envctl-repo) ENVCTL_REPO="${2:?}"; shift 2 ;;
    --nu-plugin-repo) NU_PLUGIN_REPO="${2:?}"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown argument: $1" >&2; usage; exit 2 ;;
  esac
done
[[ -n "$ENVCTL_REPO" && -n "$NU_PLUGIN_REPO" ]] || { usage >&2; exit 2; }
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENVCTL_REPO="$(realpath "$ENVCTL_REPO")"
NU_PLUGIN_REPO="$(realpath "$NU_PLUGIN_REPO")"
[[ -d "$ENVCTL_REPO" ]] || { echo "envctl repo not found" >&2; exit 1; }
[[ -d "$NU_PLUGIN_REPO" ]] || { echo "nu_plugin repo not found" >&2; exit 1; }

mkdir -p "$ENVCTL_REPO/.codex/prompts" "$ENVCTL_REPO/.codex/agents" "$NU_PLUGIN_REPO/.codex/prompts" "$NU_PLUGIN_REPO/.codex/agents"
cp "$SCRIPT_DIR/codex/AGENTS.envctl.md.template" "$ENVCTL_REPO/AGENTS.envctl-migration-automation.md"
cp "$SCRIPT_DIR/codex/AGENTS.nu_plugin.md.template" "$NU_PLUGIN_REPO/AGENTS.nu-plugin-migration-automation.md"
cp -R "$SCRIPT_DIR/prompts" "$ENVCTL_REPO/.codex/prompts/envctl-migration-automation"
cp -R "$SCRIPT_DIR/schemas" "$ENVCTL_REPO/.codex/prompts/envctl-migration-automation-schemas"
cp -R "$SCRIPT_DIR/sql" "$ENVCTL_REPO/.codex/prompts/envctl-migration-automation-sql"
cp -R "$SCRIPT_DIR/prompts" "$NU_PLUGIN_REPO/.codex/prompts/envctl-migration-automation"
cp -R "$SCRIPT_DIR/schemas" "$NU_PLUGIN_REPO/.codex/prompts/envctl-migration-automation-schemas"
cp -R "$SCRIPT_DIR/codex/agents" "$ENVCTL_REPO/.codex/agents/envctl-migration-automation"
cp -R "$SCRIPT_DIR/codex/agents" "$NU_PLUGIN_REPO/.codex/agents/envctl-migration-automation"

echo "Installed additive prompts into:"
echo "  $ENVCTL_REPO"
echo "  $NU_PLUGIN_REPO"
