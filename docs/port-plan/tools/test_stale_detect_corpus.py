# SPDX-License-Identifier: AGPL-3.0-only
import gzip
import hashlib
import json
import subprocess
import unittest
from pathlib import Path
PLAN = Path(__file__).resolve().parents[1]
ROOT = PLAN.parents[1]
PIN = '2cd89c6769a10c27ee7754b477ada521c9ef5cb7'
def load(name):
    return json.loads(gzip.decompress((PLAN/'evidence'/name).read_bytes()))
class StaleDetectCorpusTests(unittest.TestCase):
    def test_full_states_inputs_and_hashes(self):
        source=load('2026-11-02-stale-detect-source.json.gz')
        paired=load('2026-11-02-stale-detect.json.gz')
        inputs=(PLAN/'fixtures/stale-detect/cases.tsv').read_bytes()
        self.assertEqual(hashlib.sha256(inputs).hexdigest(),source['input_sha256'])
        self.assertEqual(len(paired['cases']),len(inputs.splitlines()))
        self.assertEqual(len(paired['cases']),308)
        original=[]
        for line,case in zip(inputs.splitlines(),paired['cases']):
            self.assertEqual(case['id'],line.split(b'\t')[0].decode())
            values={}
            for name,item in case['files'].items():
                values[name]=bytes.fromhex(item['hex'])
                self.assertEqual(hashlib.sha256(values[name]).hexdigest(),item['sha256'])
            self.assertEqual(values['input.bin'],line)
            self.assertEqual(values['original.bin'],values['rust.bin'])
            self.assertEqual(values['original.stderr.txt'],b'')
            self.assertIn(values['original.bin'].split(b'\t')[1],(b'0',b'1'))
            self.assertTrue(values['original.bin'].endswith(b'\tnull\n'))
            original.append(values['original.bin'])
        self.assertEqual(b''.join(original).hex(),source['original_hex'])
        self.assertTrue(any(b'\t1\t' in v for v in original))
        self.assertTrue(any(b'\t0\t' in v for v in original))
    def test_full_original_modules_and_maps(self):
        folder=PLAN/'evidence/stale-detect-oracle-2026-11-02'
        provenance=json.loads((folder/'oracle-provenance.json').read_text())
        self.assertEqual(provenance['reference_sha'],PIN)
        for name,expected in provenance['files'].items():
            self.assertEqual(hashlib.sha256((folder/name).read_bytes()).hexdigest(),expected)
        for name in ['types','stalePage']:
            original=subprocess.check_output(['git','show',PIN+':src/commands/'+name+'.ts'],cwd=ROOT)
            self.assertEqual((folder/(name+'.ts')).read_bytes(),original)
            m=json.loads((folder/(name+'.cjs.map')).read_text())
            self.assertEqual(m['sourcesContent'],[original.decode()])
