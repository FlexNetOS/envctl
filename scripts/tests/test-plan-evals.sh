#!/usr/bin/env bash
# test-plan-evals.sh — small golden evals for planning quality gates beyond file presence.
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

make_good() {
  local plan="$1"; mkdir -p "$plan"/{graph,research,findings,reports}
  cat > "$plan/targets.md" <<'EOF'
- [x] engine: shared sync library
EOF
  cat > "$plan/dimensions.md" <<'EOF'
- [x] engine/architecture · architecture · boundaries verified · deps: none
EOF
  echo '{"nodes":[{"id":"engine","spec":"shared sync library","status":"done","deps":[],"artifact_prefix":"engine"}],"edges":[],"self_revision":[{"id":"sr-1","reason":"fixture","affected":["engine"],"action":"localized"}]}' > "$plan/graph/target-dag.json"
  printf '# Target dependency graph\nready-set: engine\nSELF-REVISION: localized\n' > "$plan/graph/target-dag.md"
  for kind in symbols callgraph metrics; do echo '{"ok":true}' > "$plan/graph/engine.$kind.json"; done
  for f in graph/engine.graph.md graph/engine.diff.md reports/codemap-engine.md; do echo "# $f" > "$plan/$f"; done
  printf '# trends\n## Tool-currency & advisories\n## Sources\n' > "$plan/research/engine.trends.md"
  echo '{"url":"https://example.com","title":"Example","publisher":"Example","accessed_at":"2026-06-26","published_at":"2026-06-26","in_recency_window":true,"why_used":"fixture","claim_ids":["C1"]}' > "$plan/research/sources-engine.jsonl"
  printf -- '- CLAIM: ok | evidence: x:1 | confidence: high\n- UPGRADE: ok | axis: governance+settings+config | evidence: x:1 | blast: low | risk: low\n' > "$plan/findings/governance-config-engine.md"
  printf 'path inventory\nplacement verdict\nboundary map\nfilesystem-layout\n' > "$plan/findings/filesystem-layout-engine.md"
  printf 'tests-ran: 1\ntraceability\n## FF test-build spec\n' > "$plan/findings/test-strategy-engine.md"
  printf 'memory vector git-kb RAG ICM handoff recall\n' > "$plan/findings/memory-vector-intelligence-engine.md"
  printf 'code auto-research git-kb web auto-research 90-day recency stale invalidate\n' > "$plan/findings/autoresearch-engine.md"
  printf 'Upgrade Only No Downgrades agent org chart weave A2A background\n' > "$plan/findings/rules-policy-org-engine.md"
  printf 'Rust Lua Luau mobile AI glasses wearables Raspberry Pi Pi Zero ESP32 local cloud vendor\n' > "$plan/findings/distributed-compute-engine.md"
  printf 'prompt-architecture\ntools granted\nmodel lanes\nADR candidates: none\n' > "$plan/findings/prompt-architecture-engine.md"
  printf -- '- C1 -> CONFIRMED\n- U1 -> QUALIFIED feasible\n' > "$plan/findings/verdicts.md"
  printf '# Agent run ledger\nlane model artifact\n' > "$plan/reports/agent-run-ledger-engine.md"
  printf 'Verdict\nASCII architecture\nSequenced upgrade\nTool-evaluation\nGovernance\nFilesystem layout\nTest Strategy\nPrompt-architecture\nMemory/vector\nAuto-research\nRules/policy\nDistributed compute\nRisk policy\nConfidence\n' > "$plan/reports/engine-plan.md"
  printf 'risk_policy SUPERVISED trust-boundary secrets destructive provider/model\n' > "$plan/risk-policy.md"
  printf 'read-only-local isolated-worktree container remote-vm cloud-agent ACP A2A\n' > "$plan/agent-backend-matrix.md"
  printf 'weave mcp ACP A2A GitHub cloud agent\n' > "$plan/agent-interop.md"
  printf 'evolution scorecard self-eval\n' > "$plan/evaluation.md"
  printf 'completeness sweep CONFIRMED QUALIFIED\n' > "$plan/DONE"
}

good="$tmp/good"; make_good "$good"; bash "$gate" "$good" >/dev/null

# Eval 1: unverified claim promoted to DONE must fail through missing verifier evidence.
bad1="$tmp/bad1"; cp -R "$good" "$bad1"; printf -- '- C1 -> INCONCLUSIVE\n' > "$bad1/findings/verdicts.md"
if bash "$gate" "$bad1" >/dev/null 2>/tmp/eval-bad1.err; then echo "FAIL: accepted no confirmed/qualified verdict" >&2; exit 1; fi

# Eval 2: source ledger must be reproducible JSONL, not prose citations only.
bad2="$tmp/bad2"; cp -R "$good" "$bad2"; echo 'Source: example' > "$bad2/research/sources-engine.jsonl"
if bash "$gate" "$bad2" >/dev/null 2>/tmp/eval-bad2.err; then echo "FAIL: accepted non-JSON source ledger" >&2; exit 1; fi

# Eval 3: TDP self-revision must be represented in graph markdown.
bad3="$tmp/bad3"; cp -R "$good" "$bad3"; echo '# Target dependency graph' > "$bad3/graph/target-dag.md"
if bash "$gate" "$bad3" >/dev/null 2>/tmp/eval-bad3.err; then echo "FAIL: accepted target DAG without self-revision marker" >&2; exit 1; fi

# Eval 4: prompt architecture coupling requires its finding file.
bad4="$tmp/bad4"; cp -R "$good" "$bad4"; rm "$bad4/findings/prompt-architecture-engine.md"
if bash "$gate" "$bad4" >/dev/null 2>/tmp/eval-bad4.err; then echo "FAIL: accepted prompt-architecture gap" >&2; exit 1; fi

echo "PASS: planning eval fixtures cover verifier, source-ledger, TDP, and prompt-architecture regressions"
