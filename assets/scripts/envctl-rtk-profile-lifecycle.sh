#!/usr/bin/env bash
# Keep RTK single-owned by the canonical Yazelix Nix profile. This lifecycle
# never installs a command wrapper or a Cargo-owned RTK payload.
set -euo pipefail
export PATH=/usr/bin:/bin
umask 077

rtk_die() {
  printf 'rtk-profile: %s\n' "$*" >&2
  exit 1
}

rtk_is_owned_real_dir() {
  local path="$1" uid="$2"
  [ -d "$path" ] && [ ! -L "$path" ] \
    && [ "$(/usr/bin/readlink -f -- "$path" 2>/dev/null)" = "$path" ] \
    && [ "$(/usr/bin/stat -c '%u' -- "$path")" = "$uid" ]
}

rtk_require_safe_dir() {
  local path="$1" uid="$2" mode
  rtk_is_owned_real_dir "$path" "$uid" \
    || rtk_die "expected a canonical current-user-owned real directory: $path"
  mode="$(/usr/bin/stat -c '%a' -- "$path")"
  (( (8#$mode & 0002) == 0 )) \
    || rtk_die "world-writable directory is unsafe: $path"
}

rtk_require_safe_existing_chain() {
  local root="$1" target="$2" uid="$3" relative current part
  case "$target" in "$root"|"$root"/*) ;; *)
    rtk_die "managed path escapes its root: $target"
    ;;
  esac
  rtk_require_safe_dir "$root" "$uid"
  relative="${target#"$root"}"
  relative="${relative#/}"
  current="$root"
  IFS='/' read -r -a parts <<<"$relative"
  for part in "${parts[@]}"; do
    [ -n "$part" ] || continue
    current="$current/$part"
    if [ -e "$current" ] || [ -L "$current" ]; then
      rtk_require_safe_dir "$current" "$uid"
    else
      break
    fi
  done
}

rtk_prepare_archive_base() {
  local uid="$1" directory
  rtk_require_safe_existing_chain "$RTK_META_ROOT" \
    "$RTK_META_ROOT/var/lib/envctl/legacy-archives" "$uid"
  for directory in \
    "$RTK_META_ROOT/var" \
    "$RTK_META_ROOT/var/lib" \
    "$RTK_META_ROOT/var/lib/envctl" \
    "$RTK_META_ROOT/var/lib/envctl/legacy-archives"; do
    if [ ! -e "$directory" ] && [ ! -L "$directory" ]; then
      /usr/bin/install -d -m 755 -- "$directory"
    fi
    rtk_require_safe_dir "$directory" "$uid"
  done
}

rtk_setup() {
  RTK_META_ROOT="${META_ROOT:?META_ROOT required}"
  RTK_REAL_HOME="${ENVCTL_REAL_HOME:?ENVCTL_REAL_HOME required}"
  RTK_SOURCE_ROOT="${ENVCTL_SOURCE_ROOT:-$RTK_META_ROOT/src/envctl}"
  RTK_STORE_ROOT="${ENVCTL_RTK_STORE_ROOT:-/nix/store}"
  RTK_PROFILE="$RTK_REAL_HOME/.nix-profile"
  RTK_YAZELIX_LIFECYCLE="$RTK_SOURCE_ROOT/assets/scripts/envctl-yazelix-profile-lifecycle.sh"

  case "$RTK_STORE_ROOT" in /*) ;; *) rtk_die "store root must be absolute" ;; esac
  [ -f "$RTK_YAZELIX_LIFECYCLE" ] && [ ! -L "$RTK_YAZELIX_LIFECYCLE" ] \
    && [ -x "$RTK_YAZELIX_LIFECYCLE" ] \
    || rtk_die "missing canonical Yazelix lifecycle: $RTK_YAZELIX_LIFECYCLE"

  export RTK_META_ROOT RTK_REAL_HOME RTK_SOURCE_ROOT RTK_STORE_ROOT RTK_PROFILE
  export RTK_YAZELIX_LIFECYCLE
}

rtk_profile_binary() {
  local profile_root bin toolbin resolved_bin resolved_toolbin version magic

  # This helper is frequently called from a conditional.  Do not rely on
  # `set -e` there: Bash suppresses errexit propagation for a function used
  # as a condition, which would otherwise let a source-drifted Yazelix owner
  # be treated as valid merely because the RTK binary still exists.
  "$RTK_YAZELIX_LIFECYCLE" detect >/dev/null || return 1
  [ -L "$RTK_PROFILE" ] || return 1
  profile_root="$(/usr/bin/readlink -f -- "$RTK_PROFILE" 2>/dev/null)"
  case "$profile_root" in "$RTK_STORE_ROOT"/*-profile) ;; *) return 1 ;; esac

  bin="$RTK_PROFILE/bin/rtk"
  toolbin="$RTK_PROFILE/toolbin/rtk"
  [ -x "$bin" ] && [ -x "$toolbin" ] || return 1
  resolved_bin="$(/usr/bin/readlink -f -- "$bin" 2>/dev/null)"
  resolved_toolbin="$(/usr/bin/readlink -f -- "$toolbin" 2>/dev/null)"
  [ "$resolved_bin" = "$resolved_toolbin" ] || return 1
  case "$resolved_bin" in "$RTK_STORE_ROOT"/*-rtk-*/bin/rtk) ;; *) return 1 ;; esac
  magic="$(/usr/bin/od -An -tx1 -N4 -- "$resolved_bin" 2>/dev/null \
    | /usr/bin/tr -d '[:space:]')"
  [ "$magic" = 7f454c46 ] || return 1

  version="$(/usr/bin/env -i \
    HOME="$RTK_REAL_HOME" \
    PATH="$RTK_PROFILE/toolbin:/usr/bin:/bin" \
    "$resolved_bin" --version 2>/dev/null)" || return 1
  printf '%s\n' "$version" \
    | /usr/bin/grep -Eq '^rtk [0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.-]+)?$' \
    || return 1
  printf '%s\n' "$resolved_bin"
}

rtk_shadow_paths() {
  local path
  for path in \
    "$RTK_META_ROOT/.toolchains/cargo/bin/rtk" \
    "$RTK_META_ROOT/.cargo/bin/rtk" \
    "$RTK_META_ROOT/usr/bin/rtk" \
    "$RTK_META_ROOT/.local/bin/rtk" \
    "$RTK_REAL_HOME/.cargo/bin/rtk" \
    "$RTK_REAL_HOME/.local/bin/rtk"; do
    if [ -e "$path" ] || [ -L "$path" ]; then
      printf '%s\0' "$path"
    fi
  done
}

rtk_no_shadows() {
  local path found=0
  while IFS= read -r -d '' path; do
    printf 'rtk-profile: stale parallel RTK shadow: %s\n' "$path" >&2
    found=1
  done < <(rtk_shadow_paths)
  [ "$found" -eq 0 ]
}

rtk_archive_shadows() {
  local uid path root archive_root relative destination
  local -a shadows=()
  uid="$(/usr/bin/id -u)"
  mapfile -d '' -t shadows < <(rtk_shadow_paths)
  [ "${#shadows[@]}" -gt 0 ] || return 0

  for path in "${shadows[@]}"; do
    case "$path" in
      "$RTK_META_ROOT"/*) root="$RTK_META_ROOT" ;;
      "$RTK_REAL_HOME"/*) root="$RTK_REAL_HOME" ;;
      *) rtk_die "shadow path escaped owned roots: $path" ;;
    esac
    rtk_require_safe_existing_chain "$root" "$(/usr/bin/dirname -- "$path")" "$uid"
    [ "$(/usr/bin/stat -c '%u' -- "$path")" = "$uid" ] \
      || rtk_die "refusing foreign RTK shadow: $path"
  done

  rtk_prepare_archive_base "$uid"
  archive_root="$(/usr/bin/mktemp -d \
    "$RTK_META_ROOT/var/lib/envctl/legacy-archives/rtk-profile-shadows.XXXXXXXX")"
  for path in "${shadows[@]}"; do
    case "$path" in
      "$RTK_META_ROOT"/*) relative="meta/${path#"$RTK_META_ROOT"/}" ;;
      "$RTK_REAL_HOME"/*) relative="real-home/${path#"$RTK_REAL_HOME"/}" ;;
      *) rtk_die "shadow path escaped owned roots: $path" ;;
    esac
    destination="$archive_root/$relative"
    /usr/bin/install -d -m 700 -- "$(/usr/bin/dirname -- "$destination")"
    [ ! -e "$destination" ] && [ ! -L "$destination" ] \
      || rtk_die "RTK shadow archive collision: $destination"
    /usr/bin/mv -T --no-copy -- "$path" "$destination"
    printf 'rtk-profile: archived stale shadow %s -> %s\n' "$path" "$destination"
  done
}

rtk_repair() {
  local owner_action="$1"
  "$RTK_YAZELIX_LIFECYCLE" "$owner_action" >/dev/null \
    || rtk_die "Yazelix profile owner failed its $owner_action lifecycle"
  rtk_profile_binary >/dev/null \
    || rtk_die "canonical Yazelix profile does not own one valid RTK payload"
  rtk_archive_shadows
  rtk_no_shadows || rtk_die "parallel RTK state remains after repair"
  rtk_profile_binary >/dev/null \
    || rtk_die "profile-owned RTK changed while retiring parallel state"
}

rtk_main() {
  local action="${1:-}"
  [ "$#" -eq 1 ] \
    || rtk_die "usage: envctl-rtk-profile-lifecycle.sh detect|verify|install|fix|remove"
  rtk_setup
  case "$action" in
    detect)
      rtk_profile_binary >/dev/null
      rtk_no_shadows
      ;;
    verify)
      rtk_profile_binary >/dev/null \
        || rtk_die "canonical Yazelix profile does not own one valid RTK payload"
      rtk_no_shadows || rtk_die "parallel RTK state is present"
      printf 'rtk-profile: verified one profile-owned RTK payload and no parallel shadows\n'
      ;;
    install|fix)
      rtk_repair "$action"
      printf 'rtk-profile: %s converged through the Yazelix profile owner\n' "$action"
      ;;
    remove)
      rtk_profile_binary >/dev/null \
        || rtk_die "refusing to retire shadows without a valid profile-owned RTK replacement"
      rtk_archive_shadows
      rtk_no_shadows || rtk_die "parallel RTK state remains after removal"
      rtk_profile_binary >/dev/null \
        || rtk_die "profile-owned RTK changed while retiring parallel state"
      printf 'rtk-profile: removed only parallel shadows; Yazelix profile ownership remains\n'
      ;;
    *) rtk_die "usage: envctl-rtk-profile-lifecycle.sh detect|verify|install|fix|remove" ;;
  esac
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  rtk_main "$@"
fi
