
from __future__ import annotations
import sys, json
from pathlib import Path
from _common import read_task_graph, split_list, write_json, root, make_proof, append_proof, now

ARRAY_FIELDS = {'input_files','target_files','target_artifacts','allowed_paths','blocked_paths','depends_on','blocks','required_tools'}
BOOL_FIELDS = {'can_run_parallel','proof_required','human_approval_required'}
INT_FIELDS = {'max_parallel','priority'}


def convert(row):
    packet = {'packet_schema_version':'1.0'}
    for k,v in row.items():
        if k in ARRAY_FIELDS:
            packet[k] = split_list(v)
        elif k in BOOL_FIELDS:
            packet[k] = str(v).lower() == 'true'
        elif k in INT_FIELDS:
            try: packet[k] = int(v)
            except Exception: packet[k] = 0
        elif k == 'status':
            continue
        else:
            packet[k] = v
    packet['source_graph_uri'] = 'generated/task_graph.csv'
    packet['generated_at'] = now()
    return packet


def main():
    graph_arg = sys.argv[1] if len(sys.argv) > 1 else 'generated/task_graph.csv'
    rows = read_task_graph(graph_arg)
    packet_dir = root() / 'generated' / 'execution_packets'
    packet_dir.mkdir(parents=True, exist_ok=True)
    packets = []
    for row in rows:
        packet = convert(row)
        out = packet_dir / f"{row['task_id']}.json"
        out.write_text(json.dumps(packet, indent=2, sort_keys=False) + '\n', encoding='utf-8')
        packets.append({'task_id':row['task_id'],'packet_uri':f'generated/execution_packets/{row["task_id"]}.json','phase':row['phase'],'owner_lane':row['owner_lane'],'parallel_group':row['parallel_group'],'depends_on':split_list(row.get('depends_on',''))})
    manifest = {'schema_version':'1.0','generated_at':now(),'task_count':len(rows),'packet_count':len(packets),'packets':packets}
    write_json('generated/execution_manifest.json', manifest)
    proof = make_proof('EF-006_TASK_GRAPH_TO_PACKETS','completed','local-execution-framework','helper-packet-01','gpt-5.3-spark','execution-framework',['execution-framework/generated/execution_manifest.json'] + [f'execution-framework/generated/execution_packets/{p["task_id"]}.json' for p in packets[:20]],[f'python3 scripts/task_graph_to_packets.py {graph_arg}'],{'packet_count':len(packets),'manifest':'generated/execution_manifest.json'},['generated/execution_manifest.json','generated/execution_packets/'],'','run goal_loop.py')
    append_proof(proof)
    print(f"generated {len(packets)} execution packets")

if __name__ == '__main__':
    main()
