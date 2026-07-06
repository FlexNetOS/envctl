
from __future__ import annotations
import sys
from pathlib import Path
from _common import TASK_COLUMNS, read_task_graph, split_list, write_json, root, load_ledger, make_proof, append_proof


def has_cycle(rows):
    ids = {r['task_id'] for r in rows}
    graph = {r['task_id']: [d for d in split_list(r.get('depends_on','')) if d in ids] for r in rows}
    visiting, visited = set(), set()
    def dfs(n):
        if n in visiting: return True
        if n in visited: return False
        visiting.add(n)
        for m in graph.get(n, []):
            if dfs(m): return True
        visiting.remove(n); visited.add(n)
        return False
    return any(dfs(n) for n in graph)


def main():
    graph_arg = sys.argv[1] if len(sys.argv) > 1 else 'generated/task_graph.csv'
    rows = read_task_graph(graph_arg)
    errors, warnings = [], []
    if not rows:
        errors.append('task graph has no rows')
    columns = set(rows[0].keys()) if rows else set()
    for c in TASK_COLUMNS:
        if c not in columns:
            errors.append(f'missing required column: {c}')
    ids = [r.get('task_id','') for r in rows]
    if len(ids) != len(set(ids)):
        errors.append('task IDs are not unique')
    idset = set(ids)
    for r in rows:
        tid = r.get('task_id','')
        for dep in split_list(r.get('depends_on','')):
            if dep not in idset:
                # wildcard dependencies such as ART-* are explicit aggregate markers
                if dep.endswith('*'):
                    warnings.append(f'{tid} uses aggregate dependency marker {dep}')
                else:
                    errors.append(f'{tid} depends_on unknown task {dep}')
        for block in split_list(r.get('blocks','')):
            if block and block not in idset and not block.endswith('*'):
                errors.append(f'{tid} blocks unknown task {block}')
        if r.get('can_run_parallel','').lower() not in {'true','false'}:
            errors.append(f'{tid} can_run_parallel must be true/false')
        try:
            int(r.get('max_parallel',''))
        except Exception:
            errors.append(f'{tid} max_parallel is not an integer')
        if not r.get('owner_lane'):
            errors.append(f'{tid} missing owner_lane')
        if not (r.get('helper_id') or r.get('owner_agent')):
            errors.append(f'{tid} missing helper_id/owner_agent')
        if not r.get('model_tag'):
            errors.append(f'{tid} missing model_tag')
        if not r.get('proof_uri'):
            errors.append(f'{tid} missing proof_uri')
        if r.get('proof_required','').lower() != 'true':
            errors.append(f'{tid} proof_required must be true')
        if not (r.get('verification_command') or r.get('completion_gate')):
            errors.append(f'{tid} missing verification_command/completion_gate')
        if r.get('repo_target') in {'repo_a','repo_b','multi_repo'} and not (r.get('repo_target') and r.get('repo_path')):
            errors.append(f'{tid} two-repo task missing repo_target/repo_path')
        if r.get('repo_target') == 'filesystem' and not r.get('filesystem_scope'):
            errors.append(f'{tid} filesystem task missing filesystem_scope')
        if not r.get('allowed_paths') or not r.get('blocked_paths'):
            errors.append(f'{tid} missing allowed_paths/blocked_paths')
        if r.get('status','').lower() == 'complete' and not r.get('proof_uri'):
            errors.append(f'{tid} marked complete without proof_uri')
    if has_cycle(rows):
        errors.append('dependency cycle detected')
    # Parallel group sanity
    groups = {}
    for r in rows:
        groups.setdefault(r.get('parallel_group',''), []).append(r)
    for g, rs in groups.items():
        for r in rs:
            if r.get('can_run_parallel','').lower() == 'true' and int(r.get('max_parallel','1')) < 1:
                errors.append(f'{r["task_id"]} invalid max_parallel in group {g}')
    status = 'passed' if not errors else 'failed'
    report = {'schema_version':'1.0','status':status,'task_count':len(rows),'error_count':len(errors),'warning_count':len(warnings),'errors':errors,'warnings':warnings}
    write_json('generated/task_graph.validation_report.json', report)
    proof = make_proof('EF-005_VALIDATE_TASK_GRAPH', 'completed' if status == 'passed' else 'failed', 'local-execution-framework', 'helper-verify-01', 'gpt-5.3-spark', 'execution-framework', ['execution-framework/generated/task_graph.validation_report.json'], [f'python3 scripts/validate_task_graph.py {graph_arg}'], report, ['generated/task_graph.validation_report.json'], '' if status == 'passed' else '; '.join(errors[:10]), 'run task_graph_to_packets.py' if status == 'passed' else 'fix task graph errors')
    append_proof(proof)
    print(f"validation status={status} tasks={len(rows)} errors={len(errors)} warnings={len(warnings)}")
    if errors:
        for e in errors:
            print('ERROR:', e)
        sys.exit(1)

if __name__ == '__main__':
    main()
