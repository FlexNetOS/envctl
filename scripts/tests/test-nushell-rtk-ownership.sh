#!/usr/bin/env bash
set -euo pipefail

root="$(git rev-parse --show-toplevel)"
config="$root/home/.config/nushell/config.nu"
user_hook="$root/home/.config/yazelix/shell_nu.nu"
portability="$root/manifest/components.d/portability-links.toml"
retired="$root/home/.config/nushell/rtk-wrappers.nu"
profile_import='use ~/.nix-profile/nushell/config/rtk_wrappers.nu *'

test ! -e "$retired" || {
  echo "duplicate envctl-owned RTK Nu wrapper still exists: $retired" >&2
  exit 1
}
test "$(grep -Fxc "$profile_import" "$config")" = 1 || {
  echo "standalone login Nu must import the profile-owned RTK module exactly once" >&2
  exit 1
}
if grep -Eq '^[[:space:]]*(source|use)[[:space:]].*rtk[-_]wrappers\.nu' "$user_hook"; then
  echo "Yazelix user hook must not duplicate the managed RTK module import" >&2
  exit 1
fi
if grep -Fq '.config/nushell/rtk-wrappers.nu' "$portability"; then
  echo "portability-links must not materialize the retired duplicate RTK module" >&2
  exit 1
fi

if ! command -v nu >/dev/null 2>&1; then
  echo "test-nushell-rtk-ownership: structural PASS (nu unavailable; behavior not run)"
  exit 0
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
home="$tmp/home"
mkdir -p "$home/.config/nushell" "$home/.nix-profile/nushell/config" \
  "$home/.nix-profile/bin" "$home/.nix-profile/toolbin"
cp "$config" "$home/.config/nushell/config.nu"
cp "$root/home/.config/nushell/meta-usr-path.nu" "$home/.config/nushell/meta-usr-path.nu"
printf '%s\n' 'export def --wrapped cargo [...rest] { ^rtk cargo ...$rest }' \
  >"$home/.nix-profile/nushell/config/rtk_wrappers.nu"
{
  printf '%s\n' '#!/usr/bin/env sh'
  printf '%s\n' 'printf "%s\\n" "$*" >>"$RTK_TEST_LOG"'
  printf '%s\n' 'printf "rtk-routed\\n"'
} >"$home/.nix-profile/bin/rtk"
chmod +x "$home/.nix-profile/bin/rtk"

export HOME="$home"
export XDG_CONFIG_HOME="$home/.config"
export RTK_TEST_LOG="$tmp/rtk.log"
export PATH="/nix/store/old-lifeos-foundation-yzx/toolbin:/nix/store/old-codex-cli-0.0.0/codex-path:$home/.nix-profile/bin:$PATH"

nu -l -c '$env.PATH | each { into string } | to json' >"$tmp/path.out"
grep -Fq "${home}/.nix-profile/toolbin" "$tmp/path.out"
grep -Fq "${home}/.nix-profile/bin" "$tmp/path.out"
if grep -Fq '/nix/store/old-lifeos-foundation-yzx/toolbin' "$tmp/path.out"; then
  echo "standalone Nu preserved a stale Yazelix profile generation" >&2
  exit 1
fi
if grep -Fq '/nix/store/old-codex-cli-0.0.0/codex-path' "$tmp/path.out"; then
  echo "standalone Nu preserved a raw Codex package path" >&2
  exit 1
fi

nu -l -c 'cargo standalone-login' >"$tmp/login.out"
grep -Fqx 'rtk-routed' "$tmp/login.out"
grep -Fqx 'cargo standalone-login' "$RTK_TEST_LOG"

nu -l -c '^bash -lc "printf native-bash"' >"$tmp/bash.out"
grep -Fqx 'native-bash' "$tmp/bash.out"

managed_config="$tmp/managed.nu"
{
  printf '%s\n' "$profile_import"
  printf 'source %s\n' "$user_hook"
} >"$managed_config"
nu --config "$managed_config" -c 'cargo yazelix-managed' >"$tmp/managed.out"
grep -Fqx 'rtk-routed' "$tmp/managed.out"
grep -Fqx 'cargo yazelix-managed' "$RTK_TEST_LOG"
test "$(wc -l <"$RTK_TEST_LOG" | tr -d '[:space:]')" = 2

echo "test-nushell-rtk-ownership: PASS"
