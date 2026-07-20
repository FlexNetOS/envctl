#!/usr/bin/env bash
set -euo pipefail

root="${ENVCTL_GATE_ROOT:-$(git rev-parse --show-toplevel)}"
cd "$root"

fail() {
  printf 'agent-env active mirror test: FAIL: %s\n' "$*" >&2
  exit 1
}

mapfile -t skills < <(
  find agent-skills -mindepth 2 -maxdepth 2 -type f -name SKILL.md -printf '%h\n' \
    | sed 's#^agent-skills/##' \
    | LC_ALL=C sort
)
[ "${#skills[@]}" -gt 0 ] || fail "agent-skills contains no discoverable skills"

for skill in "${skills[@]}"; do
  source_dir="agent-skills/$skill"
  for mirror in .claude/skills .codex/skills .agents/skills; do
    destination="$mirror/$skill"
    [ -d "$destination" ] || fail "$destination is missing"
    diff -qr -- "$source_dir" "$destination" >/dev/null \
      || fail "$destination drifted from $source_dir"
  done
done

printf 'agent-env active mirror test: PASS (%s skills x 3 mirrors)\n' "${#skills[@]}"
