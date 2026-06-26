#!/usr/bin/env bash
# ci/gates/p7.sh — fail-closed p7-conformance gate for this repo's `.handoff/` Tier-A layer.
#
# META-ORG-POLICY **P7** / handoff ADR-0003 + ADR-0004 §2/§3 + ADR-0018 D1: a member repo's
# `.handoff/` durable continuity layer is git-committed. The committed continuity truth is the
# per-repo JSONL ledger export (`.handoff/ledger.events.jsonl`) plus rendered/durable text artifacts;
# local redb/RVF `ledger.db` caches are legitimate per-repo rebuild/runtime state and must be ignored,
# never committed. This gate validates the committed Tier-A artifacts and delegates durability-ignore
# swallow checks to the current `hf gitignore --check` kernel policy.
set -euo pipefail
fail() { echo "P7 GATE FAIL: $*" >&2; exit 1; }

HND=".handoff"
[ -d "$HND" ] || { echo "P7 GATE PASS (no .handoff/ — not a continuity member)"; exit 0; }

# --- Gate 1: REQUIRED Tier-A core exists (ADR-0004 §2). ---
[ -f "$HND/context/capsule.json" ] || fail "missing REQUIRED $HND/context/capsule.json"
[ -f "$HND/README.md" ]            || fail "missing REQUIRED $HND/README.md"
[ -f "$HND/ledger.events.jsonl" ]  || fail "missing REQUIRED $HND/ledger.events.jsonl (run 'hf export' and commit it)"
[ -d "$HND/tasks" ]                || fail "missing REQUIRED $HND/tasks/ dir"
[ -d "$HND/packets" ]              || fail "missing REQUIRED $HND/packets/ dir"

# --- Gate 2: schema tags pin each artifact to its versioned contract. ---
grep -q '"schema": "handoff.context_capsule.v1"' "$HND/context/capsule.json" \
  || fail "capsule.json missing/!= schema \"handoff.context_capsule.v1\""
# OPTIONAL autonomous-loop descriptors: validate the tag IFF the file exists.
if [ -f "$HND/policies/rules.toml" ]; then
  grep -Eq '^[[:space:]]*schema[[:space:]]*=[[:space:]]*"handoff.policy.rules.v1"' "$HND/policies/rules.toml" \
    || fail "policies/rules.toml missing/!= schema \"handoff.policy.rules.v1\""
fi
if [ -f "$HND/hooks/hooks.toml" ]; then
  grep -Eq '^[[:space:]]*schema[[:space:]]*=[[:space:]]*"handoff.hooks.v1"' "$HND/hooks/hooks.toml" \
    || fail "hooks/hooks.toml missing/!= schema \"handoff.hooks.v1\""
fi
# Every minted task card must carry the task schema. (nullglob: no cards yet → skipped, not an error.)
shopt -s nullglob
for card in "$HND"/tasks/*.task.json; do
  grep -q '"schema": "handoff.task.v1"' "$card" \
    || fail "$card missing/!= schema \"handoff.task.v1\""
done
shopt -u nullglob

# --- Gate 3: ledger residency/durability (ADR-0004 §3 / ADR-0018 D1). ---
# The local binary ledger is legitimate, but it is a rebuild cache. The JSONL export is the durable
# git truth; any tracked binary DB/RVF sidecar is a regression.
if ! git ls-files --error-unmatch "$HND/ledger.events.jsonl" >/dev/null 2>&1; then
  fail "$HND/ledger.events.jsonl is not git-tracked — run 'hf export' and commit it"
fi
if git ls-files "$HND" | grep -qE '\.(db|db-wal|db-shm|rvf|rvf\.lock)$|(^|/)ledger\.db$'; then
  fail "a binary ledger/cache is git-tracked under $HND — commit ledger.events.jsonl, not redb/RVF caches"
fi
if command -v hf >/dev/null 2>&1; then
  hf gitignore --check >/dev/null || fail "hf gitignore --check failed — repair the canonical .handoff durability policy"
else
  grep -qE '^\.handoff/\*\*/ledger\.db$' .gitignore \
    || fail ".gitignore is missing the canonical ledger cache guard '.handoff/**/ledger.db'"
fi

# --- Gate 4: the resume packet, if rendered, is the v2 contract (compiled, not hand-written). ---
if [ -f "$HND/packets/latest.md" ]; then
  grep -q 'handoff.packet.v2' "$HND/packets/latest.md" \
    || fail "packets/latest.md is not a handoff.packet.v2 (re-render via 'hf import && hf handoff && hf export')"
fi

echo "P7 GATE PASS"
