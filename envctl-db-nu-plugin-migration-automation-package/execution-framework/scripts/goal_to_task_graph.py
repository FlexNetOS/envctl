
from __future__ import annotations
from _common import root, make_proof, append_proof
from pathlib import Path
import json

def main():
    required = ['generated/task_graph.csv','generated/task_graph.normalized.json','generated/task_graph.index.json']
    missing = [x for x in required if not (root()/x).exists()]
    status = 'failed' if missing else 'completed'
    proof = make_proof('EF-004_GENERATE_TASK_GRAPH', status, 'local-execution-framework','helper-graph-01','gpt-5.3-spark','execution-framework',[f'execution-framework/{x}' for x in required],['python3 scripts/goal_to_task_graph.py'],{'missing':missing,'task_graph_exists':not missing},required,'; '.join(missing),'validate task graph' if not missing else 'regenerate task graph')
    append_proof(proof)
    print('task graph generation proof:', status)
    if missing:
        raise SystemExit(1)
if __name__ == '__main__':
    main()
