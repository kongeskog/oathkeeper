"""Verify a standalone lawlib release with the installed, version-pinned engine."""
import argparse
import hashlib
import json
from pathlib import Path
import oathkeeper
import sys
sys.dont_write_bytecode = True
from community_review import build

ROOT=Path(__file__).resolve().parents[1]

def check_splits(data):
    seen={}
    for split in ('train','development','held_out'):
        for entry in data.get(split,[]):
            for kind in ('matter','source_family','amendment_lineage'):
                group=entry[kind]
                if not isinstance(group,str) or not group.strip():raise ValueError('evaluation grouping must be explicit')
                key=(kind,group)
                if key in seen and seen[key]!=split:raise ValueError('evaluation leakage across splits: '+kind+' '+group)
                seen[key]=split

def check():
    manifest=json.loads((ROOT/'oath-package.json').read_text())
    if oathkeeper.__version__!=manifest['engine_version']:raise ValueError('use the exact engine version pinned by oath-package.json')
    sources=0;cases=0
    for lock in ROOT.rglob('sources.lock.json'):
        for entry in json.loads(lock.read_text())['sources']:
            path=(lock.parent/entry['file']).resolve()
            if not path.is_relative_to(ROOT) or path.is_symlink():raise ValueError('source path escapes package')
            if hashlib.sha256(path.read_bytes()).hexdigest()!=entry['sha256']:raise ValueError('source bytes changed: '+entry['file'])
            sources+=1
    pending={'long-oracle-reassessment.case.oath','long-parking-evidence-review.case.oath'}
    for path in ROOT.rglob('*.oath'):
        c=json.loads(oathkeeper.compile_file(str(path)))
        if c['case'] is not None:
            if path.name in pending:
                result=json.loads(oathkeeper.analyze_file(str(path)))
                if result['status']!='neither':raise ValueError('incomplete record unexpectedly resolved')
            else:
                json.loads(oathkeeper.verify_file(str(path)));cases+=1
            target=path.with_suffix('.json')
            if target.exists() and json.loads(target.read_text())!=c['case']:raise ValueError('stale compiled case: '+str(target))
        else:
            target=path.with_suffix('.module.json')
            if target.exists() and json.loads(target.read_text())!=c['module']:raise ValueError('stale compiled module: '+str(target))
    current=build(ROOT)
    if json.loads((ROOT/'community/current.json').read_text())!=current:raise ValueError('stale community review report')
    check_splits(json.loads((ROOT/'evaluation-groups.json').read_text()))
    return {'sources':sources,'historical_proof_paths':cases,'signed_reviews':sum(len(m['reviews']['current']) for m in current['modules']),
            'held_out_legal_accuracy':'not_measured','status':'engineering_checks_passed'}
if __name__=='__main__':
    print(json.dumps(check(),indent=2))
