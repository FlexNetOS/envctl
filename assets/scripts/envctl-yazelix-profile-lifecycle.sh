#!/usr/bin/env bash
# Own the FlexNetOS Yazelix foundation as one element of the real-home Nix profile.
# The profile is a generated runtime surface.  The editable input remains
# $META_ROOT/src/yazelix, and no META_ROOT-local or user-bin compatibility profile is created.
set -euo pipefail
export PATH=/usr/bin:/bin
umask 077

yazelix_die() {
  printf 'yazelix-profile: %s\n' "$*" >&2
  exit 1
}

yazelix_nix() {
  HOME="$YAZELIX_REAL_HOME" \
    XDG_STATE_HOME="$YAZELIX_REAL_HOME/.local/state" \
    XDG_CONFIG_HOME="$YAZELIX_REAL_HOME/.config" \
    XDG_CACHE_HOME="$YAZELIX_META_ROOT/var/cache/nix" \
    /nix/var/nix/profiles/default/bin/nix "$@"
}

yazelix_is_owned_real_dir() {
  local path="$1" uid="$2"
  [ -d "$path" ] && [ ! -L "$path" ] \
    && [ "$(/usr/bin/readlink -f -- "$path" 2>/dev/null)" = "$path" ] \
    && [ "$(/usr/bin/stat -c '%u' -- "$path")" = "$uid" ]
}

yazelix_require_safe_dir() {
  local path="$1" uid="$2" mode
  yazelix_is_owned_real_dir "$path" "$uid" \
    || yazelix_die "expected a canonical current-user-owned real directory: $path"
  mode="$(/usr/bin/stat -c '%a' -- "$path")"
  (( (8#$mode & 0002) == 0 )) || yazelix_die "world-writable directory is unsafe: $path"
}

yazelix_require_safe_existing_chain() {
  local root="$1" target="$2" uid="$3" relative current part
  case "$target" in "$root"|"$root"/*) ;; *) yazelix_die "managed path escapes its root: $target" ;; esac
  yazelix_require_safe_dir "$root" "$uid"
  relative="${target#"$root"}"
  relative="${relative#/}"
  current="$root"
  IFS='/' read -r -a parts <<<"$relative"
  for part in "${parts[@]}"; do
    [ -n "$part" ] || continue
    current="$current/$part"
    if [ -e "$current" ] || [ -L "$current" ]; then
      yazelix_require_safe_dir "$current" "$uid"
    else
      break
    fi
  done
}

yazelix_setup() {
  YAZELIX_META_ROOT="${1:?META_ROOT required}"
  YAZELIX_REAL_HOME="${2:?ENVCTL_REAL_HOME required}"
  YAZELIX_STORE_ROOT="${3:-/nix/store}"
  YAZELIX_SOURCE="$YAZELIX_META_ROOT/src/yazelix"
  YAZELIX_PROFILE_DIR="$YAZELIX_REAL_HOME/.local/state/nix"
  YAZELIX_PROFILE="$YAZELIX_PROFILE_DIR/profile"
  YAZELIX_LEGACY_PROFILE_DIR="$YAZELIX_REAL_HOME/.local/state/nix/profiles"
  YAZELIX_LEGACY_PROFILE="$YAZELIX_LEGACY_PROFILE_DIR/profile"
  YAZELIX_FRONTDOOR="$YAZELIX_REAL_HOME/.nix-profile"
  YAZELIX_ELEMENT=lifeos_foundation_yzx
  YAZELIX_INSTALLABLE="path:$YAZELIX_SOURCE#$YAZELIX_ELEMENT"
  YAZELIX_AGENT_DESKTOP_REL="share/yazelix/applications/com.flexnetos.Yazelix.Agent.desktop"
  YAZELIX_SYSTEM="$(/usr/bin/uname -m)-linux"
  case "$YAZELIX_SYSTEM" in x86_64-linux|aarch64-linux) ;; *) yazelix_die "unsupported Nix system: $YAZELIX_SYSTEM" ;; esac
  YAZELIX_ATTR="packages.$YAZELIX_SYSTEM.$YAZELIX_ELEMENT"
  export YAZELIX_META_ROOT YAZELIX_REAL_HOME YAZELIX_STORE_ROOT YAZELIX_SOURCE
  export YAZELIX_PROFILE_DIR YAZELIX_PROFILE YAZELIX_FRONTDOOR YAZELIX_ELEMENT
  export YAZELIX_LEGACY_PROFILE_DIR YAZELIX_LEGACY_PROFILE
  export YAZELIX_INSTALLABLE YAZELIX_SYSTEM YAZELIX_ATTR
  export YAZELIX_AGENT_DESKTOP_REL
}

yazelix_validate_roots() {
  local uid="$1"
  [ "$YAZELIX_META_ROOT" != "$YAZELIX_REAL_HOME" ] \
    || yazelix_die "real home must differ from META_ROOT"
  yazelix_require_safe_dir "$YAZELIX_META_ROOT" "$uid"
  yazelix_require_safe_dir "$YAZELIX_REAL_HOME" "$uid"
  if [ -e "$YAZELIX_META_ROOT/.nix-profile" ] || [ -L "$YAZELIX_META_ROOT/.nix-profile" ]; then
    yazelix_die "refusing parallel META_ROOT Nix profile: $YAZELIX_META_ROOT/.nix-profile"
  fi
}

yazelix_validate_source() {
  local uid="$1"
  yazelix_require_safe_dir "$YAZELIX_SOURCE" "$uid"
  for file in flake.nix flake.lock; do
    [ -f "$YAZELIX_SOURCE/$file" ] && [ ! -L "$YAZELIX_SOURCE/$file" ] \
      && [ "$(/usr/bin/stat -c '%u' -- "$YAZELIX_SOURCE/$file")" = "$uid" ] \
      || yazelix_die "canonical Yazelix source is incomplete or foreign: $YAZELIX_SOURCE/$file"
  done
}

yazelix_prepare_profile_layout() {
  local uid="$1" path
  yazelix_require_safe_existing_chain "$YAZELIX_REAL_HOME" "$YAZELIX_PROFILE_DIR" "$uid"
  for path in \
    "$YAZELIX_REAL_HOME/.local" \
    "$YAZELIX_REAL_HOME/.local/state" \
    "$YAZELIX_REAL_HOME/.local/state/nix" \
    "$YAZELIX_PROFILE_DIR"; do
    if [ ! -e "$path" ] && [ ! -L "$path" ]; then
      /usr/bin/install -d -m 700 -- "$path"
    fi
    yazelix_require_safe_dir "$path" "$uid"
  done
}

yazelix_profile_chain_ok() {
  local uid="$1" selector generation target resolved hops=0
  [ -L "$YAZELIX_FRONTDOOR" ] && [ "$(/usr/bin/stat -c '%u' -- "$YAZELIX_FRONTDOOR")" = "$uid" ] \
    && [ "$(/usr/bin/readlink -- "$YAZELIX_FRONTDOOR")" = "$YAZELIX_PROFILE" ] || return 1
  [ -L "$YAZELIX_PROFILE" ] && [ "$(/usr/bin/stat -c '%u' -- "$YAZELIX_PROFILE")" = "$uid" ] || return 1
  selector="$(/usr/bin/readlink -- "$YAZELIX_PROFILE")"
  [[ "$selector" =~ ^profile-[1-9][0-9]*-link(-[1-9][0-9]*-link)*$ ]] || return 1
  generation="$YAZELIX_PROFILE_DIR/$selector"
  while :; do
    [ -L "$generation" ] && [ "$(/usr/bin/stat -c '%u' -- "$generation")" = "$uid" ] || return 1
    target="$(/usr/bin/readlink -- "$generation")"
    case "$target" in
      "$YAZELIX_STORE_ROOT"/*-profile) break ;;
      profile-[1-9][0-9]*-link|profile-[1-9][0-9]*-link-[1-9][0-9]*-link*)
        [[ "$target" =~ ^profile-[1-9][0-9]*-link(-[1-9][0-9]*-link)*$ ]] || return 1
        generation="$YAZELIX_PROFILE_DIR/$target"
        hops=$((hops + 1))
        [ "$hops" -lt 16 ] || return 1
        continue
        ;;
      *) return 1 ;;
    esac
  done
  [ "$(/usr/bin/dirname -- "$target")" = "$YAZELIX_STORE_ROOT" ] \
    && [ -d "$target" ] && [ ! -L "$target" ] || return 1
  resolved="$(/usr/bin/readlink -f -- "$YAZELIX_FRONTDOOR" 2>/dev/null)"
  [ "$resolved" = "$target" ]
}

yazelix_require_profile_chain() {
  yazelix_profile_chain_ok "$1" \
    || yazelix_die "invalid real-home Nix profile ownership chain: $YAZELIX_FRONTDOOR -> $YAZELIX_PROFILE"
}

yazelix_legacy_profile_chain_ok() {
  local uid="$1" selector generation target resolved
  [ ! -e "$YAZELIX_PROFILE" ] && [ ! -L "$YAZELIX_PROFILE" ] || return 1
  [ -L "$YAZELIX_FRONTDOOR" ] \
    && [ "$(/usr/bin/stat -c '%u' -- "$YAZELIX_FRONTDOOR")" = "$uid" ] \
    && [ "$(/usr/bin/readlink -- "$YAZELIX_FRONTDOOR")" = "$YAZELIX_LEGACY_PROFILE" ] \
    || return 1
  [ -L "$YAZELIX_LEGACY_PROFILE" ] \
    && [ "$(/usr/bin/stat -c '%u' -- "$YAZELIX_LEGACY_PROFILE")" = "$uid" ] \
    || return 1
  selector="$(/usr/bin/readlink -- "$YAZELIX_LEGACY_PROFILE")"
  [[ "$selector" =~ ^profile-[1-9][0-9]*-link$ ]] || return 1
  generation="$YAZELIX_LEGACY_PROFILE_DIR/$selector"
  [ -L "$generation" ] && [ "$(/usr/bin/stat -c '%u' -- "$generation")" = "$uid" ] \
    || return 1
  target="$(/usr/bin/readlink -f -- "$generation" 2>/dev/null)"
  case "$target" in "$YAZELIX_STORE_ROOT"/*-profile) ;; *) return 1 ;; esac
  [ -d "$target" ] && [ ! -L "$target" ] || return 1
  resolved="$(/usr/bin/readlink -f -- "$YAZELIX_FRONTDOOR" 2>/dev/null)"
  [ "$resolved" = "$target" ]
}

yazelix_profile_json() {
  yazelix_nix profile list --profile "$YAZELIX_PROFILE" --json
}

yazelix_legacy_profile_json() {
  yazelix_nix profile list --profile "$YAZELIX_LEGACY_PROFILE" --json
}

yazelix_json_valid() {
  /usr/bin/jq -e '.version == 3 and (.elements | type == "object")' >/dev/null <<<"$1"
}

yazelix_element_store() {
  /usr/bin/jq -er --arg element "$YAZELIX_ELEMENT" '.elements[$element].storePaths[0]' <<<"$1"
}

yazelix_element_exact() {
  local json="$1" expected_store="${2:-}" expression
  # shellcheck disable=SC2016 # jq variables are intentionally expanded by jq, not Bash.
  expression='(.elements[$element] // null) as $item
    | $item != null
      and $item.active == true
      and $item.priority == 4
      and $item.originalUrl == $source
      and $item.url == $source
      and $item.attrPath == $attr
      and ($item.storePaths | type == "array" and length == 1)'
  if [ -n "$expected_store" ]; then
    expression="$expression and \$item.storePaths[0] == \$store"
  fi
  /usr/bin/jq -e \
    --arg element "$YAZELIX_ELEMENT" \
    --arg source "path:$YAZELIX_SOURCE" \
    --arg attr "$YAZELIX_ATTR" \
    --arg store "$expected_store" \
    "$expression" >/dev/null <<<"$json"
}

yazelix_foreign_elements() {
  /usr/bin/jq -cS --arg element "$YAZELIX_ELEMENT" '.elements | del(.[$element])' <<<"$1"
}

yazelix_only_foundation_element() {
  /usr/bin/jq -e --arg element "$YAZELIX_ELEMENT" \
    '(.elements | keys) == [$element]' >/dev/null <<<"$1"
}

yazelix_validate_one_desktop_entry() {
  local store="$1" profile_root="$2" relative="$3" name="$4" marker="$5" expected_exec="$6"
  local desktop="$store/$relative" profile_desktop exec_count resolved expected
  [ -f "$desktop" ] && [ -r "$desktop" ] || return 1
  for required in \
    '[Desktop Entry]' \
    'Version=1.5' \
    'Type=Application' \
    "$name" \
    'GenericName=Terminal Emulator' \
    'Icon=/home/flexnetos/.nix-profile/share/pixmaps/yazelix.png' \
    'StartupWMClass=mars' \
    'StartupNotify=true' \
    'Terminal=false' \
    'X-Yazelix-Managed=true' \
    "$marker" \
    'Categories=System;TerminalEmulator'; do
    [ "$(/usr/bin/grep -Fxc -- "$required" "$desktop")" -eq 1 ] || return 1
  done
  exec_count="$(/usr/bin/grep -c '^Exec=' "$desktop" || true)"
  [ "$exec_count" -eq 1 ] && /usr/bin/grep -Fqx -- "$expected_exec" "$desktop" || return 1

  if [ -n "$profile_root" ]; then
    profile_desktop="$profile_root/$relative"
    [ -f "$profile_desktop" ] && [ -r "$profile_desktop" ] || return 1
    resolved="$(/usr/bin/readlink -f -- "$profile_desktop" 2>/dev/null)"
    expected="$(/usr/bin/readlink -f -- "$desktop" 2>/dev/null)"
    [ "$resolved" = "$expected" ] || return 1
  fi
}

yazelix_validate_desktop_surface() {
  local store="$1" profile_root="${2:-}" size icon profile_icon resolved expected
  local agent_exec
  # The foundation deliberately exposes one profile-owned Agent entry.  It
  # invokes yzx directly; obsolete copied desktop-launch wrapper binaries are
  # not part of the package contract and must not become a fallback path.
  agent_exec='Exec=/home/flexnetos/.nix-profile/bin/yzx launch'
  yazelix_validate_one_desktop_entry \
    "$store" "$profile_root" "$YAZELIX_AGENT_DESKTOP_REL" \
    'Name=FlexNetOS Yazelix Agent' 'X-FlexNetOS-Managed=true' \
    "$agent_exec" \
    || return 1

  for size in 48x48 64x64 128x128 256x256; do
    icon="$store/share/icons/hicolor/$size/apps/yzx.png"
    [ -f "$icon" ] && [ -s "$icon" ] || return 1
    resolved="$(/usr/bin/readlink -f -- "$icon" 2>/dev/null)"
    case "$resolved" in "$YAZELIX_STORE_ROOT"/*) ;; *) return 1 ;; esac
    if [ -n "$profile_root" ]; then
      profile_icon="$profile_root/share/icons/hicolor/$size/apps/yzx.png"
      [ -f "$profile_icon" ] && [ -s "$profile_icon" ] || return 1
      expected="$(/usr/bin/readlink -f -- "$profile_icon" 2>/dev/null)"
      [ "$expected" = "$resolved" ] || return 1
    fi
  done
}

yazelix_validate_runtime_tree() {
  local store="$1" profile_root="${2:-}" relative entry resolved expected count=0
  local runtime_identity profile_identity
  case "$store" in "$YAZELIX_STORE_ROOT"/*-lifeos-foundation-yzx) ;; *) return 1 ;; esac
  [ -d "$store" ] && [ ! -L "$store" ] || return 1
  [ -x "$store/bin/yzx" ] || return 1
  runtime_identity="$store/share/yazelix/runtime_identity.json"
  [ -s "$runtime_identity" ] || return 1
  /usr/bin/jq -e \
    'type == "object" and .name == "Yazelix Nova"
      and (.version | type == "string" and length > 0)' \
    "$runtime_identity" >/dev/null || return 1
  resolved="$(/usr/bin/readlink -f -- "$runtime_identity" 2>/dev/null)"
  case "$resolved" in "$YAZELIX_STORE_ROOT"/*) ;; *) return 1 ;; esac
  if [ -n "$profile_root" ]; then
    profile_identity="$profile_root/share/yazelix/runtime_identity.json"
    [ -s "$profile_identity" ] || return 1
    expected="$(/usr/bin/readlink -f -- "$profile_identity" 2>/dev/null)"
    [ "$expected" = "$resolved" ] || return 1
  fi
  yazelix_validate_desktop_surface "$store" "$profile_root" || return 1
  [ -d "$store/toolbin" ] && [ ! -L "$store/toolbin" ] || return 1
  for relative in \
    bin/rtk \
    toolbin/rtk \
    toolbin/nu \
    nushell/config/config.nu \
    nushell/config/rtk_wrappers.nu; do
    [ -e "$store/$relative" ] && [ -r "$store/$relative" ] || return 1
    resolved="$(/usr/bin/readlink -f -- "$store/$relative" 2>/dev/null)"
    case "$resolved" in "$YAZELIX_STORE_ROOT"/*) ;; *) return 1 ;; esac
    if [ -n "$profile_root" ]; then
      [ -e "$profile_root/$relative" ] && [ -r "$profile_root/$relative" ] || return 1
      expected="$(/usr/bin/readlink -f -- "$profile_root/$relative" 2>/dev/null)"
      [ "$expected" = "$resolved" ] || return 1
    fi
  done
  [ "$(/usr/bin/readlink -f -- "$store/bin/rtk" 2>/dev/null)" = \
    "$(/usr/bin/readlink -f -- "$store/toolbin/rtk" 2>/dev/null)" ] || return 1
  /usr/bin/grep -Fqx 'use rtk_wrappers.nu *' "$store/nushell/config/config.nu" || return 1
  /usr/bin/grep -Fq 'export def --wrapped codex' "$store/nushell/config/rtk_wrappers.nu" || return 1
  /usr/bin/grep -Eq '^[[:space:]]*\^rtk codex' "$store/nushell/config/rtk_wrappers.nu" || return 1
  /usr/bin/grep -Fq 'export def --wrapped cargo' "$store/nushell/config/rtk_wrappers.nu" || return 1
  /usr/bin/grep -Eq '^[[:space:]]*\^rtk cargo' "$store/nushell/config/rtk_wrappers.nu" || return 1

  while IFS= read -r -d '' entry; do
    relative="${entry#"$store/"}"
    case "$relative" in bin/*|toolbin/*) ;; *) return 1 ;; esac
    [ -x "$entry" ] && { [ -f "$entry" ] || [ -L "$entry" ]; } || return 1
    resolved="$(/usr/bin/readlink -f -- "$entry" 2>/dev/null)"
    case "$resolved" in "$YAZELIX_STORE_ROOT"/*) ;; *) return 1 ;; esac
    if [ -n "$profile_root" ]; then
      [ -x "$profile_root/$relative" ] || return 1
      expected="$(/usr/bin/readlink -f -- "$profile_root/$relative" 2>/dev/null)"
      [ "$expected" = "$resolved" ] || return 1
    fi
    count=$((count + 1))
  done < <(/usr/bin/find "$store/bin" "$store/toolbin" -mindepth 1 -maxdepth 1 -print0 | LC_ALL=C /usr/bin/sort -z)
  [ "$count" -gt 1 ]
}

yazelix_shadow_paths() {
  local path applications="$YAZELIX_REAL_HOME/.local/share/applications"
  for path in "$YAZELIX_REAL_HOME/.local/bin" "$YAZELIX_REAL_HOME/.local/share" "$applications"; do
    if [ -e "$path" ] || [ -L "$path" ]; then
      yazelix_is_owned_real_dir "$path" "$(/usr/bin/id -u)" \
        || yazelix_die "unsafe real-home shadow parent: $path"
    fi
  done
  path="$YAZELIX_REAL_HOME/.local/bin/yzx"
  if [ -e "$path" ] || [ -L "$path" ]; then
    printf '%s\0' "$path"
  fi
  if [ -d "$applications" ] && [ ! -L "$applications" ]; then
    /usr/bin/find "$applications" -mindepth 1 -maxdepth 1 \
      \( -type f -o -type l \) -iname '*yazelix*.desktop' -print0
  fi
}

yazelix_no_shadows() {
  local path found=0
  while IFS= read -r -d '' path; do
    printf 'yazelix-profile: stale parallel shadow: %s\n' "$path" >&2
    found=1
  done < <(yazelix_shadow_paths)
  [ "$found" -eq 0 ]
}

yazelix_archive_shadows() {
  local uid="$1" path archive_root='' destination base directory
  while IFS= read -r -d '' path; do
    if [ -z "$archive_root" ]; then
      yazelix_require_safe_existing_chain "$YAZELIX_META_ROOT" \
        "$YAZELIX_META_ROOT/var/lib/envctl/legacy-archives" "$uid"
      for directory in \
        "$YAZELIX_META_ROOT/var" \
        "$YAZELIX_META_ROOT/var/lib" \
        "$YAZELIX_META_ROOT/var/lib/envctl" \
        "$YAZELIX_META_ROOT/var/lib/envctl/legacy-archives"; do
        if [ ! -e "$directory" ] && [ ! -L "$directory" ]; then
          /usr/bin/install -d -m 755 -- "$directory"
        fi
        yazelix_require_safe_dir "$directory" "$uid"
      done
      archive_root="$(/usr/bin/mktemp -d \
        "$YAZELIX_META_ROOT/var/lib/envctl/legacy-archives/yazelix-shadows.XXXXXXXX")"
    fi
    [ "$(/usr/bin/stat -c '%u' -- "$path")" = "$uid" ] \
      || yazelix_die "refusing foreign Yazelix shadow: $path"
    base="$(/usr/bin/basename -- "$path")"
    destination="$archive_root/$base"
    [ ! -e "$destination" ] && [ ! -L "$destination" ] \
      || yazelix_die "shadow archive collision: $destination"
    /usr/bin/mv -T --no-copy -- "$path" "$destination"
    printf 'yazelix-profile: archived stale shadow %s -> %s\n' "$path" "$destination"
  done < <(yazelix_shadow_paths)
}

yazelix_no_legacy_profile_layout() {
  local path
  while IFS= read -r -d '' path; do
    return 1
  done < <(/usr/bin/find "$YAZELIX_LEGACY_PROFILE_DIR" -mindepth 1 -maxdepth 1 \
    -type l \( -name profile -o -name 'profile-[1-9]*-link' \) -print0)
  return 0
}

yazelix_archive_legacy_profile_layout() {
  local uid="$1" path base destination directory archive_root
  local -a paths=()
  mapfile -d '' -t paths < <(/usr/bin/find "$YAZELIX_LEGACY_PROFILE_DIR" \
    -mindepth 1 -maxdepth 1 -type l \
    \( -name profile -o -name 'profile-[1-9]*-link' \) -print0 \
    | LC_ALL=C /usr/bin/sort -z)
  [ "${#paths[@]}" -gt 0 ] || return 0
  yazelix_require_profile_chain "$uid"
  yazelix_require_safe_existing_chain "$YAZELIX_META_ROOT" \
    "$YAZELIX_META_ROOT/var/lib/envctl/legacy-archives" "$uid"
  for directory in \
    "$YAZELIX_META_ROOT/var" \
    "$YAZELIX_META_ROOT/var/lib" \
    "$YAZELIX_META_ROOT/var/lib/envctl" \
    "$YAZELIX_META_ROOT/var/lib/envctl/legacy-archives"; do
    if [ ! -e "$directory" ] && [ ! -L "$directory" ]; then
      /usr/bin/install -d -m 755 -- "$directory"
    fi
    yazelix_require_safe_dir "$directory" "$uid"
  done
  archive_root="$(/usr/bin/mktemp -d \
    "$YAZELIX_META_ROOT/var/lib/envctl/legacy-archives/yazelix-profile-layout.XXXXXXXX")"
  for path in "${paths[@]}"; do
    [ "$(/usr/bin/stat -c '%u' -- "$path")" = "$uid" ] \
      || yazelix_die "refusing foreign legacy Nix profile link: $path"
    base="$(/usr/bin/basename -- "$path")"
    destination="$archive_root/$base"
    [ ! -e "$destination" ] && [ ! -L "$destination" ] \
      || yazelix_die "legacy profile archive collision: $destination"
    /usr/bin/mv -T --no-copy -- "$path" "$destination"
    printf 'yazelix-profile: archived legacy profile link %s -> %s\n' \
      "$path" "$destination"
  done
}

yazelix_validate_installed() {
  local uid="$1" json store
  yazelix_require_profile_chain "$uid"
  json="$(yazelix_profile_json)" || yazelix_die "could not read the real-home Nix profile"
  yazelix_json_valid "$json" || yazelix_die "malformed or unsupported Nix profile manifest"
  yazelix_only_foundation_element "$json" \
    || yazelix_die "real-home Nix profile has parallel elements; migrate them into the Yazelix foundation"
  yazelix_element_exact "$json" || yazelix_die "Yazelix profile element is missing or source-drifted"
  store="$(yazelix_element_store "$json")" || yazelix_die "Yazelix profile element has no store path"
  yazelix_validate_runtime_tree "$store" "$YAZELIX_FRONTDOOR" \
    || yazelix_die "profile-owned Yazelix runtime/toolbin frontdoors are incomplete or drifted"
  yazelix_no_legacy_profile_layout \
    || yazelix_die "parallel legacy real-home Nix profile selectors remain"
  yazelix_no_shadows || yazelix_die "remove/archive stale user-bin or desktop shadows with an explicit repair"
}

yazelix_build_candidate() {
  local output
  output="$(yazelix_nix build --accept-flake-config --impure --no-write-lock-file \
    --no-link --print-out-paths "$YAZELIX_INSTALLABLE")" \
    || yazelix_die "failed to build canonical Yazelix foundation"
  [ "$(printf '%s\n' "$output" | /usr/bin/sed '/^$/d' | /usr/bin/wc -l)" -eq 1 ] \
    || yazelix_die "canonical Yazelix build returned an ambiguous output set"
  yazelix_validate_runtime_tree "$output" \
    || yazelix_die "canonical Yazelix build failed its runtime/toolbin contract"
  printf '%s\n' "$output"
}

yazelix_restore_before() {
  local before="$1" attempts="$2" restored attempt
  for ((attempt=0; attempt<attempts; attempt++)); do
    yazelix_nix profile rollback --profile "$YAZELIX_PROFILE" >/dev/null 2>&1 || break
    restored="$(yazelix_profile_json 2>/dev/null || true)"
    if [ -n "$restored" ] \
      && [ "$(/usr/bin/jq -cS . <<<"$restored" 2>/dev/null)" = "$(/usr/bin/jq -cS . <<<"$before")" ]; then
      return 0
    fi
  done
  return 1
}

yazelix_adopt_legacy_profile() {
  local uid="$1" candidate="$2" post profile_root temporary_frontdoor
  yazelix_legacy_profile_chain_ok "$uid" \
    || yazelix_die "legacy real-home Nix profile chain changed during adoption"
  if ! yazelix_nix profile add --profile "$YAZELIX_PROFILE" --priority 4 \
      --accept-flake-config --impure --no-write-lock-file "$YAZELIX_INSTALLABLE"; then
    yazelix_die "failed to create the canonical Yazelix profile selector"
  fi
  post="$(yazelix_profile_json)" \
    || yazelix_die "could not read the canonical profile created during adoption"
  profile_root="$(/usr/bin/readlink -f -- "$YAZELIX_PROFILE" 2>/dev/null)"
  if [ "$(yazelix_foreign_elements "$post")" != '{}' ] \
    || ! yazelix_element_exact "$post" "$candidate" \
    || ! yazelix_validate_runtime_tree "$candidate" "$YAZELIX_PROFILE" \
    || [ -z "$profile_root" ]; then
    yazelix_die "candidate profile failed validation before frontdoor adoption"
  fi

  temporary_frontdoor="$YAZELIX_REAL_HOME/.nix-profile.envctl-adopt.$$"
  [ ! -e "$temporary_frontdoor" ] && [ ! -L "$temporary_frontdoor" ] \
    || yazelix_die "temporary frontdoor collision during profile adoption"
  /usr/bin/ln -s -- "$YAZELIX_PROFILE" "$temporary_frontdoor"
  /usr/bin/mv -T -- "$temporary_frontdoor" "$YAZELIX_FRONTDOOR"
  if ! yazelix_profile_chain_ok "$uid" \
    || ! yazelix_validate_runtime_tree "$candidate" "$YAZELIX_FRONTDOOR"; then
    /usr/bin/rm -f -- "$temporary_frontdoor"
    /usr/bin/ln -s -- "$YAZELIX_LEGACY_PROFILE" "$temporary_frontdoor"
    /usr/bin/mv -T -- "$temporary_frontdoor" "$YAZELIX_FRONTDOOR"
    yazelix_die "canonical profile failed validation; restored the legacy frontdoor"
  fi

  yazelix_archive_legacy_profile_layout "$uid"
  yazelix_archive_shadows "$uid"
  yazelix_validate_installed "$uid"
  printf 'yazelix-profile: adopted legacy selector into %s\n' "$YAZELIX_PROFILE"
}

yazelix_install_core() {
  local uid="$1" json='' before foreign candidate post store removed=0 had_profile=0 legacy_profile=0
  yazelix_validate_roots "$uid"
  yazelix_validate_source "$uid"
  yazelix_prepare_profile_layout "$uid"

  if [ -e "$YAZELIX_PROFILE" ] || [ -L "$YAZELIX_PROFILE" ]; then
    had_profile=1
    if [ -e "$YAZELIX_FRONTDOOR" ] || [ -L "$YAZELIX_FRONTDOOR" ]; then
      yazelix_require_profile_chain "$uid"
    else
      yazelix_die "profile selector exists without the normative real-home frontdoor"
    fi
    json="$(yazelix_profile_json)" || yazelix_die "could not read the real-home Nix profile"
    yazelix_json_valid "$json" || yazelix_die "malformed or unsupported Nix profile manifest"
  elif [ -e "$YAZELIX_FRONTDOOR" ] || [ -L "$YAZELIX_FRONTDOOR" ]; then
    yazelix_legacy_profile_chain_ok "$uid" \
      || yazelix_die "real-home profile frontdoor exists without a supported profile selector"
    legacy_profile=1
    json="$(yazelix_legacy_profile_json)" \
      || yazelix_die "could not read the legacy real-home Nix profile"
    yazelix_json_valid "$json" \
      || yazelix_die "malformed or unsupported legacy Nix profile manifest"
  else
    json='{"elements":{},"version":3}'
  fi

  before="$json"
  foreign="$(yazelix_foreign_elements "$before")"
  [ "$foreign" = '{}' ] \
    || yazelix_die "real-home Nix profile has parallel elements; migrate them into the Yazelix foundation"
  candidate="$(yazelix_build_candidate)"

  if [ "$legacy_profile" -eq 1 ]; then
    yazelix_adopt_legacy_profile "$uid" "$candidate"
    return 0
  fi

  # A path flake can retain the same declared URL while its content changes. Build the candidate
  # before accepting an incumbent so a source update replaces the old store path instead of
  # silently leaving the profile on stale package bytes.
  if yazelix_element_exact "$before" "$candidate"; then
    store="$(yazelix_element_store "$before")"
    if yazelix_validate_runtime_tree "$store" "$YAZELIX_FRONTDOOR"; then
      yazelix_archive_shadows "$uid"
      yazelix_validate_installed "$uid"
      printf 'yazelix-profile: already current\n'
      return 0
    fi
  fi

  if /usr/bin/jq -e --arg element "$YAZELIX_ELEMENT" '.elements[$element] != null' \
      >/dev/null <<<"$before"; then
    yazelix_nix profile remove --profile "$YAZELIX_PROFILE" "$YAZELIX_ELEMENT" \
      || yazelix_die "failed to retire the drifted Yazelix profile element"
    removed=1
    post="$(yazelix_profile_json)" || yazelix_die "profile became unreadable after Yazelix retirement"
    [ "$(yazelix_foreign_elements "$post")" = "$foreign" ] \
      || yazelix_die "unrelated profile elements changed during Yazelix retirement"
  fi

  if ! yazelix_nix profile add --profile "$YAZELIX_PROFILE" --priority 4 \
      --accept-flake-config --impure --no-write-lock-file "$YAZELIX_INSTALLABLE"; then
    if [ "$removed" -eq 1 ]; then
      yazelix_restore_before "$before" 1 \
        || yazelix_die "Yazelix add failed and the prior profile could not be restored"
    fi
    yazelix_die "failed to add the canonical Yazelix foundation to the real-home profile"
  fi

  if [ ! -e "$YAZELIX_FRONTDOOR" ] && [ ! -L "$YAZELIX_FRONTDOOR" ]; then
    /usr/bin/ln -s -- "$YAZELIX_PROFILE" "$YAZELIX_FRONTDOOR"
  fi
  post="$(yazelix_profile_json)" || yazelix_die "could not read the updated real-home Nix profile"
  if [ "$(yazelix_foreign_elements "$post")" != '{}' ] \
    || ! yazelix_element_exact "$post" "$candidate" \
    || ! yazelix_profile_chain_ok "$uid" \
    || ! yazelix_validate_runtime_tree "$candidate" "$YAZELIX_FRONTDOOR"; then
    if [ "$had_profile" -eq 1 ]; then
      yazelix_restore_before "$before" "$((removed + 1))" \
        || yazelix_die "profile verification failed and rollback was incomplete"
    else
      yazelix_nix profile remove --profile "$YAZELIX_PROFILE" "$YAZELIX_ELEMENT" >/dev/null 2>&1 || true
      /usr/bin/rm -f -- "$YAZELIX_FRONTDOOR"
    fi
    yazelix_die "updated Yazelix profile failed ownership or preservation checks"
  fi

  yazelix_archive_shadows "$uid"
  yazelix_validate_installed "$uid"
  printf 'yazelix-profile: installed %s as the one real-home profile element\n' "$candidate"
}

yazelix_remove_core() {
  local uid="$1" before foreign post
  yazelix_validate_roots "$uid"
  yazelix_validate_source "$uid"
  if [ ! -e "$YAZELIX_PROFILE" ] && [ ! -L "$YAZELIX_PROFILE" ] \
    && [ ! -e "$YAZELIX_FRONTDOOR" ] && [ ! -L "$YAZELIX_FRONTDOOR" ]; then
    printf 'yazelix-profile: already absent\n'
    return 0
  fi
  yazelix_require_profile_chain "$uid"
  before="$(yazelix_profile_json)" || yazelix_die "could not read the real-home Nix profile"
  yazelix_json_valid "$before" || yazelix_die "malformed or unsupported Nix profile manifest"
  if ! /usr/bin/jq -e --arg element "$YAZELIX_ELEMENT" '.elements[$element] != null' \
      >/dev/null <<<"$before"; then
    printf 'yazelix-profile: already absent\n'
    return 0
  fi
  yazelix_element_exact "$before" \
    || yazelix_die "refusing to remove a non-canonical element that merely reuses the owned name"
  foreign="$(yazelix_foreign_elements "$before")"
  yazelix_nix profile remove --profile "$YAZELIX_PROFILE" "$YAZELIX_ELEMENT" \
    || yazelix_die "failed to remove the owned Yazelix profile element"
  post="$(yazelix_profile_json)" || yazelix_die "profile became unreadable after Yazelix removal"
  if [ "$(yazelix_foreign_elements "$post")" != "$foreign" ] \
    || /usr/bin/jq -e --arg element "$YAZELIX_ELEMENT" '.elements[$element] != null' \
      >/dev/null <<<"$post"; then
    yazelix_die "unrelated profile elements changed during Yazelix removal"
  fi
  yazelix_require_profile_chain "$uid"
  printf 'yazelix-profile: removed only %s; profile and unrelated elements preserved\n' "$YAZELIX_ELEMENT"
}

yazelix_main() {
  local action="${1:-}" uid
  [ "$#" -eq 1 ] || yazelix_die "usage: envctl-yazelix-profile-lifecycle.sh detect|verify|install|fix|remove"
  yazelix_setup "${META_ROOT:?META_ROOT required}" \
    "${ENVCTL_REAL_HOME:?ENVCTL_REAL_HOME required}" /nix/store
  uid="$(/usr/bin/id -u)"
  case "$action" in
    detect|verify)
      yazelix_validate_roots "$uid"
      yazelix_validate_source "$uid"
      yazelix_validate_installed "$uid"
      [ "$action" = detect ] || printf 'yazelix-profile: verified canonical real-home profile ownership\n'
      ;;
    install|fix)
      yazelix_install_core "$uid"
      ;;
    remove)
      yazelix_remove_core "$uid"
      ;;
    *) yazelix_die "usage: envctl-yazelix-profile-lifecycle.sh detect|verify|install|fix|remove" ;;
  esac
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  yazelix_main "$@"
fi
