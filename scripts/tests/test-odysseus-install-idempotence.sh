#!/usr/bin/env bash
# Guard the Odysseus installer against regressing to chmod/chown of existing rootless-Podman
# volume trees. This is intentionally static and hermetic: the real install can rebuild containers,
# but CI only needs to prove the manifest keeps the rootless-idempotence contract.
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
MANIFEST="$ROOT/manifest/odysseus.toml"
DOC="$ROOT/docs/odysseus-adoption.md"

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

grep -Fq 'odysseus_ensure_state_dir()' "$MANIFEST" ||
  fail "manifest must use an idempotent state-dir helper"
grep -Fq 'if [ -d "$d" ]; then' "$MANIFEST" ||
  fail "state-dir helper must return before mutating existing dirs"
grep -Fq 'podman unshare mkdir -p "$d"' "$MANIFEST" ||
  fail "state-dir helper must have a rootless Podman namespace fallback"

if grep -Fq 'install -d -m 700 "$ROOT" "$DATA"' "$MANIFEST"; then
  fail "installer must not chmod existing DATA/LOGS via a single install -d invocation"
fi

if grep -En '\bch(own|mod)\s+-R\b.*(\$DATA|\$LOGS|odysseus)' "$MANIFEST"; then
  fail "installer must not recursively chown/chmod Odysseus state"
fi

grep -Fq 'does not chmod/chown existing data/log trees' "$DOC" ||
  fail "docs must record the Odysseus re-run idempotence contract"

echo "ODYSSEUS-INSTALL-IDEMPOTENCE TEST PASS"
