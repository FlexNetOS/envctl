#!/usr/bin/env bash
set -euo pipefail

root="$(git rev-parse --show-toplevel)"
exact='/home/flexnetos/.nix-profile/bin/rtk'
files=(
  "$root/AGENTS.md"
  "$root/home/AGENTS.md"
  "$root/home/AGENTS.rtk.md"
  "$root/home/.codex/AGENTS.rtk.md"
  "$root/home/.codex/RTK.md"
)

for file in "${files[@]}"; do
  grep -Fq "$exact" "$file" || {
    echo "RTK policy does not name the exact profile frontdoor: $file" >&2
    exit 1
  }
done

grep -Fq "$exact proxy --" "$root/home/AGENTS.rtk.md"
grep -Fq "$exact proxy --" "$root/home/.codex/AGENTS.rtk.md"
test "$(grep -Fxc '@/home/flexnetos/.codex/AGENTS.rtk.md' "$root/home/.codex/AGENTS.md")" = 1
if grep -Fq '@/home/flexnetos/.codex/RTK.md' "$root/home/.codex/AGENTS.md"; then
  echo "Codex AGENTS still imports the retired duplicate RTK policy surface" >&2
  exit 1
fi
grep -Fq 'fork_turns="none"' "$root/home/.codex/AGENTS.md" || {
  echo "Codex global guidance lost the context-minimal delegation policy" >&2
  exit 1
}

if rg -n -i \
  'run (these commands )?raw|run it raw|raw rather than|direct raw command invocation is (allowed|preferred)' \
  "$root/home/AGENTS.rtk.md" "$root/home/.codex/AGENTS.rtk.md"; then
  echo "RTK policy still authorizes an unaccounted raw-command bypass" >&2
  exit 1
fi

echo "test-rtk-command-policy: PASS"
