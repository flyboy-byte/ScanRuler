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
class NormalizeCorpusTests(unittest.TestCase):
    def test_full_states_inputs_and_hashes(self):
        source=load('2026-11-02-normalize-source.json.gz')
        paired=load('2026-11-02-normalize.json.gz')
        inputs=(PLAN/'fixtures/error-normalize/cases.tsv').read_bytes()
        self.assertEqual(hashlib.sha256(inputs).hexdigest(),source['input_sha256'])
        self.assertEqual(len(paired['cases']),len(inputs.splitlines()))
        self.assertEqual(len(paired['cases']),304)
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
            self.assertEqual(len(values['original.bin'].rstrip(b'\n').split(b'\t')),4)
            original.append(values['original.bin'])
        self.assertEqual(b''.join(original).hex(),source['original_hex'])
        self.assertTrue(any(b'CommandError,stale_page,' in v for v in original))
        self.assertTrue(any(b'\tthrow,' in v for v in original))
    def test_original_private_adapter_is_the_full_pinned_body(self):
        folder=PLAN/'evidence/normalize-oracle-2026-11-02'
        p=json.loads((folder/'provenance.json').read_text())
        for name,key in [('registry.mjs','bundle_sha256'),('registry.mjs.map','source_map_sha256')]:
            self.assertEqual(hashlib.sha256(gzip.decompress((folder/(name+'.gz')).read_bytes())).hexdigest(),p[key])
        m=json.loads(gzip.decompress((folder/'registry.mjs.map.gz').read_bytes()))
        contents=[text for name,text in zip(m['sources'],m['sourcesContent'])if name.endswith('/src/commands/registry.ts')]
        original=subprocess.check_output(['git','show',PIN+':src/commands/registry.ts'],cwd=ROOT).decode()
        self.assertEqual(contents,[original+'\nexport { asCommandError }\n'])
