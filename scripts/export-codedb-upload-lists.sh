#!/usr/bin/env bash
# Compatibility frontdoor for the catalog-backed proof renderer.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
render_dir="${1:-${CATALOG_RENDER_DIR:-}}"
[ -n "$render_dir" ] || {
  printf 'usage: export-codedb-upload-lists.sh CATALOG_RENDER_DIR\n' >&2
  printf 'render first with: envctl catalog render --out DIR\n' >&2
  exit 2
}
output_dir="${CODEDB_UPLOAD_OUTPUT_DIR:-$repo_root/docs/generated}"
exec "$repo_root/scripts/export-catalog-proof-reports.sh" "$render_dir" "$output_dir"
