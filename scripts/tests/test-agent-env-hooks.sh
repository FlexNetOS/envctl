#!/usr/bin/env bash
set -euo pipefail

root="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
lifecycle="$root/assets/scripts/envctl-claude-cleanup.sh"
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }

[ -x "$lifecycle" ] || fail "missing executable Claude profile validator"
[ ! -e "$root/profile-runtime/claude" ] \
  || fail "envctl must not own a Claude runtime tree"
grep -Fq 'share/yazelix/agent_configs/claude' "$lifecycle" \
  || fail "Claude validator does not consume profile-owned configuration inputs"
grep -Fq 'yazelix/profile-runtime/claude' "$lifecycle" \
  || fail "Claude validator does not name the volatile materialization"
if grep -Eq 'nix (build|profile)|git (checkout|switch|pull)' "$lifecycle"; then
  fail "Claude validator can build or switch its owning profile"
fi

fixture="$(mktemp -d)"
trap 'rm -rf -- "$fixture"' EXIT
real_home="$fixture/home"
store_root="$fixture/nix/store"
closure="$store_root/fixture-lifeos-foundation-yzx"
runtime="$fixture/run/yazelix/profile-runtime/claude"
mkdir -p \
  "$closure/bin" \
  "$closure/toolbin" \
  "$closure/share/yazelix/agent_configs/claude" \
  "$runtime" \
  "$real_home"
printf '%s\n' '#!/bin/sh' 'printf "claude fixture 1.0\n"' >"$closure/bin/claude"
chmod 755 "$closure/bin/claude"
ln -s ../bin/claude "$closure/toolbin/claude"
for source in settings.json.src CLAUDE.md.src RTK.md.src; do
  printf 'fixture\n' >"$closure/share/yazelix/agent_configs/claude/$source"
done
for materialized in settings.json CLAUDE.md RTK.md; do
  printf 'fixture\n' >"$runtime/$materialized"
done
ln -s "$closure" "$real_home/.nix-profile"

ENVCTL_REAL_HOME="$real_home" \
ENVCTL_NIX_STORE_ROOT="$store_root" \
XDG_RUNTIME_DIR="$fixture/run" \
  "$lifecycle" verify

printf 'AGENT-ENV PROFILE-OWNED CLAUDE CONTRACT PASS\n'
