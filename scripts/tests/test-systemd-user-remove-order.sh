#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-$(git rev-parse --show-toplevel)}"
ROOT="$(cd "$ROOT" && pwd -P)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

extract_script() {
  local manifest="$1" component_id="$2" phase="$3"
  python3 - "$manifest" "$component_id" "$phase" <<'PY'
import pathlib
import sys
import tomllib

path, component_id, phase = sys.argv[1:]
document = tomllib.loads(pathlib.Path(path).read_text())
for component in document.get("component", []):
    if component.get("id") != component_id:
        continue
    hook = component.get(phase)
    if not isinstance(hook, dict) or hook.get("kind") != "script":
        raise SystemExit(f"{component_id}.{phase} is not a script hook")
    print(hook.get("script", ""), end="")
    break
else:
    raise SystemExit(f"component {component_id!r} not found in {path}")
PY
}

FAKE_BIN="$TMP/bin"
SYSTEMCTL_LOG="$TMP/systemctl.log"
mkdir -p "$FAKE_BIN"
cat >"$FAKE_BIN/systemctl" <<'SH'
#!/bin/sh
set -eu
printf '%s\n' "$*" >>"$ENVCTL_SYSTEMCTL_LOG"
if [ "${ENVCTL_FAKE_SYSTEMCTL_FAIL:-0}" = 1 ]; then
  exit 23
fi
case "$*" in
  '--user disable --now '*)
    unit="$4"
    rm -f "$ENVCTL_REAL_HOME/.config/systemd/user/$unit"
    ;;
esac
SH
chmod 755 "$FAKE_BIN/systemctl"

ENV_CTL_REMOVE="$(extract_script "$ROOT/manifest/env-ctl.toml" env-ctl remove)"
ENV_CTL_FIX="$(extract_script "$ROOT/manifest/env-ctl.toml" env-ctl fix)"

if grep -Fq 'systemctl --user' <<<"$ENV_CTL_FIX"; then
  echo "systemd remove-order test: env-ctl fix hook must leave reload/restart to typed wiring" >&2
  exit 1
fi

setup_env_ctl() {
  local meta="$1" real="$2"
  local private="$meta/usr/libexec/envctl/secrets/bin"
  mkdir -p "$private" "$meta/usr/bin" "$meta/.config/env-ctl" \
    "$meta/.config/systemd/user" "$real/.config/systemd/user"
  printf 'secretd payload\n' >"$private/secretd"
  printf 'secretctl payload\n' >"$private/secretctl"
  printf '#!/bin/sh\nexec "%s" "$@"\n' "$private/secretd" >"$meta/usr/bin/secretd"
  printf '#!/bin/sh\nexec "%s" "$@"\n' "$private/secretctl" >"$meta/usr/bin/secretctl"
  printf 'operator config must survive\n' >"$meta/.config/env-ctl/secretd.toml"
  printf '[Service]\nExecStart=%s/secretd\n' "$private" \
    >"$meta/.config/systemd/user/env-ctl.service"
  ln -s "$meta/.config/systemd/user/env-ctl.service" \
    "$real/.config/systemd/user/env-ctl.service"
}

run_remove() {
  local script="$1" meta="$2" real="$3" fail="$4"
  env -i \
    PATH="$FAKE_BIN:/usr/bin:/bin" \
    HOME="$meta" \
    META_ROOT="$meta" \
    ENVCTL_REAL_HOME="$real" \
    ENVCTL_SYSTEMCTL_LOG="$SYSTEMCTL_LOG" \
    ENVCTL_FAKE_SYSTEMCTL_FAIL="$fail" \
    bash --noprofile --norc -c "$script"
}

assert_one_stop_call() {
  local unit="$1"
  grep -Fqx -- "--user disable --now $unit" "$SYSTEMCTL_LOG"
  test "$(wc -l <"$SYSTEMCTL_LOG" | tr -d '[:space:]')" = 1
}

# A failed stop must abort before any envctl-owned deletion. Kache is omitted:
# Yazelix owns its runtime and envctl has no Kache service/remove hook.
meta="$TMP/env-ctl-fail/meta"
real="$TMP/env-ctl-fail/home"
mkdir -p "$meta" "$real"
: >"$SYSTEMCTL_LOG"
setup_env_ctl "$meta" "$real"
cp "$meta/.config/env-ctl/secretd.toml" "$TMP/env-ctl-config.before"
if run_remove "$ENV_CTL_REMOVE" "$meta" "$real" 1; then
  echo "systemd remove-order test: env-ctl remove ignored stop failure" >&2
  exit 1
fi
assert_one_stop_call env-ctl.service
test -f "$meta/usr/libexec/envctl/secrets/bin/secretd"
test -f "$meta/usr/libexec/envctl/secrets/bin/secretctl"
test -f "$meta/usr/bin/secretd"
test -f "$meta/usr/bin/secretctl"
cmp -s "$TMP/env-ctl-config.before" "$meta/.config/env-ctl/secretd.toml"
test -f "$meta/.config/systemd/user/env-ctl.service"
test -L "$real/.config/systemd/user/env-ctl.service"

# A successful hook removes only component payload/config wiring. Canonical
# unit content remains for the executor's immediately-following generic revert;
# the successful disable may already have removed the discovery bridge.
meta="$TMP/env-ctl-success/meta"
real="$TMP/env-ctl-success/home"
mkdir -p "$meta" "$real"
setup_env_ctl "$meta" "$real"
: >"$SYSTEMCTL_LOG"
run_remove "$ENV_CTL_REMOVE" "$meta" "$real" 0
assert_one_stop_call env-ctl.service
test ! -e "$meta/usr/libexec/envctl/secrets/bin/secretd"
test ! -e "$meta/usr/libexec/envctl/secrets/bin/secretctl"
test ! -e "$meta/usr/bin/secretd"
test ! -e "$meta/usr/bin/secretctl"
test -f "$meta/.config/env-ctl/secretd.toml"
test -f "$meta/.config/systemd/user/env-ctl.service"
test ! -L "$real/.config/systemd/user/env-ctl.service"

echo "SYSTEMD USER REMOVE ORDER TEST PASS"
