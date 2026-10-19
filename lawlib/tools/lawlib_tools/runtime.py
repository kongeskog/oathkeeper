"""Shared compiler entry points and authored-source inventory."""
import hashlib
import json
import os
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DEFAULT = ROOT / 'us/wa/seattle'
IGNORED = {'.git', '.build', '.cache', '.venv', '__pycache__', 'node_modules'}


def input_files(root=ROOT):
    for directory, dirs, names in os.walk(root):
        dirs[:] = sorted(d for d in dirs if d not in IGNORED)
        if any((Path(directory) / d).is_symlink() for d in dirs):
            raise ValueError('symlinks are not authored corpus inputs')
        for name in sorted(names):
            path = Path(directory) / name
            if path.is_symlink():
                raise ValueError('symlinks are not authored corpus inputs')
            if name.endswith('.pyc') or name.startswith('.env') or name.endswith('.key.pem'):
                continue
            yield path


def inventory(root=ROOT):
    return {p.relative_to(root).as_posix():hashlib.sha256(p.read_bytes()).hexdigest() for p in input_files(root)}


def check_layout(root=ROOT):
    for path in input_files(root):
        name = path.name
        relative = path.relative_to(root)
        if name.endswith(('.pdf', '.zip', '.wasm', '.tar.gz', '.module.json', '.case.json', '.report.json', '.assessment.json')) or name in {'oath.lock.json', 'review-packet.json', 'baseline.json'} or relative.as_posix() == 'community/current.json' or 'sources' in relative.parts:
            raise ValueError('generated output or evidence in source tree: ' + relative.as_posix())
        if path.stat().st_size > 1024 * 1024:
            raise ValueError('authored file exceeds 1 MiB; split the module or use the evidence store: ' + str(relative))


def compile_source(path):
    import oathkeeper
    return json.loads(oathkeeper.compile_file(str(path)))


def combine(compilation):
    result = {**compilation['module'], **compilation['case']}
    result['constants'] = compilation['module'].get('constants', []) + compilation['case'].get('constants', [])
    return result


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2) + '\n')
