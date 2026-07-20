#!/usr/bin/env bash
set -euo pipefail

fail() { echo "FAIL: $*" >&2; exit 1; }

root="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
manifest="$root/manifest/agent-env.toml"
[ -f "$manifest" ] || fail "missing agent-env component manifest"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
meta="$tmp/meta"
repo="$tmp/repo"
ambient="$tmp/ambient"
owned="$meta/usr/libexec/envctl/cli/bin/envctl"
owned_log="$tmp/owned.log"
ambient_called="$tmp/ambient-called"
mkdir -p "$(dirname "$owned")" "$repo" "$ambient"
printf 'scope: project\nagent: codex\nskills: []\n' >"$repo/agent-env.yaml"

cat >"$owned" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf '%s|%s\n' "$0" "$*" >>"${OWNED_LOG:?}"
case "$*" in
  'agent lock --config agent-env.yaml --scope project --check --locked') ;;
  'agent sync --config agent-env.yaml --scope project --apply') ;;
  'agent clean --scope project --apply') ;;
  *) echo "unexpected owned envctl invocation: $*" >&2; exit 64 ;;
esac
SH
chmod 755 "$owned"

cat >"$ambient/envctl" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
: >"${AMBIENT_CALLED:?}"
echo "ambient envctl must never be invoked" >&2
exit 99
SH
chmod 755 "$ambient/envctl"

python3 - "$manifest" "$tmp" <<'PY'
import pathlib, sys, tomllib
data = tomllib.loads(pathlib.Path(sys.argv[1]).read_text())
component = next(c for c in data["component"] if c["id"] == "kasetto")
assert component["requires"] == [
    "envctl-cli",
    "yazelix",
    "rtk",
    "codex-global-baseline",
], component["requires"]
out = pathlib.Path(sys.argv[2])
for phase in ("detect", "install", "verify", "fix", "remove"):
    hook = component[phase]
    script = hook.get("script")
    if script is None:
        assert hook["command"] == "bash" and hook["args"][:1] == ["-lc"]
        script = hook["args"][1]
    path = out / f"agent-{phase}.sh"
    path.write_text(script)
    path.chmod(0o755)
PY

export META_ROOT="$meta"
export OWNED_LOG="$owned_log"
export AMBIENT_CALLED="$ambient_called"
export PATH="$ambient:/usr/bin:/bin"
cd "$repo"

bash "$tmp/agent-detect.sh"
mv "$owned" "$owned.off"
if bash "$tmp/agent-detect.sh"; then fail "detect passed without the owned envctl payload"; fi
mv "$owned.off" "$owned"

bash "$tmp/agent-verify.sh"
bash "$tmp/agent-install.sh"
bash "$tmp/agent-fix.sh"
bash "$tmp/agent-remove.sh"

[ ! -e "$ambient_called" ] || fail "an agent-env hook invoked ambient PATH envctl"
[ "$(wc -l <"$owned_log")" -eq 4 ] || fail "unexpected owned envctl invocation count"
grep -Fqx "$owned|agent lock --config agent-env.yaml --scope project --check --locked" "$owned_log" \
  || fail "verify did not use the owned payload"
[ "$(grep -Fc "$owned|agent sync --config agent-env.yaml --scope project --apply" "$owned_log")" -eq 2 ] \
  || fail "install/fix did not use the owned payload"
grep -Fqx "$owned|agent clean --scope project --apply" "$owned_log" \
  || fail "remove did not use the owned payload"

echo "PASS: agent-env component depends on and invokes only the owned envctl payload"
