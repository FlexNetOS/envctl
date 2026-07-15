#!/usr/bin/env bash
# Keep Codex single-owned by the canonical Yazelix Nix profile.  This component
# never downloads, copies, or exposes a parallel Codex binary under META_ROOT.
set -euo pipefail
export PATH=/usr/bin:/bin
umask 077

codex_die() {
  printf 'codex-profile: %s\n' "$*" >&2
  exit 1
}

codex_is_owned_real_dir() {
  local path="$1" uid="$2"
  [ -d "$path" ] && [ ! -L "$path" ] \
    && [ "$(/usr/bin/readlink -f -- "$path" 2>/dev/null)" = "$path" ] \
    && [ "$(/usr/bin/stat -c '%u' -- "$path")" = "$uid" ]
}

codex_require_safe_dir() {
  local path="$1" uid="$2" mode
  codex_is_owned_real_dir "$path" "$uid" \
    || codex_die "expected a canonical current-user-owned real directory: $path"
  mode="$(/usr/bin/stat -c '%a' -- "$path")"
  (( (8#$mode & 0002) == 0 )) \
    || codex_die "world-writable directory is unsafe: $path"
}

codex_require_safe_existing_chain() {
  local root="$1" target="$2" uid="$3" relative current part
  case "$target" in
    "$root"|"$root"/*) ;;
    *) codex_die "managed path escapes its root: $target" ;;
  esac
  codex_require_safe_dir "$root" "$uid"
  relative="${target#"$root"}"
  relative="${relative#/}"
  current="$root"
  IFS='/' read -r -a parts <<<"$relative"
  for part in "${parts[@]}"; do
    [ -n "$part" ] || continue
    current="$current/$part"
    if [ -e "$current" ] || [ -L "$current" ]; then
      codex_require_safe_dir "$current" "$uid"
    else
      break
    fi
  done
}

codex_prepare_archive_base() {
  local uid="$1" directory
  codex_require_safe_existing_chain "$CODEX_META_ROOT" \
    "$CODEX_META_ROOT/var/lib/envctl/legacy-archives" "$uid"
  for directory in \
    "$CODEX_META_ROOT/var" \
    "$CODEX_META_ROOT/var/lib" \
    "$CODEX_META_ROOT/var/lib/envctl" \
    "$CODEX_META_ROOT/var/lib/envctl/legacy-archives"; do
    if [ ! -e "$directory" ] && [ ! -L "$directory" ]; then
      /usr/bin/install -d -m 755 -- "$directory"
    fi
    codex_require_safe_dir "$directory" "$uid"
  done
}

codex_setup() {
  CODEX_META_ROOT="${META_ROOT:?META_ROOT required}"
  CODEX_REAL_HOME="${ENVCTL_REAL_HOME:?ENVCTL_REAL_HOME required}"
  CODEX_SOURCE_ROOT="${ENVCTL_SOURCE_ROOT:-$CODEX_META_ROOT/src/envctl}"
  CODEX_STORE_ROOT="${ENVCTL_CODEX_STORE_ROOT:-/nix/store}"
  CODEX_PROFILE="$CODEX_REAL_HOME/.nix-profile"
  CODEX_YAZELIX_LIFECYCLE="$CODEX_SOURCE_ROOT/assets/scripts/envctl-yazelix-profile-lifecycle.sh"
  CODEX_CLEANUP="$CODEX_SOURCE_ROOT/assets/scripts/envctl-codex-cleanup.sh"

  case "$CODEX_STORE_ROOT" in
    /*) ;;
    *) codex_die "store root must be absolute" ;;
  esac
  [ -f "$CODEX_YAZELIX_LIFECYCLE" ] && [ ! -L "$CODEX_YAZELIX_LIFECYCLE" ] \
    && [ -x "$CODEX_YAZELIX_LIFECYCLE" ] \
    || codex_die "missing canonical Yazelix lifecycle: $CODEX_YAZELIX_LIFECYCLE"
  [ -f "$CODEX_CLEANUP" ] && [ ! -L "$CODEX_CLEANUP" ] && [ -x "$CODEX_CLEANUP" ] \
    || codex_die "missing guarded Codex cleanup helper: $CODEX_CLEANUP"

  export CODEX_META_ROOT CODEX_REAL_HOME CODEX_SOURCE_ROOT CODEX_STORE_ROOT CODEX_PROFILE
  export CODEX_YAZELIX_LIFECYCLE CODEX_CLEANUP
}

codex_profile_binary() {
  local profile_root bin toolbin resolved_bin resolved_toolbin package_root metadata version package

  "$CODEX_YAZELIX_LIFECYCLE" detect >/dev/null
  [ -L "$CODEX_PROFILE" ] || return 1
  profile_root="$(/usr/bin/readlink -f -- "$CODEX_PROFILE" 2>/dev/null)"
  case "$profile_root" in "$CODEX_STORE_ROOT"/*-profile) ;; *) return 1 ;; esac

  bin="$CODEX_PROFILE/bin/codex"
  toolbin="$CODEX_PROFILE/toolbin/codex"
  [ -x "$bin" ] && [ -x "$toolbin" ] || return 1
  resolved_bin="$(/usr/bin/readlink -f -- "$bin" 2>/dev/null)"
  resolved_toolbin="$(/usr/bin/readlink -f -- "$toolbin" 2>/dev/null)"
  [ "$resolved_bin" = "$resolved_toolbin" ] || return 1
  case "$resolved_bin" in "$CODEX_STORE_ROOT"/*-codex-cli-*/bin/codex) ;; *) return 1 ;; esac

  package_root="$(/usr/bin/dirname -- "$(/usr/bin/dirname -- "$resolved_bin")")"
  metadata="$package_root/codex-package.json"
  [ -f "$metadata" ] && [ ! -L "$metadata" ] && [ -r "$metadata" ] || return 1
  version="$(/usr/bin/jq -er '
    select(.layoutVersion == 1 and .variant == "codex" and .entrypoint == "bin/codex")
    | .version
    | select(type == "string" and test("^[0-9]+\\.[0-9]+\\.[0-9]+([.-][0-9A-Za-z.-]+)?$"))
  ' "$metadata" 2>/dev/null)" || return 1
  package="$(/usr/bin/basename -- "$package_root")"
  case "$package" in *-codex-cli-"$version") ;; *) return 1 ;; esac

  printf '%s\n' "$resolved_bin"
}

codex_profile_tool() {
  local name="$1" bin toolbin resolved_bin resolved_toolbin
  bin="$CODEX_PROFILE/bin/$name"
  toolbin="$CODEX_PROFILE/toolbin/$name"
  [ -x "$bin" ] && [ -x "$toolbin" ] || return 1
  resolved_bin="$(/usr/bin/readlink -f -- "$bin" 2>/dev/null)"
  resolved_toolbin="$(/usr/bin/readlink -f -- "$toolbin" 2>/dev/null)"
  [ "$resolved_bin" = "$resolved_toolbin" ] || return 1
  case "$resolved_bin" in "$CODEX_STORE_ROOT"/*/bin/"$name") ;; *) return 1 ;; esac
  printf '%s\n' "$resolved_bin"
}

codex_shadow_paths() {
  local path root
  for path in \
    "$CODEX_META_ROOT/usr/bin/codex" \
    "$CODEX_META_ROOT/usr/bin/codex-alpha" \
    "$CODEX_META_ROOT/.local/bin/codex" \
    "$CODEX_META_ROOT/.local/bin/codex-alpha" \
    "$CODEX_META_ROOT/.toolchains/openai-codex" \
    "$CODEX_META_ROOT/.local/share/codex" \
    "$CODEX_META_ROOT/.local/state/codex" \
    "$CODEX_META_ROOT/.toolchains/bun/bin/codex" \
    "$CODEX_META_ROOT/.toolchains/.bun/bin/codex" \
    "$CODEX_META_ROOT/.toolchains/bun/install/global/node_modules/.bin/codex" \
    "$CODEX_META_ROOT/.toolchains/.bun/install/global/node_modules/.bin/codex" \
    "$CODEX_META_ROOT/.toolchains/bun/install/global/node_modules/@openai/codex" \
    "$CODEX_META_ROOT/.toolchains/.bun/install/global/node_modules/@openai/codex" \
    "$CODEX_REAL_HOME/.local/bin/codex" \
    "$CODEX_REAL_HOME/.local/bin/codex-alpha" \
    "$CODEX_REAL_HOME/.local/share/codex" \
    "$CODEX_REAL_HOME/.local/state/codex"; do
    if [ -e "$path" ] || [ -L "$path" ]; then
      printf '%s\0' "$path"
    fi
  done

  for root in \
    "$CODEX_META_ROOT/.toolchains/bun/install/cache" \
    "$CODEX_META_ROOT/.toolchains/.bun/install/cache"; do
    [ -d "$root" ] || continue
    /usr/bin/find "$root" -mindepth 1 -maxdepth 1 -name '@openai-codex*' -print0
  done
}

codex_package_refs_absent() {
  ! /usr/bin/grep -Rqs '"@openai/codex"' \
    "$CODEX_META_ROOT/.toolchains/bun/install/global/package.json" \
    "$CODEX_META_ROOT/.toolchains/bun/install/global/bun.lock" \
    "$CODEX_META_ROOT/.toolchains/.bun/install/global/package.json" \
    "$CODEX_META_ROOT/.toolchains/.bun/install/global/bun.lock" 2>/dev/null
}

codex_no_shadows() {
  local path found=0
  while IFS= read -r -d '' path; do
    printf 'codex-profile: stale parallel Codex shadow: %s\n' "$path" >&2
    found=1
  done < <(codex_shadow_paths)
  codex_package_refs_absent || {
    printf 'codex-profile: stale @openai/codex package record under META_ROOT\n' >&2
    found=1
  }
  "$CODEX_CLEANUP" verify || found=1
  [ "$found" -eq 0 ]
}

codex_remove_package_refs() {
  local bun bun_root
  codex_package_refs_absent && return 0
  bun="$(codex_profile_tool bun)" \
    || codex_die \
      "cannot retire stale @openai/codex records without one profile-owned Bun frontdoor"
  for bun_root in "$CODEX_META_ROOT/.toolchains/bun" "$CODEX_META_ROOT/.toolchains/.bun"; do
    if /usr/bin/grep -Rqs '"@openai/codex"' \
        "$bun_root/install/global/package.json" "$bun_root/install/global/bun.lock" 2>/dev/null; then
      /usr/bin/timeout --kill-after=2s 20s /usr/bin/env -i \
        HOME="$CODEX_REAL_HOME" \
        META_ROOT="$CODEX_META_ROOT" \
        BUN_INSTALL="$bun_root" \
        XDG_CACHE_HOME="$CODEX_META_ROOT/var/cache/bun" \
        PATH="$CODEX_PROFILE/toolbin:/usr/bin:/bin" \
        "$bun" remove -g @openai/codex >/dev/null \
        || codex_die "profile-owned Bun could not retire stale @openai/codex records"
    fi
  done
  codex_package_refs_absent \
    || codex_die "could not remove stale @openai/codex package records without touching foreign packages"
}

codex_archive_shadows() {
  local uid path root archive_root relative destination
  local -a shadows=()
  uid="$(/usr/bin/id -u)"
  mapfile -d '' -t shadows < <(codex_shadow_paths)
  [ "${#shadows[@]}" -gt 0 ] || return 0

  for path in "${shadows[@]}"; do
    case "$path" in
      "$CODEX_META_ROOT"/*) root="$CODEX_META_ROOT" ;;
      "$CODEX_REAL_HOME"/*) root="$CODEX_REAL_HOME" ;;
      *) codex_die "shadow path escaped owned roots: $path" ;;
    esac
    codex_require_safe_existing_chain "$root" "$(/usr/bin/dirname -- "$path")" "$uid"
    [ "$(/usr/bin/stat -c '%u' -- "$path")" = "$uid" ] \
      || codex_die "refusing foreign Codex shadow: $path"
  done

  codex_prepare_archive_base "$uid"
  archive_root="$(/usr/bin/mktemp -d \
    "$CODEX_META_ROOT/var/lib/envctl/legacy-archives/codex-profile-shadows.XXXXXXXX")"
  for path in "${shadows[@]}"; do
    case "$path" in
      "$CODEX_META_ROOT"/*) relative="meta/${path#"$CODEX_META_ROOT"/}" ;;
      "$CODEX_REAL_HOME"/*) relative="real-home/${path#"$CODEX_REAL_HOME"/}" ;;
      *) codex_die "shadow path escaped owned roots: $path" ;;
    esac
    destination="$archive_root/$relative"
    /usr/bin/install -d -m 700 -- "$(/usr/bin/dirname -- "$destination")"
    [ ! -e "$destination" ] && [ ! -L "$destination" ] \
      || codex_die "shadow archive collision: $destination"
    /usr/bin/mv -T --no-copy -- "$path" "$destination"
    printf 'codex-profile: archived stale shadow %s -> %s\n' "$path" "$destination"
  done
}

codex_repair() {
  local owner_action="$1"
  "$CODEX_YAZELIX_LIFECYCLE" "$owner_action"
  codex_profile_binary >/dev/null \
    || codex_die "canonical Yazelix profile does not own one valid Codex binary"

  codex_archive_shadows
  if ! "$CODEX_CLEANUP" verify; then
    "$CODEX_CLEANUP" clean
    "$CODEX_CLEANUP" verify \
      || codex_die "guarded legacy Codex cleanup did not converge"
  fi
  codex_remove_package_refs
  codex_no_shadows || codex_die "parallel Codex state remains after repair"
  codex_profile_binary >/dev/null \
    || codex_die "profile-owned Codex changed while retiring parallel state"
}

codex_main() {
  local action="${1:-}"
  [ "$#" -eq 1 ] \
    || codex_die "usage: envctl-codex-profile-lifecycle.sh detect|verify|install|fix|remove"
  codex_setup
  case "$action" in
    detect)
      codex_profile_binary >/dev/null
      codex_no_shadows
      ;;
    verify)
      codex_profile_binary >/dev/null \
        || codex_die "canonical Yazelix profile does not own one valid Codex binary"
      codex_no_shadows || codex_die "parallel Codex state is present"
      printf 'codex-profile: verified single profile-owned Codex runtime\n'
      ;;
    install|fix)
      codex_repair "$action"
      printf 'codex-profile: %s converged through the Yazelix profile owner\n' "$action"
      ;;
    remove)
      codex_profile_binary >/dev/null \
        || codex_die "refusing to retire shadows without a valid profile-owned Codex replacement"
      codex_repair detect
      printf 'codex-profile: removed only parallel shadows; Yazelix profile ownership remains\n'
      ;;
    *) codex_die "usage: envctl-codex-profile-lifecycle.sh detect|verify|install|fix|remove" ;;
  esac
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  codex_main "$@"
fi
