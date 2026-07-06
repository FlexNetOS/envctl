#!/usr/bin/env python3
from pathlib import Path
import hashlib, json, sys
root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).resolve().parents[1]
files = []
for p in sorted(root.rglob('*')):
    if not p.is_file():
        continue
    if p.name == 'PACKAGE_MANIFEST.json':
        continue
    if '.git' in p.parts:
        continue
    data = p.read_bytes()
    files.append({'path': str(p.relative_to(root)), 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()})
print(json.dumps({'package': root.name, 'file_count': len(files), 'files': files}, indent=2))
