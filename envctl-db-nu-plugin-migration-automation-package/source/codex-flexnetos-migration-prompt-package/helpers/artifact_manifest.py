#!/usr/bin/env python3
import json, sys, hashlib
from pathlib import Path
from datetime import datetime, timezone

root = Path(sys.argv[1] if len(sys.argv) > 1 else 'migration-artifacts').resolve()
root.mkdir(parents=True, exist_ok=True)
items = []
for p in sorted(root.rglob('*')):
    if p.is_file():
        rel = p.relative_to(root).as_posix()
        if rel.startswith('_raw/') and p.stat().st_size > 2_000_000:
            sha = None
        else:
            h = hashlib.sha256()
            try:
                with p.open('rb') as f:
                    for chunk in iter(lambda: f.read(1024 * 1024), b''):
                        h.update(chunk)
                sha = h.hexdigest()
            except OSError:
                sha = None
        status = 'unknown'
        try:
            text = p.read_text(encoding='utf-8', errors='ignore')[:4000]
            for marker in ['status: complete', 'status: partial', 'status: unknown', 'status: blocked']:
                if marker in text:
                    status = marker.split(': ',1)[1]
                    break
        except Exception:
            pass
        items.append({'path': rel, 'bytes': p.stat().st_size, 'sha256': sha, 'status': status})
manifest = {
    'generated_at_utc': datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ'),
    'root': str(root),
    'artifact_count': len(items),
    'artifacts': items,
}
(root / 'artifact-manifest.json').write_text(json.dumps(manifest, indent=2, sort_keys=True) + '\n', encoding='utf-8')
md = ['# Artifact Manifest', '', f"Generated: `{manifest['generated_at_utc']}`", '', '| Path | Status | Bytes | SHA-256 |', '|---|---:|---:|---|']
for item in items:
    link = item['path'].replace(' ', '%20')
    sha = item['sha256'] or ''
    md.append(f"| [{item['path']}]({link}) | {item['status']} | {item['bytes']} | `{sha[:16]}` |")
(root / 'artifact-manifest.md').write_text('\n'.join(md) + '\n', encoding='utf-8')
print(str(root / 'artifact-manifest.json'))
