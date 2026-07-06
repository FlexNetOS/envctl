#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
manifest_default="/home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json"
manifest_path="${1:-$manifest_default}"
output_dir="$repo_root/docs/generated"

if [[ ! -f "$manifest_path" ]]; then
  echo "manifest not found: $manifest_path" >&2
  exit 1
fi

mkdir -p "$output_dir"

all_targets="$output_dir/codedb-import-targets.txt"
blob_targets="$output_dir/codedb-content-blob-targets.txt"
metadata_targets="$output_dir/codedb-metadata-only-targets.txt"

jq -r '[.[] | .absolute_path] | sort[]' "$manifest_path" > "$all_targets"
jq -r '[.[] | select(.import_mode=="content_blob") | .absolute_path] | sort[]' "$manifest_path" > "$blob_targets"
jq -r '[.[] | select(.import_mode=="metadata_only") | .absolute_path] | sort[]' "$manifest_path" > "$metadata_targets"

echo "wrote:"
echo "  $all_targets"
echo "  $blob_targets"
echo "  $metadata_targets"
