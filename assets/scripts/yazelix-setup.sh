#!/usr/bin/env bash
# Compatibility entrypoint for the retired envctl first-login installer.
# Runtime installation and final cutover belong exclusively to a clean checkout
# of merged FlexNetOS/yazelix origin/main.
set -euo pipefail
export PATH=/usr/bin:/bin

meta_root="${META_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)}"
source_root="${ENVCTL_SOURCE_ROOT:-$meta_root/src/envctl}"
validator="$source_root/assets/scripts/envctl-yazelix-profile-lifecycle.sh"
[ -x "$validator" ] || {
  printf 'yazelix-setup: missing profile validator: %s\n' "$validator" >&2
  exit 1
}
"$validator" verify
printf 'yazelix-setup: installed profile is valid; no envctl cutover was performed\n'
