"""Reproduce compiler artifacts without writing into the authored tree."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path
import shutil
import tarfile
import tempfile

import oathkeeper
from . import audit_tax, benchmark, community_review, review_packet, sources
from .runtime import ROOT, check_layout, combine, compile_source, input_files, inventory, write_json

MANIFEST = 'build-manifest.json'


def generate(root, output):
    check_layout(root)
    manifest = json.loads((root / 'oath-package.json').read_text())
    if oathkeeper.__version__ != manifest['engine_version']:
        raise ValueError('install the exact engine_version from oath-package.json')
    evidence = sources.catalog(root)  # Validate metadata; compilation never fetches evidence.
    seattle = root / 'us/wa/seattle'
    coverage = json.loads((seattle / 'coverage.json').read_text())
    unresolved = {seattle / n for n in coverage['pending_cases'] + coverage['assessment_cases']}
    compiled = 0
    for path in input_files(root):
        if path.suffix != '.oath':
            continue
        result = compile_source(path)
        relative = path.relative_to(root)
        compiled += 1
        if result['case'] is None:
            write_json((output / relative).with_suffix('.module.json'), result['module'])
        else:
            write_json((output / relative).with_suffix('.json'), result['case'])
            stem = str(relative).removesuffix('.case.oath')
            bundle = combine(result)
            write_json(output / (stem + '.bundle.json'), bundle)
            if path in unresolved:
                report = json.loads(oathkeeper.analyze_file(str(path)))
                if report['status'] != 'neither':
                    raise ValueError('incomplete record unexpectedly resolved: ' + str(relative))
                write_json(output / (stem + '.analysis.json'), report)
            else:
                write_json(output / (stem + '.report.json'), json.loads(oathkeeper.verify_file(str(path))))
        write_json((output / relative).with_suffix('.imports.json'),
                   {'format':'oathkeeper-import-lock/1', 'snapshot_sha256':result['snapshot_sha256'], 'sources':result['lock']})
    c = compile_source(seattle / 'parking.oath')
    write_json(output / 'us/wa/seattle/oath.lock.json',
               {'format':'oathkeeper-import-lock/1','snapshot_sha256':c['snapshot_sha256'],'sources':c['lock']})
    packet = review_packet.packet(seattle)
    packet['reviews'] = review_packet.validate_reviews(json.loads((seattle / 'review.json').read_text()), packet)
    write_json(output / 'us/wa/seattle/review-packet.json', packet)
    baseline = benchmark.evaluate(seattle)
    if baseline['passed'] != baseline['total']:
        raise ValueError('engineering baseline failed')
    write_json(output / 'us/wa/seattle/baseline.json', baseline)
    audit_tax.audit(True, base=root / 'us/tax', output=output / 'us/tax', evidence=False)
    write_json(output / 'community/current.json', community_review.build(root))
    report = {'format':'oathkeeper.build/1', 'package':manifest['name'], 'package_version':manifest['version'],
              'engine_version':oathkeeper.__version__, 'inputs':inventory(root), 'files':inventory(output),
              'evidence_sha256':sorted(s['sha256'] for s in evidence), 'evidence_bytes':'not_checked_by_compilation',
              'compiled_sources':compiled, 'timings':'omitted for reproducibility'}
    write_json(output / MANIFEST, report)
    return report


def build(root=ROOT, output=None, *, check=False):
    root = root.resolve()
    output = (output or root / '.build').absolute()
    if output.is_symlink():
        raise ValueError('build output cannot be a symlink')
    output = output.resolve()
    if output == root or root.is_relative_to(output) or (output.is_relative_to(root) and output.relative_to(root).parts[0] != '.build'):
        raise ValueError('output inside the source checkout must be under .build/')
    if output.exists() and any(output.iterdir()) and not (output / MANIFEST).is_file():
        raise ValueError('refusing to replace a directory without an Oathkeeper build manifest')
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='oathkeeper-build-') as temp:
        staging = Path(temp) / 'artifacts'
        staging.mkdir()
        report = generate(root, staging)
        if check:
            if inventory(output) != inventory(staging):
                raise ValueError('generated artifacts are stale; run python tools/build.py')
        else:
            if output.exists():
                shutil.rmtree(output)
            shutil.copytree(staging, output)
    return report


def archive(output, destination):
    """Byte-reproducible build archive; timestamps and host ownership excluded."""
    if destination.resolve().is_relative_to(output.resolve()):
        raise ValueError('release archive must be outside the generated directory')
    with destination.open('wb') as raw:
        with gzip.GzipFile(fileobj=raw, mode='wb', filename='', mtime=0) as compressed:
            with tarfile.open(fileobj=compressed, mode='w') as tar:
                for path in sorted(output.rglob('*')):
                    if not path.is_file() or path.is_symlink():
                        continue
                    info = tar.gettarinfo(str(path), arcname=path.relative_to(output).as_posix())
                    info.mtime = 0
                    info.uid = info.gid = 0
                    info.uname = info.gname = ''
                    info.mode = 0o644
                    with path.open('rb') as stream:
                        tar.addfile(info, stream)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=ROOT)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--check', action='store_true', help='rebuild and compare every generated byte')
    parser.add_argument('--archive', type=Path, help='optional reproducible .tar.gz release asset')
    args = parser.parse_args()
    output = args.output or args.root / '.build'
    report = build(args.root, output, check=args.check)
    if args.archive:
        archive(output, args.archive)
    print(json.dumps({'status':'reproduced' if args.check else 'built','output':str(output),
                      'compiled_sources':report['compiled_sources'],'artifacts':len(report['files'])}, indent=2))


if __name__ == '__main__':
    main()
