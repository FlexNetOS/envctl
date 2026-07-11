#!/usr/bin/env bash
# agent-env.sh - agent-env (absorbed kasetto v3.2.0) drift gate. (TASK-0040)
#
# Closes a claimed-but-unwired enforcement: CLAUDE.md long stated "CI enforces with
# `envctl agent ... --locked`", but no gate existed because the config files were never
# migrated off the retired kasetto binary's names (kasetto.yaml/.lock), so the absorbed CLI
# could not find `agent-env.yaml` and any gate would have failed "config not found". With the
# config migrated (TASK-0040), this gate makes the enforcement real.
#
# Fail-closed: the committed agent-env.yaml must match agent-env.lock. The gate runs
# `envctl agent lock --config agent-env.yaml --check --locked`: read-only, zero-network
# (no fetch), and exits 1 on config<->lock drift. Sibling to ci/gates/{no-c,shape,enable,p7}.sh.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

if [ -x "$PWD/../../.toolchains/cargo/bin/cargo" ]; then
  export PATH="$PWD/../../.toolchains/cargo/bin:$PWD/../../usr/bin:$PATH"
fi

echo "AGENT-ENV GATE: build envctl"
cargo build -q -p envctl
BIN="${CARGO_TARGET_DIR:-target}/debug/envctl"

render_tmp="$(mktemp -d)"
trap 'rm -rf "$render_tmp"' EXIT
echo "AGENT-ENV GATE: render catalog"
"$BIN" catalog render --out "$render_tmp/catalog" --target-root "$(pwd)" >/dev/null
echo "AGENT-ENV GATE: verify Yazelix MCP mirror"
ENVCTL_RENDERED_CODEX_CONFIG="$render_tmp/catalog/.codex/config.toml" bash scripts/tests/test-agent-mcp-yazelix-mirror.sh

echo "AGENT-ENV GATE: verify agent-env lock"
if "$BIN" agent lock --config agent-env.yaml --check --locked; then
  echo "AGENT-ENV GATE PASS"
else
  echo "AGENT-ENV GATE FAIL - agent-env.yaml drifted from agent-env.lock" >&2
  echo "  fix: 'envctl agent lock --config agent-env.yaml' to rewrite the lock, then commit agent-env.lock" >&2
  exit 1
fi
