#!/usr/bin/env bash
# ci/gates/kdf-feature-off.sh — fail-closed gate: the test-speed Argon2-floor knob must be OFF by default.
#
# TASK-0032 added a non-default `low-cost-kdf-tests` feature to envctl-secrets-engine that lowers the
# Argon2 downgrade floor + Default params so the CI `test` job's argon2id derivations are cheap. It is
# TEST-SPEED ONLY and must NEVER be in the crate's `default` feature set — otherwise every default
# build (release, doctor manifest, clippy, downstream consumers) would silently ship a 256 MiB → 8 KiB
# downgraded KDF floor. This gate proves, from the AUTHORITATIVE resolved metadata, that the feature
# is absent from `features.default`. Fail-closed (like no-c.sh): a metadata error or a parse failure
# aborts the gate CLOSED rather than passing on empty output. Run from the repo root.
set -euo pipefail

fail() { echo "KDF-FEATURE-OFF GATE FAIL: $*" >&2; exit 1; }

# Capture-first: `cargo metadata` failure must abort (fail-closed), not be misread as "feature off".
METADATA=$(cargo metadata --format-version 1 --no-deps) || fail "cargo metadata failed"

echo "$METADATA" | python3 -c '
import json, sys
m = json.load(sys.stdin)
crate = "envctl-secrets-engine"
pkg = next((p for p in m["packages"] if p["name"] == crate), None)
if pkg is None:
    sys.stderr.write("KDF-FEATURE-OFF GATE FAIL: "+crate+" not found in workspace metadata\n"); sys.exit(1)
features = pkg.get("features", {})
default = features.get("default", [])
if "low-cost-kdf-tests" not in features:
    sys.stderr.write("KDF-FEATURE-OFF GATE FAIL: feature low-cost-kdf-tests missing from "+crate+" (expected to exist, off by default)\n"); sys.exit(1)
if "low-cost-kdf-tests" in default:
    sys.stderr.write("KDF-FEATURE-OFF GATE FAIL: low-cost-kdf-tests is in "+crate+" features.default ("+str(default)+") — test-speed Argon2 floor must NEVER be a default feature (TASK-0032)\n"); sys.exit(1)
print("default features for "+crate+" = "+str(default)+" (low-cost-kdf-tests correctly OFF by default)")
'

echo "KDF-FEATURE-OFF GATE PASS"
