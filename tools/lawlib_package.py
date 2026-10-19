"""Fetch an exact lawlib dependency into ignored .deps/, or verify it offline.

Only the lockfile belongs in the software repository. Fetching never executes
package code and never follows a moving branch or tag. Pinning a new commit is
an explicit operation separate from installing the existing lock.
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
from urllib.parse import urlsplit

ROOT = Path(__file__).resolve().parents[1]
LOCK = ROOT / 'lawlib.lock.json'
DEST = ROOT / '.deps/lawlib'
IGNORED = {'.build', '.cache', '__pycache__'}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def safe_path(name):
    p = PurePosixPath(name)
    if not p.parts or p.is_absolute() or '..' in p.parts or any(part in {'.git', '.venv', 'node_modules', *IGNORED} for part in p.parts) or '\\' in name:
        raise ValueError('unsafe package path: ' + name)
    return p


def inventory(directory):
    if directory.is_symlink():
        raise ValueError('symlinks are forbidden in a lawlib snapshot')
    files = {}
    for path in sorted(directory.rglob('*')):
        if any(part in IGNORED for part in path.relative_to(directory).parts):
            continue
        if path.is_symlink():
            raise ValueError('symlinks are forbidden in a lawlib snapshot')
        if path.is_file():
            files[path.relative_to(directory).as_posix()] = digest(path.read_bytes())
    return files


def validate_lock(lock):
    if lock.get('format') != 'oathkeeper.lawlib-lock/1' or not re.fullmatch('[a-f0-9]{40}', lock.get('revision', '')):
        raise ValueError('lock requires an exact 40-character commit SHA; branches and tags are not locks')
    url = urlsplit(lock.get('repository', ''))
    if url.scheme != 'https' or not url.hostname or url.username or url.password or url.query or url.fragment:
        raise ValueError('lawlib repository must be a credential-free HTTPS URL')
    if not isinstance(lock.get('files'), dict) or 'oath-package.json' not in lock['files']:
        raise ValueError('lock requires a complete authored file inventory')
    for name, sha in lock['files'].items():
        safe_path(name)
        if not isinstance(sha, str) or not re.fullmatch('[a-f0-9]{64}', sha):
            raise ValueError('invalid package file SHA-256: ' + name)
    return lock


def read_lock():
    return validate_lock(json.loads(LOCK.read_text()))


def verify_snapshot(directory, lock):
    if not directory.is_dir():
        raise ValueError('lawlib dependency is missing; run python tools/lawlib_package.py sync')
    actual = inventory(directory)
    if actual != lock['files']:
        changed = sorted(k for k in set(actual) | set(lock['files']) if actual.get(k) != lock['files'].get(k))
        raise ValueError('lawlib dependency differs from its immutable lock: ' + ', '.join(changed))
    manifest = json.loads((directory / 'oath-package.json').read_text())
    if manifest.get('format') != 'oathkeeper.law-package/1' or manifest['version'] != lock['package_version'] or manifest['repository'] != lock['repository']:
        raise ValueError('package identity/version mismatch')
    return {'status': 'verified', 'revision': lock['revision'], 'files': len(actual), 'repository': lock['repository']}


def verify():
    return verify_snapshot(DEST, read_lock())


def unpack(checkout, revision, destination):
    if not re.fullmatch('[a-f0-9]{40}', revision):
        raise ValueError('supply an exact 40-character commit SHA; branches and tags are not locks')
    archive = subprocess.check_output(['git', '-C', str(checkout), 'archive', '--format=tar', revision])
    if len(archive) > 64 * 1024 * 1024:
        raise ValueError('package archive exceeds 64 MiB')
    destination.mkdir()
    with tarfile.open(fileobj=io.BytesIO(archive), mode='r:') as tar:
        members = tar.getmembers()
        if len(members) > 10000:
            raise ValueError('package file count exceeds limit')
        seen = set()
        for member in members:
            p = safe_path(member.name)
            if member.name in seen:
                raise ValueError('duplicate package path')
            seen.add(member.name)
            if member.isdir():
                continue
            if not member.isfile() or member.size > 32 * 1024 * 1024:
                raise ValueError('only bounded regular package files are allowed')
            target = destination.joinpath(*p.parts)
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(tar.extractfile(member).read())


def install(snapshot, lock):
    """Verify staging before replacing the ignored dependency, with rollback."""
    verify_snapshot(snapshot, lock)
    if DEST.is_symlink() or DEST.parent.is_symlink():
        raise ValueError('dependency directory cannot be a symlink')
    DEST.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='.lawlib-install-', dir=DEST.parent) as tmp:
        staged = Path(tmp) / 'new'
        backup = Path(tmp) / 'previous'
        shutil.copytree(snapshot, staged)
        if DEST.exists():
            DEST.rename(backup)
        try:
            staged.rename(DEST)
        except OSError:
            if backup.exists():
                backup.rename(DEST)
            raise
    return verify_snapshot(DEST, lock)


def sync(checkout=None):
    """Install the existing lock; a verified local dependency needs no network."""
    lock = read_lock()
    if DEST.exists() or DEST.is_symlink():
        return verify_snapshot(DEST, lock)
    with tempfile.TemporaryDirectory(prefix='oathkeeper-lawlib-') as tmp:
        if checkout is None:
            checkout = Path(tmp) / 'git'
            subprocess.run(['git', 'init', '--bare', '-q', str(checkout)], check=True)
            subprocess.run(['git', '-C', str(checkout), 'remote', 'add', 'origin', lock['repository']], check=True)
            subprocess.run(['git', '-C', str(checkout), 'fetch', '--quiet', '--no-tags', '--depth=1', 'origin', lock['revision']], check=True)
            actual = subprocess.check_output(['git', '-C', str(checkout), 'rev-parse', 'FETCH_HEAD'], text=True).strip()
            if actual != lock['revision']:
                raise ValueError('fetched revision does not match lock')
        snapshot = Path(tmp) / 'snapshot'
        unpack(checkout, lock['revision'], snapshot)
        return install(snapshot, lock)


def pin(checkout, revision):
    """Adopt a reviewed commit, update the lock and prepare its local dependency."""
    with tempfile.TemporaryDirectory(prefix='oathkeeper-lawlib-') as tmp:
        snapshot = Path(tmp) / 'snapshot'
        unpack(checkout, revision, snapshot)
        manifest = json.loads((snapshot / 'oath-package.json').read_text())
        lock = validate_lock({'format': 'oathkeeper.lawlib-lock/1', 'repository': manifest['repository'], 'revision': revision,
                             'package_version': manifest['version'], 'files': inventory(snapshot)})
        result = install(snapshot, lock)
        LOCK.write_text(json.dumps(lock, indent=2) + '\n')
        return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='action', required=True)
    sub.add_parser('verify', help='verify the installed dependency without network')
    prepare = sub.add_parser('sync', help='install the exact existing lock into ignored .deps/lawlib/')
    prepare.add_argument('--from-checkout', type=Path, help='use an existing Git checkout for offline installation')
    update = sub.add_parser('pin', help='explicitly update the lock to a reviewed commit')
    update.add_argument('--from-checkout', type=Path, required=True)
    update.add_argument('--revision', required=True)
    args = parser.parse_args()
    if args.action == 'verify':
        result = verify()
    elif args.action == 'sync':
        result = sync(args.from_checkout)
    else:
        result = pin(args.from_checkout, args.revision)
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
