#!/usr/bin/env python3
from pathlib import Path
import sys

ROOT = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).resolve().parents[1]
ORDER = [
    'prompts/MASTER_PROMPT_ENVCTL_DB_NU_PLUGIN.md',
    'prompts/STRATEGY_DECISION.md',
    'prompts/GAP_ANALYSIS.md',
    'prompts/UTILIZE_FLEXNETOS_PACKAGE.md',
    'prompts/DATABASE_FEATURE_SPEC.md',
    'prompts/NU_PLUGIN_CONTRACT.md',
    'prompts/AGENT_CONTROL_PROTOCOL.md',
    'prompts/LIVE_VISUALS_AND_HUMAN_CONTROL.md',
    'prompts/ANY_TARGET_EXTENSION_SPEC.md',
    'prompts/SECURITY_REPRODUCIBILITY_MODEL.md',
    'prompts/IMPLEMENTATION_PHASES.md',
    'prompts/ACCEPTANCE_CRITERIA.md',
    'prompts/ISSUE_ENVCTL.md',
    'prompts/ISSUE_NU_PLUGIN.md',
    'prompts/ISSUE_SHARED_PROTOCOL.md',
]
for path in sorted((ROOT/'prompts/spark_helpers').glob('*.md')):
    ORDER.append(str(path.relative_to(ROOT)))
ORDER += [
    'source/current-user-request.md',
    'source/previous-migration-artifact-context.md',
    'source/codex-flexnetos-migration-prompt-package/PROMPT_PACKAGE_COMBINED.md',
]

out = []
for rel in ORDER:
    p = ROOT / rel
    if not p.exists():
        raise SystemExit(f'Missing required prompt component: {rel}')
    out.append(f'\n\n---\n\n# FILE: {rel}\n\n')
    out.append(p.read_text(encoding='utf-8'))
print(''.join(out).strip() + '\n')
