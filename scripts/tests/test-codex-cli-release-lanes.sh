#!/usr/bin/env bash
# test-codex-cli-release-lanes.sh — guard envctl's Node-independent Codex Rust release lanes.
set -euo pipefail

ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
MANIFEST="$ROOT/manifest/ai-clis.toml"

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

grep -Fq 'pinned upstream release rust-v0.142.3' "$MANIFEST" \
  || fail "stable Codex CLI component must pin rust-v0.142.3"

if grep -Fq '0.142.0' "$MANIFEST"; then
  grep -Fn '0.142.0' "$MANIFEST" >&2
  fail "stale rust-v0.142.0 default remains"
fi

stable_defaults="$(grep -Fc 'CODEX_VERSION:-0.142.3' "$MANIFEST")"
[ "$stable_defaults" -ge 3 ] \
  || fail "stable component must use CODEX_VERSION override with 0.142.3 default in detect/install/fix"

grep -Fq 'id = "codex-cli-alpha"' "$MANIFEST" \
  || fail "missing opt-in codex-cli-alpha component"
grep -Fq 'CODEX_ALPHA_VERSION:-0.143.0-alpha.29' "$MANIFEST" \
  || fail "alpha lane must pin current candidate rust-v0.143.0-alpha.29 behind CODEX_ALPHA_VERSION"
grep -Fq 'envctl codex alpha wrapper' "$MANIFEST" \
  || fail "alpha lane must expose a distinct codex-alpha wrapper"
grep -Fq 'codex-alpha' "$MANIFEST" \
  || fail "alpha lane must expose codex-alpha without repointing codex"

python3 - "$MANIFEST" <<'PY'
import sys
import tomllib
from pathlib import Path

manifest = Path(sys.argv[1])
data = tomllib.loads(manifest.read_text())
components = {component["id"]: component for component in data["component"]}
for required in ("codex-cli", "codex-cli-alpha"):
    if required not in components:
        raise SystemExit(f"missing component {required}")

alpha_text = manifest.read_text().split('id = "codex-cli-alpha"', 1)[1].split("[[component]]", 1)[0]
for forbidden in (
    'ln -sfn "$VER" "$CUR"',
    'openai-codex/current/bin/codex',
    'CODEX_BIN_PATH="${CODEX_BIN_PATH:-$META_ROOT/.toolchains/openai-codex/${VER}/bin/codex}"',
):
    if forbidden in alpha_text:
        raise SystemExit(f"alpha lane must not repoint or use stable current: {forbidden}")

stable_remove = " ".join(components["codex-cli"]["remove"]["args"])
if '@openai/codex' not in stable_remove:
    raise SystemExit("stable remove path must still clean stale npm/Bun @openai/codex shims")
if "timeout --kill-after=2s 20s bun remove -g @openai/codex" not in manifest.read_text():
    raise SystemExit("stale npm/Bun cleanup must be timeout-bounded and non-blocking")
PY

echo "PASS: Codex Rust release lanes are Node-independent and alpha is opt-in"
