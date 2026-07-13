#!/usr/bin/env bash
set -euo pipefail
# Match the lifecycle's extraction contract when constructing the synthetic incumbent/archive.
# The test runner commonly inherits 0002, while production deliberately normalizes to 0022.
umask 022

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source_lifecycle="$repo_root/assets/scripts/envctl-llvm-clang-lifecycle.sh"
component_manifest="$repo_root/manifest/components.d/epic-h-toolchains.toml"
tmp="$(mktemp -d)"
trap '/usr/bin/rm -rf --one-file-system -- "$tmp"' EXIT
meta="$tmp/meta"
fixture_lifecycle="$tmp/envctl-llvm-clang-lifecycle.sh"
shadow="$tmp/shadow"
marker="$tmp/shadow-executed"
uid="$(id -u)"
tag=llvmorg-21.1.8
archive_digest=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa

/usr/bin/install -d -m755 "$meta/.toolchains" "$meta/usr/bin" "$shadow"
/usr/bin/install -d -m700 "$meta/var/tmp/envctl"

make_generation() {
  local destination="$meta/.toolchains/llvm"
  /usr/bin/rm -rf --one-file-system -- "$destination"
  /usr/bin/install -d -m755 "$destination/bin" "$destination/lib/clang/21/include"
  /usr/bin/chmod 755 "$destination"
  printf '%s\n' '#!/bin/sh' 'printf "%s\n" "fixture clang 21.1.8"' >"$destination/bin/clang-21"
  /usr/bin/chmod 755 "$destination/bin/clang-21"
  /usr/bin/ln "$destination/bin/clang-21" "$destination/bin/clang"
  /usr/bin/ln "$destination/bin/clang-21" "$destination/bin/clang++"
  printf '%s\n' '#!/bin/sh' 'printf "%s\n" "fixture llvm-ar 21.1.8"' >"$destination/bin/llvm-ar"
  /usr/bin/chmod 755 "$destination/bin/llvm-ar"
  /usr/bin/ln "$destination/bin/llvm-ar" "$destination/bin/llvm-ranlib"
  printf '%s\n' '#!/bin/sh' 'printf "%s\n" "fixture llvm-nm 21.1.8"' >"$destination/bin/llvm-nm"
  /usr/bin/chmod 755 "$destination/bin/llvm-nm"
  printf '%s\n' 'typedef __SIZE_TYPE__ size_t;' >"$destination/lib/clang/21/include/stddef.h"
  clang_digest="$(/usr/bin/sha256sum "$destination/bin/clang" | /usr/bin/awk '{print $1}')"
  ar_digest="$(/usr/bin/sha256sum "$destination/bin/llvm-ar" | /usr/bin/awk '{print $1}')"
  resource_digest="$(
    cd "$destination/lib/clang/21"
    /usr/bin/find . -type f -print0 \
      | LC_ALL=C /usr/bin/sort -z \
      | /usr/bin/xargs -0 /usr/bin/sha256sum \
      | /usr/bin/sha256sum \
      | /usr/bin/awk '{print $1}'
  )"
  # shellcheck source=../../assets/scripts/envctl-llvm-clang-lifecycle.sh
  source "$fixture_lifecycle"
  generation_digest="$(llvm_generation_digest "$destination" "$uid")"
  printf '%s\n' "$tag $archive_digest $resource_digest $generation_digest" >"$destination/.envctl-release"
  /usr/bin/chmod 644 "$destination/.envctl-release"
  llvm_write_frontdoor "$meta/usr/bin/clang" "$destination/bin/clang"
  llvm_write_frontdoor "$meta/usr/bin/clang++" "$destination/bin/clang++"
  llvm_write_frontdoor "$meta/usr/bin/clang-21" "$destination/bin/clang-21"
  llvm_write_frontdoor "$meta/usr/bin/llvm-ar" "$destination/bin/llvm-ar"
  llvm_write_frontdoor "$meta/usr/bin/llvm-ranlib" "$destination/bin/llvm-ranlib"
  llvm_write_frontdoor "$meta/usr/bin/llvm-nm" "$destination/bin/llvm-nm"
}

assert_complete() {
  META_ROOT="$meta" ENVCTL_SOURCE_ROOT="$tmp/does-not-exist" /usr/bin/bash "$fixture_lifecycle" detect
  [ -d "$meta/.toolchains/llvm" ]
  [ -x "$meta/usr/bin/clang" ]
  [ -x "$meta/usr/bin/clang++" ]
  [ -x "$meta/usr/bin/llvm-ar" ]
  [ -x "$meta/usr/bin/llvm-ranlib" ]
}

for command in curl gh secretctl id readlink stat sha256sum awk; do
  printf '%s\n' '#!/bin/sh' "printf '%s\\n' '$command' >>'$marker'" 'exit 99' >"$shadow/$command"
  /usr/bin/chmod 755 "$shadow/$command"
done

/usr/bin/cp "$source_lifecycle" "$fixture_lifecycle"
/usr/bin/chmod 755 "$fixture_lifecycle"

# Replace only the active x86_64 identity so the public lifecycle dispatch can exercise tiny,
# hermetic payloads. The production constants remain checked separately by enable.sh.
make_generation

# Build a tiny release archive from the valid incumbent. The archive intentionally carries a
# throwaway marker (Install rewrites it after extraction), and includes clang-21 because the
# production lifecycle reconstructs clang/clang++ as hardlinks to that immutable release binary.
fixture_archive_root="$tmp/archive-root"
fixture_archive="$tmp/LLVM-21.1.8-Linux-X64.tar.xz"
/usr/bin/install -d -m755 "$fixture_archive_root/LLVM-fixture"
/usr/bin/cp -a "$meta/.toolchains/llvm/." "$fixture_archive_root/LLVM-fixture/"
/usr/bin/tar -cJf "$fixture_archive" -C "$fixture_archive_root" LLVM-fixture
archive_digest="$(/usr/bin/sha256sum "$fixture_archive" | /usr/bin/awk '{print $1}')"
printf '%s\n' "$tag $archive_digest $resource_digest $generation_digest" >"$meta/.toolchains/llvm/.envctl-release"
/usr/bin/sed -i \
  -e "s/b3b7f2801d15d50736acea3c73982994d025b01c2f035b91ae3b49d1b575732b/$archive_digest/g" \
  -e "s/d85d72c5c33bedce519504c4646b1356bf5205188ec60e4f753d1c4197cb0687/$clang_digest/g" \
  -e "s/2a69b1406070607c758117083752bd05597a5539ec13888a524b9aa4bdb85703/$ar_digest/g" \
  -e "s/8103c17f58639c829047e9166a65f0ba68d94c9cd1f55ae0dc6db526187af142/$resource_digest/g" \
  -e "s/89df891585a701ce617b1d91ec1757db9f00337b4a70e7bd4dd28cb68e1dacac/$generation_digest/g" \
  "$fixture_lifecycle"

# Exercise the manifest-level Detect preflight, not only the lifecycle asset. Its first operation
# pins PATH before id/readlink/stat/sha256sum/awk can authorize the helper, so caller-writable
# shadows cannot forge source ownership, mode, or digest evidence.
manifest_source="$tmp/manifest-source"
manifest_detect="$tmp/manifest-detect.sh"
/usr/bin/install -d -m755 "$manifest_source/assets/scripts"
/usr/bin/install -m755 "$fixture_lifecycle" \
  "$manifest_source/assets/scripts/envctl-llvm-clang-lifecycle.sh"
production_lifecycle_digest="$(/usr/bin/sha256sum "$source_lifecycle" | /usr/bin/awk '{print $1}')"
fixture_lifecycle_digest="$(/usr/bin/sha256sum "$fixture_lifecycle" | /usr/bin/awk '{print $1}')"
python3 - "$component_manifest" "$manifest_detect" "$production_lifecycle_digest" \
  "$fixture_lifecycle_digest" <<'PY'
import pathlib, sys, tomllib

data = tomllib.loads(pathlib.Path(sys.argv[1]).read_text())
component = next(item for item in data["component"] if item["id"] == "llvm-clang")
script = component["detect"]["args"][1]
assert "export PATH=/usr/bin:/bin" in script
script = script.replace(sys.argv[3], sys.argv[4])
path = pathlib.Path(sys.argv[2])
path.write_text(script)
path.chmod(0o755)
PY
PATH="$shadow:/usr/bin:/bin" META_ROOT="$meta" ENVCTL_SOURCE_ROOT="$manifest_source" \
  /usr/bin/bash "$manifest_detect"
[ ! -e "$marker" ]

# Retain one legacy managed symlink frontdoor. Remove must validate it while the payload exists,
# then avoid re-resolving the intentionally dangling target during the retirement transaction.
/usr/bin/rm -f "$meta/usr/bin/clang++"
/usr/bin/ln -s "$meta/.toolchains/llvm/bin/clang++" "$meta/usr/bin/clang++"

# Hosted CI must expose the same coreutils atomic primitive production requires, and the fixture
# exercises the exchange itself rather than merely grepping --help.
left="$tmp/left"
right="$tmp/right"
/usr/bin/install -d -m755 "$left" "$right"
printf left >"$left/value"
printf right >"$right/value"
/usr/bin/mv -T --exchange --no-copy -- "$left" "$right"
[ "$(cat "$left/value")" = right ] && [ "$(cat "$right/value")" = left ]
/usr/bin/mv -T --exchange --no-copy -- "$left" "$right"

# Detect and Verify are zero-network and source-checkout independent. Caller PATH tripwires for
# curl/gh/secretctl cannot run because the lifecycle pins its utility PATH and only Install fetches.
PATH="$shadow:/usr/bin:/bin" META_ROOT="$meta" ENVCTL_SOURCE_ROOT="$tmp/does-not-exist" \
  /usr/bin/bash "$fixture_lifecycle" detect
PATH="$shadow:/usr/bin:/bin" META_ROOT="$meta" ENVCTL_SOURCE_ROOT="$tmp/does-not-exist" \
  /usr/bin/bash "$fixture_lifecycle" verify >/dev/null
[ ! -e "$marker" ]

# A second Install and Fix over the exact incumbent are true no-op/idempotent operations: no
# archive fetch, extraction, generation exchange, or frontdoor rewrite occurs.
clang_inode_before="$(stat -c '%d:%i' "$meta/.toolchains/llvm/bin/clang")"
front_inode_before="$(stat -c '%d:%i' "$meta/usr/bin/clang")"
PATH="$shadow:/usr/bin:/bin" META_ROOT="$meta" ENVCTL_SOURCE_ROOT="$tmp/does-not-exist" \
  /usr/bin/bash "$fixture_lifecycle" install
PATH="$shadow:/usr/bin:/bin" META_ROOT="$meta" ENVCTL_SOURCE_ROOT="$tmp/does-not-exist" \
  /usr/bin/bash "$fixture_lifecycle" fix
[ "$(stat -c '%d:%i' "$meta/.toolchains/llvm/bin/clang")" = "$clang_inode_before" ]
[ "$(stat -c '%d:%i' "$meta/usr/bin/clang")" = "$front_inode_before" ]
[ ! -e "$marker" ]

# A marker-backed generation can be repaired when a legacy managed symlink has become dangling.
# This invokes the real Fix core through a deterministic download seam, proving preflight accepts
# the symlink by its own inode ownership and normalized lexical target before replacing the payload.
/usr/bin/rm -f "$meta/.toolchains/llvm/bin/clang++"
[ -L "$meta/usr/bin/clang++" ] && [ ! -e "$meta/usr/bin/clang++" ]
(
  # A hostile caller umask must not alter archive modes or the generation digest.
  umask 077
  # shellcheck source=../../assets/scripts/envctl-llvm-clang-lifecycle.sh
  source "$fixture_lifecycle"
  llvm_download_archive() { /usr/bin/cp "$fixture_archive" "$2"; }
  llvm_install_generation "$meta" "$tag" 21.1.8 X64 \
    "$archive_digest" "$clang_digest" "$ar_digest" "$resource_digest" "$generation_digest"
)
assert_complete
[ -L "$meta/usr/bin/clang++" ]
[ "$(readlink -m -- "$meta/usr/bin/clang++")" = "$meta/.toolchains/llvm/bin/clang++" ]

# Every installed archive member is identity-bound, including tools that are not one of the four
# critical compiler/archiver pins. A same-mode replacement that exits zero must fail Detect and be
# repaired from the pinned complete-generation archive.
llvm_nm_digest="$(/usr/bin/sha256sum "$meta/.toolchains/llvm/bin/llvm-nm" | /usr/bin/awk '{print $1}')"
printf '%s\n' '#!/bin/sh' 'exit 0' >"$meta/.toolchains/llvm/bin/llvm-nm"
/usr/bin/chmod 755 "$meta/.toolchains/llvm/bin/llvm-nm"
if META_ROOT="$meta" /usr/bin/bash "$fixture_lifecycle" detect; then
  echo 'complete-generation Detect accepted a replacement llvm-nm' >&2
  exit 1
fi
(
  source "$fixture_lifecycle"
  llvm_download_archive() { /usr/bin/cp "$fixture_archive" "$2"; }
  llvm_install_generation "$meta" "$tag" 21.1.8 X64 \
    "$archive_digest" "$clang_digest" "$ar_digest" "$resource_digest" "$generation_digest"
)
assert_complete
[ "$(/usr/bin/sha256sum "$meta/.toolchains/llvm/bin/llvm-nm" | /usr/bin/awk '{print $1}')" = "$llvm_nm_digest" ]

# If a foreign frontdoor appears after the atomic generation exchange, activation must roll back
# to the complete incumbent and preserve the foreign path for an operator to inspect.
printf '%s\n' '#!/bin/sh' 'exit 0' >"$meta/.toolchains/llvm/bin/llvm-nm"
/usr/bin/chmod 755 "$meta/.toolchains/llvm/bin/llvm-nm"
tampered_nm_digest="$(/usr/bin/sha256sum "$meta/.toolchains/llvm/bin/llvm-nm" | /usr/bin/awk '{print $1}')"
if (
  source "$fixture_lifecycle"
  original_commit="$(declare -f llvm_commit_generation)"
  eval "${original_commit/llvm_commit_generation ()/llvm_commit_generation_real ()}"
  llvm_download_archive() { /usr/bin/cp "$fixture_archive" "$2"; }
  llvm_commit_generation() {
    llvm_commit_generation_real "$@"
    printf '%s\n' '#!/bin/sh' 'echo foreign-post-exchange' >"$meta/usr/bin/clang"
    /usr/bin/chmod 755 "$meta/usr/bin/clang"
  }
  llvm_install_generation "$meta" "$tag" 21.1.8 X64 \
    "$archive_digest" "$clang_digest" "$ar_digest" "$resource_digest" "$generation_digest"
); then
  echo 'expected post-exchange foreign-frontdoor refusal' >&2
  exit 1
fi
[ "$(/usr/bin/sha256sum "$meta/.toolchains/llvm/bin/llvm-nm" | /usr/bin/awk '{print $1}')" = "$tampered_nm_digest" ]
/usr/bin/grep -Fqx 'echo foreign-post-exchange' "$meta/usr/bin/clang"
/usr/bin/rm -f "$meta/usr/bin/clang"
(
  source "$fixture_lifecycle"
  llvm_write_frontdoor "$meta/usr/bin/clang" "$meta/.toolchains/llvm/bin/clang"
  llvm_download_archive() { /usr/bin/cp "$fixture_archive" "$2"; }
  llvm_install_generation "$meta" "$tag" 21.1.8 X64 \
    "$archive_digest" "$clang_digest" "$ar_digest" "$resource_digest" "$generation_digest"
)
assert_complete

# A generation-rename failure leaves the complete incumbent untouched.
if (
  # shellcheck source=../../assets/scripts/envctl-llvm-clang-lifecycle.sh
  source "$fixture_lifecycle"
  calls=0
  llvm_mv() {
    calls=$((calls + 1))
    [ "$calls" -ne 1 ] || return 1
    /usr/bin/mv "$@"
  }
  llvm_remove_generation "$meta" "$tag" "$archive_digest" "$clang_digest" "$ar_digest" "$resource_digest" "$generation_digest"
); then
  echo 'expected injected LLVM generation retirement failure' >&2
  exit 1
fi
assert_complete

# A parent-fsync failure is reversed before the lifecycle returns non-zero.
if (
  source "$fixture_lifecycle"
  calls=0
  llvm_sync() {
    calls=$((calls + 1))
    [ "$calls" -ne 1 ] || return 1
    /usr/bin/sync "$@"
  }
  llvm_remove_generation "$meta" "$tag" "$archive_digest" "$clang_digest" "$ar_digest" "$resource_digest" "$generation_digest"
); then
  echo 'expected injected LLVM generation fsync failure' >&2
  exit 1
fi
assert_complete

# A frontdoor-rename failure occurs after generation retirement; both namespaces must roll back.
if (
  source "$fixture_lifecycle"
  calls=0
  llvm_mv() {
    calls=$((calls + 1))
    [ "$calls" -ne 2 ] || return 1
    /usr/bin/mv "$@"
  }
  llvm_remove_generation "$meta" "$tag" "$archive_digest" "$clang_digest" "$ar_digest" "$resource_digest" "$generation_digest"
); then
  echo 'expected injected LLVM frontdoor retirement failure' >&2
  exit 1
fi
assert_complete

# Remove is also source-checkout independent, preserves no stale frontdoor, and never resolves a
# caller shadow. A second Remove is idempotent.
PATH="$shadow:/usr/bin:/bin" META_ROOT="$meta" ENVCTL_SOURCE_ROOT="$tmp/does-not-exist" \
  /usr/bin/bash "$fixture_lifecycle" remove
PATH="$shadow:/usr/bin:/bin" META_ROOT="$meta" ENVCTL_SOURCE_ROOT="$tmp/does-not-exist" \
  /usr/bin/bash "$fixture_lifecycle" remove
[ ! -e "$meta/.toolchains/llvm" ]
for command in clang clang++ llvm-ar llvm-ranlib; do
  [ ! -e "$meta/usr/bin/$command" ] && [ ! -L "$meta/usr/bin/$command" ]
done
[ ! -e "$marker" ]

# Rollback failures are never suppressed. Inject failure in both the forward frontdoor retirement
# and the subsequent generation restoration, and require an explicit incomplete-rollback error.
make_generation
rollback_failure_log="$tmp/rollback-failure.log"
if (
  source "$fixture_lifecycle"
  calls=0
  llvm_mv() {
    calls=$((calls + 1))
    [ "$calls" -ne 2 ] && [ "$calls" -ne 3 ] || return 1
    /usr/bin/mv "$@"
  }
  llvm_remove_generation "$meta" "$tag" "$archive_digest" "$clang_digest" "$ar_digest" \
    "$resource_digest" "$generation_digest"
) 2>"$rollback_failure_log"; then
  echo 'expected injected LLVM rollback failure' >&2
  exit 1
fi
/usr/bin/grep -Fq 'rollback was incomplete' "$rollback_failure_log"

echo 'llvm-clang lifecycle fixture: PASS'
