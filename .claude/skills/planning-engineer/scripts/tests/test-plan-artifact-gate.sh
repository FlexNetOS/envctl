#!/usr/bin/env bash
# test-plan-artifact-gate.sh — hermetic tests for the runtime .handoff/loop/plan artifact gate.
set -euo pipefail
root="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
resolve_script_under_test() {
  local rel="$1"
  local candidate
  for candidate in \
    "$root/scripts/$rel" \
    "$root/harness/skills/planning-engineer/scripts/$rel" \
    "$root/.claude/skills/planning-engineer/scripts/$rel" \
    "$root/.agents/skills/planning-engineer/scripts/$rel"; do
    [ -x "$candidate" ] && { printf '%s\n' "$candidate"; return 0; }
  done
  return 1
}
gate="$(resolve_script_under_test plan-artifact-gate.sh)" || { echo "FAIL: missing executable plan-artifact-gate.sh" >&2; exit 1; }

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
plan="$tmp/plan"
mkdir -p "$plan"/{graph,research,findings,reports}

cat > "$plan/targets.md" <<'EOF'
- [x] engine: shared sync library
EOF
cat > "$plan/dimensions.md" <<'EOF'
- [x] engine/architecture · architecture · boundaries verified · deps: none
- [x] engine/prompt-architecture · prompt architecture · prompt/tool couplings verified · deps: engine/architecture
EOF
cat > "$plan/graph/target-dag.json" <<'EOF'
{"nodes":[{"id":"engine","spec":"shared sync library","status":"done","deps":[],"artifact_prefix":"engine"}],"edges":[],"self_revision":[{"id":"sr-1","reason":"verifier refuted one claim","affected":["engine"],"action":"localized downstream update"}]}
EOF
cat > "$plan/graph/target-dag.md" <<'EOF'
# Target dependency graph
ready-set: engine
SELF-REVISION: sr-1 localized downstream update
EOF
for kind in symbols callgraph metrics; do echo '{"ok":true}' > "$plan/graph/engine.$kind.json"; done
cat > "$plan/graph/engine.graph.md" <<'EOF'
# engine graph
EOF
cat > "$plan/graph/engine.diff.md" <<'EOF'
# engine diff
EOF
cat > "$plan/reports/codemap-engine.md" <<'EOF'
# codemap engine
EOF
cat > "$plan/research/engine.trends.md" <<'EOF'
# engine trends
## Tool-currency & advisories
| tool | latest version | released | breaking? | CVE / advisory | recommend |
## Sources
1. Source — 2026-06-26
EOF
cat > "$plan/research/sources-engine.jsonl" <<'EOF'
{"url":"https://example.com","title":"Example","publisher":"Example","accessed_at":"2026-06-26","published_at":"2026-06-26","in_recency_window":true,"why_used":"fixture","claim_ids":["C1"]}
EOF
cat > "$plan/findings/governance-config-engine.md" <<'EOF'
# governance config
- CLAIM: hooks are present | evidence: AGENTS.md:1 | confidence: high
- UPGRADE: lock config | axis: governance+settings+config | rationale: safety | evidence: AGENTS.md:1 | blast: bounded | risk: low
EOF
cat > "$plan/findings/filesystem-layout-engine.md" <<'EOF'
# filesystem-layout
## Path inventory
## Placement verdict
## Boundary map
- UPGRADE: move cache | axis: filesystem-layout | evidence: path | risk: low
EOF
cat > "$plan/findings/test-strategy-engine.md" <<'EOF'
# test strategy
traceability matrix
tests-ran: 2
## FF test-build spec
EOF
cat > "$plan/findings/memory-vector-intelligence-engine.md" <<'EOF'
# memory-vector-intelligence
Memory inventory: ICM and .handoff present. Vector intelligence map: git-kb graph and RAG index freshness. Recall guarantees recorded.
EOF
cat > "$plan/findings/autoresearch-engine.md" <<'EOF'
# autoresearch
Code auto-research: git-kb refresh. Web auto-research: 90-day recency. stale evidence invalidation enabled.
EOF
cat > "$plan/findings/rules-policy-org-engine.md" <<'EOF'
# rules-policy-org
Upgrade Only. No Downgrades. agent org chart. weave/A2A communication. background agents required.
EOF
cat > "$plan/findings/distributed-compute-engine.md" <<'EOF'
# distributed-compute
Rust and Lua/Luau plan for mobile, AI glasses/wearables, Raspberry Pi / Pi Zero, ESP32, local and cloud vendor mesh.
EOF
cat > "$plan/findings/prompt-architecture-engine.md" <<'EOF'
# prompt-architecture
instruction surfaces, tools granted, model lanes, hidden couplings
ADR candidates: none; no-ADR rationale recorded
EOF
cat > "$plan/findings/verdicts.md" <<'EOF'
- C1 -> CONFIRMED | evidence: fixture
- U1 -> QUALIFIED feasible | condition: fixture
EOF
cat > "$plan/reports/agent-run-ledger-engine.md" <<'EOF'
# Agent run ledger
| lane | model | artifact | verdict |
| code-graph | claude-opus-4-8 | graph/engine.symbols.json | PASS |
EOF
cat > "$plan/reports/engine-plan.md" <<'EOF'
# Verdict
Confidence: High
## ASCII architecture
## Sequenced upgrade roadmap
## Tool-evaluation
## Governance, settings & config
## Filesystem layout
## Test Strategy & Coverage
## Memory/vector intelligence
## Auto-research
## Rules/policy/org
## Distributed compute
## Prompt-architecture
## Risk policy
## Confidence
EOF
cat > "$plan/risk-policy.md" <<'EOF'
risk_policy: SUPERVISED for destructive, trust-boundary, secrets, filesystem migration, provider/model changes.
EOF
cat > "$plan/agent-backend-matrix.md" <<'EOF'
read-only-local | isolated-worktree | container | remote-vm | cloud-agent | ACP | A2A
EOF
cat > "$plan/agent-interop.md" <<'EOF'
weave, mcp, ACP, A2A, GitHub cloud agent routing registry
EOF
cat > "$plan/evaluation.md" <<'EOF'
evolution scorecard self-eval
EOF
cat > "$plan/DONE" <<'EOF'
completeness sweep: CONFIRMED no major unexamined area. Evidence: QUALIFIED graph sweep.
EOF

bash "$gate" "$plan" >/tmp/plan-artifact-good.out

defect="$tmp/defect"
cp -R "$plan" "$defect"
rm "$defect/findings/prompt-architecture-engine.md"
if bash "$gate" "$defect" >/tmp/plan-artifact-bad.out 2>/tmp/plan-artifact-bad.err; then
  echo "FAIL: gate accepted missing prompt architecture finding" >&2
  exit 1
fi
grep -q 'prompt-architecture-engine.md' /tmp/plan-artifact-bad.err || { cat /tmp/plan-artifact-bad.err >&2; echo "FAIL: missing-artifact error did not name prompt architecture" >&2; exit 1; }

defect2="$tmp/defect2"
cp -R "$plan" "$defect2"
sed -i 's/- \[x\] engine/- [~] engine/' "$defect2/targets.md"
if bash "$gate" "$defect2" >/tmp/plan-artifact-bad2.out 2>/tmp/plan-artifact-bad2.err; then
  echo "FAIL: gate accepted DONE with nonterminal target" >&2
  exit 1
fi
grep -q 'not terminal' /tmp/plan-artifact-bad2.err || { cat /tmp/plan-artifact-bad2.err >&2; echo "FAIL: nonterminal DONE error missing" >&2; exit 1; }

defect3="$tmp/defect3"
cp -R "$plan" "$defect3"
rm "$defect3/DONE"
sed -i 's/- \[x\] engine/- [~] engine/' "$defect3/targets.md"
cat > "$defect3/loop_state.md" <<'EOF'
status: COMPLETE
cycles_total: 7
last_wrapup_total: 5
EOF
if bash "$gate" "$defect3" >/tmp/plan-artifact-bad3.out 2>/tmp/plan-artifact-bad3.err; then
  echo "FAIL: gate accepted terminal loop_state with nonterminal target" >&2
  exit 1
fi
grep -q 'terminal plan state' /tmp/plan-artifact-bad3.err || { cat /tmp/plan-artifact-bad3.err >&2; echo "FAIL: terminal loop_state error missing" >&2; exit 1; }

defect4="$tmp/defect4"
cp -R "$plan" "$defect4"
rm "$defect4/DONE" "$defect4/targets.md"
cat > "$defect4/loop_state.md" <<'EOF'
status: COMPLETE
cycles_total: 7
last_wrapup_total: 5
EOF
if bash "$gate" "$defect4" >/tmp/plan-artifact-bad4.out 2>/tmp/plan-artifact-bad4.err; then
  echo "FAIL: gate accepted terminal loop_state with zero target rows" >&2
  exit 1
fi
grep -q 'no target rows' /tmp/plan-artifact-bad4.err || { cat /tmp/plan-artifact-bad4.err >&2; echo "FAIL: zero-target terminal loop_state error missing" >&2; exit 1; }

echo "PASS: plan artifact gate rejects incomplete runtime artifacts, DONE drift, and zero-target terminal roll-ups"
