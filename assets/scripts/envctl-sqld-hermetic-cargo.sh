#!/usr/bin/env bash
# Shared, source-only helpers for the sqld component's reproducible, isolated secretctl build. The
# caller preflights this file by exact SHA-256 and provides validated META-owned source/toolchain
# paths. This binds META Rust/LLVM/Cargo inputs; the GNU target still consumes host Ubuntu headers,
# CRT, libc/libgcc, and compiler runtime DSOs, which remain an explicit OS/sysroot residual.

configure_hermetic_build_environment() {
  local build_workspace="$1"
  local build_target="$2"
  local target_upper linker_dir
  hermetic_path="$rust_toolchain_dir/bin:$llvm_bin:/usr/bin:/bin"
  linker_dir="$build_workspace/toolchain-bin"
  install -d -m700 "$linker_dir"
  hermetic_lld="$linker_dir/ld.lld"
  ln -s "$rust_lld_bin" "$hermetic_lld"
  [ -L "$hermetic_lld" ] && [ "$(readlink -f -- "$hermetic_lld")" = "$rust_lld_bin" ] \
    || { echo "FATAL: could not pin the run-private ld.lld alias" >&2; return 1; }
  target_key="${build_target//-/_}"
  target_key="${target_key//./_}"
  target_upper="${target_key^^}"
  cargo_linker_var="CARGO_TARGET_${target_upper}_LINKER"
  cargo_rustflags_var="CARGO_TARGET_${target_upper}_RUSTFLAGS"
  hermetic_cflags="--no-default-config -ffile-prefix-map=$build_workspace=/envctl-build -fdebug-prefix-map=$build_workspace=/envctl-build -ffile-prefix-map=$source_root=/envctl-source -fdebug-prefix-map=$source_root=/envctl-source -ffile-prefix-map=$M=/meta -fdebug-prefix-map=$M=/meta"
  cargo_rustflags="--remap-path-prefix=$M=/meta --remap-path-prefix=$source_root=/envctl-source --remap-path-prefix=$build_workspace=/envctl-build --remap-path-prefix=$private_cargo_home=/envctl-cargo-home -C codegen-units=1 -C embed-bitcode=yes -C llvm-args=-rng-seed=1 -C link-arg=--no-default-config -C link-arg=--ld-path=$hermetic_lld -C link-arg=-Wl,--build-id=sha1"
}

validate_toolchain_resource_tree() {
  local root node target resolved
  for root in "$clang_resource_root" "$rust_toolchain_lib_root" "$llvm_generation_root"; do
    owned_real_dir "$root" \
      || { echo "FATAL: unsafe META compiler-input root: $root" >&2; return 1; }
    while IFS= read -r -d '' node; do
      [ "$(/usr/bin/stat -c '%u' -- "$node")" = "$uid" ] \
        || { echo "FATAL: foreign META compiler input: $node" >&2; return 1; }
      if [ -L "$node" ]; then
        target="$(/usr/bin/readlink -- "$node")" \
          || { echo "FATAL: unreadable META compiler-input symlink: $node" >&2; return 1; }
        case "$target" in
          ''|/*) echo "FATAL: META compiler-input symlink must use a relative target: $node" >&2; return 1 ;;
        esac
        resolved="$(/usr/bin/readlink -f -- "$node")" \
          || { echo "FATAL: dangling META compiler-input symlink: $node" >&2; return 1; }
        case "$resolved" in
          "$root"|"$root"/*) ;;
          *) echo "FATAL: META compiler-input symlink escapes its root: $node" >&2; return 1 ;;
        esac
        { [ -f "$resolved" ] || [ -d "$resolved" ]; } \
          || { echo "FATAL: META compiler-input symlink targets a special node: $node" >&2; return 1; }
      elif ! { [ -f "$node" ] || [ -d "$node" ]; }; then
        echo "FATAL: unsafe META compiler input: $node" >&2
        return 1
      fi
    done < <(/usr/bin/find "$root" -print0)
  done
}

stage_hermetic_cargo_home() {
  local build_workspace="$1" shared_cache shared_index private_cache private_index archive staged
  local checksum name name_key version source count index_count sparse_path index_source index_staged
  registry_id="index.crates.io-1949cf8c6b5b557f"
  shared_cache="$M/.toolchains/cargo/registry/cache/$registry_id"
  shared_index="$M/.toolchains/cargo/registry/index/$registry_id"
  validate_managed_chain "$shared_cache"
  validate_managed_chain "$shared_index"
  owned_real_dir "$shared_cache" \
    || { echo "FATAL: missing or unsafe META-owned Cargo registry cache" >&2; return 1; }
  owned_real_dir "$shared_index" \
    || { echo "FATAL: missing or unsafe META-owned Cargo sparse index" >&2; return 1; }
  private_cargo_home="$build_workspace/cargo-home"
  private_cache="$private_cargo_home/registry/cache/$registry_id"
  private_index="$private_cargo_home/registry/index/$registry_id"
  install -d -m700 "$private_cache" "$private_index/.cache"
  index_source="$shared_index/config.json"
  index_staged="$private_index/config.json"
  [ -f "$index_source" ] && [ ! -L "$index_source" ] && canonical_is_lexical "$index_source" \
    && [ "$(stat -c '%u' "$index_source")" = "$uid" ] \
    || { echo "FATAL: unsafe Cargo sparse-index config" >&2; return 1; }
  install -m444 "$index_source" "$index_staged"
  count=0
  index_count=0
  while IFS='|' read -r source name version checksum; do
    case "$source" in
      registry+https://github.com/rust-lang/crates.io-index) ;;
      *) echo "FATAL: unsupported non-crates.io Cargo.lock source: $source" >&2; return 1 ;;
    esac
    case "$checksum" in ''|*[!0-9a-f]*) echo "FATAL: invalid Cargo.lock checksum for $name $version" >&2; return 1 ;; esac
    [ "${#checksum}" = 64 ] \
      || { echo "FATAL: invalid Cargo.lock checksum length for $name $version" >&2; return 1; }
    name_key="${name,,}"
    case "${#name_key}" in
      1) sparse_path="1/$name_key" ;;
      2) sparse_path="2/$name_key" ;;
      3) sparse_path="3/${name_key:0:1}/$name_key" ;;
      *) sparse_path="${name_key:0:2}/${name_key:2:2}/$name_key" ;;
    esac
    index_source="$shared_index/.cache/$sparse_path"
    index_staged="$private_index/.cache/$sparse_path"
    if [ ! -e "$index_staged" ] && [ ! -L "$index_staged" ]; then
      [ -f "$index_source" ] && [ ! -L "$index_source" ] && canonical_is_lexical "$index_source" \
        && [ "$(stat -c '%u' "$index_source")" = "$uid" ] \
        || { echo "FATAL: missing or unsafe Cargo sparse-index entry: $index_source" >&2; return 1; }
      install -d -m700 "$(dirname "$index_staged")"
      install -m444 "$index_source" "$index_staged"
      index_count=$((index_count + 1))
    fi
    archive="$shared_cache/${name}-${version}.crate"
    [ -e "$archive" ] || [ -L "$archive" ] || continue
    [ -f "$archive" ] && [ ! -L "$archive" ] && canonical_is_lexical "$archive" \
      && [ "$(stat -c '%u' "$archive")" = "$uid" ] \
      || { echo "FATAL: unsafe META-owned crate archive: $archive" >&2; return 1; }
    staged="$private_cache/${name}-${version}.crate"
    install -m444 "$archive" "$staged"
    [ "$(/usr/bin/sha256sum "$staged" | /usr/bin/awk '{print $1}')" = "$checksum" ] \
      || { echo "FATAL: crate archive differs from Cargo.lock: $archive" >&2; return 1; }
    count=$((count + 1))
  done < <(/usr/bin/awk '
    function emit() { if (source != "") print source "|" name "|" version "|" checksum }
    /^\[\[package\]\]$/ { emit(); name=version=source=checksum=""; next }
    /^name = / { name=$0; sub(/^name = "/,"",name); sub(/"$/,"",name); next }
    /^version = / { version=$0; sub(/^version = "/,"",version); sub(/"$/,"",version); next }
    /^source = / { source=$0; sub(/^source = "/,"",source); sub(/"$/,"",source); next }
    /^checksum = / { checksum=$0; sub(/^checksum = "/,"",checksum); sub(/"$/,"",checksum); next }
    END { emit() }
  ' "$source_root/Cargo.lock")
  [ "$count" -gt 0 ] || { echo "FATAL: no Cargo.lock crate archives were staged" >&2; return 1; }
  [ "$index_count" -gt 0 ] || { echo "FATAL: no Cargo.lock sparse-index entries were staged" >&2; return 1; }
}

run_hermetic_cargo_build() {
  local build_workspace="$1" build_target="$2" target_dir="$3" home_dir tmp_dir
  configure_hermetic_build_environment "$build_workspace" "$build_target"
  home_dir="$build_workspace/home"
  tmp_dir="$build_workspace/tmp"
  install -d -m700 "$home_dir" "$tmp_dir"
  (
    cd /
    /usr/bin/env -i \
      HOME="$home_dir" TMPDIR="$tmp_dir" LANG=C.UTF-8 LC_ALL=C.UTF-8 TERM=dumb \
      TZ=UTC SOURCE_DATE_EPOCH=1 ZERO_AR_DATE=1 \
      PATH="$hermetic_path" CARGO_HOME="$private_cargo_home" RUSTUP_HOME="$M/.toolchains/rustup" \
      CARGO_TARGET_DIR="$target_dir" CARGO_NET_OFFLINE=true CARGO_INCREMENTAL=0 \
      RUSTC="$rustc_bin" RUSTDOC="$rustdoc_bin" \
      CARGO_BUILD_RUSTC="$rustc_bin" CARGO_BUILD_RUSTDOC="$rustdoc_bin" \
      CC="$clang_bin" CXX="$clangxx_bin" AR="$llvm_ar_bin" RANLIB="$llvm_ranlib_bin" \
      CFLAGS="$hermetic_cflags" CXXFLAGS="$hermetic_cflags" \
      "CC_${target_key}=$clang_bin" "CXX_${target_key}=$clangxx_bin" \
      "AR_${target_key}=$llvm_ar_bin" "RANLIB_${target_key}=$llvm_ranlib_bin" \
      "CFLAGS_${target_key}=$hermetic_cflags" "CXXFLAGS_${target_key}=$hermetic_cflags" \
      "$cargo_linker_var=$clang_bin" "$cargo_rustflags_var=$cargo_rustflags" \
      "$cargo_bin" build --quiet --frozen --locked --offline --release \
        --manifest-path "$source_root/Cargo.toml" --package envctl-secretctl --bin secretctl
  )
}

collect_used_crate_archives() {
  local source_dir cargo_ok leaf archive
  crate_archive_args=()
  source_dir="$private_cargo_home/registry/src/$registry_id"
  [ -d "$source_dir" ] && [ ! -L "$source_dir" ] \
    || { echo "FATAL: Cargo did not freshly unpack registry sources" >&2; return 1; }
  while IFS= read -r -d '' cargo_ok; do
    [ -f "$cargo_ok" ] && [ ! -L "$cargo_ok" ] \
      || { echo "FATAL: fresh Cargo extraction marker is unsafe: $cargo_ok" >&2; return 1; }
    leaf="$(basename "$(dirname "$cargo_ok")")"
    archive="$private_cargo_home/registry/cache/$registry_id/$leaf.crate"
    [ -f "$archive" ] && [ ! -L "$archive" ] && canonical_is_lexical "$archive" \
      && [ "$(stat -c '%u' "$archive")" = "$uid" ] && [ "$(stat -c '%a' "$archive")" = 444 ] \
      || { echo "FATAL: freshly used crate archive is unsafe: $archive" >&2; return 1; }
    crate_archive_args+=(--crate-archive "$archive")
  done < <(/usr/bin/find "$source_dir" -mindepth 2 -maxdepth 2 -type f -name .cargo-ok -print0 \
    | LC_ALL=C /usr/bin/sort -z)
  [ "${#crate_archive_args[@]}" -gt 0 ] \
    || { echo "FATAL: fresh Cargo build exposed no registry input archive set" >&2; return 1; }
}
