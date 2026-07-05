
from __future__ import annotations
from _common import load_ledger, write_json, now

def main():
    records = load_ledger()
    by_task = {r.get('task_id'): r for r in records if r.get('task_id')}
    write_json('generated/proof_ledger.merged.json', {'schema_version':'1.0','generated_at':now(),'proof_count':len(by_task),'proofs':list(by_task.values())})
    print(f'merged {len(by_task)} proof records')
if __name__ == '__main__':
    main()
