#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
GATE="$ROOT/ci/gates/strict-profile-owner.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

fixture="$TMP/repo"
mkdir -p "$fixture/ci" "$fixture/home" "$fixture/docs/generated"
git -C "$fixture" init -q
printf 'profile=/home/flexnetos/.nix-profile\n' >"$fixture/README.md"
printf 'historical projection\n' >"$fixture/docs/generated/receipt.md"
git -C "$fixture" add README.md docs/generated/receipt.md

"$GATE" "$fixture" >/dev/null

agent_name="$(printf '%s' codex)"
printf 'retired=/home/flexnetos/.%s\n' "$agent_name" >"$fixture/CLAUDE.md"
git -C "$fixture" add CLAUDE.md
if "$GATE" "$fixture" >"$TMP/forbidden.out" 2>"$TMP/forbidden.err"; then
  echo "strict profile owner gate accepted a forbidden home-agent reference" >&2
  exit 1
fi
grep -Fq 'forbidden maintained reference: CLAUDE.md' "$TMP/forbidden.err"

rm "$fixture/CLAUDE.md"
git -C "$fixture" add -u
mkdir -p "$fixture/home/.$agent_name"
printf 'projection\n' >"$fixture/home/.$agent_name/config.toml"
git -C "$fixture" add "home/.$agent_name/config.toml"
if "$GATE" "$fixture" >"$TMP/path.out" 2>"$TMP/path.err"; then
  echo "strict profile owner gate accepted a forbidden home-agent source path" >&2
  exit 1
fi
grep -Fq 'forbidden maintained path:' "$TMP/path.err"

printf 'test-strict-profile-owner-gate: PASS\n'
