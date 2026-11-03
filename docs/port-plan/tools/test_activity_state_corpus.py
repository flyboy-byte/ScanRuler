# SPDX-License-Identifier: AGPL-3.0-only
import gzip,hashlib,json,subprocess,unittest
from pathlib import Path
PLAN=Path(__file__).resolve().parents[1]
def load(name):return json.loads(gzip.decompress((PLAN/'evidence'/name).read_bytes()))
class ActivityStateCorpusTests(unittest.TestCase):
    def test_full_snapshot_identity_listener_fault_traces_and_retained_prefix(self):
        source=load('2026-11-03-activity-state-source.json.gz');paired=load('2026-11-03-activity-state.json.gz');inputs=(PLAN/'fixtures/activity-state/cases.tsv').read_bytes()
        self.assertEqual(hashlib.sha256(inputs).hexdigest(),source['input_sha256']);self.assertEqual(source['cases'],256);self.assertEqual(len(paired['cases']),256);self.assertTrue(paired['identity'].endswith('dirty: '));original=[]
        for line,case in zip(inputs.splitlines(),paired['cases'],strict=True):
            self.assertEqual(case['id'],line.split(b'\t')[0].decode());values={}
            for name,item in case['files'].items():
                values[name]=bytes.fromhex(item['hex']);self.assertEqual(hashlib.sha256(values[name]).hexdigest(),item['sha256'])
            self.assertEqual(values['input.bin'],line);self.assertEqual(values['original.bin'],values['rust.bin']);self.assertEqual(values['original.stderr.txt'],b'');self.assertIn(b'held:',values['original.bin']);original.append(values['original.bin'])
        raw=b''.join(original);self.assertEqual(raw.hex(),source['original_hex']);self.assertEqual(hashlib.sha256(raw[:source['prior_prefix_bytes']]).hexdigest(),source['prior_prefix_sha256'])
        expected=''.join(f'case-{m*16+s:03}\t{m}\t{s}\n'for m in range(16)for s in range(16)).encode();self.assertEqual(inputs,expected)
    def test_complete_pinned_activity_module_and_zustand_provenance(self):
        folder=PLAN/'evidence/activity-state-oracle-2026-11-03';p=json.loads((folder/'provenance.json').read_text())
        for name,key in [('activity.mjs','bundle_sha256'),('activity.mjs.map','source_map_sha256')]:self.assertEqual(hashlib.sha256(gzip.decompress((folder/(name+'.gz')).read_bytes())).hexdigest(),p[key])
        m=json.loads(gzip.decompress((folder/'activity.mjs.map.gz').read_bytes()));path='src/commands/activity.ts';contents=[text for name,text in zip(m['sources'],m['sourcesContent'])if name.endswith('/'+path)]
        original=subprocess.check_output(['git','show',p['reference_sha']+':'+path],cwd=PLAN.parents[1]).decode();self.assertEqual(contents,[original])
        lock=json.loads((PLAN.parents[1]/'package-lock.json').read_text());self.assertEqual(p['zustand_version'],lock['packages']['node_modules/zustand']['version'])
        for filename,key in [('vanilla.js','zustand_vanilla_sha256'),('esm/vanilla.mjs','zustand_vanilla_esm_sha256')]:self.assertEqual(hashlib.sha256((PLAN.parents[1]/'node_modules/zustand'/filename).read_bytes()).hexdigest(),p[key])
        self.assertEqual((PLAN.parents[1]/'rust/licenses/ZUSTAND-MIT.txt').read_bytes(),(PLAN.parents[1]/'node_modules/zustand/LICENSE').read_bytes())
