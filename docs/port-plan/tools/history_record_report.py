# SPDX-License-Identifier: AGPL-3.0-only
"""Archive lossless history-record observations against frozen source capture."""
import gzip
import hashlib
import json
from pathlib import Path

from schema_report import Reader

ROOT = Path(__file__).resolve().parents[3]
PLAN = ROOT / 'docs/port-plan'
SOURCE = PLAN / 'evidence/2026-11-02-history-record-source-110.json.gz'


def load(path):
    return json.loads(gzip.decompress(path.read_bytes()))


def frame(data):
    reader = Reader(data)
    if reader.take(4) != b'HRC1':
        raise ValueError('History frame magic')
    root = reader.value()
    reader.end()
    if root[0] != 6 or not root[1]:
        raise ValueError('Empty history observations')
    keys = ['error', 'workspace', 'past', 'future', 'revision', 'restoring',
            'importing', 'blocked', 'current', 'pending', 'log']
    for row in root[1]:
        if row[0] != 7 or [key for key, _ in row[1]] != keys:
            raise ValueError('History observation shape/order')
        values = dict(row[1])
        if any(values[key][0] != 6 for key in ['past', 'future', 'current', 'pending', 'log']):
            raise ValueError('History list shape')
        for entry in values['past'][1] + values['future'][1]:
            if entry[0] != 7 or [key for key, _ in entry[1]] != [
                'before', 'after', 'label', 'workspace', 'group'
            ]:
                raise ValueError('History entry shape/order')
    return root


def compare(original, rust):
    frame(original)
    frame(rust)
    if original != rust:
        raise ValueError('Complete history state/graph mismatch')


def archive():
    run = max((ROOT / 'rust/target/history-record-diff').glob('run-*'),
              key=lambda path: int(path.name[4:]))
    source = load(SOURCE)
    frozen = {row['id']: row for row in source['cases']}
    manifest_path = PLAN / 'fixtures/history-record/manifest.json'
    manifest = json.loads(manifest_path.read_text())
    rows = []
    observations = 0
    for case in manifest['cases']:
        directory = run / case['id']
        original = (directory / 'original.bin').read_bytes()
        rust = (directory / 'rust.bin').read_bytes()
        compare(bytes.fromhex(frozen[case['id']]['hex']), original)
        compare(original, rust)
        count = len(frame(rust)[1])
        observations += count
        files = {}
        for name in ['input.bin', 'original.bin', 'rust.bin', 'original.stderr.txt']:
            data = (directory / name).read_bytes()
            files[name] = dict(bytes=len(data), sha256=hashlib.sha256(data).hexdigest(),
                               **({'hex': data.hex()} if name in ['rust.bin', 'original.stderr.txt'] else {}))
        if files['input.bin']['sha256'] != case['sha256'] or files['original.stderr.txt']['bytes']:
            raise ValueError('Input/stderr mismatch')
        rows.append(dict(id=case['id'], observations=count, files=files))
    if {path.name for path in run.iterdir() if path.is_dir()} != set(frozen):
        raise ValueError('Missing/extra history cases')
    report = dict(contract='engine.session.history.record.bytes.v1',
                  candidate_sha=(run / 'candidate.txt').read_text().strip(),
                  reference_sha=source['reference_sha'],
                  identity=(run / 'identity.txt').read_text(),
                  source_capture=SOURCE.name,
                  source_capture_sha256=hashlib.sha256(SOURCE.read_bytes()).hexdigest(),
                  manifest_sha256=hashlib.sha256(manifest_path.read_bytes()).hexdigest(),
                  seeds=manifest['seeds'], scenario_count=len(rows),
                  operation_observations=observations, browser_runtime_claimed=False,
                  undo_redo_travel_claimed=False, cases=rows)
    output = PLAN / 'evidence/2026-11-02-history-record.json.gz'
    output.write_bytes(gzip.compress((json.dumps(report, indent=2) + '\n').encode(), mtime=0))
    print(report['candidate_sha'], len(rows), 'scenarios', observations, 'observations')


if __name__ == '__main__':
    archive()
