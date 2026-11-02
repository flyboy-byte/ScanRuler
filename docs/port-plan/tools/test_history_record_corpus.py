# SPDX-License-Identifier: AGPL-3.0-only
import gzip
import hashlib
import json
import re
import unittest
from pathlib import Path

import history_record_report as hr

ROOT = Path(__file__).resolve().parents[3]
PLAN = ROOT / 'docs/port-plan'


class HistoryRecordCorpusTests(unittest.TestCase):
    def test_complete_original_and_rust_bytes(self):
        report = hr.load(PLAN / 'evidence/2026-11-02-history-record.json.gz')
        source = hr.load(hr.SOURCE)
        self.assertEqual(report['scenario_count'], 110)
        self.assertFalse(report['undo_redo_travel_claimed'])
        self.assertFalse(report['browser_runtime_claimed'])
        for case, frozen in zip(report['cases'], source['cases']):
            self.assertEqual(case['id'], frozen['id'])
            original = bytes.fromhex(frozen['hex'])
            rust = bytes.fromhex(case['files']['rust.bin']['hex'])
            hr.compare(original, rust)
            self.assertEqual(case['observations'], len(hr.frame(rust)[1]))
            for name, info in case['files'].items():
                data = ((PLAN / 'fixtures/history-record' / (case['id'] + '.bin')).read_bytes()
                        if name == 'input.bin' else original if name == 'original.bin'
                        else bytes.fromhex(info['hex']))
                self.assertEqual((len(data), hashlib.sha256(data).hexdigest()),
                                 (info['bytes'], info['sha256']))

    def test_source_versions_and_fixed_inputs(self):
        current = hr.load(hr.SOURCE)
        for count, name in [(103, '2026-11-02-history-record-source.json.gz'),
                            (107, '2026-11-02-history-record-source-107.json.gz')]:
            old = hr.load(PLAN / 'evidence' / name)
            self.assertEqual(old['cases'], current['cases'][:count])
        path = PLAN / 'fixtures/history-record/manifest.json'
        manifest = json.loads(path.read_text())
        ids = [case['id'] for case in manifest['cases']]
        self.assertEqual(len(ids), 110)
        self.assertEqual(len(set(ids)), 110)
        self.assertEqual(manifest['seeds'], [1, 7, 99, 1039])
        self.assertEqual(ids, re.findall(r'id: "([^"]+)"',
                         (ROOT / 'rust/crates/port-engine/tests/support/history_record_cases.rs').read_text()))
        self.assertEqual(set(ids), {p.stem for p in (PLAN / 'fixtures/history-record').glob('*.bin')})
        for case in manifest['cases']:
            data = (PLAN / 'fixtures/history-record' / (case['id'] + '.bin')).read_bytes()
            self.assertEqual((len(data), hashlib.sha256(data).hexdigest()),
                             (case['bytes'], case['sha256']))
            reader = hr.Reader(data)
            reader.value()
            reader.end()
        self.assertEqual(hashlib.sha256(path.read_bytes()).hexdigest(), current['manifest_sha256'])

    def test_archived_compiled_original(self):
        folder = PLAN / 'evidence/history-record-oracle-2026-11-02'
        provenance = json.loads((folder / 'provenance.json').read_text())
        source = hr.load(hr.SOURCE)
        for name, key in [('history.mjs', 'bundle_sha256'),
                          ('history.mjs.map', 'source_map_sha256')]:
            self.assertEqual(hashlib.sha256(gzip.decompress((folder / (name + '.gz')).read_bytes())).hexdigest(),
                             provenance[key])
        self.assertEqual(source['bundle_sha256'], provenance['bundle_sha256'])
        self.assertEqual(source['package_lock_sha256'], provenance['package_lock_sha256'])
        self.assertEqual(source['reference_sha'], provenance['reference_sha'])

    def test_mutation_of_real_recorded_graph_and_tail_rejected(self):
        source = hr.load(hr.SOURCE)
        original = bytes.fromhex(source['cases'][0]['hex'])
        for needle in ['first'.encode('utf-16-le'), 'value'.encode('utf-16-le')]:
            mutated = bytearray(original)
            index = mutated.index(needle)
            mutated[index] ^= 1
            hr.frame(bytes(mutated))
            with self.assertRaises(ValueError):
                hr.compare(original, bytes(mutated))
        for broken in [b'', b'HRC1', original[:-1], original + b'\0']:
            with self.assertRaises(ValueError):
                hr.frame(broken)
