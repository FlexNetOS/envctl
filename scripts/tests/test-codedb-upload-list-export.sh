#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
tmp="$(mktemp -d)"
trap '/usr/bin/rm -rf --one-file-system -- "$tmp"' EXIT
render="$tmp/render"
output="$tmp/output"
mkdir -p "$render/catalog/tables"

cat >"$render/catalog/scan.json" <<JSON
{"repo_root":"$repo_root","manifest_dir":"$repo_root/manifest"}
JSON
cat >"$render/catalog/tables/codedb_file_imports.json" <<'JSON'
[
  {"absolute_path":"/stable/content.toml","import_mode":"content_blob"},
  {"absolute_path":"/stable/metadata.bin","import_mode":"metadata_only"}
]
JSON
cat >"$render/catalog/tables/config_files.json" <<'JSON'
[{"path":"manifest/base.toml","file_kind":"toml","read_status":"ok","parse_status":"ok"}]
JSON
cat >"$render/catalog/tables/env_vars.json" <<'JSON'
[{"var_name":"PROFILE_TEST","producer":"fixture","scope":"test","source":"fixture","sensitive":false,"effective_value":"ok"}]
JSON
cat >"$render/catalog/tables/paths.json" <<'JSON'
[{"path_id":"profile","path":"/profile","path_kind":"binary","source":"fixture"}]
JSON
cat >"$render/catalog/tables/settings.json" <<'JSON'
[{"setting_key":"profile.owner","source_file":"fixture","source_kind":"test","value":"nix"}]
JSON
for table in components nix_components component_hooks agent_assets registries migration_evidence observed_facts; do
  printf '[]\n' >"$render/catalog/tables/$table.json"
done

CODEDB_UPLOAD_OUTPUT_DIR="$output" \
  "$repo_root/scripts/export-codedb-upload-lists.sh" "$render" >/dev/null

diff -u <(printf '%s\n' /stable/content.toml /stable/metadata.bin) \
  "$output/codedb-import-targets.txt"
diff -u <(printf '%s\n' /stable/content.toml) \
  "$output/codedb-content-blob-targets.txt"
diff -u <(printf '%s\n' /stable/metadata.bin) \
  "$output/codedb-metadata-only-targets.txt"
grep -Fq -- '- total rows: `2`' "$output/codedb-upload-inventory.md"
grep -Fq 'catalog snapshot SHA-256' "$output/codedb-upload-inventory.md"
grep -Fq '| `codedb_file_imports` | `2` |' "$output/catalog-table-inventory.md"

printf '%s\n' 'CodeDB catalog-backed export contract: PASS'
