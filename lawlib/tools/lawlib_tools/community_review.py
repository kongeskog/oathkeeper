"""Check community review journals against exact source, interpretation and coverage artifacts."""
import argparse
import json
from pathlib import Path
import oathkeeper

ROOT=Path(__file__).resolve().parents[2]

def request(operation, **fields):
    return json.loads(oathkeeper.assess_policy(json.dumps({'operation':operation,**fields})))

def build(base):
    package=json.loads((base/'oath-package.json').read_text())
    journal=json.loads((base/'community/reviews.json').read_text())
    trust=json.loads((base/'community/trusted-keys.json').read_text())
    records=[]
    for entry in package['modules']:
        compilation=json.loads(oathkeeper.compile_file(str(base/entry['entrypoint'])))
        artifact={'module_snapshot_sha256':compilation['snapshot_sha256'],
                  'interpretations':json.loads((base/entry['interpretations']).read_text()),
                  'sources':json.loads((base/entry['sources']).read_text())}
        artifact_hash=request('digest',value=artifact)['sha256']
        records.append({'id':entry['id'],'entrypoint':entry['entrypoint'],'module_snapshot_sha256':compilation['snapshot_sha256'],
                        'coverage':compilation['module'].get('coverage',[]),
                        'reviews':request('reviews',artifact_sha256=artifact_hash,journal=journal,trust=trust,authors=['oathkeeper.machine-assisted-author','kongeskog'])})
    return {'format':'oathkeeper.community-review/1','package':package['name'],'package_version':package['version'],
            'repository':package['repository'],'modules':records,
            'review_policy':'Open community review. No professional credential is required. Statements, authorship and claimed independence remain visible; signatures do not establish legal correctness.'}

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--lawlib',type=Path,default=ROOT)
    parser.add_argument('--write',action='store_true')
    parser.add_argument('--output',type=Path)
    args=parser.parse_args(); current=build(args.lawlib)
    path=args.output or args.lawlib/'.build/community/current.json'
    if args.write:
        path.parent.mkdir(parents=True,exist_ok=True)
        path.write_text(json.dumps(current,indent=2)+'\n')
    elif json.loads(path.read_text())!=current:raise ValueError('community review report is stale')
    print(json.dumps({'modules':len(current['modules']),'signed_reviews':sum(len(m['reviews']['current']) for m in current['modules']),'status':'checked'}))
if __name__=='__main__':main()
