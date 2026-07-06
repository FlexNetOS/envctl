#!/usr/bin/env python3
import sys
from pathlib import Path
from datetime import datetime, timezone

root = Path(sys.argv[1] if len(sys.argv) > 1 else 'migration-artifacts').resolve()
root.mkdir(parents=True, exist_ok=True)
sections = {}
for p in sorted(root.rglob('*.md')):
    rel = p.relative_to(root).as_posix()
    if rel in {'index.md', 'wiki-home.md'} or rel.startswith('_raw/'):
        continue
    top = rel.split('/')[0] if '/' in rel else 'root'
    sections.setdefault(top, []).append(rel)

lines = ['# Migration Artifact Wiki Index', '', f"Generated: `{datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ')}`", '', '## Start here', '', '- [Persistent Memory](MIGRATION_MEMORY.md)', '- [Evidence Register](evidence-register.md)', '- [Artifact Manifest](artifact-manifest.md)', '- [Link Graph](link-graph.md)', '']
for top, paths in sorted(sections.items()):
    lines.append(f'## {top}')
    lines.append('')
    for rel in paths:
        title = Path(rel).stem.replace('-', ' ').title()
        lines.append(f'- [{title}]({rel})')
    lines.append('')
(root / 'index.md').write_text('\n'.join(lines), encoding='utf-8')
(root / 'wiki-home.md').write_text('\n'.join(lines), encoding='utf-8')
print(str(root / 'index.md'))
