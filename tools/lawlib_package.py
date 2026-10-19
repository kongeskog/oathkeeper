"""Vendor an immutable lawlib Git commit, or verify the existing offline snapshot.

Never execute files from the package. Reject links, traversal, unlisted files, and
changed bytes. The lock pins a Git commit AND a complete SHA-256 file inventory.
"""
import argparse
import hashlib
import io
import json
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import tarfile
import tempfile

ROOT = Path(__file__).resolve().parents[1]
LOCK = ROOT / 'lawlib.lock.json'
DEST = ROOT / 'lawlib'

def digest(data):
    return hashlib.sha256(data).hexdigest()

def inventory(directory):
    files = {}
    for path in sorted(directory.rglob('*')):
        if any(part in {'.build', '.cache', '__pycache__'} for part in path.relative_to(directory).parts):
            continue
        if path.is_symlink():
            raise ValueError('symlinks are forbidden in a lawlib snapshot')
        if path.is_file():
            files[path.relative_to(directory).as_posix()] = digest(path.read_bytes())
    return files

def verify():
    lock = json.loads(LOCK.read_text())
    if lock['format'] != 'oathkeeper.lawlib-lock/1' or not re.fullmatch('[a-f0-9]{40}', lock['revision']):
        raise ValueError('invalid package lock')
    actual = inventory(DEST)
    if actual != lock['files']:
        changed = sorted(k for k in set(actual) | set(lock['files']) if actual.get(k) != lock['files'].get(k))
        raise ValueError('lawlib snapshot differs from its immutable lock: ' + ', '.join(changed))
    manifest = json.loads((DEST / 'oath-package.json').read_text())
    if manifest['version'] != lock['package_version']:
        raise ValueError('package version mismatch')
    return {'status': 'verified', 'revision': lock['revision'], 'files': len(actual), 'repository': lock['repository']}

def vendor(checkout, revision):
    if not re.fullmatch('[a-f0-9]{40}', revision):
        raise ValueError('supply an exact 40-character commit SHA; branches and tags are not locks')
    archive = subprocess.check_output(['git', '-C', str(checkout), 'archive', '--format=tar', revision])
    if len(archive) > 64 * 1024 * 1024:
        raise ValueError('package archive exceeds 64 MiB')
    with tempfile.TemporaryDirectory(prefix='oathkeeper-lawlib-') as temp:
        unpacked = Path(temp) / 'snapshot'
        unpacked.mkdir()
        with tarfile.open(fileobj=io.BytesIO(archive), mode='r:') as tar:
            members = tar.getmembers()
            if len(members) > 10000:
                raise ValueError('package file count exceeds limit')
            for member in members:
                p = PurePosixPath(member.name)
                if p.is_absolute() or '..' in p.parts or any(part in {'.git', '.venv', '.build', '.cache', 'node_modules', '__pycache__'} for part in p.parts) or '\\' in member.name:
                    raise ValueError('unsafe package path')
                if member.isdir():
                    continue
                if not member.isfile() or member.size > 32 * 1024 * 1024:
                    raise ValueError('only bounded regular package files are allowed')
                target = unpacked.joinpath(*p.parts)
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(tar.extractfile(member).read())
        manifest = json.loads((unpacked / 'oath-package.json').read_text())
        if manifest['format'] != 'oathkeeper.law-package/1':
            raise ValueError('unknown package format')
        lock = {'format': 'oathkeeper.lawlib-lock/1', 'repository': manifest['repository'], 'revision': revision,
                'package_version': manifest['version'], 'files': inventory(unpacked)}
        # The caller explicitly requests replacing this generated vendored directory.
        if DEST.exists():
            shutil.rmtree(DEST)
        shutil.copytree(unpacked, DEST)
        LOCK.write_text(json.dumps(lock, indent=2) + '\n')
    return verify()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='action', required=True)
    sub.add_parser('verify')
    update = sub.add_parser('vendor')
    update.add_argument('--from-checkout', type=Path, required=True)
    update.add_argument('--revision', required=True)
    args = parser.parse_args()
    print(json.dumps(verify() if args.action == 'verify' else vendor(args.from_checkout, args.revision), indent=2))

if __name__ == '__main__':
    main()
