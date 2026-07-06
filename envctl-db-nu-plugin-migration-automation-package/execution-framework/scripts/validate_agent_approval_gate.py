#!/usr/bin/env python3
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

from _common import load_ledger, now, root, write_json
from goal_loop import load_agent_approval


TASK_ID = '__APPROVAL_GATE_SELF_TEST__'


def backup(path: Path) -> bytes | None:
    return path.read_bytes() if path.exists() else None


def restore(path: Path, data: bytes | None) -> None:
    if data is None:
        path.unlink(missing_ok=True)
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)


def assert_invalid(label: str, expected: str) -> None:
    proofs = {p.get('task_id'): p for p in load_ledger() if p.get('task_id')}
    result = load_agent_approval(TASK_ID, proofs)
    if not result or result.get('valid'):
        raise AssertionError(f'{label}: malformed approval unexpectedly accepted')
    reason = result.get('reason', '')
    if expected not in reason:
        raise AssertionError(f'{label}: expected reason containing {expected!r}, got {reason!r}')
    print(f'PASS {label}: {reason}')


def full_record(review_artifact: str) -> dict:
    return {
        'schema_version': '1.0',
        'task_id': TASK_ID,
        'reviewer': 'gpt-5.5',
        'model': 'gpt-5.5',
        'decision': 'approved',
        'reviewed_at': now(),
        'review_artifact': review_artifact,
        'summary': 'self-test approval record that must fail closed',
        'instruction_on_denial': '',
        'fallback_reason': 'self-test fallback marker',
        'evidence': [review_artifact],
    }


def main() -> int:
    approval_path = root() / 'approvals' / f'{TASK_ID}.agent_approval.json'
    proof_path = root() / 'proof_records' / f'APPROVAL-{TASK_ID}.proof.json'
    ledger_path = root() / 'proof_records' / 'proof_ledger.jsonl'
    review_path = root() / 'reviews' / 'GPT55_APPROVAL_REVIEW.md'
    if not review_path.exists():
        review_path = root() / 'reviews' / 'APPROVAL_GATE_REVIEW_REQUEST.md'
    if not review_path.exists():
        raise SystemExit('no review artifact available for approval-gate validation')
    rel_review = str(review_path.relative_to(root()))

    original_approval = backup(approval_path)
    original_proof = backup(proof_path)
    original_ledger = backup(ledger_path)
    try:
        write_json(approval_path, {'decision': 'approved', 'reviewer': 'gpt-5.5', 'evidence': [rel_review]})
        assert_invalid('minimal hand-written approval', 'missing required fields')

        record = full_record('reviews/DOES_NOT_EXIST.md')
        write_json(approval_path, record)
        assert_invalid('missing review artifact', 'review artifact missing')

        record = full_record(rel_review)
        write_json(approval_path, record)
        restore(proof_path, None)
        assert_invalid('missing approval proof', f'matching proof record missing: APPROVAL-{TASK_ID}')

        denied = subprocess.run(
            [
                sys.executable,
                str(root() / 'scripts' / 'agent_approval_gate.py'),
                TASK_ID,
                '--reviewer',
                'gpt-5.5',
                '--decision',
                'denied',
                '--review-artifact',
                rel_review,
                '--model',
                'gpt-5.5',
                '--summary',
                'self-test denied approval',
                '--fallback-reason',
                'Claude Opus unavailable during self-test',
            ],
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            check=False,
        )
        if denied.returncode == 0:
            raise AssertionError('blank denial instructions unexpectedly accepted')
        if '--instruction-on-denial is required' not in denied.stdout:
            raise AssertionError(f'blank denial instructions failed for wrong reason: {denied.stdout!r}')
        print('PASS blank denial instructions: rejected')
    finally:
        restore(approval_path, original_approval)
        restore(proof_path, original_proof)
        restore(ledger_path, original_ledger)

    print('agent approval gate validation passed')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
