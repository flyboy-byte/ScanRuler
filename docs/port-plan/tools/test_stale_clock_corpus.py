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
class StaleClockCorpusTests(unittest.TestCase):
    def test_full_states_inputs_and_hashes(self):
        source=load('2026-11-02-stale-clock-source.json.gz')
        paired=load('2026-11-02-stale-clock.json.gz')
        inputs=(PLAN/'fixtures/stale-clock/cases.tsv').read_bytes()
        self.assertEqual(hashlib.sha256(inputs).hexdigest(),source['input_sha256'])
        self.assertEqual(len(paired['cases']),len(inputs.splitlines()))
        self.assertEqual(len(paired['cases']),64)
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
            self.assertEqual(len(values['original.bin'].rstrip(b'\n').split(b'\t')),2)
            for frame in values['original.bin'].rstrip(b'\n').split(b'\t')[1].split(b';'):
                self.assertEqual(len(frame.split(b',')),3)
            original.append(values['original.bin'])
        self.assertEqual(b''.join(original).hex(),source['original_hex'])
