# SPDX-License-Identifier: AGPL-3.0-only
import gzip
import hashlib
import json
import unittest
from pathlib import Path

PLAN = Path(__file__).resolve().parents[1]
def load(name):
    return json.loads(gzip.decompress((PLAN / 'evidence' / name).read_bytes()))

class LocalHistoryCorpusTests(unittest.TestCase):
    def test_complete_original_rust_observations_and_inputs(self):
        source = load('2026-11-02-history-local-source.json.gz')
        paired = load('2026-11-02-history-local.json.gz')
        inputs = (PLAN / 'fixtures/history-local/cases.tsv').read_bytes()
        self.assertEqual(hashlib.sha256(inputs).hexdigest(), source['input_sha256'])
        self.assertEqual(len(paired['cases']), 192)
        originals = []
        for case, line in zip(paired['cases'], inputs.splitlines()):
            self.assertEqual(case['id'], line.split(b'\t')[0].decode())
            values = {}
            for name, item in case['files'].items():
                values[name] = bytes.fromhex(item['hex'])
                self.assertEqual(hashlib.sha256(values[name]).hexdigest(), item['sha256'])
            self.assertEqual(values['input.bin'], line)
            self.assertEqual(values['original.bin'], values['rust.bin'])
            self.assertEqual(values['original.stderr.txt'], b'')
            self.assertEqual(len(values['rust.bin'].rstrip(b'\n').split(b'\t')), 9)
            originals.append(values['original.bin'])
        self.assertEqual(b''.join(originals).hex(), source['output_hex'])

    def test_compiled_original_identity(self):
        source = load('2026-11-02-history-local-source.json.gz')
        folder = PLAN / 'evidence/history-local-oracle-2026-11-02'
        p = json.loads((folder / 'provenance.json').read_text())
        self.assertEqual(p, source['provenance'])
        for name,key in [('history.mjs','bundle_sha256'),('history.mjs.map','source_map_sha256')]:
            self.assertEqual(hashlib.sha256(gzip.decompress((folder / (name+'.gz')).read_bytes())).hexdigest(), p[key])
