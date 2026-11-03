# SPDX-License-Identifier: AGPL-3.0-only
import gzip,hashlib,json,runpy,subprocess,unittest
from pathlib import Path
PLAN=Path(__file__).resolve().parents[1]
def load(name):return json.loads(gzip.decompress((PLAN/'evidence'/name).read_bytes()))
class RecentResultCorpusTests(unittest.TestCase):
    def test_full_typed_values_bits_lengths_and_copy_state(self):
        source=load('2026-11-03-recent-result-source.json.gz');paired=load('2026-11-03-recent-result.json.gz');inputs=(PLAN/'fixtures/recent-result/cases.tsv').read_bytes()
        self.assertEqual(hashlib.sha256(inputs).hexdigest(),source['input_sha256']);self.assertEqual(len(paired['cases']),413);self.assertEqual(len(inputs.splitlines()),413);original=[]
        for line,case in zip(inputs.splitlines(),paired['cases']):
            self.assertEqual(case['id'],line.split(b'\t')[0].decode());values={}
            for name,item in case['files'].items():
                values[name]=bytes.fromhex(item['hex']);self.assertEqual(hashlib.sha256(values[name]).hexdigest(),item['sha256'])
            self.assertEqual(values['input.bin'],line);self.assertEqual(values['original.bin'],values['rust.bin']);self.assertEqual(values['original.stderr.txt'],b'')
            parts=values['original.bin'].rstrip(b'\n').split(b'\t')
            self.assertEqual(len(parts),3);self.assertEqual(parts[1],parts[2])
            self.assertTrue(parts[1]==b'absent' or parts[1].startswith((b'value:',b'omit:')))
            original.append(values['original.bin'])
        self.assertEqual(b''.join(original).hex(),source['original_hex'])
    def test_original_complete_registry_module_and_reproducible_inputs(self):
        folder=PLAN/'evidence/recent-result-oracle-2026-11-03';p=json.loads((folder/'provenance.json').read_text())
        for name,key in [('registry.mjs','bundle_sha256'),('registry.mjs.map','source_map_sha256')]:self.assertEqual(hashlib.sha256(gzip.decompress((folder/(name+'.gz')).read_bytes())).hexdigest(),p[key])
        m=json.loads(gzip.decompress((folder/'registry.mjs.map.gz').read_bytes()));contents=[text for name,text in zip(m['sources'],m['sourcesContent'])if name.endswith('/src/commands/registry.ts')]
        original=subprocess.check_output(['git','show',p['reference_sha']+':src/commands/registry.ts'],cwd=PLAN.parents[1]).decode();self.assertEqual(contents,[original+'\nexport { keptResult }\n'])
        module=runpy.run_path(str(PLAN/'tools/generate_recent_result_cases.py'));self.assertEqual(module['generate'](),(PLAN/'fixtures/recent-result/cases.tsv').read_bytes())
