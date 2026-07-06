#!/usr/bin/env python3
from pathlib import Path
import json, sys, subprocess, sqlite3, tempfile
root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).resolve().parents[1]
required = [
 'README.md','RUN_WITH_CODEX_ENVCTL.sh','INSTALL_IN_REPOS.sh','PROMPT_PACKAGE_COMBINED.md',
 'prompts/MASTER_PROMPT_ENVCTL_DB_NU_PLUGIN.md','prompts/STRATEGY_DECISION.md','prompts/GAP_ANALYSIS.md',
 'prompts/UTILIZE_FLEXNETOS_PACKAGE.md','prompts/DATABASE_FEATURE_SPEC.md','prompts/NU_PLUGIN_CONTRACT.md',
 'schemas/target_descriptor.schema.json','schemas/migration_recipe.schema.json','schemas/run_event.schema.json',
 'sql/001_migration_automation_schema.sql','sql/002_views_and_indexes.sql',
 'source/codex-flexnetos-migration-prompt-package/PROMPT_PACKAGE_COMBINED.md',
]
missing = [p for p in required if not (root/p).exists()]
if missing:
    raise SystemExit('Missing required files:\n' + '\n'.join(missing))
for p in (root/'schemas').glob('*.json'):
    json.loads(p.read_text(encoding='utf-8'))
conn = sqlite3.connect(':memory:')
for rel in ['sql/001_migration_automation_schema.sql','sql/002_views_and_indexes.sql']:
    conn.executescript((root/rel).read_text(encoding='utf-8'))
conn.close()
print('OK: package structure, JSON schemas, and SQLite baseline SQL validated')
