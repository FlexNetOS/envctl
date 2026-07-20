#!/usr/bin/env bash
# canonical: scripts/tests/blueprint/t3_embedder_wiring.sh
# T3 — MiniLM embedder wiring assertion (read-only).
#   Two conditions, both must hold for GREEN:
#     1. every row of the `codebase` table carries a 384-d MiniLM embedding
#        (count(embedding_minilm) == total row count, expected 5157).
#     2. the agentdb manifest's embedder `model` is no longer the "fallback"
#        embedder (a real local model is wired).
#   RED now:  only a partial subset of rows are MiniLM-embedded AND the manifest
#             still reads "agentdb fallback embedder (no local model wired yet)".
#   GREEN:    after R3 re-embeds `codebase` with the real MiniLM model and updates
#             the manifest model off the fallback.
#   flip-on:  once GREEN, wire beside ci/gates/no-c.sh / shape.sh.
#
# Strictly read-only: SELECT count(*) only; reads the manifest JSON. Touches no
# production rows.
set -uo pipefail

META_ROOT="${META_ROOT:?META_ROOT required}"
ENVCTL_REAL_HOME="${ENVCTL_REAL_HOME:?ENVCTL_REAL_HOME required}"
PSQL="${PSQL_BIN:-$ENVCTL_REAL_HOME/.nix-profile/toolbin/psql}"
PGPORT="${PGPORT:-5432}"
PGHOST="${PGHOST_DIR:-$META_ROOT/var/lib/postgresql}"
DB="${RUVECTOR_DB:-ruvector}"
MANIFEST="${RUVECTOR_MANIFEST:-$META_ROOT/var/lib/ruvector/agents/_manifest.json}"
RUV_DIR="${RUVECTOR_DIR:-$META_ROOT/var/lib/ruvector}"

fail=0
q() { "$PSQL" -h "$PGHOST" -p "$PGPORT" -d "$DB" -tAc "$1" 2>&1; }

echo "== T3: MiniLM embedder wiring =="

# 1) codebase MiniLM coverage
total="$(q 'SELECT count(*) FROM codebase')"
minilm="$(q 'SELECT count(*) FROM codebase WHERE embedding_minilm IS NOT NULL')"
echo "codebase: total=$total  minilm_embedded=$minilm"
if ! printf '%s' "$total" | grep -qE '^[0-9]+$'; then
  echo "FAIL: could not read codebase row count (psql said: $total)"
  fail=1
  total=0
fi
if [ "$minilm" = "$total" ] && [ "$total" -gt 0 ]; then
  echo "PASS: all $total codebase rows carry a MiniLM (384-d) embedding"
else
  echo "FAIL: only $minilm / $total codebase rows are MiniLM-embedded (embedder not fully wired until R3)"
  fail=1
fi

# 2) manifest embedder model is not the fallback
model="$(cd "$RUV_DIR" && RUV_MANIFEST="$MANIFEST" bun -e 'const fs=require("fs");const m=JSON.parse(fs.readFileSync(process.env.RUV_MANIFEST,"utf8"));process.stdout.write(String((m.uniform_params||{}).model||""))' 2>&1)"
echo "manifest model = \"$model\""
if printf '%s' "$model" | grep -qi 'fallback'; then
  echo "FAIL: manifest embedder model is still the fallback (real model not wired until R3)"
  fail=1
else
  echo "PASS: manifest embedder model is not the fallback"
fi

if [ "$fail" -eq 0 ]; then
  echo "T3 GREEN"
  exit 0
fi
echo "T3 RED (expected until R3)"
exit 1
