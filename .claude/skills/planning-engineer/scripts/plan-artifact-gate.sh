#!/usr/bin/env bash
# plan-artifact-gate.sh — validate a real .handoff/loop/plan/ run before DONE.
#
# This is the runtime counterpart to the prompt/contract tests: it proves that an actual planning
# target produced the required graph/research/findings/report/control-plane artifacts and that DONE is
# not written over weak or missing evidence.
set -euo pipefail
PLAN_DIR="${1:-.handoff/loop/plan}"
python3 - "$PLAN_DIR" <<'PY'
from __future__ import annotations
import json, re, sys
from pathlib import Path

plan = Path(sys.argv[1])
errors: list[str] = []
SLUG = re.compile(r"^[a-z0-9][a-z0-9-]*$")
TARGET_ROW = re.compile(r"^- \[( |x|~|!|!!)\] ([^:]+):\s*(.+)$")
DIM_ROW = re.compile(r"^- \[( |x|~|!)\] ([a-z0-9][a-z0-9-]*/[^\s]+)")


def err(msg: str) -> None:
    errors.append(msg)


def text(rel: str) -> str:
    p = plan / rel
    try:
        return p.read_text(encoding="utf-8")
    except FileNotFoundError:
        err(f"missing required artifact: {rel}")
        return ""


def require_file(rel: str) -> Path:
    p = plan / rel
    if not p.is_file():
        err(f"missing required artifact: {rel}")
    elif p.stat().st_size == 0:
        err(f"empty required artifact: {rel}")
    return p


def require_json(rel: str) -> object | None:
    p = require_file(rel)
    if not p.is_file():
        return None
    try:
        return json.loads(p.read_text(encoding="utf-8"))
    except Exception as exc:
        err(f"invalid JSON in {rel}: {exc}")
        return None


def require_contains(rel: str, patterns: list[str], *, mode: str = "all") -> None:
    s = text(rel)
    if not s:
        return
    hits = [pat for pat in patterns if re.search(pat, s, re.I | re.M)]
    if mode == "all" and len(hits) != len(patterns):
        missing = [pat for pat in patterns if pat not in hits]
        err(f"{rel} missing markers: {', '.join(missing)}")
    if mode == "any" and not hits:
        err(f"{rel} missing one of markers: {', '.join(patterns)}")
    if re.search(r"\b(TODO|TBD|placeholder evidence|citation needed)\b", s, re.I):
        err(f"{rel} contains placeholder/TODO evidence")


def parse_targets() -> list[tuple[str, str, str]]:
    rows: list[tuple[str, str, str]] = []
    p = plan / "targets.md"
    if not p.exists():
        if (plan / "DONE").exists():
            err("DONE exists but targets.md is missing")
        return rows
    seen: set[str] = set()
    for n, line in enumerate(p.read_text(encoding="utf-8").splitlines(), 1):
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        m = TARGET_ROW.match(line)
        if not m:
            err(f"targets.md:{n}: invalid row: {line}")
            continue
        status, slug, scope = m.groups()
        if not SLUG.match(slug):
            err(f"targets.md:{n}: invalid target slug: {slug}")
        if slug in seen:
            err(f"targets.md:{n}: duplicate target slug: {slug}")
        seen.add(slug)
        if not scope.strip():
            err(f"targets.md:{n}: missing scope for {slug}")
        rows.append((status, slug, scope))
    return rows


def validate_source_ledger(target: str) -> None:
    rel = f"research/sources-{target}.jsonl"
    p = require_file(rel)
    if not p.is_file():
        return
    required = {"url", "title", "publisher", "accessed_at", "published_at", "in_recency_window", "why_used", "claim_ids"}
    count = 0
    for n, line in enumerate(p.read_text(encoding="utf-8").splitlines(), 1):
        if not line.strip():
            continue
        count += 1
        try:
            row = json.loads(line)
        except Exception as exc:
            err(f"{rel}:{n}: invalid JSONL row: {exc}")
            continue
        missing = required - set(row)
        if missing:
            err(f"{rel}:{n}: missing keys {sorted(missing)}")
        if not isinstance(row.get("claim_ids"), list) or not row.get("claim_ids"):
            err(f"{rel}:{n}: claim_ids must be a non-empty list")
    if count == 0:
        err(f"{rel}: no source rows")


def validate_target_dag(targets: list[str]) -> None:
    dag = require_json("graph/target-dag.json")
    require_contains("graph/target-dag.md", [r"target dependency graph", r"SELF-REVISION", r"ready-set|topological"])
    if not isinstance(dag, dict):
        return
    nodes = dag.get("nodes")
    edges = dag.get("edges", [])
    if not isinstance(nodes, list) or not nodes:
        err("graph/target-dag.json: nodes must be a non-empty list")
        return
    node_ids = {str(n.get("id")) for n in nodes if isinstance(n, dict) and n.get("id")}
    for t in targets:
        if t not in node_ids:
            err(f"graph/target-dag.json: missing target node {t}")
    if not isinstance(edges, list):
        err("graph/target-dag.json: edges must be a list")
    for n in nodes:
        if not isinstance(n, dict):
            err("graph/target-dag.json: each node must be an object")
            continue
        for key in ("id", "spec", "status", "deps", "artifact_prefix"):
            if key not in n:
                err(f"graph/target-dag.json node {n.get('id','<unknown>')}: missing {key}")


def validate_global_artifacts(targets: list[str]) -> None:
    validate_target_dag(targets)
    require_contains("risk-policy.md", [r"risk_policy", r"SUPERVISED", r"trust-boundary|secrets|destructive|provider/model"])
    require_contains("agent-backend-matrix.md", [r"read-only-local", r"isolated-worktree", r"container", r"remote-vm", r"cloud-agent", r"ACP|A2A"])
    require_contains("agent-interop.md", [r"weave", r"mcp", r"ACP", r"A2A", r"GitHub cloud agent"])
    require_contains("evaluation.md", [r"evolution|scorecard|self-eval"])


def validate_target(target: str) -> None:
    for rel in [
        f"graph/{target}.symbols.json",
        f"graph/{target}.callgraph.json",
        f"graph/{target}.metrics.json",
    ]:
        require_json(rel)
    for rel in [
        f"graph/{target}.graph.md",
        f"graph/{target}.diff.md",
        f"reports/codemap-{target}.md",
        f"research/{target}.trends.md",
        f"findings/governance-config-{target}.md",
        f"findings/filesystem-layout-{target}.md",
        f"findings/test-strategy-{target}.md",
        f"findings/memory-vector-intelligence-{target}.md",
        f"findings/autoresearch-{target}.md",
        f"findings/rules-policy-org-{target}.md",
        f"findings/distributed-compute-{target}.md",
        f"findings/prompt-architecture-{target}.md",
        f"reports/agent-run-ledger-{target}.md",
        f"reports/{target}-plan.md",
    ]:
        require_file(rel)
    validate_source_ledger(target)
    require_contains(f"findings/governance-config-{target}.md", [r"CLAIM", r"UPGRADE", r"governance\+settings\+config"])
    require_contains(f"findings/filesystem-layout-{target}.md", [r"path inventory", r"placement verdict", r"boundary map", r"filesystem-layout"])
    require_contains(f"findings/test-strategy-{target}.md", [r"tests-ran\s*[:=]\s*[1-9]", r"traceability", r"FF test-build spec"])
    require_contains(f"findings/memory-vector-intelligence-{target}.md", [r"memory", r"vector|git-kb|RAG", r"ICM|handoff", r"recall"])
    require_contains(f"findings/autoresearch-{target}.md", [r"code auto-research|git-kb", r"web auto-research|90-day|recency", r"stale|invalidate"])
    require_contains(f"findings/rules-policy-org-{target}.md", [r"Upgrade Only", r"No Downgrades", r"agent org chart", r"weave|A2A", r"background"] )
    require_contains(f"findings/distributed-compute-{target}.md", [r"Rust", r"Lua|Luau", r"mobile", r"AI glasses|wearables", r"Pi Zero|Raspberry Pi", r"ESP32", r"vendor|cloud|local"])
    require_contains(f"findings/prompt-architecture-{target}.md", [r"prompt-architecture", r"tool grants|tools granted", r"model lanes", r"ADR"])
    require_contains(f"reports/agent-run-ledger-{target}.md", [r"agent run ledger", r"lane", r"model", r"artifact"])
    require_contains(f"research/{target}.trends.md", [r"Tool-currency & advisories", r"Sources"])
    require_contains(f"reports/{target}-plan.md", [
        r"Verdict", r"ASCII architecture", r"Sequenced upgrade", r"Tool-evaluation",
        r"Governance", r"Filesystem layout", r"Memory/vector", r"Auto-research",
        r"Rules/policy", r"Distributed compute", r"Test Strategy", r"Prompt-architecture",
        r"Risk policy", r"Confidence"
    ])


def validate_verdicts() -> None:
    require_contains("findings/verdicts.md", [r"VERDICT|->", r"CONFIRMED|QUALIFIED|REFUTED|INCONCLUSIVE"], mode="all")
    s = text("findings/verdicts.md")
    if s and not re.search(r"CONFIRMED|QUALIFIED", s, re.I):
        err("findings/verdicts.md: completed plans require at least one CONFIRMED or QUALIFIED verdict")
    if s and re.search(r"UPGRADE", s, re.I) and not re.search(r"feasible|infeasible|feasibility", s, re.I):
        err("findings/verdicts.md: UPGRADE rows must include feasibility verdicts")


def validate_done(target_rows: list[tuple[str, str, str]]) -> None:
    done = plan / "DONE"
    if not done.exists():
        return
    if not target_rows:
        err("DONE exists but no target rows were parsed")
    for status, slug, _ in target_rows:
        if status not in {"x", "!"}:
            err(f"DONE exists but target {slug} is not terminal: [{status or ' '}]")
    dims = plan / "dimensions.md"
    if not dims.exists():
        err("DONE exists but dimensions.md is missing")
    else:
        for n, line in enumerate(dims.read_text(encoding="utf-8").splitlines(), 1):
            if not line.strip() or line.lstrip().startswith("#"):
                continue
            m = DIM_ROW.match(line)
            if not m:
                err(f"dimensions.md:{n}: invalid dimension row: {line}")
                continue
            status = m.group(1)
            if status not in {"x", "!"}:
                err(f"DONE exists but dimension row is not terminal: dimensions.md:{n}: {line}")
    d = done.read_text(encoding="utf-8")
    if not re.search(r"completeness sweep", d, re.I) or not re.search(r"CONFIRMED|QUALIFIED", d):
        err("DONE must record completeness sweep and confirmed/qualified evidence")


target_rows = parse_targets()
terminal_planned = [slug for status, slug, _ in target_rows if status == "x"]
if terminal_planned:
    validate_global_artifacts([slug for _, slug, _ in target_rows])
    validate_verdicts()
    for slug in terminal_planned:
        validate_target(slug)
validate_done(target_rows)

if errors:
    for e in errors:
        print(f"FAIL: {e}", file=sys.stderr)
    raise SystemExit(1)
print(f"PASS: plan artifact gate validated {len(terminal_planned)} planned target(s) under {plan}")
PY
