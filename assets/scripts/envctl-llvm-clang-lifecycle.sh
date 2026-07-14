#!/usr/bin/env bash
# Managed LLVM/clang lifecycle used by both Install and Fix. The component pins immutable upstream
# archive and critical-tool digests in main(); helpers accept explicit values so hermetic tests can
# exercise the same staging/validation/commit code with tiny fixtures.
set -euo pipefail
export PATH=/usr/bin:/bin
# GNU tar applies the caller's umask while extracting as an unprivileged user. Pin it so archive
# modes—and therefore the complete-generation identity—cannot vary with the invoking shell.
umask 022

llvm_die() { echo "llvm: $*" >&2; exit 1; }

llvm_canonical_is_lexical() {
  [ "$(readlink -f -- "$1" 2>/dev/null)" = "$1" ]
}

llvm_owned_real_dir() {
  local path="$1" uid="$2"
  [ -d "$path" ] && [ ! -L "$path" ] && llvm_canonical_is_lexical "$path" \
    && [ "$(stat -c '%u' "$path")" = "$uid" ]
}

llvm_validate_chain() {
  local meta="$1" target="$2" uid="$3" relative current component
  case "$target" in "$meta"|"$meta"/*) ;; *) llvm_die "managed path escapes META_ROOT: $target" ;; esac
  llvm_owned_real_dir "$meta" "$uid" || llvm_die "META_ROOT is not a canonical current-user-owned real directory"
  case "$(stat -c '%a' "$meta")" in 700|755|775) ;; *) llvm_die "unsafe META_ROOT mode" ;; esac
  relative="${target#"$meta"}"
  relative="${relative#/}"
  current="$meta"
  IFS='/' read -r -a components <<<"$relative"
  for component in "${components[@]}"; do
    [ -n "$component" ] || continue
    current="$current/$component"
    if [ -e "$current" ] || [ -L "$current" ]; then
      llvm_owned_real_dir "$current" "$uid" || llvm_die "unsafe managed parent: $current"
      case "$(stat -c '%a' "$current")" in 700|755|775) ;; *) llvm_die "unsafe managed parent mode: $current" ;; esac
    else
      break
    fi
  done
}

llvm_ensure_layout() {
  local meta="$1" uid="$2"
  llvm_validate_chain "$meta" "$meta/.toolchains" "$uid"
  llvm_validate_chain "$meta" "$meta/usr/bin" "$uid"
  llvm_validate_chain "$meta" "$meta/var/tmp/envctl" "$uid"
  [ -e "$meta/.toolchains" ] || /usr/bin/install -d -m755 "$meta/.toolchains"
  [ -e "$meta/usr" ] || /usr/bin/install -d -m755 "$meta/usr"
  [ -e "$meta/usr/bin" ] || /usr/bin/install -d -m755 "$meta/usr/bin"
  [ -e "$meta/var" ] || /usr/bin/install -d -m755 "$meta/var"
  [ -e "$meta/var/tmp" ] || /usr/bin/install -d -m755 "$meta/var/tmp"
  [ -e "$meta/var/tmp/envctl" ] || /usr/bin/install -d -m700 "$meta/var/tmp/envctl"
  llvm_owned_real_dir "$meta/.toolchains" "$uid" || llvm_die "unsafe .toolchains directory"
  llvm_owned_real_dir "$meta/usr/bin" "$uid" || llvm_die "unsafe usr/bin directory"
  llvm_owned_real_dir "$meta/var/tmp/envctl" "$uid" \
    && [ "$(stat -c '%a' "$meta/var/tmp/envctl")" = 700 ] \
    || llvm_die "unsafe envctl scratch directory"
}

llvm_validate_file() {
  local path="$1" mode="$2" digest="$3" uid="$4"
  [ -x "$path" ] && [ -f "$path" ] && [ ! -L "$path" ] \
    && llvm_canonical_is_lexical "$path" \
    && [ "$(stat -c '%a' "$path")" = "$mode" ] \
    && [ "$(stat -c '%u' "$path")" = "$uid" ] \
    && printf '%s  %s\n' "$digest" "$path" | /usr/bin/sha256sum --check --status
}

llvm_validate_resource_tree() {
  local root="$1" expected_digest="$2" uid="$3" node actual
  llvm_owned_real_dir "$root" "$uid" || return 1
  while IFS= read -r -d '' node; do
    [ ! -L "$node" ] && [ "$(stat -c '%u' "$node")" = "$uid" ] \
      && { [ -f "$node" ] || [ -d "$node" ]; } || return 1
  done < <(/usr/bin/find "$root" -print0)
  actual="$(
    cd "$root"
    /usr/bin/find . -type f -print0 \
      | LC_ALL=C /usr/bin/sort -z \
      | /usr/bin/xargs -0 /usr/bin/sha256sum \
      | /usr/bin/sha256sum \
      | /usr/bin/awk '{print $1}'
  )"
  [ "$actual" = "$expected_digest" ]
}

# Bind the complete installed generation, not only the compiler entrypoints. The record is stable
# across extraction locations and owners: every node contributes its relative path, kind, mode,
# and either regular-file bytes or symlink target. Ownership is validated separately against the
# current uid, and special files are rejected. The envctl marker is excluded because it carries
# this digest.
llvm_generation_digest() {
  local directory="$1" uid="$2"
  (
    cd "$directory"
    while IFS= read -r -d '' node; do
      [ "$node" != ./.envctl-release ] || continue
      [ "$(stat -c '%u' -- "$node")" = "$uid" ] || exit 1
      relative="${node#./}"
      mode="$(stat -c '%a' -- "$node")"
      if [ -L "$node" ]; then
        printf 'l\0%s\0%s\0%s\0' "$relative" "$mode" "$(readlink -- "$node")"
      elif [ -d "$node" ]; then
        printf 'd\0%s\0%s\0' "$relative" "$mode"
      elif [ -f "$node" ]; then
        printf 'f\0%s\0%s\0%s\0' "$relative" "$mode" \
          "$(/usr/bin/sha256sum "$node" | /usr/bin/awk '{print $1}')"
      else
        exit 1
      fi
    done < <(/usr/bin/find . -mindepth 1 -print0 | LC_ALL=C /usr/bin/sort -z)
  ) | /usr/bin/sha256sum | /usr/bin/awk '{print $1}'
}

llvm_require_atomic_mv() {
  /usr/bin/mv --help 2>&1 | /usr/bin/grep -Fq -- '--exchange' \
    && /usr/bin/mv --help 2>&1 | /usr/bin/grep -Fq -- '--no-copy' \
    || llvm_die "host mv lacks required atomic --exchange/--no-copy support"
}

# Small exact-command seams let the hermetic fixture inject namespace/fsync failures without
# allowing production callers to redirect a command through PATH or environment variables.
llvm_mv() { /usr/bin/mv "$@"; }
llvm_sync() { /usr/bin/sync "$@"; }

llvm_download_archive() {
  local url="$1" archive="$2" download_root="$3"
  /usr/bin/env -i HOME="$download_root" PATH=/usr/bin:/bin \
    /usr/bin/curl --disable --proto '=https' --tlsv1.2 --fail --location \
      --silent --show-error "$url" -o "$archive"
}

llvm_validate_generation() {
  local directory="$1" tag="$2" archive_digest="$3" clang_digest="$4" ar_digest="$5"
  local resource_digest="$6" generation_digest="$7" uid="$8"
  llvm_owned_real_dir "$directory" "$uid" && [ "$(stat -c '%a' "$directory")" = 755 ] || return 1
  local marker="$directory/.envctl-release"
  [ -f "$marker" ] && [ ! -L "$marker" ] && llvm_canonical_is_lexical "$marker" \
    && [ "$(stat -c '%a' "$marker")" = 644 ] && [ "$(stat -c '%u' "$marker")" = "$uid" ] \
    && [ "$(cat "$marker")" = "$tag $archive_digest $resource_digest $generation_digest" ] || return 1
  llvm_validate_file "$directory/bin/clang" 755 "$clang_digest" "$uid" || return 1
  llvm_validate_file "$directory/bin/clang++" 755 "$clang_digest" "$uid" || return 1
  llvm_validate_file "$directory/bin/llvm-ar" 755 "$ar_digest" "$uid" || return 1
  llvm_validate_file "$directory/bin/llvm-ranlib" 755 "$ar_digest" "$uid" || return 1
  llvm_validate_resource_tree "$directory/lib/clang/21" "$resource_digest" "$uid" || return 1
  [ "$(llvm_generation_digest "$directory" "$uid")" = "$generation_digest" ] || return 1
}

llvm_validate_managed_identity() {
  local directory="$1" tag="$2" archive_digest="$3" resource_digest="$4" generation_digest="$5" uid="$6"
  local marker="$directory/.envctl-release"
  llvm_owned_real_dir "$directory" "$uid" && [ "$(stat -c '%a' "$directory")" = 755 ] \
    && [ -f "$marker" ] && [ ! -L "$marker" ] && llvm_canonical_is_lexical "$marker" \
    && [ "$(stat -c '%a' "$marker")" = 644 ] && [ "$(stat -c '%u' "$marker")" = "$uid" ] \
    && [ "$(cat "$marker")" = "$tag $archive_digest $resource_digest $generation_digest" ]
}

llvm_wrapper_body() {
  local private="$1"
  printf '%s\n' '#!/bin/sh' "exec \"$private\" \"\$@\""
}

llvm_managed_frontdoor() {
  local front="$1" private="$2" uid="$3"
  [ -e "$front" ] || [ -L "$front" ] || return 1
  if [ -L "$front" ]; then
    # GNU stat without -L is an lstat: ownership is bound to the symlink inode, not its target.
    # `readlink -m` follows and lexically normalizes the link while tolerating a missing payload,
    # which lets Fix recognize a known managed legacy frontdoor and repair its generation.
    [ "$(stat -c '%u' -- "$front")" = "$uid" ] \
      && [ "$(readlink -m -- "$front" 2>/dev/null || true)" = "$(readlink -m -- "$private" 2>/dev/null || true)" ]
    return
  fi
  [ -f "$front" ] && llvm_canonical_is_lexical "$front" \
    && [ "$(stat -c '%u' "$front")" = "$uid" ] && [ "$(stat -c '%a' "$front")" = 755 ] \
    && [ "$(cat "$front")" = "$(llvm_wrapper_body "$private")" ]
}

llvm_preflight_frontdoors() {
  local meta="$1" destination="$2" uid="$3" binary front private
  for binary in clang clang++ clang-cpp clang-21 llvm-config llc llvm-ar llvm-ranlib llvm-nm llvm-objcopy llvm-objdump lld ld.lld; do
    front="$meta/usr/bin/$binary"
    private="$destination/bin/$binary"
    if [ -e "$front" ] || [ -L "$front" ]; then
      llvm_managed_frontdoor "$front" "$private" "$uid" \
        || llvm_die "refusing foreign LLVM frontdoor: $front"
    fi
  done
}

llvm_validate_frontdoor_set() {
  local meta="$1" destination="$2" uid="$3" binary front private
  for binary in clang clang++ clang-cpp clang-21 llvm-config llc llvm-ar llvm-ranlib llvm-nm llvm-objcopy llvm-objdump lld ld.lld; do
    front="$meta/usr/bin/$binary"
    private="$destination/bin/$binary"
    if [ -e "$private" ] && "$private" --version >/dev/null 2>&1; then
      llvm_managed_frontdoor "$front" "$private" "$uid" \
        || llvm_die "managed LLVM frontdoor is missing or drifted: $front"
    elif [ -e "$front" ] || [ -L "$front" ]; then
      llvm_die "stale LLVM frontdoor exists for an unavailable payload: $front"
    fi
  done
}

llvm_frontdoor_set_ok() {
  local meta="$1" destination="$2" uid="$3" binary front private
  for binary in clang clang++ clang-cpp clang-21 llvm-config llc llvm-ar llvm-ranlib llvm-nm llvm-objcopy llvm-objdump lld ld.lld; do
    front="$meta/usr/bin/$binary"
    private="$destination/bin/$binary"
    if [ -e "$private" ] && "$private" --version >/dev/null 2>&1; then
      llvm_managed_frontdoor "$front" "$private" "$uid" || return 1
    elif [ -e "$front" ] || [ -L "$front" ]; then
      return 1
    fi
  done
}

llvm_write_frontdoor() {
  local front="$1" private="$2" parent staged
  parent="$(dirname "$front")"
  staged="$(mktemp "$parent/.llvm-frontdoor.XXXXXX")"
  llvm_wrapper_body "$private" >"$staged"
  chmod 755 "$staged"
  if ! llvm_mv -T --no-copy -- "$staged" "$front"; then
    /usr/bin/rm -f -- "$staged"
    return 1
  fi
  llvm_sync -f "$parent"
}

llvm_commit_generation() {
  local parent="$1" staged="$2" current="$3"
  llvm_activation_fallback="$staged"
  llvm_activation_was_initial=0
  llvm_require_atomic_mv
  if [ -e "$current" ] || [ -L "$current" ]; then
    llvm_mv -T --exchange --no-copy -- "$staged" "$current" \
      || llvm_die "atomic LLVM generation exchange failed"
    if ! llvm_sync -f "$parent"; then
      if llvm_mv -T --exchange --no-copy -- "$staged" "$current" \
        && llvm_sync -f "$parent"; then
        llvm_die "LLVM generation exchange was rolled back after sync failure"
      fi
      llvm_die "LLVM generation rollback failed; a complete generation may remain active"
    fi
  else
    llvm_activation_was_initial=1
    llvm_mv -T --no-copy -- "$staged" "$current" \
      || llvm_die "initial LLVM generation activation failed"
    if ! llvm_sync -f "$parent"; then
      if llvm_mv -T --no-copy -- "$current" "$staged" \
        && llvm_sync -f "$parent"; then
        llvm_die "initial LLVM activation was rolled back after sync failure"
      fi
      llvm_die "initial LLVM activation rollback failed; a complete generation may remain active"
    fi
  fi
}

llvm_rollback_activation() {
  local parent="$1" current="$2" fallback="$3" was_initial="$4"
  if [ "$was_initial" = 1 ]; then
    llvm_mv -T --no-copy -- "$current" "$fallback" || return 1
  else
    llvm_mv -T --exchange --no-copy -- "$fallback" "$current" || return 1
  fi
  llvm_sync -f "$parent" || return 1
  # `fallback` now names only the failed candidate generation.
  /usr/bin/rm -rf --one-file-system -- "$fallback" || return 1
  llvm_sync -f "$parent"
}

llvm_rollback_frontdoors_and_activation() {
  local meta="$1" parent="$2" current="$3" fallback="$4" was_initial="$5"
  local created_name="$6" pruned_fronts_name="$7" pruned_backups_name="$8" index failed
  local -n created_ref="$created_name" pruned_fronts_ref="$pruned_fronts_name" pruned_backups_ref="$pruned_backups_name"
  failed=0
  for ((index=${#created_ref[@]} - 1; index >= 0; index--)); do
    /usr/bin/rm -f -- "${created_ref[$index]}" || failed=1
  done
  for ((index=${#pruned_backups_ref[@]} - 1; index >= 0; index--)); do
    llvm_mv -T --no-copy -- "${pruned_backups_ref[$index]}" "${pruned_fronts_ref[$index]}" || failed=1
  done
  llvm_sync -f "$meta/usr/bin" || failed=1
  llvm_rollback_activation "$parent" "$current" "$fallback" "$was_initial" || failed=1
  [ "$failed" = 0 ]
}

llvm_install_generation() {
  local meta="$1" tag="$2" version="$3" asset_arch="$4"
  local archive_digest="$5" clang_digest="$6" ar_digest="$7" resource_digest="$8" generation_digest="$9"
  local uid destination parent url download_root staged_root archive binary src front backup cleanup_failed
  local -a created_fronts=() pruned_fronts=() pruned_backups=()
  uid="$(id -u)"
  destination="$meta/.toolchains/llvm"
  parent="$meta/.toolchains"
  llvm_ensure_layout "$meta" "$uid"
  if [ -e "$destination" ] || [ -L "$destination" ]; then
    llvm_validate_managed_identity "$destination" "$tag" "$archive_digest" "$resource_digest" "$generation_digest" "$uid" \
      || llvm_die "refusing unsafe or foreign existing LLVM generation"
  fi
  llvm_preflight_frontdoors "$meta" "$destination" "$uid"
  if [ -e "$destination" ] \
    && llvm_validate_generation "$destination" "$tag" "$archive_digest" "$clang_digest" "$ar_digest" "$resource_digest" "$generation_digest" "$uid" \
    && llvm_frontdoor_set_ok "$meta" "$destination" "$uid"; then
    return 0
  fi

  url="https://github.com/llvm/llvm-project/releases/download/${tag}/LLVM-${version}-Linux-${asset_arch}.tar.xz"
  download_root="$(mktemp -d "$meta/var/tmp/envctl/llvm-download.XXXXXX")"
  staged_root="$(mktemp -d "$parent/.llvm.next.XXXXXX")"
  trap '/usr/bin/rm -rf --one-file-system -- "${download_root:-}" "${staged_root:-}"' EXIT
  archive="$download_root/llvm.tar.xz"
  llvm_download_archive "$url" "$archive" "$download_root"
  printf '%s  %s\n' "$archive_digest" "$archive" | /usr/bin/sha256sum --check --status \
    || llvm_die "release archive SHA-256 mismatch"
  /usr/bin/tar -xJf "$archive" -C "$staged_root" --strip-components=1
  chmod 755 "$staged_root"
  /usr/bin/rm -f -- "$staged_root/bin/clang" "$staged_root/bin/clang++" "$staged_root/bin/llvm-ranlib"
  /usr/bin/ln "$staged_root/bin/clang-21" "$staged_root/bin/clang"
  /usr/bin/ln "$staged_root/bin/clang-21" "$staged_root/bin/clang++"
  /usr/bin/ln "$staged_root/bin/llvm-ar" "$staged_root/bin/llvm-ranlib"
  chmod 755 "$staged_root/bin/clang" "$staged_root/bin/clang++" \
    "$staged_root/bin/llvm-ar" "$staged_root/bin/llvm-ranlib"
  [ "$(llvm_generation_digest "$staged_root" "$uid")" = "$generation_digest" ] \
    || llvm_die "staged LLVM complete-generation SHA-256 mismatch"
  printf '%s\n' "$tag $archive_digest $resource_digest $generation_digest" >"$staged_root/.envctl-release"
  chmod 644 "$staged_root/.envctl-release"
  llvm_validate_generation "$staged_root" "$tag" "$archive_digest" "$clang_digest" "$ar_digest" "$resource_digest" "$generation_digest" "$uid" \
    || llvm_die "staged LLVM generation failed critical payload validation"
  "$staged_root/bin/clang" --version >/dev/null \
    || llvm_die "staged pinned clang is not executable on this host"
  "$staged_root/bin/llvm-ar" --version >/dev/null \
    || llvm_die "staged pinned llvm-ar is not executable on this host"

  llvm_commit_generation "$parent" "$staged_root" "$destination"
  staged_root=""
  for binary in clang clang++ clang-cpp clang-21 llvm-config llc llvm-ar llvm-ranlib llvm-nm llvm-objcopy llvm-objdump lld ld.lld; do
    src="$destination/bin/$binary"
    front="$meta/usr/bin/$binary"
    if [ -e "$src" ] && "$src" --version >/dev/null 2>&1; then
      if [ -e "$front" ] || [ -L "$front" ]; then
        # The wrapper names the stable generation path, so an already-managed frontdoor remains
        # correct across a payload exchange and need not be rewritten.
        if ! llvm_managed_frontdoor "$front" "$src" "$uid"; then
          llvm_rollback_frontdoors_and_activation "$meta" "$parent" "$destination" \
            "$llvm_activation_fallback" "$llvm_activation_was_initial" \
            created_fronts pruned_fronts pruned_backups \
            || llvm_die "foreign LLVM frontdoor appeared after activation and rollback was incomplete: $front"
          llvm_die "foreign LLVM frontdoor appeared after activation; activation was rolled back: $front"
        fi
      else
        created_fronts+=("$front")
        if ! llvm_write_frontdoor "$front" "$src"; then
          llvm_rollback_frontdoors_and_activation "$meta" "$parent" "$destination" \
            "$llvm_activation_fallback" "$llvm_activation_was_initial" \
            created_fronts pruned_fronts pruned_backups \
            || llvm_die "LLVM frontdoor creation failed and activation rollback was incomplete"
          llvm_die "LLVM frontdoor creation failed; activation was rolled back"
        fi
      fi
    elif [ -e "$front" ] || [ -L "$front" ]; then
      if ! llvm_managed_frontdoor "$front" "$src" "$uid"; then
        llvm_rollback_frontdoors_and_activation "$meta" "$parent" "$destination" \
          "$llvm_activation_fallback" "$llvm_activation_was_initial" \
          created_fronts pruned_fronts pruned_backups \
          || llvm_die "foreign LLVM frontdoor appeared during pruning and rollback was incomplete: $front"
        llvm_die "foreign LLVM frontdoor appeared during pruning; activation was rolled back: $front"
      fi
      backup="$(mktemp "$meta/usr/bin/.llvm-prune.${binary}.XXXXXX")"
      /usr/bin/rm -f -- "$backup"
      if ! llvm_mv -T --no-copy -- "$front" "$backup"; then
        llvm_rollback_frontdoors_and_activation "$meta" "$parent" "$destination" \
          "$llvm_activation_fallback" "$llvm_activation_was_initial" \
          created_fronts pruned_fronts pruned_backups \
          || llvm_die "LLVM frontdoor pruning failed and activation rollback was incomplete"
        llvm_die "LLVM frontdoor pruning failed; activation was rolled back"
      fi
      pruned_fronts+=("$front")
      pruned_backups+=("$backup")
    fi
  done
  if ! llvm_sync -f "$meta/usr/bin"; then
    llvm_rollback_frontdoors_and_activation "$meta" "$parent" "$destination" \
      "$llvm_activation_fallback" "$llvm_activation_was_initial" \
      created_fronts pruned_fronts pruned_backups \
      || llvm_die "LLVM frontdoor fsync failed and activation rollback was incomplete"
    llvm_die "LLVM frontdoor fsync failed; activation was rolled back"
  fi
  cleanup_failed=0
  for backup in "${pruned_backups[@]}"; do
    /usr/bin/rm -f -- "$backup" || cleanup_failed=1
  done
  if [ "$llvm_activation_was_initial" = 0 ]; then
    /usr/bin/rm -rf --one-file-system -- "$llvm_activation_fallback" || cleanup_failed=1
  fi
  llvm_sync -f "$meta/usr/bin" || cleanup_failed=1
  llvm_sync -f "$parent" || cleanup_failed=1
  [ "$cleanup_failed" = 0 ] || llvm_die "LLVM activation committed but strict retired-state cleanup failed"
  /usr/bin/rm -rf --one-file-system -- "$download_root"
  download_root=""
  trap - EXIT
}

llvm_remove_generation() {
  local meta="$1" tag="$2" archive_digest="$3" clang_digest="$4" ar_digest="$5" resource_digest="$6"
  local generation_digest="$7"
  local uid destination parent binary front private retired backup index cleanup_failed
  local -a fronts=() backups=()
  uid="$(id -u)"
  destination="$meta/.toolchains/llvm"
  parent="$meta/.toolchains"
  llvm_require_atomic_mv
  llvm_validate_chain "$meta" "$parent" "$uid"
  llvm_validate_chain "$meta" "$meta/usr/bin" "$uid"
  if [ ! -e "$destination" ] && [ ! -L "$destination" ]; then
    for binary in clang clang++ clang-cpp clang-21 llvm-config llc llvm-ar llvm-ranlib llvm-nm llvm-objcopy llvm-objdump lld ld.lld; do
      [ ! -e "$meta/usr/bin/$binary" ] && [ ! -L "$meta/usr/bin/$binary" ] \
        || llvm_die "refusing partial LLVM state with no managed generation"
    done
    return 0
  fi
  [ ! -e "$meta/.toolchains/sqld/bin/sqld" ] && [ ! -L "$meta/.toolchains/sqld/bin/sqld" ] \
    && [ ! -e "$meta/usr/libexec/envctl/sqld/bin/current/secretctl" ] \
    && [ ! -L "$meta/usr/libexec/envctl/sqld/bin/current/secretctl" ] \
    || llvm_die "refusing to remove LLVM while the sqld helper provenance chain is installed"
  llvm_validate_generation "$destination" "$tag" "$archive_digest" "$clang_digest" "$ar_digest" "$resource_digest" "$generation_digest" "$uid" \
    || llvm_die "refusing to remove unsafe, foreign, or drifted LLVM generation"
  llvm_preflight_frontdoors "$meta" "$destination" "$uid"
  llvm_validate_frontdoor_set "$meta" "$destination" "$uid"

  # Retire the complete payload first. Any later namespace or fsync failure restores both this
  # directory and every frontdoor from same-directory backups before returning non-zero.
  retired="$(mktemp -d "$parent/.llvm.retired.XXXXXX")"
  /usr/bin/rmdir "$retired"
  llvm_mv -T --no-copy -- "$destination" "$retired" \
    || llvm_die "LLVM generation retirement failed"
  if ! llvm_sync -f "$parent"; then
    if llvm_mv -T --no-copy -- "$retired" "$destination" && llvm_sync -f "$parent"; then
      llvm_die "LLVM removal was rolled back after generation fsync failure"
    fi
    llvm_die "LLVM generation retirement rollback failed"
  fi

  for binary in clang clang++ clang-cpp clang-21 llvm-config llc llvm-ar llvm-ranlib llvm-nm llvm-objcopy llvm-objdump lld ld.lld; do
    front="$meta/usr/bin/$binary"
    private="$destination/bin/$binary"
    if [ -e "$front" ] || [ -L "$front" ]; then
      # The complete set was validated immediately before retirement. Do not re-resolve a legacy
      # managed symlink here: its stable destination path is intentionally dangling until commit
      # or rollback completes.
      backup="$(mktemp "$meta/usr/bin/.llvm-remove.${binary}.XXXXXX")"
      /usr/bin/rm -f -- "$backup"
      if ! llvm_mv -T --no-copy -- "$front" "$backup"; then
        cleanup_failed=0
        for ((index=${#backups[@]} - 1; index >= 0; index--)); do
          llvm_mv -T --no-copy -- "${backups[$index]}" "${fronts[$index]}" || cleanup_failed=1
        done
        llvm_sync -f "$meta/usr/bin" || cleanup_failed=1
        llvm_mv -T --no-copy -- "$retired" "$destination" || cleanup_failed=1
        llvm_sync -f "$parent" || cleanup_failed=1
        [ "$cleanup_failed" = 0 ] \
          || llvm_die "LLVM frontdoor retirement failed and incumbent rollback was incomplete"
        llvm_die "LLVM frontdoor retirement failed; incumbent was restored"
      fi
      fronts+=("$front")
      backups+=("$backup")
    fi
  done
  if ! llvm_sync -f "$meta/usr/bin"; then
    cleanup_failed=0
    for ((index=${#backups[@]} - 1; index >= 0; index--)); do
      llvm_mv -T --no-copy -- "${backups[$index]}" "${fronts[$index]}" || cleanup_failed=1
    done
    llvm_sync -f "$meta/usr/bin" || cleanup_failed=1
    llvm_mv -T --no-copy -- "$retired" "$destination" || cleanup_failed=1
    llvm_sync -f "$parent" || cleanup_failed=1
    if [ "$cleanup_failed" = 0 ]; then
      llvm_die "LLVM removal was rolled back after frontdoor fsync failure"
    fi
    llvm_die "LLVM removal rollback failed after frontdoor fsync failure"
  fi
  cleanup_failed=0
  for backup in "${backups[@]}"; do
    /usr/bin/rm -f -- "$backup" || cleanup_failed=1
  done
  /usr/bin/rm -rf --one-file-system -- "$retired" || cleanup_failed=1
  llvm_sync -f "$meta/usr/bin" || cleanup_failed=1
  llvm_sync -f "$parent" || cleanup_failed=1
  [ "$cleanup_failed" = 0 ] || llvm_die "LLVM removal committed but strict cleanup/fsync failed"
}

llvm_main() {
  local mode="${1:-}" meta uid tag version asset_arch archive_digest clang_digest ar_digest resource_digest generation_digest
  meta="${META_ROOT:?META_ROOT required}"
  tag="llvmorg-21.1.8"
  version="21.1.8"
  case "$(uname -m)" in
    x86_64)
      asset_arch="X64"
      archive_digest="b3b7f2801d15d50736acea3c73982994d025b01c2f035b91ae3b49d1b575732b"
      clang_digest="d85d72c5c33bedce519504c4646b1356bf5205188ec60e4f753d1c4197cb0687"
      ar_digest="2a69b1406070607c758117083752bd05597a5539ec13888a524b9aa4bdb85703"
      resource_digest="8103c17f58639c829047e9166a65f0ba68d94c9cd1f55ae0dc6db526187af142"
      generation_digest="89df891585a701ce617b1d91ec1757db9f00337b4a70e7bd4dd28cb68e1dacac"
      ;;
    aarch64)
      asset_arch="ARM64"
      archive_digest="65ce0b329514e5643407db2d02a5bd34bf33d159055dafa82825c8385bd01993"
      clang_digest="470d20e410527de249ca560ef5519b6d7c772cb186c1e0c6af760ece9c794b32"
      ar_digest="21826438587f955364bfd91babd2a8e019c1c50fc79d4f1cdd8b2dfc4f814fa5"
      resource_digest="48fc701c3989881594c5ec7c8a8c354ccbe36c76fca3fad7868d5edc5aea407a"
      generation_digest="c8b5bc8d10d092c8d3edb996ffcf3dd4e78992b8a0fc9f88f910a5b3cc0c1383"
      ;;
    *) llvm_die "unsupported architecture" ;;
  esac
  case "$mode" in
    install|fix)
      llvm_install_generation "$meta" "$tag" "$version" "$asset_arch" \
        "$archive_digest" "$clang_digest" "$ar_digest" "$resource_digest" "$generation_digest"
      ;;
    detect|verify)
      uid="$(id -u)"
      llvm_validate_chain "$meta" "$meta/.toolchains/llvm" "$uid"
      llvm_validate_chain "$meta" "$meta/usr/bin" "$uid"
      llvm_validate_generation "$meta/.toolchains/llvm" "$tag" "$archive_digest" "$clang_digest" "$ar_digest" "$resource_digest" "$generation_digest" "$uid" \
        || llvm_die "installed LLVM generation does not match its pinned identity"
      llvm_preflight_frontdoors "$meta" "$meta/.toolchains/llvm" "$uid"
      llvm_validate_frontdoor_set "$meta" "$meta/.toolchains/llvm" "$uid"
      if [ "$mode" = verify ]; then
        "$meta/.toolchains/llvm/bin/clang" --version
        "$meta/.toolchains/llvm/bin/llvm-ar" --version
      fi
      ;;
    remove) llvm_remove_generation "$meta" "$tag" "$archive_digest" "$clang_digest" "$ar_digest" "$resource_digest" "$generation_digest" ;;
    *) llvm_die "usage: $0 {detect|install|verify|fix|remove}" ;;
  esac
}

if [ "${BASH_SOURCE[0]}" = "$0" ]; then
  llvm_main "$@"
fi
