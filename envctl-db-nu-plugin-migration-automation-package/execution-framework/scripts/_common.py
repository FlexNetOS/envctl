
from __future__ import annotations
import csv, json, hashlib, os, sys, time
from pathlib import Path
from datetime import datetime, timezone

TASK_COLUMNS = ['task_id', 'parent_id', 'phase', 'title', 'goal', 'owner_lane', 'owner_agent', 'helper_id', 'model_tag', 'agent_runtime', 'shell_mode', 'repo_target', 'repo_path', 'filesystem_scope', 'input_files', 'target_files', 'target_artifacts', 'allowed_paths', 'blocked_paths', 'depends_on', 'blocks', 'parallel_group', 'can_run_parallel', 'max_parallel', 'start_after', 'priority', 'status', 'execution_cell', 'required_tools', 'command_template', 'verification_command', 'completion_gate', 'proof_required', 'proof_uri', 'heartbeat_file', 'logs_uri', 'rollback_plan', 'risk_level', 'human_approval_required', 'notes']
PACKET_EXTRA_FIELDS = ['packet_schema_version','source_graph_uri','generated_at']
PROOF_FIELDS = ['proof_schema_version','task_id','status','started_at','completed_at','actor','helper_id','model_tag','repo_path','files_changed','commands_run','verification_output','checksums','logs_uri','rollback_point','evidence','failure_reason','next_action']

def now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat()

def root() -> Path:
    return Path(__file__).resolve().parents[1]

def package_root() -> Path:
    return root().parent

def read_task_graph(path: str | Path):
    p = Path(path)
    if not p.is_absolute():
        p = root() / p
    with p.open(newline='', encoding='utf-8') as f:
        rows = list(csv.DictReader(f))
    return rows

def write_json(path: str | Path, obj):
    p = Path(path)
    if not p.is_absolute():
        p = root() / p
    p.parent.mkdir(parents=True, exist_ok=True)
    with p.open('w', encoding='utf-8') as f:
        json.dump(obj, f, indent=2, sort_keys=False)
        f.write('\n')

def read_json(path: str | Path):
    p = Path(path)
    if not p.is_absolute():
        p = root() / p
    return json.loads(p.read_text(encoding='utf-8'))

def split_list(value: str):
    if value is None:
        return []
    value = str(value).strip()
    if not value:
        return []
    return [x.strip() for x in value.split('|') if x.strip()]

def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open('rb') as f:
        for chunk in iter(lambda: f.read(1024*1024), b''):
            h.update(chunk)
    return h.hexdigest()

def file_checksums(paths):
    out = {}
    base = package_root()
    for raw in paths:
        p = Path(raw)
        if not p.is_absolute():
            p = base / raw
        if p.exists() and p.is_file():
            try:
                out[str(p.relative_to(base))] = sha256_file(p)
            except ValueError:
                out[str(p)] = sha256_file(p)
    return out

def load_ledger():
    ledger = root() / 'proof_records' / 'proof_ledger.jsonl'
    records = []
    if ledger.exists():
        for line in ledger.read_text(encoding='utf-8').splitlines():
            line = line.strip()
            if not line:
                continue
            try:
                records.append(json.loads(line))
            except json.JSONDecodeError:
                records.append({'task_id':'__LEDGER_PARSE_ERROR__','status':'failed','failure_reason':'Invalid JSONL line'})
    return records

def append_proof(record: dict):
    pr = root() / 'proof_records'
    pr.mkdir(parents=True, exist_ok=True)
    proof_path = pr / f"{record['task_id']}.proof.json"
    proof_path.write_text(json.dumps(record, indent=2, sort_keys=False) + '\n', encoding='utf-8')
    ledger = pr / 'proof_ledger.jsonl'
    # Avoid duplicate task IDs by rebuilding ledger with latest record for task.
    records = [r for r in load_ledger() if r.get('task_id') != record.get('task_id')]
    records.append(record)
    ledger.write_text(''.join(json.dumps(r, sort_keys=False) + '\n' for r in records), encoding='utf-8')
    return proof_path

def make_proof(task_id: str, status: str, actor: str, helper_id: str, model_tag: str, repo_path: str, files_changed: list[str], commands_run: list[str], verification_output, evidence: list[str], failure_reason: str = '', next_action: str = ''):
    ts = now()
    return {
        'proof_schema_version':'1.0',
        'task_id':task_id,
        'status':status,
        'started_at':ts,
        'completed_at':ts,
        'actor':actor,
        'helper_id':helper_id,
        'model_tag':model_tag,
        'repo_path':repo_path,
        'files_changed':files_changed,
        'commands_run':commands_run,
        'verification_output':verification_output,
        'checksums':file_checksums(files_changed),
        'logs_uri':f'logs/{task_id}.log',
        'rollback_point':'history/pre_execution_framework_manifest.json',
        'evidence':evidence,
        'failure_reason':failure_reason,
        'next_action':next_action,
    }

def task_lookup(rows):
    return {r['task_id']: r for r in rows}
