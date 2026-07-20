#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
LIFECYCLE="$ROOT/assets/scripts/envctl-codex-profile-lifecycle.sh"

fail() {
  printf 'FAIL: %s\n' "$*" >&2
  exit 1
}

assert_absent() {
  [ ! -e "$1" ] && [ ! -L "$1" ] || fail "expected absent path: $1"
}

tmp="$(mktemp -d -t envctl-codex-profile-contract.XXXXXXXX)"
trap 'rm -rf "$tmp"' EXIT
meta="$tmp/meta"
real_home="$tmp/home"
store="$tmp/store"
source_root="$tmp/source"
fixture_lifecycle="$source_root/assets/scripts/envctl-codex-profile-lifecycle.sh"
owner_lifecycle="$source_root/assets/scripts/envctl-yazelix-profile-lifecycle.sh"

install -d -m 755 "$meta" "$real_home" "$store" "$source_root/assets/scripts"
install -m 755 "$LIFECYCLE" "$fixture_lifecycle"
install -m 755 "$ROOT/assets/scripts/envctl-codex-cleanup.sh" \
  "$source_root/assets/scripts/envctl-codex-cleanup.sh"

cat >"$owner_lifecycle" <<'OWNER'
#!/usr/bin/env bash
set -euo pipefail
action="${1:?action required}"
profile="$ENVCTL_REAL_HOME/.nix-profile"
store="${ENVCTL_CODEX_STORE_ROOT:?test store root required}"
ready="$ENVCTL_REAL_HOME/.owner-ready"

provision() {
  package="$store/fixture-codex-cli-9.9.9"
  profile_store="$store/fixture-profile"
  mkdir -p "$package/bin" "$profile_store/bin" "$profile_store/toolbin"
  cat >"$package/bin/codex" <<'CODEX'
#!/bin/sh
printf '%s\n' 'codex-cli 9.9.9'
CODEX
  chmod 755 "$package/bin/codex"
  printf '%s\n' \
    '{"layoutVersion":1,"version":"9.9.9","variant":"codex","entrypoint":"bin/codex"}' \
    >"$package/codex-package.json"
  ln -sfn "$package/bin/codex" "$profile_store/bin/codex"
  ln -sfn "$package/bin/codex" "$profile_store/toolbin/codex"
  ln -sfn "$profile_store" "$profile"
  : >"$ready"
}

case "$action" in
  detect|verify)
    [ -f "$ready" ] && [ -L "$profile" ]
    ;;
  install|fix)
    printf '%s\n' "$action" >>"$META_ROOT/owner-actions.log"
    provision
    ;;
  remove)
    printf '%s\n' remove >>"$META_ROOT/owner-actions.log"
    rm -f "$profile" "$ready"
    ;;
  *) exit 2 ;;
esac
OWNER
chmod 755 "$owner_lifecycle"

run_lifecycle() {
  env -i \
    HOME="$real_home" \
    META_ROOT="$meta" \
    ENVCTL_REAL_HOME="$real_home" \
    ENVCTL_SOURCE_ROOT="$source_root" \
    ENVCTL_CODEX_STORE_ROOT="$store" \
    PATH=/usr/bin:/bin \
    "$fixture_lifecycle" "$@"
}

if run_lifecycle detect >/dev/null 2>&1; then
  fail "detect accepted a missing Yazelix profile"
fi
assert_absent "$real_home/.nix-profile"
assert_absent "$meta/var/lib/envctl/legacy-archives"

run_lifecycle install >/dev/null
grep -Fqx install "$meta/owner-actions.log" \
  || fail "install did not delegate to the Yazelix owner"
[ "$(run_lifecycle detect)" = "" ] || fail "healthy detect emitted unexpected output"
[ "$(run_lifecycle verify)" = "codex-profile: verified single profile-owned Codex runtime" ] \
  || fail "verify did not report the single-owner contract"
resolved="$(readlink -f "$real_home/.nix-profile/toolbin/codex")"
case "$resolved" in "$store"/*-codex-cli-9.9.9/bin/codex) ;; *) fail "unexpected profile Codex: $resolved" ;; esac

archives_before=0
if [ -d "$meta/var/lib/envctl/legacy-archives" ]; then
  archives_before="$(find "$meta/var/lib/envctl/legacy-archives" -mindepth 1 -maxdepth 1 -type d | wc -l)"
fi
run_lifecycle install >/dev/null
archives_after=0
if [ -d "$meta/var/lib/envctl/legacy-archives" ]; then
  archives_after="$(find "$meta/var/lib/envctl/legacy-archives" -mindepth 1 -maxdepth 1 -type d | wc -l)"
fi
[ "$archives_before" -eq "$archives_after" ] \
  || fail "idempotent install created an empty shadow archive"

install -d -m 755 \
  "$meta/usr/bin" \
  "$meta/.toolchains/openai-codex/0.142.3/bin" \
  "$meta/.local/share/codex" \
  "$meta/.local/state/codex" \
  "$meta/.toolchains/bun/install/cache/@openai-codex-stale" \
  "$real_home/.local/bin"
printf '#!/bin/sh\nexit 0\n' >"$meta/usr/bin/codex"
chmod 755 "$meta/usr/bin/codex"
cp "$meta/usr/bin/codex" "$meta/usr/bin/codex-alpha"
cp "$meta/usr/bin/codex" "$meta/.toolchains/openai-codex/0.142.3/bin/codex"
: >"$meta/.local/share/codex/stale-config"
: >"$meta/.local/state/codex/stale-state"
ln -s "$meta/usr/bin/codex" "$real_home/.local/bin/codex"
ln -s "$meta/usr/bin/codex-alpha" "$real_home/.local/bin/codex-alpha"

if run_lifecycle detect >"$tmp/shadow.out" 2>"$tmp/shadow.err"; then
  fail "detect accepted parallel Codex state"
fi
grep -Fq 'stale parallel Codex shadow' "$tmp/shadow.err" \
  || fail "detect did not explain the shadow refusal"

run_lifecycle fix >/dev/null
for path in \
  "$meta/usr/bin/codex" \
  "$meta/usr/bin/codex-alpha" \
  "$meta/.toolchains/openai-codex" \
  "$meta/.local/share/codex" \
  "$meta/.local/state/codex" \
  "$meta/.toolchains/bun/install/cache/@openai-codex-stale" \
  "$real_home/.local/bin/codex"; do
  assert_absent "$path"
done
find "$meta/var/lib/envctl/legacy-archives" -path '*/meta/usr/bin/codex' -type f -print -quit \
  | grep -q . || fail "repair did not archive the Meta-root wrapper"
find "$meta/var/lib/envctl/legacy-archives" -path '*/real-home/.local/bin/codex' -type l -print -quit \
  | grep -q . || fail "repair did not archive the real-home shadow"
find "$meta/var/lib/envctl/legacy-archives" -path '*/real-home/.local/bin/codex-alpha' -type l -print -quit \
  | grep -q . || fail "repair did not archive the obsolete alpha shadow"
run_lifecycle verify >/dev/null

install -d -m 755 "$meta/.toolchains/bun/install/global"
printf '{"dependencies":{"@openai/codex":"0.142.3"}}\n' \
  >"$meta/.toolchains/bun/install/global/package.json"
if run_lifecycle fix >"$tmp/package.out" 2>"$tmp/package.err"; then
  fail "repair accepted a stale package record without an owned Bun remover"
fi
grep -Fq 'cannot retire stale @openai/codex records without one profile-owned Bun frontdoor' \
  "$tmp/package.err" \
  || fail "package-record refusal was not explicit"
[ -f "$meta/.toolchains/bun/install/global/package.json" ] \
  || fail "fail-closed package repair deleted the unresolved record"
rm -f "$meta/.toolchains/bun/install/global/package.json"

profile_store="$(readlink -f "$real_home/.nix-profile")"
bun_package="$store/fixture-bun-1.2.3"
install -d -m 755 "$bun_package/bin" "$meta/.toolchains/bun/bin"
cat >"$bun_package/bin/bun" <<'BUN'
#!/usr/bin/env bash
set -euo pipefail
[ "${1:-}" = remove ] && [ "${2:-}" = -g ] && [ "${3:-}" = @openai/codex ]
: "${BUN_INSTALL:?BUN_INSTALL required}"
: >"$META_ROOT/profile-bun-called"
printf '{}\n' >"$BUN_INSTALL/install/global/package.json"
rm -f "$BUN_INSTALL/install/global/bun.lock"
BUN
chmod 755 "$bun_package/bin/bun"
ln -sfn "$bun_package/bin/bun" "$profile_store/bin/bun"
ln -sfn "$bun_package/bin/bun" "$profile_store/toolbin/bun"
cat >"$meta/.toolchains/bun/bin/bun" <<'HOSTILE'
#!/usr/bin/env bash
set -euo pipefail
: >"$META_ROOT/meta-bun-called"
printf '{}\n' >"$META_ROOT/.toolchains/bun/install/global/package.json"
HOSTILE
chmod 755 "$meta/.toolchains/bun/bin/bun"
printf '{"dependencies":{"@openai/codex":"0.142.3"}}\n' \
  >"$meta/.toolchains/bun/install/global/package.json"
run_lifecycle fix >/dev/null
[ -f "$meta/profile-bun-called" ] \
  || fail "repair did not use the profile-owned Bun remover"
assert_absent "$meta/meta-bun-called"
grep -Fq '"@openai/codex"' "$meta/.toolchains/bun/install/global/package.json" \
  && fail "profile-owned Bun remover left the stale Codex package record"

second_package="$store/fixture-codex-cli-9.9.8"
install -d -m 755 "$second_package/bin"
printf '#!/bin/sh\nprintf "%%s\\n" "codex-cli 9.9.8"\n' >"$second_package/bin/codex"
chmod 755 "$second_package/bin/codex"
ln -sfn "$second_package/bin/codex" "$real_home/.nix-profile/toolbin/codex"
if run_lifecycle detect >/dev/null 2>&1; then
  fail "detect accepted split profile Codex binaries"
fi
ln -sfn "$store/fixture-codex-cli-9.9.9/bin/codex" "$real_home/.nix-profile/toolbin/codex"

install -d -m 755 "$meta/usr/bin"
printf '#!/bin/sh\nexit 0\n' >"$meta/usr/bin/codex"
chmod 755 "$meta/usr/bin/codex"
run_lifecycle remove >/dev/null
[ -L "$real_home/.nix-profile" ] || fail "remove retired the Yazelix-owned profile"
assert_absent "$meta/usr/bin/codex"
if grep -Fqx remove "$meta/owner-actions.log"; then
  fail "Codex remove delegated destructive profile removal to Yazelix"
fi
run_lifecycle verify >/dev/null

rm -rf "${meta:?}/var"
archive_escape="$tmp/archive-escape"
install -d -m 755 "$archive_escape" "$meta/usr/bin"
ln -s "$archive_escape" "$meta/var"
printf '#!/bin/sh\nexit 0\n' >"$meta/usr/bin/codex"
chmod 755 "$meta/usr/bin/codex"
if run_lifecycle fix >/dev/null 2>&1; then
  fail "repair accepted a symlinked archive parent"
fi
[ -f "$meta/usr/bin/codex" ] \
  || fail "unsafe archive refusal moved the Codex shadow"
[ -z "$(find "$archive_escape" -mindepth 1 -print -quit)" ] \
  || fail "unsafe archive refusal wrote through the symlinked parent"
rm "$meta/var"
rm -f "$meta/usr/bin/codex"

if META_ROOT="$meta" ENVCTL_REAL_HOME="$real_home" ENVCTL_SOURCE_ROOT="$source_root" \
    ENVCTL_CODEX_STORE_ROOT=relative "$fixture_lifecycle" detect >/dev/null 2>&1; then
  fail "relative store-root override was accepted"
fi

echo "PASS: Codex lifecycle is profile-owned, idempotent, shadow-free, and fail-closed"
