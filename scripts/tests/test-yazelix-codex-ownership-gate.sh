#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

fail() {
  printf 'FAIL: %s\n' "$*" >&2
  exit 1
}

assert_not_contains() {
  local path="$1"
  local pattern="$2"
  local why="$3"

  [[ -e "$path" ]] || return 0
  if grep -Fq -- "$pattern" "$path"; then
    fail "$path contains forbidden pattern '$pattern' ($why)"
  fi
}

assert_absent() {
  local path="$1"
  local why="$2"

  if [[ -e "$path" ]]; then
    fail "$path exists ($why)"
  fi
}

assert_contains() {
  local path="$1"
  local pattern="$2"
  local why="$3"

  [[ -e "$path" ]] || fail "$path missing ($why)"
  if ! grep -Fq -- "$pattern" "$path"; then
    fail "$path does not contain required pattern '$pattern' ($why)"
  fi
}

active_mcp_sources=(
  "$ROOT/agent-env.yaml"
  "$ROOT/agent-env.lock"
  "$ROOT/.mcp.json"
  "$ROOT/.codex/AGENTS.md"
)

for path in "${active_mcp_sources[@]}"; do
  assert_not_contains "$path" "n8n-mcp" "envctl MCP baseline must remain profile-compatible and exa-only"
  assert_not_contains "$path" "/home/drdave/Desktop/meta" "active envctl MCP sources must not point at the retired workspace root"
done

assert_absent "$ROOT/agent-skills/mcps/n8n-mcp.json" "retired MCP must not remain as a source-pack asset"

if [[ -d "$ROOT/agent-skills/mcps" ]]; then
  while IFS= read -r -d '' path; do
    assert_not_contains "$path" "/home/drdave/Desktop/meta" "active MCP asset must use the current FlexNetOS workspace root"
  done < <(find "$ROOT/agent-skills/mcps" -type f -name '*.json' -print0)
fi

assert_contains "$ROOT/AGENTS.md" "Codex binary/runtime ownership must mirror the Yazelix binary/runtime model" \
  "repo instructions must preserve the Yazelix-as-authority contract"
assert_contains "$ROOT/AGENTS.md" "User-bin shadows, repo-cache materializations, temp plugin bundles, marketplace caches, and generated-output files are never alternate active locations." \
  "repo instructions must forbid parallel non-Yazelix ownership paths"

# shellcheck disable=SC2016 # Manifest literals must not expand in the test shell.
assert_not_contains "$ROOT/manifest/ai-clis.toml" 'ln -sfn "$LINK" "$real_link"' \
  "Codex manifest must not create real-home user-bin Codex shadows"
# shellcheck disable=SC2016 # Manifest literals must not expand in the test shell.
assert_not_contains "$ROOT/manifest/ai-clis.toml" 'ln -sfn "$link" "$real_link"' \
  "Codex manifest must not preserve fallback user-bin Codex wrappers"
# shellcheck disable=SC2016 # Manifest literals must not expand in the test shell.
assert_not_contains "$ROOT/manifest/ai-clis.toml" 'tar -C "$ENVCTL_REAL_HOME/.codex"' \
  "Codex manifest must not copy legacy real-home generated config as source"

# shellcheck disable=SC2016 # Manifest literals must not expand in the test shell.
assert_not_contains "$ROOT/manifest/components.d/codex-global-baseline.toml" 'ln -s "$CODEX_HOME_DIR" "$R/.codex"' \
  "Codex global baseline must not turn active ~/.codex into a generated symlink shadow"
# shellcheck disable=SC2016 # Manifest literals must not expand in the test shell.
assert_not_contains "$ROOT/manifest/components.d/codex-global-baseline.toml" 'ln -sfn "$M/usr/bin/codex" "$R/.local/bin/codex"' \
  "Codex global baseline must not create real-home user-bin Codex shadows"

CODEX_PROFILE_LIFECYCLE="$ROOT/assets/scripts/envctl-codex-profile-lifecycle.sh"
assert_contains "$CODEX_PROFILE_LIFECYCLE" "\"\$CODEX_YAZELIX_LIFECYCLE\" detect" \
  "stable Codex verification must delegate source/profile ownership to Yazelix"
assert_contains "$CODEX_PROFILE_LIFECYCLE" "\"\$CODEX_PROFILE/bin/codex\"" \
  "stable Codex must verify the exported profile bin frontdoor"
assert_contains "$CODEX_PROFILE_LIFECYCLE" "\"\$CODEX_PROFILE/toolbin/codex\"" \
  "stable Codex must verify the exported profile toolbin frontdoor"
assert_contains "$CODEX_PROFILE_LIFECYCLE" "\"\$CODEX_META_ROOT/usr/bin/codex\"" \
  "stable Codex must classify the former Meta wrapper as a shadow"
assert_contains "$CODEX_PROFILE_LIFECYCLE" "\"\$CODEX_META_ROOT/usr/bin/codex-alpha\"" \
  "stable Codex must classify the obsolete alpha wrapper as a shadow"
assert_contains "$CODEX_PROFILE_LIFECYCLE" "\"\$CODEX_META_ROOT/.toolchains/openai-codex\"" \
  "stable Codex must classify the former mutable toolchain as a shadow"

CODEX_GLOBAL_LIFECYCLE="$ROOT/assets/scripts/envctl-codex-global-baseline-lifecycle.sh"
# shellcheck disable=SC2016 # Lifecycle source literals must not expand in the test shell.
assert_contains "$CODEX_GLOBAL_LIFECYCLE" '"$CODEX_GLOBAL_PROFILE_LIFECYCLE" "$profile_action"' \
  "global policy repair must validate the Yazelix-owned Codex profile first"
assert_contains "$CODEX_GLOBAL_LIFECYCLE" 'name != "exa" && name != "openaiDeveloperDocs"' \
  "global policy must carry the exact remote-only MCP allowlist"
assert_contains "$CODEX_GLOBAL_LIFECYCLE" 'forbidden active-home plugin or marketplace table' \
  "global policy must reject plugin and marketplace runtime authority"
# shellcheck disable=SC2016 # Lifecycle source literals must not expand in the test shell.
assert_contains "$CODEX_GLOBAL_LIFECYCLE" '"$codex" features disable "$feature"' \
  "global repair must use the profile-owned official feature mutation surface"
assert_contains "$CODEX_GLOBAL_LIFECYCLE" 'features.remote_plugin must be explicitly false' \
  "global verification must prevent remote plugin-cache regeneration"
# shellcheck disable=SC2016 # Lifecycle source literals must not expand in the test shell.
assert_contains "$CODEX_GLOBAL_LIFECYCLE" '"$CODEX_GLOBAL_CONFIG_ROOT/plugins"' \
  "global policy must classify generated plugin bundles as shadows"

python3 - "$ROOT/manifest/ai-clis.toml" <<'PY'
import sys
import tomllib
from pathlib import Path

manifest = Path(sys.argv[1])
components = {item["id"]: item for item in tomllib.loads(manifest.read_text())["component"]}
stable = components.get("codex-cli")
if stable is None:
    raise SystemExit("missing stable Codex profile contract")
if stable.get("requires") != ["yazelix"]:
    raise SystemExit("stable Codex must have Yazelix as its sole owner dependency")
for phase in ("detect", "install", "verify", "fix", "remove"):
    hook = stable.get(phase, {})
    if hook.get("kind") != "shipped_script":
        raise SystemExit(f"stable Codex {phase} bypasses the shipped profile lifecycle")
    if hook.get("path") != "$ENVCTL_SOURCE_ROOT/assets/scripts/envctl-codex-profile-lifecycle.sh":
        raise SystemExit(f"stable Codex {phase} points outside the profile lifecycle")
    if hook.get("args") != [phase]:
        raise SystemExit(f"stable Codex {phase} does not preserve lifecycle phase parity")
PY

python3 - "$ROOT/manifest/components.d/codex-global-baseline.toml" <<'PY'
import sys
import tomllib
from pathlib import Path

manifest = Path(sys.argv[1])
components = tomllib.loads(manifest.read_text())["component"]
if len(components) != 1 or components[0].get("id") != "codex-global-baseline":
    raise SystemExit("missing singular Codex global policy component")
component = components[0]
if component.get("requires") != ["codex-cli"]:
    raise SystemExit("Codex global policy must depend only on the profile-owned Codex contract")
expected = "$ENVCTL_SOURCE_ROOT/assets/scripts/envctl-codex-global-baseline-lifecycle.sh"
for phase in ("detect", "install", "verify", "fix", "remove"):
    hook = component.get(phase, {})
    if hook.get("kind") != "shipped_script" or hook.get("path") != expected:
        raise SystemExit(f"Codex global {phase} bypasses the shipped policy lifecycle")
    if hook.get("args") != [phase]:
        raise SystemExit(f"Codex global {phase} does not preserve lifecycle phase parity")
PY

if python3 - "$ROOT" <<'PY'
import re
import sys
from pathlib import Path

root = Path(sys.argv[1])
needle = "/home/flexnetos/.local/share/yazelix"
forbidden = (
    re.compile(r"(?:edit|write)\s+(?:under|to|at)\s+[`'\"]?" + re.escape(needle), re.I),
    re.compile(re.escape(needle) + r".{0,80}\b(?:is|as)\b.{0,30}\b(?:editable|writable|writeable|source|input)\b", re.I),
)
targets = [
    root / "agent-env.yaml",
    root / "manifest",
    root / "agent-skills",
    root / ".codex/AGENTS.md",
]
for target in targets:
    paths = [target] if target.is_file() else target.rglob("*")
    for path in paths:
        if not path.is_file():
            continue
        try:
            lines = path.read_text(errors="ignore").splitlines()
        except OSError:
            continue
        for line_number, line in enumerate(lines, 1):
            if needle in line and any(pattern.search(line) for pattern in forbidden):
                print(f"{path}:{line_number}:{line}", file=sys.stderr)
                raise SystemExit(0)
raise SystemExit(1)
PY
then
  fail "source/config text appears to treat /home/flexnetos/.local/share/yazelix as an editable input"
fi

printf 'PASS: Yazelix/Codex ownership source gate\n'
