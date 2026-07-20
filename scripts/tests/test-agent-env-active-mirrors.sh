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
  if [ "$name" = meta-gitkb-review ]; then
    grep -A3 '^  meta-gitkb-review:$' agent-skills/skill-catalog/catalog.yaml \
      | grep -Fx '    source: https://github.com/FlexNetOS/meta' >/dev/null \
      || fail "catalog Meta review source is not canonical"
    grep -A3 '^  meta-gitkb-review:$' agent-skills/skill-catalog/catalog.yaml \
      | grep -Fx '    source_ref: fb7273a7c8d05dce0bac649ded940a86ad41e107' >/dev/null \
      || fail "catalog Meta review source is not pinned"
    grep -A3 '^  meta-gitkb-review:$' agent-skills/skill-catalog/catalog.yaml \
      | grep -Fx '    sub_dir: agent-env/skills' >/dev/null \
      || fail "catalog Meta review source has the wrong subdirectory"
    continue
  fi
  [ -f "agent-skills/$name/SKILL.md" ] \
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
declare -A expected_remote=()
for source in "${sources[@]}"; do
  case "$source" in
    ./agent-skills/skill-catalog) name=skill-catalog ;;
    ./agent-skills/*) name="${source##*/}" ;;
    ./.kb/skills/*) name="${source##*/}" ;;
    https://github.com/FlexNetOS/meta)
      name=meta-gitkb-review
      grep -A4 '^  - source: https://github.com/FlexNetOS/meta$' agent-env.active.yaml \
        | grep -Fx '    ref: fb7273a7c8d05dce0bac649ded940a86ad41e107' >/dev/null \
        || fail "active Meta review source is not pinned"
      grep -A4 '^  - source: https://github.com/FlexNetOS/meta$' agent-env.active.yaml \
        | grep -Fx '    sub-dir: agent-env/skills' >/dev/null \
        || fail "active Meta review source has the wrong subdirectory"
      expected_remote["$name"]=1
      ;;
    *) fail "unmanaged or forbidden active source: $source" ;;
  esac
  if [ "${expected_remote[$name]:-}" != 1 ]; then
    [ -f "${source#./}/SKILL.md" ] || fail "active source is missing SKILL.md: $source"
  fi
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
    if [ "${expected_remote[$name]:-}" != 1 ]; then
      diff -qr -- "$source" "$mirror/$name" >/dev/null || fail "$mirror/$name drifted from $source"
    fi
  done
done

diff -qr -- .codex/skills/meta-gitkb-review .agents/skills/meta-gitkb-review >/dev/null \
  || fail "remote Meta review projections disagree"
grep -F 'source: https://github.com/FlexNetOS/meta' agent-env.lock >/dev/null \
  || fail "lock lost Meta review provenance"
grep -F 'source_revision: ref:fb7273a7c8d05dce0bac649ded940a86ad41e107' agent-env.lock >/dev/null \
  || fail "lock lost Meta review pin"

printf 'agent-env active mirror test: PASS (%s active / %s canonical skills)\n' \
  "${#expected[@]}" \
  "$(( $(find agent-skills -mindepth 2 -maxdepth 2 -name SKILL.md | wc -l) + $(find .kb/skills -mindepth 2 -maxdepth 2 -name SKILL.md | wc -l) ))"
