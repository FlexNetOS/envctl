#!/usr/bin/env bash
# Guarded cleanup for stale Codex install/cache targets.
# Safety rule: scan every target for Rust/native/package artifacts and preserve
# target contents plus checksums under var/lib/envctl/legacy-archives before removal.
set -euo pipefail

M="${META_ROOT:?META_ROOT required}"
REAL_HOME="${ENVCTL_REAL_HOME:-$HOME}"
ARCH_BASE="$M/var/lib/envctl/legacy-archives"

usage() {
  cat <<USAGE
usage: ${0##*/} scan|clean|verify

scan   list remaining Codex conflict candidates and artifact hints (read-only)
clean  archive checksums/contents, then remove only safe stale targets
verify fail if removable stale Codex conflict candidates remain
USAGE
}

safe_name() {
  printf '%s' "$1" | sed 's#^/##; s#[^A-Za-z0-9._-]#__#g'
}

path_on_path() {
  local needle="$1" part
  IFS=':' read -r -a parts <<< "${PATH:-}"
  for part in "${parts[@]}"; do
    [ "$part" = "$needle" ] && return 0
  done
  return 1
}

lock_is_live() {
  local lock="$1"
  [ -e "$lock" ] || return 1
  if command -v fuser >/dev/null 2>&1 && fuser "$lock" >/dev/null 2>&1; then
    return 0
  fi
  return 1
}

arg0_is_live() {
  local d="$1"
  path_on_path "$d" && return 0
  lock_is_live "$d/.lock" && return 0
  return 1
}

has_preservable_artifacts() {
  local t="$1"
  [ -e "$t" ] || return 1
  if [ -d "$t" ]; then
    find "$t" -type f \( \
      -name '*.rs' -o -name 'Cargo.toml' -o -name 'Cargo.lock' -o -name 'build.rs' -o \
      -name '*.wasm' -o -name '*.node' -o -name 'package.json' -o -name 'pnpm-lock.yaml' -o \
      -name 'package-lock.json' -o -name 'bun.lock' -o -name 'bun.lockb' \
    \) -print -quit 2>/dev/null | grep -q .
  else
    case "$(basename "$t")" in
      *.rs|Cargo.toml|Cargo.lock|build.rs|*.wasm|*.node|package.json|pnpm-lock.yaml|package-lock.json|bun.lock|bun.lockb) return 0 ;;
      *) return 1 ;;
    esac
  fi
}

candidate_targets() {
  [ -e "$M/.local/archive/codex" ] && printf '%s\n' "$M/.local/archive/codex"
  if [ -d "$M/.local/archive" ]; then
    find "$M/.local/archive" -maxdepth 1 -mindepth 1 -type d -name 'codex-flat-*' -print 2>/dev/null
  fi
  for root in "$REAL_HOME/.npm/_npx" "$M/.npm/_npx"; do
    [ -d "$root" ] || continue
    find "$root" \( \
      -path '*/node_modules/@openai/codex' -o \
      -path '*/node_modules/@claude-flow/codex' -o \
      -path '*/node_modules/.bin/*codex*' \
    \) -print 2>/dev/null
  done
  if [ -d "$M/.local/share/codex/tmp/arg0" ]; then
    find "$M/.local/share/codex/tmp/arg0" -maxdepth 1 -mindepth 1 -type d -name 'codex-arg0*' -print 2>/dev/null
  fi
}

is_removable_now() {
  local t="$1"
  [ -e "$t" ] || return 1
  case "$t" in
    "$M/.local/share/codex/tmp/arg0"/codex-arg0*)
      ! arg0_is_live "$t"
      ;;
    *)
      return 0
      ;;
  esac
}

scan_one() {
  local t="$1" status="candidate" kind="unknown" count="0" artifacts="no"
  [ -e "$t" ] || status="absent"
  if [ -d "$t" ]; then
    kind="dir"
    count="$(find "$t" -type f 2>/dev/null | wc -l | tr -d ' ')"
  elif [ -L "$t" ]; then
    kind="symlink:$(readlink "$t" 2>/dev/null || true)"
    count="1"
  elif [ -f "$t" ]; then
    kind="file:$(file -b "$t" 2>/dev/null | tr '\t' ' ' | cut -c1-120)"
    count="1"
  fi
  has_preservable_artifacts "$t" && artifacts="yes"
  if [ -d "$t" ] && [[ "$t" == "$M/.local/share/codex/tmp/arg0"/codex-arg0* ]]; then
    if arg0_is_live "$t"; then status="live-skip"; else status="stale-removable"; fi
  fi
  printf '%s\t%s\t%s\t%s\t%s\n' "$status" "$artifacts" "$count" "$kind" "$t"
}

write_manifest() {
  local target="$1" manifest="$2"
  {
    printf 'target\t%s\n' "$target"
    printf 'timestamp_utc\t%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    printf 'preservable_artifacts\t'
    if has_preservable_artifacts "$target"; then printf 'yes\n'; else printf 'no\n'; fi
    printf 'entries\n'
    if [ -d "$target" ]; then
      find "$target" \( -type f -o -type l \) -print0 2>/dev/null | sort -z | while IFS= read -r -d '' p; do
        if [ -L "$p" ]; then
          printf 'L\t%s\t%s\n' "${p#$target/}" "$(readlink "$p" 2>/dev/null || true)"
        elif [ -f "$p" ]; then
          printf 'F\t%s\t%s\n' "${p#$target/}" "$(sha256sum "$p" | awk '{print $1}')"
        fi
      done
    elif [ -L "$target" ]; then
      printf 'L\t.\t%s\n' "$(readlink "$target" 2>/dev/null || true)"
    elif [ -f "$target" ]; then
      printf 'F\t.\t%s\n' "$(sha256sum "$target" | awk '{print $1}')"
    fi
  } > "$manifest"
}

archive_then_remove() {
  local target="$1" archive_root="$2" name dest manifest
  [ -e "$target" ] || return 0
  is_removable_now "$target" || { echo "skip live target: $target" >&2; return 0; }
  name="$(safe_name "$target")"
  dest="$archive_root/removed/$name"
  manifest="$archive_root/manifests/$name.tsv"
  install -d -m 700 "$(dirname "$dest")" "$(dirname "$manifest")"
  write_manifest "$target" "$manifest"
  if [ -L "$target" ]; then
    cp -a "$target" "$dest"
    rm -f "$target"
  elif [ -d "$target" ] || [ -f "$target" ]; then
    mv "$target" "$dest"
  fi
  echo "archived and removed: $target -> $dest"
}

scan_cmd() {
  printf 'status\tpreservable_artifacts\tfiles\tkind\ttarget\n'
  candidate_targets | sort -u | while IFS= read -r target; do
    scan_one "$target"
  done
}

clean_cmd() {
  local archive_root="$ARCH_BASE/codex-cleanup-$(date -u +%Y%m%d-%H%M%S)" any=0
  install -d -m 700 "$archive_root/manifests" "$archive_root/removed"
  scan_cmd > "$archive_root/pre-clean-scan.tsv"
  while IFS= read -r target; do
    [ -e "$target" ] || continue
    if is_removable_now "$target"; then
      any=1
      archive_then_remove "$target" "$archive_root"
    else
      echo "skip live target: $target" >&2
    fi
  done < <(candidate_targets | sort -u)
  scan_cmd > "$archive_root/post-clean-scan.tsv"
  if [ "$any" -eq 0 ]; then
    echo "no removable Codex cleanup targets; scan archived at $archive_root"
  else
    echo "Codex cleanup archive: $archive_root"
  fi
}

verify_cmd() {
  local bad=0 target
  if [ -e "$M/.local/archive/codex" ]; then
    echo "stale Codex archive remains: $M/.local/archive/codex" >&2
    bad=1
  fi
  if [ -d "$M/.local/archive" ] && find "$M/.local/archive" -maxdepth 3 -path '*/codex/bin/codex' -print -quit 2>/dev/null | grep -q .; then
    echo "stale executable Codex flat archive remains under $M/.local/archive" >&2
    bad=1
  fi
  for root in "$REAL_HOME/.npm/_npx" "$M/.npm/_npx"; do
    [ -d "$root" ] || continue
    if find "$root" \( -path '*/node_modules/@openai/codex' -o -path '*/node_modules/@claude-flow/codex' -o -path '*/node_modules/.bin/*codex*' \) -print -quit 2>/dev/null | grep -q .; then
      echo "stale codex-named npx cache remains under $root" >&2
      bad=1
    fi
  done
  if [ -d "$M/.local/share/codex/tmp/arg0" ]; then
    while IFS= read -r target; do
      if ! arg0_is_live "$target"; then
        echo "stale unlocked Codex arg0 dir remains: $target" >&2
        bad=1
      fi
    done < <(find "$M/.local/share/codex/tmp/arg0" -maxdepth 1 -mindepth 1 -type d -name 'codex-arg0*' -print 2>/dev/null)
  fi
  return "$bad"
}

case "${1:-}" in
  scan) scan_cmd ;;
  clean) clean_cmd ;;
  verify) verify_cmd ;;
  -h|--help|help) usage ;;
  *) usage >&2; exit 2 ;;
esac
