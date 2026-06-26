#!/usr/bin/env bash
# test-plan-contract.sh — locks the `.handoff/loop/plan/` row/artifact contract the plan-loop must
# produce. No single validator script exists for these, so this test carries small inline validators
# and asserts them against synthetic GOOD/BAD fixtures, AND against the literal examples grepped out of
# the real references/state-contract.md (so the doc can't silently drift from the gate):
#   * targets.md rows: status marker + lowercase-kebab slug + "scope" present
#   * graph artifact names: graph/<T>.{symbols,callgraph,metrics}.json + .{graph,diff}.md
#   * JSON validity of a synthetic symbols.json via jq (SKIPPED with a note if jq is absent)
#   * the documented examples parse under the very same validators
#
# Self-contained: no external script under test, no network, tmpdir for fixtures.
set -euo pipefail

fail() { echo "FAIL: $*" >&2; exit 1; }
# Locate the packaged state-contract.md regardless of which of the two byte-identical copies is running
# (mirrored into envctl/scripts/tests/ and the harness_hub plugin). Walk up from this script to the
# meta-worktree root (holding both envctl/ and harness_hub/) and descend to the plugin references.
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; root="$here"
# Prefer the repo-local ejected copy first; a sibling harness_hub checkout can be older than this PR.
repo_probe="$(git -C "$here" rev-parse --show-toplevel 2>/dev/null)"
CONTRACT="$repo_probe/.claude/skills/planning-engineer/references/state-contract.md"
[ -f "$CONTRACT" ] || CONTRACT="$repo_probe/harness/skills/planning-engineer/references/state-contract.md"
REL="harness_hub/harness/skills/planning-engineer/references/state-contract.md"
if [ ! -f "$CONTRACT" ]; then
  while [ "$root" != "/" ] && [ ! -f "$root/$REL" ]; do root="$(dirname "$root")"; done
  CONTRACT="$root/$REL"
fi
[ -f "$CONTRACT" ] || { echo "FAIL: planning-engineer state-contract.md not found from $here" >&2; exit 1; }
repo_root="$(git -C "$here" rev-parse --show-toplevel 2>/dev/null)"

# Codex prompt front doors: the recovered planning-engineer harness must be reachable from Codex,
# not only as Claude/.agents skills. The compatibility alias covers the historically-mistyped
# `/plan-engineering-loop` request and routes it to `/plan-loop`.
for prompt in planning-engineer plan-loop plan-engineering-loop; do
  [ -f "$repo_root/.codex/prompts/$prompt.md" ] || fail "missing Codex prompt front door: $prompt"
done
grep -q '.agents/skills/planning-engineer/SKILL.md' "$repo_root/.codex/prompts/planning-engineer.md" \
  || fail "planning-engineer prompt does not point at the authoritative .agents skill"
grep -q '.agents/skills/plan-loop/SKILL.md' "$repo_root/.codex/prompts/plan-loop.md" \
  || fail "plan-loop prompt does not point at the authoritative .agents skill"
grep -q '.codex/prompts/plan-loop.md' "$repo_root/.codex/prompts/plan-engineering-loop.md" \
  || fail "plan-engineering-loop alias does not route to plan-loop"
for agent in \
  plan-analyst \
  plan-architect \
  plan-cartographer \
  plan-governance-config-auditor \
  plan-filesystem-layout-auditor \
  plan-opus-bg-code-graph \
  plan-opus-bg-governance \
  plan-opus-bg-rusty-idd-north-star \
  plan-opus-bg-settings-config \
  plan-opus-bg-web-trends \
  plan-test-strategist \
  plan-trend-researcher \
  plan-verifier; do
  [ -f "$repo_root/.codex/agents/$agent.toml" ] || fail "missing Codex planning subagent: $agent"
done

# PromptHub / owner-intent alignment: the loop must preserve the recovered upstream prompt contract,
# including 5x Opus 4.8 background lanes, rusty-idd first-run surfacing, and graph-first code intel.
grep -q 'prompt_hub/prompts/planning-engineer-loop.prompt.yml' "$repo_root/.codex/prompts/plan-loop.md"   || fail "plan-loop prompt does not cite the PromptHub source of truth"
grep -q 'prompt_hub/prompts/planning-engineer-loop.prompt.yml' "$repo_root/.agents/skills/plan-loop/SKILL.md"   || fail "plan-loop skill does not cite the PromptHub source of truth"
grep -q '5× Opus 4.8' "$repo_root/.agents/skills/plan-loop/SKILL.md"   || fail "plan-loop skill does not require 5x Opus 4.8 background lanes"
grep -q 'Use weave when running from Codex' "$repo_root/.agents/skills/plan-loop/SKILL.md"   || fail "plan-loop skill does not route Codex Opus lanes through weave"
grep -q 'foreground chat remains interactive' "$repo_root/.agents/skills/plan-loop/SKILL.md"   || fail "plan-loop skill does not protect foreground interactivity"
grep -q 'rusty-idd' "$repo_root/.agents/skills/plan-loop/SKILL.md"   || fail "plan-loop skill does not seed/surface rusty-idd"
grep -q 'git-kb code' "$repo_root/.agents/skills/planning-engineer/SKILL.md"   || fail "planning-engineer skill does not require git-kb code intelligence"
grep -q 'git-kb code doctor' "$repo_root/.agents/skills/planning-engineer/SKILL.md"   || fail "planning-engineer skill does not enumerate git-kb code doctor/index/query flow"
grep -q 'meta↔envctl' "$repo_root/.codex/prompts/plan-loop.md"   || fail "plan-loop prompt does not capture the meta/envctl/prompt_hub relationship"
grep -q 'filesystem-layout' "$repo_root/.agents/skills/planning-engineer/SKILL.md"   || fail "planning-engineer skill does not include the filesystem-layout axis"
[ -f "$repo_root/.agents/skills/plan-filesystem-layout/SKILL.md" ]   || fail "missing plan-filesystem-layout skill"
grep -q 'FHS/XDG' "$repo_root/.agents/skills/plan-filesystem-layout/SKILL.md"   || fail "plan-filesystem-layout skill does not name FHS/XDG standards"
grep -q 'findings/filesystem-layout-<T>.md' "$repo_root/.agents/skills/plan-filesystem-layout/SKILL.md"   || fail "plan-filesystem-layout skill does not define its finding artifact"
for lane in code-graph web-trends governance settings-config rusty-idd-north-star; do
  [ -f "$repo_root/.codex/agents/plan-opus-bg-$lane.toml" ] || fail "missing Opus background lane agent: $lane"
  ! grep -q '^model = "claude-opus-4-8"' "$repo_root/.codex/agents/plan-opus-bg-$lane.toml"     || fail "Opus background lane $lane pins unsupported Codex model instead of using weave"
  grep -q 'weave' "$repo_root/.codex/agents/plan-opus-bg-$lane.toml"     || fail "Opus background lane $lane does not instruct weave transport"
done
if grep -R '^model = "claude-opus-4-8"' "$repo_root/.codex/agents"/plan-*.toml >/dev/null; then
  fail "Codex plan agents must not pin unsupported claude-opus-4-8 directly; use weave transport"
fi

[ -x "$repo_root/scripts/plan-weave-dispatch.sh" ] || fail "missing executable plan weave dispatch helper"
grep -q 'plan-weave-dispatch.sh' "$repo_root/.agents/skills/plan-loop/SKILL.md"   || fail "plan-loop skill does not point at the weave dispatch helper"
grep -q 'weave_dispatch' "$repo_root/.agents/skills/planning-engineer/scripts/loop_state.template.md"   || fail "loop_state template does not record weave dispatch artifact"

python3 - "$repo_root" <<'PY'
from pathlib import Path
import sys
import tomllib

root = Path(sys.argv[1])
for path in sorted((root / ".codex/agents").glob("plan-*.toml")):
    data = tomllib.loads(path.read_text())
    for key in ("name", "description", "developer_instructions"):
        if key not in data or not str(data[key]).strip():
            raise SystemExit(f"FAIL: {path} missing non-empty {key}")
PY

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

SLUG_RE='^[a-z0-9][a-z0-9-]*$'

# valid_target_row "<row>" — a targets.md row: "- [<m>] <slug>: <scope>"; <m> in { space,x,~,!,!! }.
# Rejects bad status markers, slugs with spaces/uppercase/bad chars, and a missing scope.
# Regex parse (brackets/literals are explicit — no glob ambiguity from `case`/`${#}` patterns).
valid_target_row() {
  local row="$1" slug scope
  [[ "$row" =~ ^-\ \[(\ |x|~|!|!!)\]\ ([^:]+):[[:space:]]*(.+)$ ]] || return 1
  slug="${BASH_REMATCH[2]}"
  scope="${BASH_REMATCH[3]}"
  [ -n "$scope" ] || return 1                        # scope must be non-empty
  [[ "$slug" =~ $SLUG_RE ]] || return 1              # lowercase-kebab slug, no spaces/uppercase
  return 0
}

# valid_graph_artifact "<name>" — graph/<T>.<kind> per the documented scheme.
valid_graph_artifact() {
  local n="$1" base T kind
  case "$n" in graph/*) base="${n#graph/}" ;; *) return 1 ;; esac
  T="${base%%.*}"
  [[ "$T" =~ $SLUG_RE ]] || return 1
  kind="${base#*.}"
  case "$kind" in
    symbols.json|callgraph.json|metrics.json|graph.md|diff.md) return 0 ;;
    *) return 1 ;;
  esac
}

# ---- targets.md row fixtures ----
GOOD_ROWS=(
  "- [ ] secrets-proto: gRPC contract crate"
  "- [x] engine: the single shared sync library"
  "- [!] foo: blocked: upstream crate missing"
  "- [~] secretd: planned with open gaps"
  "- [!!] reset-flow: SUPERVISED destructive path"
)
BAD_ROWS=(
  "- [?] secrets-proto: bad status marker"
  "- [ ] Secrets Proto: slug has spaces and uppercase"
  "- [ ] secrets_proto-OK: uppercase in slug"
  "- [ ] secretd"
  "* [ ] secrets-proto: wrong bullet"
)
for r in "${GOOD_ROWS[@]}"; do valid_target_row "$r" || fail "GOOD targets row rejected: $r"; done
for r in "${BAD_ROWS[@]}";  do valid_target_row "$r" && fail "BAD targets row accepted: $r" || true; done

# duplicate-slug detection over a synthetic targets.md
cat > "$tmp/targets.md" <<'EOF'
- [ ] secrets-proto: gRPC contract crate
- [x] engine: shared sync library
- [~] secrets-proto: duplicate slug must be caught
EOF
dups="$(grep -oE '^- \[[^]]*\] [a-z0-9-]+:' "$tmp/targets.md" \
        | sed -E 's/^- \[[^]]*\] ([a-z0-9-]+):/\1/' | sort | uniq -d)"
[ "$dups" = "secrets-proto" ] || fail "duplicate-slug detector failed (got: '${dups:-<none>}')"

# ---- graph artifact naming fixtures ----
GOOD_ART=(
  "graph/secrets-proto.symbols.json"
  "graph/secrets-proto.callgraph.json"
  "graph/secrets-proto.metrics.json"
  "graph/secrets-proto.graph.md"
  "graph/secrets-proto.diff.md"
)
BAD_ART=(
  "graph/secrets-proto.symbol.json"
  "graph/secrets-proto.json"
  "graph/Secrets-Proto.symbols.json"
  "secrets-proto.symbols.json"
  "graph/secrets-proto.symbols.txt"
)
for a in "${GOOD_ART[@]}"; do valid_graph_artifact "$a" || fail "GOOD artifact rejected: $a"; done
for a in "${BAD_ART[@]}";  do valid_graph_artifact "$a" && fail "BAD artifact accepted: $a" || true; done

# ---- JSON validity (jq) ----
if command -v jq >/dev/null 2>&1; then
  printf '{"symbols":[],"count":0}\n' > "$tmp/secrets-proto.symbols.json"
  jq -e . "$tmp/secrets-proto.symbols.json" >/dev/null || fail "well-formed symbols.json failed jq parse"
  printf '{"symbols":[], "count":0\n' > "$tmp/bad.symbols.json"      # missing closing brace
  jq -e . "$tmp/bad.symbols.json" >/dev/null 2>&1 && fail "malformed JSON passed jq parse" || true
  jq_note="jq checks RAN"
else
  jq_note="jq absent — JSON sub-checks SKIPPED"
fi

# ---- documented examples parse under the same validators (anti-drift vs the real doc) ----
# targets.md example line in the doc uses <T>/<one-line scope> placeholders; substitute concrete
# values and strip the trailing "# comment", then run it through valid_target_row.
doc_target="$(grep -E '^- \[ \] <T>: <one-line scope>' "$CONTRACT" | head -n1)"
[ -n "$doc_target" ] || fail "could not find the targets.md row example in state-contract.md"
doc_target="${doc_target%%#*}"; doc_target="${doc_target%"${doc_target##*[![:space:]]}"}"  # rtrim
doc_target="${doc_target//<T>/secrets-proto}"
doc_target="${doc_target//<one-line scope>/gRPC contract crate}"
valid_target_row "$doc_target" || fail "documented targets.md example does not satisfy the validator: '$doc_target'"

# graph artifact names documented in the doc (graph/<T>.<kind> ...) must satisfy valid_graph_artifact.
mapfile -t doc_arts < <(grep -oE 'graph/<T>\.[a-z]+\.(json|md)' "$CONTRACT" | sort -u)
[ "${#doc_arts[@]}" -ge 4 ] || fail "expected >=4 documented graph artifacts, found ${#doc_arts[@]}"
for da in "${doc_arts[@]}"; do
  concrete="${da//<T>/secrets-proto}"
  valid_graph_artifact "$concrete" || fail "documented graph artifact does not satisfy the validator: '$da'"
done

echo "PASS: plan contract locked — targets rows, graph artifact names, JSON validity, and the documented examples all conform ($jq_note)"
