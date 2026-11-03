# SPDX-License-Identifier: AGPL-3.0-only
import gzip,hashlib,json,runpy,subprocess,unittest
from pathlib import Path
PLAN=Path(__file__).resolve().parents[1]
def load(name):return json.loads(gzip.decompress((PLAN/'evidence'/name).read_bytes()))
class WatchStoresCorpusTests(unittest.TestCase):
    def test_full_guard_identity_values_and_callback_state(self):
        source=load('2026-11-03-watch-stores-source.json.gz');paired=load('2026-11-03-watch-stores.json.gz');inputs=(PLAN/'fixtures/watch-stores/cases.tsv').read_bytes()
        self.assertEqual(hashlib.sha256(inputs).hexdigest(),source['input_sha256']);self.assertEqual(len(paired['cases']),80);self.assertEqual(len(inputs.splitlines()),80);original=[]
        for line,case in zip(inputs.splitlines(),paired['cases']):
            self.assertEqual(case['id'],line.split(b'\t')[0].decode());values={}
            for name,item in case['files'].items():
                values[name]=bytes.fromhex(item['hex']);self.assertEqual(hashlib.sha256(values[name]).hexdigest(),item['sha256'])
            self.assertEqual(values['input.bin'],line);self.assertEqual(values['original.bin'],values['rust.bin']);self.assertEqual(values['original.stderr.txt'],b'')
            frames=values['original.bin'].rstrip(b'\n').split(b'\t')[1].split(b';');self.assertGreaterEqual(len(frames),6)
            for frame in frames:
                self.assertEqual(len(frame.split(b'|')),3);self.assertTrue(frame.endswith(b'|0'))
            original.append(values['original.bin'])
        self.assertEqual(b''.join(original).hex(),source['original_hex'])
    def test_original_complete_activity_module_and_reproducible_inputs(self):
        folder=PLAN/'evidence/watch-stores-oracle-2026-11-03';p=json.loads((folder/'provenance.json').read_text())
        for name,key in [('activity.mjs','bundle_sha256'),('activity.mjs.map','source_map_sha256')]:self.assertEqual(hashlib.sha256(gzip.decompress((folder/(name+'.gz')).read_bytes())).hexdigest(),p[key])
        m=json.loads(gzip.decompress((folder/'activity.mjs.map.gz').read_bytes()));contents=[text for name,text in zip(m['sources'],m['sourcesContent'])if name.endswith('/src/commands/activity.ts')]
        original=subprocess.check_output(['git','show',p['reference_sha']+':src/commands/activity.ts'],cwd=PLAN.parents[1]).decode();self.assertEqual(contents,[original+'\nexport { sessionStores }\n'])
        module=runpy.run_path(str(PLAN/'tools/generate_watch_store_cases.py'));self.assertEqual(module['generate'](),(PLAN/'fixtures/watch-stores/cases.tsv').read_bytes())
