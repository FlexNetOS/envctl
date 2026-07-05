
from __future__ import annotations
import subprocess
import sys, json
from collections import defaultdict
from pathlib import Path
from _common import read_task_graph, split_list, write_json, root, package_root, load_ledger, make_proof, append_proof, now, sha256_file

TERMINAL_COMPLETE = {'completed','complete','passed'}
TERMINAL_FAILED = {'failed','error'}
APPROVAL_DECISION_APPROVED = {'approved', 'approve'}
APPROVAL_DECISION_DENIED = {'denied', 'deny'}
REQUIRED_APPROVAL_FIELDS = {'task_id', 'reviewer', 'model', 'decision', 'reviewed_at', 'review_artifact', 'summary', 'evidence'}
ALLOWED_APPROVAL_DIRS = {'approvals', 'reviews', 'proof_records'}


def rel_to_root(path: Path) -> str:
    return str(path.relative_to(root()))


def package_rel(path: Path) -> str:
    return str(path.relative_to(package_root()))


def fail_approval(approval_path: Path, reason: str, extra: dict | None = None):
    out = {'valid': False, 'reason': reason, 'path': rel_to_root(approval_path)}
    if extra:
        out.update(extra)
    return out


def resolve_approval_path(value: str) -> Path | None:
    if not value or Path(value).is_absolute():
        return None
    candidate = (root() / value).resolve()
    try:
        rel = candidate.relative_to(root().resolve())
    except ValueError:
        return None
    if not rel.parts or rel.parts[0] not in ALLOWED_APPROVAL_DIRS:
        return None
    return candidate


def git_index_contains(path: Path) -> bool:
    try:
        repo_root = package_root().parent
        rel = path.relative_to(repo_root)
    except ValueError:
        return False
    result = subprocess.run(
        ['git', '-C', str(repo_root), 'ls-files', '--cached', '--', str(rel)],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    return str(rel) in result.stdout.splitlines()


def load_agent_approval(task_id: str, proof_by_task: dict):
    approval_path = root() / 'approvals' / f'{task_id}.agent_approval.json'
    if not approval_path.exists():
        return None
    try:
        approval = json.loads(approval_path.read_text(encoding='utf-8'))
    except json.JSONDecodeError as exc:
        return fail_approval(approval_path, f'invalid approval json: {exc}')

    missing = sorted(REQUIRED_APPROVAL_FIELDS - set(approval))
    if missing:
        return fail_approval(approval_path, f'agent approval missing required fields: {", ".join(missing)}')
    if approval.get('task_id') != task_id:
        return fail_approval(approval_path, f'agent approval task_id mismatch: {approval.get("task_id") or "missing"}')

    decision = str(approval.get('decision', '')).lower()
    reviewer = str(approval.get('reviewer', '')).strip()
    model = str(approval.get('model', '')).strip()
    summary = str(approval.get('summary', '')).strip()
    review_artifact = str(approval.get('review_artifact', '')).strip()
    evidence = approval.get('evidence') or []
    if not isinstance(evidence, list):
        return fail_approval(approval_path, 'agent reviewer evidence must be a list')
    if decision not in APPROVAL_DECISION_APPROVED:
        reason = f'agent reviewer decision is {decision or "missing"}'
        if decision in APPROVAL_DECISION_DENIED:
            denial = str(approval.get('instruction_on_denial', '')).strip()
            reason = f'agent reviewer denied approval: {denial or "instructions missing"}'
            return fail_approval(approval_path, reason, {'denial_instructions': denial})
        return fail_approval(approval_path, reason)
    if not reviewer:
        return fail_approval(approval_path, 'agent reviewer missing')
    if not model:
        return fail_approval(approval_path, 'agent reviewer model missing')
    if not summary:
        return fail_approval(approval_path, 'agent reviewer summary missing')
    if not evidence:
        return fail_approval(approval_path, 'agent reviewer evidence missing')
    review_path = resolve_approval_path(review_artifact)
    if not review_path or not review_path.exists() or not review_path.is_file():
        return fail_approval(approval_path, f'agent review artifact missing or outside approved area: {review_artifact or "missing"}')
    if review_artifact not in evidence:
        return fail_approval(approval_path, 'agent reviewer evidence does not include review_artifact')
    evidence_paths = []
    for raw in evidence:
        if not isinstance(raw, str):
            return fail_approval(approval_path, 'agent reviewer evidence entries must be paths')
        path = resolve_approval_path(raw)
        if not path or not path.exists() or not path.is_file():
            return fail_approval(approval_path, f'agent reviewer evidence missing or outside approved area: {raw}')
        evidence_paths.append(path)

    if reviewer.lower() not in {'claude-opus-4.8', 'claude-opus', 'opus'}:
        fallback_reason = str(approval.get('fallback_reason', '')).strip()
        if not fallback_reason:
            return fail_approval(approval_path, 'fallback reviewer approval missing fallback_reason')

    proof_id = f'APPROVAL-{task_id}'
    proof = proof_by_task.get(proof_id)
    if not proof:
        return fail_approval(approval_path, f'matching proof record missing: {proof_id}')
    if str(proof.get('status', '')).lower() not in TERMINAL_COMPLETE:
        return fail_approval(approval_path, f'matching proof record is not complete: {proof.get("status") or "missing"}')
    if proof.get('actor') != 'agent-approval-gate':
        return fail_approval(approval_path, f'matching proof actor is not agent-approval-gate: {proof.get("actor") or "missing"}')
    if proof.get('model_tag') != model:
        return fail_approval(approval_path, 'matching proof model_tag does not match approval model')
    verification = proof.get('verification_output') or {}
    if verification.get('reviewer') != reviewer:
        return fail_approval(approval_path, 'matching proof reviewer does not match approval reviewer')
    if verification.get('decision') != decision:
        return fail_approval(approval_path, 'matching proof decision does not match approval decision')
    proof_evidence = set(proof.get('evidence') or [])
    required_evidence = {rel_to_root(approval_path), review_artifact}
    missing_evidence = sorted(required_evidence - proof_evidence)
    if missing_evidence:
        return fail_approval(approval_path, f'matching proof missing evidence: {", ".join(missing_evidence)}')
    checksums = proof.get('checksums') or {}
    required_checksums = {
        package_rel(approval_path): sha256_file(approval_path),
        package_rel(review_path): sha256_file(review_path),
    }
    bad_checksums = [path for path, checksum in required_checksums.items() if checksums.get(path) != checksum]
    if bad_checksums:
        return fail_approval(approval_path, f'matching proof checksum mismatch: {", ".join(bad_checksums)}')

    tracked_paths = [approval_path, *evidence_paths]
    untracked = [package_rel(p) for p in tracked_paths if not git_index_contains(p)]
    if untracked:
        return fail_approval(approval_path, f'agent approval artifacts not tracked in git index: {", ".join(untracked)}')

    return {'valid': True, 'path': rel_to_root(approval_path), 'reviewer': reviewer, 'model': model, 'decision': decision, 'review_artifact': review_artifact}


def compute(rows, proofs):
    proof_by_task = {p.get('task_id'): p for p in proofs if p.get('task_id')}
    statuses = {}
    for r in rows:
        p = proof_by_task.get(r['task_id'])
        if p and str(p.get('status','')).lower() in TERMINAL_COMPLETE:
            statuses[r['task_id']] = 'complete'
        elif p and str(p.get('status','')).lower() in TERMINAL_FAILED:
            statuses[r['task_id']] = 'failed'
        else:
            statuses[r['task_id']] = r.get('status','pending') or 'pending'
    runnable, blocked, approval = [], [], []
    idset = {r['task_id'] for r in rows}
    for r in rows:
        tid = r['task_id']
        if statuses[tid] in {'complete','failed'}:
            continue
        deps = [d for d in split_list(r.get('depends_on','')) if not d.endswith('*')]
        unknown = [d for d in deps if d not in idset]
        unmet = [d for d in deps if statuses.get(d) != 'complete']
        if unknown:
            blocked.append({'task_id':tid,'reason':'unknown dependencies','dependencies':unknown})
        elif unmet:
            blocked.append({'task_id':tid,'reason':'unmet dependencies','dependencies':unmet})
        elif r.get('human_approval_required','').lower() == 'true':
            agent_approval = load_agent_approval(tid, proof_by_task)
            if agent_approval and agent_approval.get('valid'):
                runnable.append({'task_id':tid,'parallel_group':r.get('parallel_group',''),'max_parallel':int(r.get('max_parallel','1') or 1),'packet_uri':f'generated/execution_packets/{tid}.json','approval_source':'agent_review','approval_uri':agent_approval['path'],'approval_reviewer':agent_approval['reviewer']})
            else:
                reason = 'agent approval required'
                if agent_approval:
                    reason = agent_approval.get('reason', reason)
                approval.append({'task_id':tid,'reason':reason,'packet_uri':f'generated/execution_packets/{tid}.json','agent_approval_uri':f'approvals/{tid}.agent_approval.json'})
        else:
            runnable.append({'task_id':tid,'parallel_group':r.get('parallel_group',''),'max_parallel':int(r.get('max_parallel','1') or 1),'packet_uri':f'generated/execution_packets/{tid}.json'})
    # Respect max parallel per group in dispatch list.
    group_counts = defaultdict(int)
    dispatch = []
    for r in runnable:
        if group_counts[r['parallel_group']] < r['max_parallel']:
            dispatch.append(r); group_counts[r['parallel_group']] += 1
    complete = sum(1 for s in statuses.values() if s == 'complete')
    failed = sum(1 for s in statuses.values() if s == 'failed')
    state = {
        'schema_version':'1.0','generated_at':now(),'task_count':len(rows),'complete_count':complete,'failed_count':failed,'pending_count':len(rows)-complete-failed,'runnable_count':len(runnable),'dispatch_count':len(dispatch),'approval_blocker_count':len(approval),'blocked_count':len(blocked),'runnable_tasks':runnable,'dispatch_packets':dispatch,'approval_blockers':approval,'blocked_tasks':blocked,'statuses':statuses
    }
    return state


def main():
    graph_arg = sys.argv[1] if len(sys.argv) > 1 else 'generated/task_graph.csv'
    rows = read_task_graph(graph_arg)
    state = compute(rows, load_ledger())
    write_json('state/goal_loop_state.json', state)
    write_json('generated/status_report.json', state)
    proof = make_proof('EF-007_GOAL_LOOP','completed','local-execution-framework','helper-goal-01','gpt-5.3-spark','execution-framework',['execution-framework/state/goal_loop_state.json','execution-framework/generated/status_report.json'],[f'python3 scripts/goal_loop.py {graph_arg}'],{'dispatch_count':state['dispatch_count'],'approval_blocker_count':state['approval_blocker_count'],'blocked_count':state['blocked_count']},['state/goal_loop_state.json','generated/status_report.json'],'','run verify_history_and_completeness.py')
    append_proof(proof)
    # Recompute including own proof.
    state = compute(rows, load_ledger())
    write_json('state/goal_loop_state.json', state)
    write_json('generated/status_report.json', state)
    print(f"goal loop status: complete={state['complete_count']} runnable={state['runnable_count']} approvals={state['approval_blocker_count']} blocked={state['blocked_count']}")

if __name__ == '__main__':
    main()
