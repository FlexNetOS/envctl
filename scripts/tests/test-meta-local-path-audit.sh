#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
AUDIT="$ROOT/scripts/audit-meta-local-paths.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

meta="$TMP/meta"
home="$TMP/home"
store="$TMP/store"
profile_generation="$store/0123456789abcdefghijklmnopqrstuv-profile"
mkdir -p "$meta" "$home" "$profile_generation/bin" "$profile_generation/toolbin"

for command in yzx codex claude rtk bun bunx nu nix git-kb icm; do
  printf '#!/bin/sh\nexit 0\n' >"$profile_generation/bin/$command"
  chmod +x "$profile_generation/bin/$command"
done
ln -s "$profile_generation" "$home/.nix-profile-1-link"
ln -s .nix-profile-1-link "$home/.nix-profile"

common=(
  --require-yazelix-profile
  --nix-store-root "$store"
  --meta-root "$meta"
  --real-home "$home"
  --envctl-home-source "$ROOT/home"
)

"$AUDIT" "${common[@]}" >"$TMP/clean.out"
grep -Fq 'strict profile ownership audit: PASS' "$TMP/clean.out"

dot_local=".$(printf '%s' local)"
dot_codex=".$(printf '%s' codex)"
dot_claude=".$(printf '%s' claude)"
mkdir -p "$home/$dot_local" "$home/$dot_codex" "$home/$dot_claude"
printf 'credential fixture\n' >"$home/$dot_codex/auth.json"

if "$AUDIT" "${common[@]}" >"$TMP/detect.out" 2>"$TMP/detect.err"; then
  echo "expected competing ownership paths to fail detection" >&2
  exit 1
fi
test "$(grep -Fc 'forbidden competing ownership path:' "$TMP/detect.err")" = 3

"$AUDIT" --apply "${common[@]}" >"$TMP/apply.out"
for path in "$home/$dot_local" "$home/$dot_codex" "$home/$dot_claude"; do
  test ! -e "$path"
done
archive="$(find "$meta/var/lib/envctl/archives/strict-profile-owner" -mindepth 1 -maxdepth 1 -type d -print -quit)"
test -n "$archive"
test -d "$archive/$dot_local"
test -f "$archive/$dot_codex/auth.json"
test -d "$archive/$dot_claude"
sha256sum --check "$archive/receipt.txt.sha256" >/dev/null

wrong="$TMP/wrong-home"
mkdir -p "$wrong"
ln -s "$profile_generation" "$wrong/.nix-profile"
if "$AUDIT" --require-yazelix-profile --nix-store-root "$store" \
  --meta-root "$meta" --real-home "$wrong" >"$TMP/wrong.out" 2>"$TMP/wrong.err"; then
  echo "expected a non-generation selector to fail closed" >&2
  exit 1
fi
grep -Fq 'invalid direct profile selector' "$TMP/wrong.err"

printf 'test-meta-local-path-audit: PASS\n'
