#!/usr/bin/env bash
# Compatibility gate name retained for CI. The former multi-prefix policy has
# been superseded by the strict single-profile ownership contract.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
exec bash "$root/ci/gates/strict-profile-owner.sh"
