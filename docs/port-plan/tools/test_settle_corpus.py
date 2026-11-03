# SPDX-License-Identifier: AGPL-3.0-only
import gzip, hashlib, json, subprocess, unittest
from pathlib import Path
PLAN=Path(__file__).resolve().parents[1]
ROOT=PLAN.parents[1]
def load(name):return json.loads(gzip.decompress((PLAN/'evidence'/name).read_bytes()))
class SettleCorpusTests(unittest.TestCase):
    def test_complete_ordered_states_and_original_bytes(self):
        source=load('2026-11-02-settle-source.json.gz');paired=load('2026-11-02-settle.json.gz')
        inputs=(PLAN/'fixtures/settle-run/cases.tsv').read_bytes()
        self.assertEqual(hashlib.sha256(inputs).hexdigest(),source['input_sha256'])
        self.assertEqual(len(paired['cases']),64);self.assertEqual(len(inputs.splitlines()),64)
        self.assertEqual(source['seeds'],[1,7,99,1039]);original=[]
        for line,case in zip(inputs.splitlines(),paired['cases']):
            self.assertEqual(case['id'],line.split(b'\t')[0].decode());values={}
            for name,item in case['files'].items():
                values[name]=bytes.fromhex(item['hex']);self.assertEqual(hashlib.sha256(values[name]).hexdigest(),item['sha256'])
            self.assertEqual(values['input.bin'],line);self.assertEqual(values['original.bin'],values['rust.bin']);self.assertEqual(values['original.stderr.txt'],b'')
            states=values['original.bin'].rstrip(b'\n').split(b'\t')[1].split(b';')
            self.assertGreaterEqual(len(states),3)
            for state in states:self.assertTrue(state.endswith(b'|0'))
            original.append(values['original.bin'])
        self.assertEqual(b''.join(original).hex(),source['original_hex'])
        for status in [b'null|',b'pending|',b'done|']:self.assertTrue(any(status in s for s in original))
    def test_complete_pinned_private_export_source(self):
        folder=PLAN/'evidence/settle-oracle-2026-11-02';p=json.loads((folder/'provenance.json').read_text())
        for name,key in [('registry.mjs','bundle_sha256'),('registry.mjs.map','source_map_sha256')]:self.assertEqual(hashlib.sha256(gzip.decompress((folder/(name+'.gz')).read_bytes())).hexdigest(),p[key])
        m=json.loads(gzip.decompress((folder/'registry.mjs.map.gz').read_bytes()))
        contents=[text for name,text in zip(m['sources'],m['sourcesContent'])if name.endswith('/src/commands/registry.ts')]
        original=subprocess.check_output(['git','show',p['reference_sha']+':src/commands/registry.ts'],cwd=ROOT).decode()
        self.assertEqual(contents,[original+'\nexport { settle }\n'])
