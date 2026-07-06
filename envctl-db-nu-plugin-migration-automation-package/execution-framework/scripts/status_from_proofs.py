
from __future__ import annotations
from _common import read_task_graph, load_ledger, write_json, now

def main():
    rows = read_task_graph('generated/task_graph.csv')
    proofs = {p.get('task_id'): p for p in load_ledger()}
    status = []
    for r in rows:
        p = proofs.get(r['task_id'])
        status.append({'task_id':r['task_id'],'phase':r['phase'],'owner_lane':r['owner_lane'],'status':p.get('status','pending') if p else 'pending','proof_uri':r['proof_uri']})
    write_json('generated/status_from_proofs.json', {'schema_version':'1.0','generated_at':now(),'tasks':status})
    print(f'wrote status for {len(status)} tasks')
if __name__ == '__main__':
    main()
