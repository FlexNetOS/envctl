#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

fail() {
  echo "GUI FONT STACK TEST FAIL: $*" >&2
  exit 1
}

metadata="$(mktemp)"
features="$(mktemp)"
trap 'rm -f "$metadata" "$features"' EXIT

cargo metadata --locked --format-version 1 >"$metadata"
cargo tree --locked -p envctl-gui -e features >"$features"

for feature in default_fonts glow x11 wayland; do
  grep -Fq "eframe feature \"$feature\"" "$features" \
    || fail "envctl-gui does not resolve eframe feature $feature"
done

epaint_manifest="$({
  jq -r '.packages[] | select(.name == "epaint" and .version == "0.33.3") | .manifest_path' \
    "$metadata"
} | head -n 1)"
[ "$epaint_manifest" = "$ROOT/third_party/epaint-0.33.3/Cargo.toml" ] \
  || fail "epaint 0.33.3 is not resolved from the reviewed local security patch"

for forbidden in ab_glyph owned_ttf_parser ttf-parser; do
  if jq -e --arg name "$forbidden" '.packages[] | select(.name == $name)' "$metadata" \
      >/dev/null; then
    fail "unmaintained font dependency remains resolved: $forbidden"
  fi
done

jq -e '.packages[] | select(.name == "skrifa" and .version == "0.37.0")' "$metadata" \
  >/dev/null || fail "reviewed skrifa 0.37.0 dependency is missing"
jq -e '.packages[] | select(.name == "vello_cpu" and .version == "0.0.4")' "$metadata" \
  >/dev/null || fail "reviewed vello_cpu 0.0.4 dependency is missing"

if grep -Fq 'skrifa feature "traversal"' "$features"; then
  fail "skrifa's unnecessary traversal feature is enabled"
fi

patch_doc="$ROOT/third_party/epaint-0.33.3/ENVCTL-PATCH.md"
[ -f "$patch_doc" ] || fail "missing epaint patch provenance"
grep -Fq '009d0dd3c2163823a0abdb899451ecbc78798dec545ee91b43aff1fa790bab62' \
  "$patch_doc" || fail "epaint archive checksum is not locked in provenance"
grep -Fq '609dd2d28edfadd544f53cec39b38564eb4fcb75' "$patch_doc" \
  || fail "font migration commit is not locked in provenance"
grep -Fq '6277a310b93f2f07834e920baabe43409334c973' "$patch_doc" \
  || fail "traversal correction commit is not locked in provenance"

[ "$(sha256sum "$ROOT/third_party/epaint-0.33.3/LICENSE-APACHE" | awk '{print $1}')" = \
  '8173d5c29b4f956d532781d2b86e4e30f83e6b7878dce18c919451d6ba707c90' ] \
  || fail "vendored Apache-2.0 license does not match upstream 0.33.3"
[ "$(sha256sum "$ROOT/third_party/epaint-0.33.3/LICENSE-MIT" | awk '{print $1}')" = \
  '95ca92f5f8ea5231f1580b3a2a799e8260af3114b900e1def5355a7f44bcf60c' ] \
  || fail "vendored MIT license does not match upstream 0.33.3"

vcs_info="$ROOT/third_party/epaint-0.33.3/.cargo_vcs_info.json"
[ "$(jq -r '.git.sha1' "$vcs_info")" = \
  '44cdd653e2317d300fb8a6c9c36b03f23991e803' ] \
  || fail "vendored source does not identify the exact upstream 0.33.3 commit"

echo "GUI FONT STACK TEST PASS"
