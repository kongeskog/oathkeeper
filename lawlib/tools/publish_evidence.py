"""Publish verified evidence objects without ever overwriting an existing asset.

Normally imports from the local cache. --from-revision bootstraps the original
tracked snapshots from an exact historical commit during this migration.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
from lawlib_tools import sources


def github(repository, tag):
    # The by-tag endpoint does not return an unpublished draft. Enumerate
    # authorized releases so interrupted uploads can resume that same draft.
    result = subprocess.run(['gh','api','--paginate','--slurp',f'repos/{repository}/releases?per_page=100'],text=True,capture_output=True)
    if result.returncode:
        raise RuntimeError('GitHub release lookup failed: ' + result.stderr)
    matches=[release for page in json.loads(result.stdout) for release in page if release['tag_name']==tag]
    if len(matches)>1:
        raise ValueError('multiple releases have this tag; resolve duplicate drafts before publishing')
    return matches[0] if matches else None


def read_release(repository, release_id):
    return json.loads(subprocess.check_output(['gh','api',f'repos/{repository}/releases/{release_id}'],text=True))


def create_release(repository, tag, target, title, notes):
    # Use the creation response directly: list/by-tag reads may not yet expose
    # a just-created draft. Its numeric ID is immediately usable for uploads.
    payload={'tag_name':tag,'target_commitish':target,'draft':True,'name':title,'body':notes}
    return json.loads(subprocess.check_output(
        ['gh','api','--method','POST',f'repos/{repository}/releases','--input','-'],
        input=json.dumps(payload),text=True))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--tag',required=True)
    parser.add_argument('--from-revision',help='exact historical commit containing the original source documents')
    args = parser.parse_args()
    if not re.fullmatch('[A-Za-z0-9][A-Za-z0-9._-]{0,79}',args.tag):
        parser.error('invalid evidence release tag')
    root=sources.ROOT
    package=json.loads((root/'oath-package.json').read_text())
    repository=package['repository'].removeprefix('https://github.com/')
    if not re.fullmatch('[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+',repository):
        parser.error('expected a GitHub repository')
    evidence=sources.catalog(root)
    if args.from_revision:
        if not re.fullmatch('[a-f0-9]{40}',args.from_revision):
            parser.error('--from-revision requires a full commit SHA')
        subprocess.run(['git','-C',str(root),'fetch','--depth=1','origin',args.from_revision],check=True)
        paths={}
        for module in package['modules']:
            lock=root/module['sources']
            for entry in json.loads(lock.read_text())['sources']:
                paths[entry['sha256']]=(lock.parent/entry['file']).relative_to(root).as_posix()
        for source in evidence:
            data=subprocess.check_output(['git','-C',str(root),'show',args.from_revision+':'+paths[source['sha256']]])
            import io
            sources.store_stream(io.BytesIO(data),source)
    for source in evidence:
        sources.require(source)
    release=github(repository,args.tag)
    if release is None:
        target=subprocess.check_output(['git','-C',str(root),'rev-parse','HEAD'],text=True).strip()
        release=create_release(repository,args.tag,target,'Evidence archive: '+args.tag,
            'Exact-byte source snapshots named by SHA-256. Original authority, retrieval metadata and rights are recorded in sources.lock.json. Never replace an asset to represent a new edition.')
    existing={asset['name']:asset for asset in release['assets']}
    with tempfile.TemporaryDirectory(prefix='oathkeeper-evidence-publish-') as temp:
        for source in evidence:
            name=source['sha256']+Path(source['file']).suffix
            if name in existing:
                asset=existing[name]
                if asset['size'] != source['size_bytes'] or asset.get('digest') != 'sha256:'+source['sha256']:
                    raise ValueError('existing evidence asset differs; refusing to overwrite: '+name)
                continue
            path=Path(temp)/name
            path.write_bytes(sources.require(source).read_bytes())
            subprocess.run(['gh','release','upload',args.tag,str(path),'--repo',repository],check=True)
    release=read_release(repository,release['id'])
    existing={asset['name']:asset for asset in release['assets']}
    for source in evidence:
        name=source['sha256']+Path(source['file']).suffix
        asset=existing[name]
        if asset['size'] != source['size_bytes'] or asset.get('digest') != 'sha256:'+source['sha256']:
            raise ValueError('uploaded asset verification failed: '+name)
    if release['draft']:
        subprocess.run(['gh','release','edit',args.tag,'--repo',repository,'--draft=false','--latest=false'],check=True)
        release=read_release(repository,release['id'])
    print(json.dumps({'status':'published','objects':len(evidence),'release':release['html_url']}))


if __name__=='__main__':main()
