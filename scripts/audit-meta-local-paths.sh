#!/usr/bin/env bash
set -euo pipefail

apply=0
require_profile=0
meta_root="${META_ROOT:-/home/flexnetos/meta}"
real_home="${ENVCTL_REAL_HOME:-/home/flexnetos}"
nix_store_root="${ENVCTL_NIX_STORE_ROOT:-/nix/store}"

while (($# > 0)); do
  case "$1" in
    --apply) apply=1 ;;
    --require-yazelix-profile) require_profile=1 ;;
    --profile-shadow-guard-only) ;;
    --meta-root) shift; meta_root="${1:?missing value for --meta-root}" ;;
    --real-home) shift; real_home="${1:?missing value for --real-home}" ;;
    --nix-store-root) shift; nix_store_root="${1:?missing value for --nix-store-root}" ;;
    --envctl-home-source) shift; : "${1:?missing value for --envctl-home-source}" ;;
    *) printf 'unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

[[ -n "$meta_root" && "$meta_root" != / ]] || {
  echo "refusing unsafe meta root" >&2
  exit 2
}
[[ -n "$real_home" && "$real_home" != / && "$real_home" != "$meta_root" ]] || {
  echo "real home must be an explicit path distinct from the meta root" >&2
  exit 2
}

profile="$real_home/.nix-profile"
if ((require_profile)); then
  [[ -L "$profile" ]] || { echo "missing direct profile selector: $profile" >&2; exit 1; }
  selector="$(readlink "$profile")"
  case "$selector" in
    .nix-profile-[0-9]*-link) ;;
    *) echo "invalid direct profile selector: $profile -> $selector" >&2; exit 1 ;;
  esac
  resolved="$(readlink -f "$profile" 2>/dev/null || true)"
  case "$resolved" in
    "$nix_store_root"/*-profile) ;;
    *) echo "invalid profile generation target: $resolved" >&2; exit 1 ;;
  esac

  for command in yzx codex claude rtk bun bunx nu nix git-kb icm; do
    if [[ ! -x "$profile/bin/$command" && ! -x "$profile/toolbin/$command" ]]; then
      echo "missing profile-owned command: $command" >&2
      exit 1
    fi
  done
fi

dot_local=".$(printf '%s' local)"
dot_codex=".$(printf '%s' codex)"
dot_claude=".$(printf '%s' claude)"
forbidden=(
  "$real_home/$dot_local"
  "$real_home/$dot_codex"
  "$real_home/$dot_claude"
)

declare -a present=()
for path in "${forbidden[@]}"; do
  [[ -e "$path" || -L "$path" ]] && present+=("$path")
done

if ((${#present[@]} > 0)) && ((apply == 0)); then
  printf 'forbidden competing ownership path: %s\n' "${present[@]}" >&2
  exit 1
fi

if ((${#present[@]} > 0)); then
  stamp="$(date -u +%Y%m%dT%H%M%SZ)"
  archive="$meta_root/var/lib/envctl/archives/strict-profile-owner/$stamp"
  mkdir -p "$archive"
  for path in "${present[@]}"; do
    name="$(basename "$path")"
    [[ ! -e "$archive/$name" && ! -L "$archive/$name" ]] || {
      echo "archive collision: $archive/$name" >&2
      exit 1
    }
    mv -- "$path" "$archive/$name"
    printf 'archived competing ownership path: %s -> %s\n' "$path" "$archive/$name"
  done
  printf 'profile=%s\nsource_count=%d\n' "$profile" "${#present[@]}" \
    >"$archive/receipt.txt"
  chmod 0600 "$archive/receipt.txt"
  sha256sum "$archive/receipt.txt" >"$archive/receipt.txt.sha256"
fi

for path in "${forbidden[@]}"; do
  [[ ! -e "$path" && ! -L "$path" ]] || {
    echo "competing ownership path remains after repair: $path" >&2
    exit 1
  }
done

printf 'strict profile ownership audit: PASS\n'
