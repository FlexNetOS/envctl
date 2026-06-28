#!/usr/bin/env bash
# agent-env.sh — agent-env (absorbed kasetto v3.2.0) drift gate. (TASK-0040)
#
# Closes a claimed-but-unwired enforcement: CLAUDE.md long stated "CI enforces with
# `envctl agent ... --locked`", but no gate existed — because the config files were never
# migrated off the retired kasetto binary's names (kasetto.yaml/.lock), so the absorbed CLI
# could not find `agent-env.yaml` and any gate would have failed "config not found". With the
# config migrated (TASK-0040), this gate makes the enforcement real.
#
# Fail-closed: the committed agent-env.yaml must match agent-env.lock. The gate runs
# `envctl agent lock --config agent-env.yaml --check --locked`: read-only, zero-network
# (no fetch), and exits 1 on config<->lock drift. Sibling to ci/gates/{no-c,shape,enable,p7}.sh.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

bash scripts/tests/test-mcp-memory-meta-root.sh

BIN=""
for c in target/release/envctl target/debug/envctl; do
  [ -x "$c" ] && BIN="$c" && break
done
if [ -z "$BIN" ]; then
  cargo build -q -p envctl
  BIN=target/debug/envctl
fi

if "$BIN" agent lock --config agent-env.yaml --check --locked; then
  echo "AGENT-ENV GATE PASS"
else
  echo "AGENT-ENV GATE FAIL — agent-env.yaml drifted from agent-env.lock" >&2
  echo "  fix: 'envctl agent lock --config agent-env.yaml' to rewrite the lock, then commit agent-env.lock" >&2
  exit 1
fi
