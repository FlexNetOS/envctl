#!/usr/bin/env bash
# test-codex-cli-release-lanes.sh — guard the single profile-owned Codex release lane.
set -euo pipefail

ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
MANIFEST="$ROOT/manifest/ai-clis.toml"

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

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

stable_text = manifest.read_text().split('id = "codex-cli"', 1)[1].split("[[component]]", 1)[0]
if components["codex-cli"].get("requires") != ["yazelix"]:
    raise SystemExit("stable Codex must delegate ownership exclusively to the Yazelix profile")
if components["codex-cli-alpha"].get("requires") != ["codex-cli"]:
    raise SystemExit("the retired alpha compatibility component must depend on stable profile ownership")

alpha_text = manifest.read_text().split('id = "codex-cli-alpha"', 1)[1].split("[[component]]", 1)[0]
for forbidden in (
    "CODEX_VERSION",
    "CODEX_ALPHA_VERSION",
    "0.142.3",
    "0.143.0-alpha.29",
    "releases/download/rust-v",
    ".toolchains/openai-codex",
    'LINK="$M/usr/bin/codex"',
    "envctl codex wrapper",
    "envctl codex alpha wrapper",
    "codex-alpha",
    "CODEX_BUILD_FROM_SOURCE",
    "CODEX_CARGO_JOBS",
    "CARGO_PROFILE_RELEASE_LTO",
    "CARGO_PROFILE_RELEASE_CODEGEN_UNITS",
    "CARGO_PROFILE_RELEASE_INCREMENTAL",
):
    if forbidden in stable_text or forbidden in alpha_text:
        raise SystemExit(f"Codex must not retain a parallel Meta-root release lane: {forbidden}")

for component_id in ("codex-cli", "codex-cli-alpha"):
    for phase in ("detect", "install", "verify", "fix", "remove"):
        hook = components[component_id][phase]
        if hook.get("kind") != "shipped_script":
            raise SystemExit(f"{component_id} {phase} must use the validated shipped-script boundary")
        if hook.get("path") != "$ENVCTL_SOURCE_ROOT/assets/scripts/envctl-codex-profile-lifecycle.sh":
            raise SystemExit(f"{component_id} {phase} must use the profile lifecycle script")
        if hook.get("args") != [phase]:
            raise SystemExit(f"{component_id} {phase} must delegate to the profile lifecycle")

group_detect = components["group-ai-clis"]["detect"]["args"][-1]
if "envctl-codex-profile-lifecycle.sh" not in group_detect:
    raise SystemExit("AI CLI group detect must use the profile-owned Codex contract")
if "command -v codex" in group_detect or "for c in claude codex" in group_detect:
    raise SystemExit("AI CLI group detect must not search the Meta-first PATH for Codex")
PY

echo "PASS: Codex release ownership is singular and Yazelix-profile-owned"
