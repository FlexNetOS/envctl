#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

from _common import make_proof, append_proof, now, root, write_json


def main() -> int:
    parser = argparse.ArgumentParser(description='Record an agent reviewer approval decision for a task gate.')
    parser.add_argument('task_id')
    parser.add_argument('--reviewer', required=True)
    parser.add_argument('--decision', choices=['approved', 'denied'], required=True)
    parser.add_argument('--review-artifact', required=True)
    parser.add_argument('--model', default='claude-opus')
    parser.add_argument('--summary', required=True)
    parser.add_argument('--instruction-on-denial', default='')
    parser.add_argument('--fallback-reason', default='')
    args = parser.parse_args()

    if not args.reviewer.strip():
        raise SystemExit('reviewer must not be blank')
    if not args.model.strip():
        raise SystemExit('model must not be blank')
    if not args.summary.strip():
        raise SystemExit('summary must not be blank')
    if args.decision == 'denied' and not args.instruction_on_denial.strip():
        raise SystemExit('--instruction-on-denial is required when --decision denied')
    if args.reviewer.lower() not in {'claude-opus-4.8', 'claude-opus', 'opus'} and not args.fallback_reason.strip():
        raise SystemExit('--fallback-reason is required when reviewer is not Claude Opus')

    review_path = Path(args.review_artifact)
    if not review_path.is_absolute():
        review_path = root() / review_path
    if not review_path.exists():
        raise SystemExit(f'review artifact not found: {review_path}')
    try:
        rel_review = str(review_path.relative_to(root()))
    except ValueError as exc:
        raise SystemExit(f'review artifact must be under execution-framework: {review_path}') from exc
    if not rel_review.startswith('reviews/'):
        raise SystemExit(f'review artifact must be under reviews/: {rel_review}')

    record = {
        'schema_version': '1.0',
        'task_id': args.task_id,
        'reviewer': args.reviewer.strip(),
        'model': args.model.strip(),
        'decision': args.decision,
        'reviewed_at': now(),
        'review_artifact': rel_review,
        'summary': args.summary.strip(),
        'instruction_on_denial': args.instruction_on_denial.strip(),
        'fallback_reason': args.fallback_reason.strip(),
        'evidence': [rel_review],
    }
    out = root() / 'approvals' / f'{args.task_id}.agent_approval.json'
    write_json(out, record)

    rel_approval = str(out.relative_to(root()))
    status = 'completed' if args.decision == 'approved' else 'failed'
    proof = make_proof(
        f'APPROVAL-{args.task_id}',
        status,
        'agent-approval-gate',
        'helper-agent-approval-01',
        record['model'],
        'execution-framework',
        [f'execution-framework/{rel_approval}', f'execution-framework/{rel_review}'],
        ['python3 scripts/agent_approval_gate.py ...'],
        {
            'decision': args.decision,
            'reviewer': record['reviewer'],
            'model': record['model'],
            'review_artifact': rel_review,
            'approval_artifact': rel_approval,
            'fallback_reason': record['fallback_reason'],
        },
        [rel_approval, rel_review],
        '' if args.decision == 'approved' else record['instruction_on_denial'],
        f'run goal_loop.py; {args.task_id} becomes dispatchable only when approved',
    )
    append_proof(proof)
    print(f'{args.task_id}: {args.decision} by {args.reviewer}')
    return 0 if args.decision == 'approved' else 1


if __name__ == '__main__':
    raise SystemExit(main())
