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
# Locate the maintained state contract. In envctl, `agent-skills/` is the
# tracked projection; in harness_hub package CI, `harness/skills/` is the
# package source. Never depend on a live repo-local `.claude/` projection.
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; root="$here"
repo_probe="$(git -C "$here" rev-parse --show-toplevel 2>/dev/null)"
CONTRACT="$repo_probe/agent-skills/planning-engineer/references/state-contract.md"
[ -f "$CONTRACT" ] || CONTRACT="$repo_probe/harness/skills/planning-engineer/references/state-contract.md"
REL="harness_hub/harness/skills/planning-engineer/references/state-contract.md"
if [ ! -f "$CONTRACT" ]; then
  while [ "$root" != "/" ] && [ ! -f "$root/$REL" ]; do root="$(dirname "$root")"; done
  CONTRACT="$root/$REL"
fi
[ -f "$CONTRACT" ] || { echo "FAIL: planning-engineer state-contract.md not found from $here" >&2; exit 1; }
repo_root="$(git -C "$here" rev-parse --show-toplevel 2>/dev/null)"
if [[ "$CONTRACT" == "$repo_root/agent-skills/"* ]]; then
  SOURCE_LAYOUT="envctl"
else
  SOURCE_LAYOUT="package"
fi

# Codex prompt front doors: the recovered planning-engineer harness must be reachable from Codex,
# not only as Claude/.agents skills. The compatibility alias covers the historically-mistyped
# `/plan-engineering-loop` request and routes it to `/plan-loop`.
for prompt in planning-engineer plan-loop plan-engineering-loop; do
  [ -f "$repo_root/.codex/prompts/$prompt.md" ] || fail "missing Codex prompt front door: $prompt"
done
grep -q 'agent-skills/planning-engineer/SKILL.md' "$repo_root/.codex/prompts/planning-engineer.md" \
  || fail "planning-engineer prompt does not point at the authoritative .agents skill"
grep -q 'agent-skills/plan-loop/SKILL.md' "$repo_root/.codex/prompts/plan-loop.md" \
  || fail "plan-loop prompt does not point at the authoritative .agents skill"
grep -q '.codex/prompts/plan-loop.md' "$repo_root/.codex/prompts/plan-engineering-loop.md" \
  || fail "plan-engineering-loop alias does not route to plan-loop"

# Claude project material is an explicit ejection output, not a checked-in
# authority or a prerequisite for this clean-checkout gate. The hermetic eject
# test proves that projection separately.
[ ! -e "$repo_root/.claude" ] || fail "repo-local .claude projection must not be an active source tree"
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
grep -q 'prompt_hub/prompts/planning-engineer-loop.prompt.yml' "$repo_root/agent-skills/plan-loop/SKILL.md"   || fail "plan-loop skill does not cite the PromptHub source of truth"
grep -q '5× Opus 4.8' "$repo_root/agent-skills/plan-loop/SKILL.md"   || fail "plan-loop skill does not require 5x Opus 4.8 background lanes"
grep -q 'Use weave when running from Codex' "$repo_root/agent-skills/plan-loop/SKILL.md"   || fail "plan-loop skill does not route Codex Opus lanes through weave"
grep -q 'foreground chat remains interactive' "$repo_root/agent-skills/plan-loop/SKILL.md"   || fail "plan-loop skill does not protect foreground interactivity"
grep -q 'rusty-idd' "$repo_root/agent-skills/plan-loop/SKILL.md"   || fail "plan-loop skill does not seed/surface rusty-idd"
grep -q 'git-kb code' "$repo_root/agent-skills/planning-engineer/SKILL.md"   || fail "planning-engineer skill does not require git-kb code intelligence"
grep -q 'git-kb code doctor' "$repo_root/agent-skills/planning-engineer/SKILL.md"   || fail "planning-engineer skill does not enumerate git-kb code doctor/index/query flow"
grep -q 'meta↔envctl' "$repo_root/.codex/prompts/plan-loop.md"   || fail "plan-loop prompt does not capture the meta/envctl/prompt_hub relationship"
grep -q 'filesystem-layout' "$repo_root/agent-skills/planning-engineer/SKILL.md"   || fail "planning-engineer skill does not include the filesystem-layout axis"
[ -f "$repo_root/agent-skills/plan-filesystem-layout/SKILL.md" ]   || fail "missing plan-filesystem-layout skill"
grep -q 'FHS/XDG' "$repo_root/agent-skills/plan-filesystem-layout/SKILL.md"   || fail "plan-filesystem-layout skill does not name FHS/XDG standards"
grep -q 'findings/filesystem-layout-<T>.md' "$repo_root/agent-skills/plan-filesystem-layout/SKILL.md"   || fail "plan-filesystem-layout skill does not define its finding artifact"
for lane in code-graph web-trends governance settings-config rusty-idd-north-star; do
  [ -f "$repo_root/.codex/agents/plan-opus-bg-$lane.toml" ] || fail "missing Opus background lane agent: $lane"
  ! grep -q '^model = "claude-opus-4-8"' "$repo_root/.codex/agents/plan-opus-bg-$lane.toml"     || fail "Opus background lane $lane pins unsupported Codex model instead of using weave"
  grep -q 'weave' "$repo_root/.codex/agents/plan-opus-bg-$lane.toml"     || fail "Opus background lane $lane does not instruct weave transport"
done
if grep -R '^model = "claude-opus-4-8"' "$repo_root/.codex/agents"/plan-*.toml >/dev/null; then
  fail "Codex plan agents must not pin unsupported claude-opus-4-8 directly; use weave transport"
fi

[ -x "$repo_root/scripts/plan-weave-dispatch.sh" ] || fail "missing executable plan weave dispatch helper"
grep -q 'plan-weave-dispatch.sh' "$repo_root/agent-skills/plan-loop/SKILL.md"   || fail "plan-loop skill does not point at the weave dispatch helper"
grep -q 'weave_dispatch' "$repo_root/agent-skills/planning-engineer/scripts/loop_state.template.md"   || fail "loop_state template does not record weave dispatch artifact"

# P0-P2 runtime upgrades from June 2026 research: artifact gate, TDP DAG, prompt architecture,
# observability/backend/risk/evals, source ledger, and interop registry.
[ -x "$repo_root/scripts/plan-artifact-gate.sh" ] || fail "missing executable plan artifact gate"
for t in test-plan-artifact-gate.sh test-plan-evals.sh; do
  [ -x "$repo_root/scripts/tests/$t" ] || fail "missing executable planning eval/gate test: $t"
done
for skill in plan-memory-vector-intelligence plan-autoresearch-loop plan-rules-policy-org plan-distributed-compute plan-dependency-graph plan-prompt-architecture; do
  [ -f "$repo_root/agent-skills/$skill/SKILL.md" ] || fail "missing planning P0 skill: $skill"
done
for agent in plan-memory-vector-intelligence-auditor plan-autoresearch-loop-auditor plan-rules-policy-org-auditor plan-distributed-compute-auditor plan-dependency-graph-auditor plan-prompt-architecture-auditor; do
  [ -f "$repo_root/.codex/agents/$agent.toml" ] || fail "missing Codex P0 planning agent: $agent"
done
grep -q 'plan-artifact-gate.sh' "$repo_root/agent-skills/planning-engineer/SKILL.md" || fail "planning-engineer does not require artifact gate"
grep -q 'graph/target-dag.json' "$repo_root/agent-skills/plan-loop/SKILL.md" || fail "plan-loop does not require TDP target DAG"
grep -q 'SELF-REVISION' "$repo_root/agent-skills/plan-dependency-graph/SKILL.md" || fail "TDP skill missing SELF-REVISION"
grep -q 'findings/prompt-architecture-<T>.md' "$repo_root/agent-skills/plan-prompt-architecture/SKILL.md" || fail "prompt architecture skill missing artifact contract"
grep -q 'sources-<T>.jsonl' "$repo_root/agent-skills/plan-trend-research/SKILL.md" || fail "trend researcher missing source ledger contract"
grep -q 'agent-run-ledger' "$repo_root/agent-skills/plan-synthesis/SKILL.md" || fail "synthesis missing agent-run-ledger lift"
grep -q 'agent_interop' "$repo_root/agent-skills/planning-engineer/scripts/loop_state.template.md" || fail "loop state missing interop registry"

# Owner critical architecture-loop axes: persistent memory/vector intelligence, constant research,
# policy/org/A2A, Rust+Lua, distributed owner hardware, and multi-vendor local+cloud mesh.
grep -q 'Owner north-star architecture-loop upgrade' "$repo_root/agent-skills/planning-engineer/SKILL.md" || fail "planning-engineer missing owner critical architecture-loop section"
grep -q 'memory-vector-intelligence-<T>.md' "$repo_root/agent-skills/planning-engineer/SKILL.md" || fail "planning-engineer missing memory/vector artifact"
grep -q 'autoresearch-<T>.md' "$repo_root/agent-skills/planning-engineer/SKILL.md" || fail "planning-engineer missing autoresearch artifact"
grep -q 'rules-policy-org-<T>.md' "$repo_root/agent-skills/planning-engineer/SKILL.md" || fail "planning-engineer missing rules/policy/org artifact"
grep -q 'distributed-compute-<T>.md' "$repo_root/agent-skills/planning-engineer/SKILL.md" || fail "planning-engineer missing distributed compute artifact"
grep -q 'Pi Zero' "$repo_root/agent-skills/plan-distributed-compute/SKILL.md" || fail "distributed compute skill missing Pi Zero target"
grep -q 'ESP32' "$repo_root/agent-skills/plan-distributed-compute/SKILL.md" || fail "distributed compute skill missing ESP32 target"
grep -q 'Lua' "$repo_root/agent-skills/plan-distributed-compute/SKILL.md" || fail "distributed compute skill missing Lua target"
grep -q 'No Downgrades' "$repo_root/agent-skills/plan-rules-policy-org/SKILL.md" || fail "rules-policy skill missing no-downgrades rule"
grep -q 'ICM' "$repo_root/agent-skills/plan-memory-vector-intelligence/SKILL.md" || fail "memory-vector skill missing ICM"
grep -q 'stale-evidence' "$repo_root/agent-skills/plan-autoresearch-loop/SKILL.md" || fail "autoresearch skill missing stale-evidence invalidation"
if grep -nE 'directory = /tmp/\.tmp' "$repo_root/home/.gitconfig" >/tmp/envctl-safe-dir-grep.txt 2>/dev/null; then
  cat /tmp/envctl-safe-dir-grep.txt >&2
  fail "tracked home/.gitconfig must not trust ephemeral /tmp/.tmp* safe.directory paths"
fi

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

# ---- self-eval + self-upgrade after EVERY cycle is wired (anti-drift on the harness-evolution contract) ----
PE_DIR="$(dirname "$(dirname "$CONTRACT")")"
SKILLS_DIR="$(dirname "$PE_DIR")"
HARNESS_ROOT="$(dirname "$SKILLS_DIR")"
if [ "$SOURCE_LAYOUT" = "envctl" ]; then
  AGENTS_DIR="$repo_root/.codex/agents"
  AGENT_SUFFIX="toml"
  EVO_AGENT="$SKILLS_DIR/harness-evolution/SKILL.md"
else
  AGENTS_DIR="$HARNESS_ROOT/agents"
  AGENT_SUFFIX="md"
  EVO_AGENT="$AGENTS_DIR/evolution-steward.md"
fi
agent_path() { printf '%s/%s.%s\n' "$AGENTS_DIR" "$1" "$AGENT_SUFFIX"; }
PE_SKILL="$PE_DIR/SKILL.md"
PLAN_LOOP_SKILL="$SKILLS_DIR/plan-loop/SKILL.md"
for f in "$PE_SKILL" "$PLAN_LOOP_SKILL" "$EVO_AGENT"; do
  [ -f "$f" ] || fail "self-eval contract: required file missing: $f"
done
# planning-engineer single cycle: the every-cycle self-eval phase + the harness-evolution method.
grep -qiE 'SELF-EVAL \(every cycle\)' "$PE_SKILL" || fail "planning-engineer SKILL.md lost the 'SELF-EVAL (every cycle)' phase"
grep -qi  'harness-evolution'          "$PE_SKILL" || fail "planning-engineer SKILL.md no longer references the harness-evolution method"
# plan-loop: must self-evaluate AND self-upgrade every cycle (not only at the batch boundary), fail-closed.
grep -qiE 'self-eval.*self-upgrade'    "$PLAN_LOOP_SKILL" || fail "plan-loop SKILL.md must state per-cycle self-eval + self-upgrade"
grep -qiE 'after every cycle'          "$PLAN_LOOP_SKILL" || fail "plan-loop SKILL.md must run the evolution after every cycle"
grep -qi  'harness-evolution'          "$PLAN_LOOP_SKILL" || fail "plan-loop SKILL.md must reference the harness-evolution method"
grep -qiE 'never weaken'               "$PLAN_LOOP_SKILL" || fail "plan-loop SKILL.md must keep the never-weaken-a-gate guard"
# shared evolution contract: fires at every run boundary, fail-closed, never mid-cycle.
grep -qiE 'every cycle|every run boundary|end of any.*harness run' "$EVO_AGENT" || fail "evolution contract must run at every cycle boundary"
grep -qiE 'never mid-cycle'            "$EVO_AGENT" || fail "evolution-steward must keep the never-mid-cycle rule"
echo "PASS: self-eval+self-upgrade-every-cycle contract locked (planning-engineer Phase 5 · plan-loop · evolution-steward)"

# ---- prompt-parity: the loop has what the north-star prompt describes (laws · P4 · P5 · P6 · P8) ----
# (reuses the maintained skills and runtime-specific agent definitions resolved above.)
TREND_SKILL="$SKILLS_DIR/plan-trend-research/SKILL.md"
SYNTH_SKILL="$SKILLS_DIR/plan-synthesis/SKILL.md"
TSTRAT_SKILL="$SKILLS_DIR/plan-test-strategy/SKILL.md"
ANALYST="$(agent_path plan-analyst)"
CARTO="$(agent_path plan-cartographer)"
GCAUD="$(agent_path plan-governance-config-auditor)"
DDRIVE="$PE_DIR/scripts/differential-drive.sh"
for f in "$TREND_SKILL" "$SYNTH_SKILL" "$TSTRAT_SKILL" "$ANALYST" "$CARTO" "$GCAUD" "$DDRIVE"; do
  [ -f "$f" ] || fail "prompt-parity: required file missing: $f"
done
# LAW: latest-toolchain standing rule (bun-not-pnpm + shimmy/ruvllm-don't-remove-ollama-until-proven).
grep -qiE 'ruvllm'        "$PE_SKILL"  || fail "planning-engineer SKILL must carry the shimmy/ruvllm toolchain law"
grep -qiE '\bbun\b'       "$PE_SKILL"  || fail "planning-engineer SKILL must carry the bun-not-pnpm rule"
# P4: the control-plane diagram is REQUIRED (not just a prose governance section).
grep -qiE 'control-plane' "$SYNTH_SKILL" || fail "plan-synthesis must require the control-plane diagram"
# P5: the UPGRADE row carries the full schema (4th axis + risk-tier + acceptance↔test + reversibility).
grep -qiE 'governance\+settings\+config' "$ANALYST" || fail "plan-analyst UPGRADE row must include the governance+settings+config axis"
grep -qiE 'risk-tier'     "$ANALYST"  || fail "plan-analyst UPGRADE row must carry risk-tier APPLY/PROPOSE/REGENERATE"
grep -qiE 'acceptance'    "$ANALYST"  || fail "plan-analyst UPGRADE row must carry the acceptance criterion (1:1 with the P8 test)"
grep -qiE 'reversibility' "$ANALYST"  || fail "plan-analyst UPGRADE row must carry NORTH-STAR reversibility"
# P2: HuggingFace research + cross-repo-reference edges.
grep -qiE 'hugging ?face' "$TREND_SKILL" || fail "plan-trend-research must list the Hugging Face research tool"
grep -qiE 'cross-repo'    "$CARTO"    || fail "plan-cartographer must map cross-repo edges via cross-repo-reference"
# P8: the differential-drive driver EXISTS (was vaporware), is fail-closed, and is wired into the strategy.
grep -q  'tests-ran must be > 0' "$DDRIVE"      || fail "differential-drive.sh must enforce the fail-closed tests-ran>0 gate"
grep -qiE 'differential-drive\.sh' "$TSTRAT_SKILL" || fail "plan-test-strategy must drive cases via differential-drive.sh"
grep -qiE 'tests-ran'     "$TSTRAT_SKILL" || fail "plan-test-strategy must keep the tests-ran>0 count-verify"
echo "PASS: prompt-parity contract locked (toolchain law · P4 control-plane diagram · P5 row schema · P2 HF/cross-repo · P8 differential-drive + count-verify)"

# Inactive packs need not be materialized under `.agents/skills`; the tracked
# `agent-skills/` content above is the parity carrier and has already passed.
echo "PASS: maintained skill parity locked"

# ---- source-of-truth + transport + terminal artifact-gate hardening ----
PE_SCRIPT_DIR="$PE_DIR/scripts"
ARTIFACT_GATE="$PE_SCRIPT_DIR/plan-artifact-gate.sh"
WEAVE_DISPATCH="$PE_SCRIPT_DIR/plan-weave-dispatch.sh"
LOOP_TEMPLATE="$PE_SCRIPT_DIR/loop_state.template.md"
for f in "$ARTIFACT_GATE" "$WEAVE_DISPATCH" "$LOOP_TEMPLATE"; do
  [ -f "$f" ] || fail "source/transport contract: required file missing: $f"
done
# Source-of-truth/PromptHub contract: package first, then ejected mirrors; preserve owner prompt intent.
grep -q  'harness_hub'          "$PE_SKILL"        || fail "planning-engineer must name harness_hub as package source-of-truth"
grep -q  'harness_hub'          "$PLAN_LOOP_SKILL" || fail "plan-loop must name harness_hub as package source-of-truth"
grep -q  'PromptHub'            "$PE_SKILL"        || fail "planning-engineer must preserve upstream PromptHub intent"
grep -q  'PromptHub'            "$PLAN_LOOP_SKILL" || fail "plan-loop must preserve upstream PromptHub intent"
# Transport contract: no model downgrade; weave dispatch is the required background lane escape hatch.
grep -q  'plan-weave-dispatch.sh' "$PE_SKILL"        || fail "planning-engineer must route background Opus via plan-weave-dispatch.sh"
grep -q  'plan-weave-dispatch.sh' "$PLAN_LOOP_SKILL" || fail "plan-loop must route background Opus via plan-weave-dispatch.sh"
grep -q  'claude-opus-4-8'        "$WEAVE_DISPATCH"  || fail "weave dispatcher must preserve the claude-opus-4-8 capability contract"
grep -q  'rusty-idd-north-star'   "$WEAVE_DISPATCH"  || fail "weave dispatcher must include the rusty-idd north-star lane"
grep -q  'weave_dispatch'         "$LOOP_TEMPLATE"   || fail "loop_state template must include weave_dispatch"
# Durable state contract: the P8/P9-style ledgers and artifact gate must be first-class fields.
for field in target_dag artifact_gate source_ledger agent_run_ledger risk_policy agent_backend_matrix agent_interop; do
  grep -q "$field" "$LOOP_TEMPLATE" || fail "loop_state template missing field: $field"
done
grep -q 'plan-artifact-gate.sh' "$PE_SKILL"        || fail "planning-engineer DONE gate must invoke plan-artifact-gate.sh"
grep -q 'plan-artifact-gate.sh' "$PLAN_LOOP_SKILL" || fail "plan-loop DONE gate must invoke plan-artifact-gate.sh"
grep -q 'terminal plan state'   "$ARTIFACT_GATE"   || fail "artifact gate must reject terminal zero-target/nonterminal roll-ups"
# P9: concurrent fan-out peer artifacts are PENDING until the producer lane reports done.
for agent in "$(agent_path plan-analyst)" \
             "$(agent_path plan-governance-config-auditor)" \
             "$(agent_path plan-memory-vector-intelligence-auditor)" \
             "$(agent_path plan-prompt-architecture-auditor)"; do
  grep -q 'Concurrent peer-artifact rule (P9)' "$agent" || fail "agent missing P9 peer-artifact pending rule: $agent"
  grep -q 'PENDING' "$agent" || fail "agent must record not-yet-produced peer artifacts as PENDING: $agent"
done
# New runtime/ejection regression tests must be packaged so envctl can mirror them.
for t in test-plan-artifact-gate.sh test-plan-evals.sh test-plan-weave-dispatch.sh; do
  [ -f "$PE_SCRIPT_DIR/tests/$t" ] || fail "new planning regression test missing: $t"
done
echo "PASS: source-of-truth + weave transport + terminal artifact-gate contract locked"
echo "PASS: plan contract locked — targets rows, graph artifact names, JSON validity, and the documented examples all conform ($jq_note)"
