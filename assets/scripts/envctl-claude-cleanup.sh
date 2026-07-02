#!/usr/bin/env bash
# Guarded cleanup for stale Claude Code executable install targets.
# Keeps Claude config/state, but archives/removes old executable locations after
# the envctl-managed toolchain binary and wrappers are verified.
set -euo pipefail

M="${META_ROOT:?META_ROOT required}"
REAL_HOME="${ENVCTL_REAL_HOME:-$HOME}"
ARCH_BASE="$M/var/lib/envctl/legacy-archives"
CURRENT_BIN="$M/.toolchains/claude/current/bin/claude"

usage() {
  cat <<USAGE
usage: ${0##*/} scan|clean|verify

scan   list remaining Claude executable conflict candidates (read-only)
clean  archive checksums/contents, then remove only safe stale executable targets
verify fail if stale Claude executable targets remain
USAGE
}

safe_name() {
  printf '%s' "$1" | sed 's#^/##; s#[^A-Za-z0-9._-]#__#g'
}

same_file() {
  local a="$1" b="$2"
  [ -e "$a" ] && [ -e "$b" ] && [ "$(readlink -f "$a" 2>/dev/null)" = "$(readlink -f "$b" 2>/dev/null)" ]
}

is_live() {
  local target="$1"
  [ -e "$target" ] || return 1
  if command -v fuser >/dev/null 2>&1 && fuser "$target" >/dev/null 2>&1; then
    return 0
  fi
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
  [ -e "$M/.toolchains/claude/bin/claude" ] && printf '%s\n' "$M/.toolchains/claude/bin/claude"
  for dir in "$M/.local/share/claude/versions" "$REAL_HOME/.local/share/claude/versions" "$REAL_HOME/.claude/local"; do
    [ -d "$dir" ] || continue
    find "$dir" -maxdepth 1 -mindepth 1 \( -type f -o -type l \) -print 2>/dev/null
  done | sort -u
}

is_removable_now() {
  local t="$1"
  [ -e "$t" ] || return 1
  same_file "$t" "$CURRENT_BIN" && return 1
  ! is_live "$t"
}

scan_one() {
  local t="$1" status="candidate" kind="unknown" artifacts="no"
  [ -e "$t" ] || status="absent"
  if same_file "$t" "$CURRENT_BIN"; then
    status="current-skip"
  elif is_live "$t"; then
    status="live-skip"
  elif is_removable_now "$t"; then
    status="stale-removable"
  fi
  if [ -L "$t" ]; then
    kind="symlink:$(readlink "$t" 2>/dev/null || true)"
  elif [ -f "$t" ]; then
    kind="file:$(file -b "$t" 2>/dev/null | tr '\t' ' ' | cut -c1-120)"
  elif [ -d "$t" ]; then
    kind="dir"
  fi
  has_preservable_artifacts "$t" && artifacts="yes"
  printf '%s\t%s\t%s\t%s\n' "$status" "$artifacts" "$kind" "$t"
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
  is_removable_now "$target" || { echo "skip live/current target: $target" >&2; return 0; }
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
  printf 'status\tpreservable_artifacts\tkind\ttarget\n'
  candidate_targets | sort -u | while IFS= read -r target; do
    scan_one "$target"
  done
}

clean_cmd() {
  local archive_root="$ARCH_BASE/claude-cleanup-$(date -u +%Y%m%d-%H%M%S)" any=0
  install -d -m 700 "$archive_root/manifests" "$archive_root/removed"
  scan_cmd > "$archive_root/pre-clean-scan.tsv"
  while IFS= read -r target; do
    [ -e "$target" ] || continue
    if is_removable_now "$target"; then
      any=1
      archive_then_remove "$target" "$archive_root"
    else
      echo "skip live/current target: $target" >&2
    fi
  done < <(candidate_targets | sort -u)
  scan_cmd > "$archive_root/post-clean-scan.tsv"
  if [ "$any" -eq 0 ]; then
    echo "no removable Claude cleanup targets; scan archived at $archive_root"
  else
    echo "Claude cleanup archive: $archive_root"
  fi
}

verify_cmd() {
  local bad=0 target
  while IFS= read -r target; do
    if is_removable_now "$target"; then
      echo "stale Claude executable target remains: $target" >&2
      bad=1
    fi
  done < <(candidate_targets | sort -u)
  return "$bad"
}

case "${1:-}" in
  scan) scan_cmd ;;
  clean) clean_cmd ;;
  verify) verify_cmd ;;
  -h|--help|help) usage ;;
  *) usage >&2; exit 2 ;;
esac
