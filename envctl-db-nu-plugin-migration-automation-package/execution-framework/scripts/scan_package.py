
from __future__ import annotations
import argparse
from pathlib import Path
from _common import package_root, root, write_json, sha256_file, now, make_proof, append_proof

FOLDERS = ['specs','prompts','codex','schemas','examples','helpers','sql','source','expected-output','history','execution-framework','execution-templates']

def scan():
    base = package_root()
    top = []
    for child in sorted(base.iterdir()):
        top.append({'name':child.name,'type':'directory' if child.is_dir() else 'file'})
    scanned = {}
    for folder in FOLDERS:
        p = base / folder
        files=[]
        if p.exists():
            for f in sorted(p.rglob('*')):
                if f.is_file():
                    files.append({'path':str(f.relative_to(base)),'size_bytes':f.stat().st_size,'sha256':sha256_file(f)})
        scanned[folder]={'exists':p.exists(),'file_count':len(files),'files':files}
    return {'schema_version':'1.0','generated_at':now(),'package_root':str(base),'top_level_entries':top,'scanned_folders':scanned}

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--gaps', action='store_true')
    args = ap.parse_args()
    obj = scan()
    write_json('generated/package_scan.json', obj)
    if args.gaps:
        # Preserve curated gap report if present; otherwise create a minimal one.
        gap_path = root() / 'generated' / 'gap_report.json'
        if not gap_path.exists():
            write_json('generated/gap_report.json', {'schema_version':'1.0','generated_at':now(),'gaps':[]})
    proof = make_proof('EF-001_SCAN_PACKAGE','completed','local-execution-framework','helper-scan-01','gpt-5.3-spark','execution-framework',['execution-framework/generated/package_scan.json'],['python3 scripts/scan_package.py' + (' --gaps' if args.gaps else '')],{'scanned_folders':list(obj['scanned_folders'].keys())},['generated/package_scan.json'],'','generate/validate task graph')
    append_proof(proof)
    print(f"scanned {len(obj['top_level_entries'])} top-level entries")

if __name__ == '__main__':
    main()
