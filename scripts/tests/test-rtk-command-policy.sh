#!/usr/bin/env bash
set -euo pipefail

root="$(git rev-parse --show-toplevel)"
exact='/home/flexnetos/.nix-profile/bin/rtk'
files=(
  "$root/AGENTS.md"
  "$root/home/AGENTS.md"
  "$root/home/AGENTS.rtk.md"
)

for file in "${files[@]}"; do
  grep -Fq "$exact" "$file" || {
    echo "RTK policy does not name the exact profile frontdoor: $file" >&2
    exit 1
  }
done

grep -Fq "$exact proxy --" "$root/home/AGENTS.rtk.md"
grep -Fq 'Do not hide several independent diagnostics' "$root/home/AGENTS.rtk.md"
if [ -e "$root/profile-runtime" ]; then
  echo "repo-local profile-runtime must not be an installed-agent authority" >&2
  exit 1
fi

if rg -n -i \
  'run (these commands )?raw|run it raw|raw rather than|direct raw command invocation is (allowed|preferred)' \
  "$root/home/AGENTS.rtk.md"; then
  echo "RTK policy still authorizes an unaccounted raw-command bypass" >&2
  exit 1
fi

echo "test-rtk-command-policy: PASS"
