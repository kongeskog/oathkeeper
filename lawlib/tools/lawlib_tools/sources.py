"""Fetch and verify content-addressed evidence outside the source checkout.

No network access occurs during compilation. Fetching is an explicit command;
every cache read checks the pinned size and SHA-256 again.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import tempfile
from urllib.parse import urlsplit
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parents[2]
MAX_BYTES = 128 * 1024 * 1024


def cache_root():
    configured = os.environ.get('OATHKEEPER_CACHE_DIR')
    return Path(configured).expanduser() if configured else Path(
        os.environ.get('XDG_CACHE_HOME', Path.home() / '.cache')) / 'oathkeeper'


def validate(source):
    if not re.fullmatch(r'[a-f0-9]{64}', source.get('sha256', '')):
        raise ValueError('source requires a lowercase SHA-256 digest')
    size = source.get('size_bytes')
    if type(size) is not int or size <= 0:
        raise ValueError('source requires a positive byte count')
    name = PurePosixPath(source.get('file', ''))
    if not name.parts or name.is_absolute() or '..' in name.parts or '\\' in str(name):
        raise ValueError('unsafe logical source filename')
    if source.get('media_type') not in {'application/pdf', 'text/plain'}:
        raise ValueError('unsupported evidence media type')
    if not source.get('mirrors'):
        raise ValueError('source requires at least one exact-byte mirror')
    for uri in source['mirrors']:
        url = urlsplit(uri)
        if url.scheme != 'https' or not url.hostname or url.username or url.password or url.fragment:
            raise ValueError('evidence mirrors must be credential-free HTTPS URLs')


def catalog(root=ROOT, modules=None, selected=None):
    manifest = json.loads((root / 'oath-package.json').read_text())
    module_ids = {m['id'] for m in manifest['modules']}
    if modules and set(modules) - module_ids:
        raise ValueError('unknown module: ' + ', '.join(sorted(set(modules) - module_ids)))
    locks = sorted({m['sources'] for m in manifest['modules'] if not modules or m['id'] in modules})
    sources = {}
    for filename in locks:
        lock_path = root / filename
        if not lock_path.resolve().is_relative_to(root.resolve()) or lock_path.is_symlink():
            raise ValueError('source manifest escapes package')
        lock = json.loads(lock_path.read_text())
        if lock.get('format') != 'oathkeeper.sources/2':
            raise ValueError('unsupported source lock; expected oathkeeper.sources/2')
        seen = set()
        for source in lock['sources']:
            validate(source)
            if source['id'] in seen:
                raise ValueError('duplicate source ID in manifest')
            seen.add(source['id'])
            prior = sources.get(source['sha256'])
            if prior and (prior['size_bytes'], prior['media_type']) != (source['size_bytes'], source['media_type']):
                raise ValueError('conflicting metadata for the same evidence digest')
            sources.setdefault(source['sha256'], source)
    if selected:
        unknown = set(selected) - {s['id'] for s in sources.values()} - set(sources)
        if unknown:
            raise ValueError('unknown source: ' + ', '.join(sorted(unknown)))
        sources = {h:s for h,s in sources.items() if h in selected or s['id'] in selected}
    return [sources[h] for h in sorted(sources)]


def object_path(source, cache=None):
    validate(source)
    base = (cache or cache_root()).resolve()
    path = base / 'sha256' / source['sha256'][:2] / source['sha256']
    for item in (path.parent.parent, path.parent, path):
        if item.is_symlink():
            raise ValueError('symlinks are forbidden in the evidence object store')
    return path


def verify_file(path, source):
    validate(source)
    if path.is_symlink() or not path.is_file():
        raise ValueError('evidence object must be a regular file')
    if path.stat().st_size != source['size_bytes']:
        raise ValueError('source bytes changed (size mismatch): ' + source['id'])
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(65536), b''):
            digest.update(block)
    if digest.hexdigest() != source['sha256']:
        raise ValueError('source bytes changed (SHA-256 mismatch): ' + source['id'])
    return path


def require(source, cache=None):
    path = object_path(source, cache)
    if not path.exists():
        raise FileNotFoundError('evidence unavailable offline: ' + source['id'] +
                                '; run python tools/sources.py fetch --source ' + source['id'])
    return verify_file(path, source)


def store_stream(stream, source, cache=None, max_bytes=MAX_BYTES):
    if source['size_bytes'] > max_bytes:
        raise ValueError('evidence exceeds configured download limit')
    target = object_path(source, cache)
    target.parent.mkdir(parents=True, exist_ok=True)
    if target.exists():
        return verify_file(target, source)
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(dir=target.parent, prefix='.partial-', delete=False) as output:
            temporary = Path(output.name)
            remaining = source['size_bytes'] + 1
            while remaining:
                block = stream.read(min(65536, remaining))
                if not block:
                    break
                output.write(block)
                remaining -= len(block)
            output.flush()
            os.fsync(output.fileno())
        verify_file(temporary, source)
        # Check again before replacement; concurrent successful downloads contain
        # the same verified bytes. Never overwrite a poisoned cache silently.
        if target.exists():
            return verify_file(target, source)
        object_path(source, cache)
        os.replace(temporary, target)
        return target
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)


def fetch(source, cache=None, max_bytes=MAX_BYTES):
    path = object_path(source, cache)
    if path.exists():
        return verify_file(path, source)
    if source['size_bytes'] > max_bytes:
        raise ValueError('evidence exceeds configured download limit')
    failures = []
    for uri in source['mirrors']:
        try:
            request = Request(uri, headers={'User-Agent': 'Oathkeeper evidence cache/1', 'Accept-Encoding': 'identity'})
            with urlopen(request, timeout=30) as response:
                if urlsplit(response.geturl()).scheme != 'https':
                    raise ValueError('evidence redirect must retain HTTPS')
                if response.headers.get('Content-Length') and int(response.headers['Content-Length']) != source['size_bytes']:
                    raise ValueError('mirror byte count differs from pinned evidence')
                return store_stream(response, source, cache, max_bytes)
        except (OSError, ValueError) as error:
            failures.append(type(error).__name__)
    raise ValueError('no mirror supplied the pinned evidence for ' + source['id'] + ': ' + ', '.join(failures))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['list', 'fetch', 'verify', 'import'])
    parser.add_argument('--root', type=Path, default=ROOT)
    parser.add_argument('--cache', type=Path)
    parser.add_argument('--module', action='append')
    parser.add_argument('--source', action='append')
    parser.add_argument('--file', type=Path, help='local exact-byte artifact for import; select one source')
    parser.add_argument('--max-bytes', type=int, default=MAX_BYTES)
    args = parser.parse_args()
    sources = catalog(args.root, args.module, args.source)
    if args.action == 'import' and (len(sources) != 1 or not args.file):
        parser.error('import requires --source selecting one object and --file')
    results = []
    for source in sources:
        if args.action == 'fetch':
            path = fetch(source, args.cache, args.max_bytes)
        elif args.action == 'verify':
            path = require(source, args.cache)
        elif args.action == 'import':
            verify_file(args.file, source)
            with args.file.open('rb') as stream:
                path = store_stream(stream, source, args.cache, args.max_bytes)
        else:
            path = object_path(source, args.cache)
        results.append({'id':source['id'], 'sha256':source['sha256'], 'size_bytes':source['size_bytes'], 'cache_path':str(path)})
    print(json.dumps({'action':args.action, 'objects':results}, indent=2))


if __name__ == '__main__':
    main()
