#!/usr/bin/env bash
set -euo pipefail

root="${ENVCTL_GATE_ROOT:-$(git rev-parse --show-toplevel)}"
cd "$root"

fail() { printf 'agent-env active mirror test: FAIL: %s\n' "$*" >&2; exit 1; }

[ -f agent-skills/skill-catalog/catalog.yaml ] || fail "catalog owner is missing"
[ -f agent-env.active.yaml ] || fail "generated active projection is missing"

# Catalog roots are canonical source only, never discovery projections.  Every
# declared catalog skill must retain a SKILL.md even when inactive.
while IFS= read -r name; do
  [ -f "agent-skills/capability-packs/$name/SKILL.md" ] \
    || [ -f ".kb/skills/$name/SKILL.md" ] \
    || [ "$name" = skill-catalog ] && continue
  fail "catalog skill $name has no canonical source"
done < <(awk '
  $0 == "skills:" { in_skills = 1; next }
  in_skills && /^  [A-Za-z0-9][A-Za-z0-9-]*:$/ {
    line = $0
    sub(/^  /, "", line)
    sub(/:$/, "", line)
    print line
  }
' agent-skills/skill-catalog/catalog.yaml)

mapfile -t sources < <(awk '$1 == "-" && $2 == "source:" {print $3}' agent-env.active.yaml)
[ "${#sources[@]}" -gt 0 ] || fail "active projection declares no source"

declare -A expected=()
declare -A expected_source=()
for source in "${sources[@]}"; do
  case "$source" in
    ./agent-skills/skill-catalog) name=skill-catalog ;;
    ./agent-skills/capability-packs/*) name="${source##*/}" ;;
    ./.kb/skills/*) name="${source##*/}" ;;
    *) fail "unmanaged or forbidden active source: $source" ;;
  esac
  [ -f "${source#./}/SKILL.md" ] || fail "active source is missing SKILL.md: $source"
  expected["$name"]=1
  expected_source["$name"]="${source#./}"
done

for mirror in .codex/skills .agents/skills; do
  [ -d "$mirror" ] || fail "$mirror is missing"
  mapfile -t actual < <(find -L "$mirror" -mindepth 2 -maxdepth 2 -name SKILL.md -printf '%h\n' | sed "s#^$mirror/##" | LC_ALL=C sort)
  [ "${#actual[@]}" = "${#expected[@]}" ] || fail "$mirror has ${#actual[@]} active skills; expected ${#expected[@]}"
  for name in "${actual[@]}"; do
    [ "${expected[$name]:-}" = 1 ] || fail "$mirror/$name is a stale active projection"
    source="${expected_source[$name]}"
    diff -qr -- "$source" "$mirror/$name" >/dev/null || fail "$mirror/$name drifted from $source"
  done
done

printf 'agent-env active mirror test: PASS (%s active / %s canonical skills)\n' \
  "${#expected[@]}" \
  "$(( $(find agent-skills/capability-packs -mindepth 2 -maxdepth 2 -name SKILL.md | wc -l) + $(find .kb/skills -mindepth 2 -maxdepth 2 -name SKILL.md | wc -l) ))"
