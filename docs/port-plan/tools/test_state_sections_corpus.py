# SPDX-License-Identifier: AGPL-3.0-only
import gzip,hashlib,json,subprocess,unittest
from pathlib import Path
PLAN=Path(__file__).resolve().parents[1]
def load(name):return json.loads(gzip.decompress((PLAN/'evidence'/name).read_bytes()))
class StateSectionsCorpusTests(unittest.TestCase):
    def test_full_live_readout_values_identity_prototype_faults_and_retained_prefix(self):
        source=load('2026-11-03-state-sections-source.json.gz');paired=load('2026-11-03-state-sections.json.gz');inputs=(PLAN/'fixtures/state-sections/cases.tsv').read_bytes()
        self.assertEqual(hashlib.sha256(inputs).hexdigest(),source['input_sha256']);self.assertEqual(source['cases'],192);self.assertEqual(len(paired['cases']),192);self.assertTrue(paired['identity'].endswith('dirty: '));original=[]
        for line,case in zip(inputs.splitlines(),paired['cases'],strict=True):
            self.assertEqual(case['id'],line.split(b'\t')[0].decode());values={}
            for name,item in case['files'].items():
                values[name]=bytes.fromhex(item['hex']);self.assertEqual(hashlib.sha256(values[name]).hexdigest(),item['sha256'])
            self.assertEqual(values['input.bin'],line);self.assertEqual(values['original.bin'],values['rust.bin']);self.assertEqual(values['original.stderr.txt'],b'');self.assertIn(b'read:',values['original.bin']);original.append(values['original.bin'])
        raw=b''.join(original);self.assertEqual(raw.hex(),source['original_hex']);self.assertEqual(hashlib.sha256(raw[:source['prior_prefix_bytes']]).hexdigest(),source['prior_prefix_sha256'])
        expected=''.join(f'case-{m*8+s:03}\t{m}\t{s}\n'for m in range(24)for s in range(8)).encode();self.assertEqual(inputs,expected)
        self.assertIn(b'^null',raw);self.assertIn(b'^@obj',raw);self.assertIn(b'^@arr',raw);self.assertIn(b'^@proto',raw);self.assertIn(b'^o{',raw);self.assertIn(b'fault:same',raw)
    def test_complete_unchanged_pinned_module_and_source_map(self):
        folder=PLAN/'evidence/state-sections-oracle-2026-11-03';p=json.loads((folder/'provenance.json').read_text())
        for name,key in [('activity.mjs','bundle_sha256'),('activity.mjs.map','source_map_sha256')]:self.assertEqual(hashlib.sha256(gzip.decompress((folder/(name+'.gz')).read_bytes())).hexdigest(),p[key])
        m=json.loads(gzip.decompress((folder/'activity.mjs.map.gz').read_bytes()));path='src/commands/stateSections.ts';contents=[text for name,text in zip(m['sources'],m['sourcesContent'])if name.endswith('/'+path)]
        original=subprocess.check_output(['git','show',p['reference_sha']+':'+path],cwd=PLAN.parents[1]).decode();self.assertEqual(contents,[original])
