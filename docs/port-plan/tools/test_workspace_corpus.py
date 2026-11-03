# SPDX-License-Identifier: AGPL-3.0-only
import gzip,hashlib,json,runpy,subprocess,unittest
from pathlib import Path
PLAN=Path(__file__).resolve().parents[1]
def load(name):return json.loads(gzip.decompress((PLAN/'evidence'/name).read_bytes()))
class WorkspaceCorpusTests(unittest.TestCase):
    def test_full_typed_values_bits_lengths_and_copy_state(self):
        source=load('2026-11-03-workspaces-source.json.gz');paired=load('2026-11-03-workspaces.json.gz');inputs=(PLAN/'fixtures/workspaces/cases.tsv').read_bytes()
        self.assertEqual(hashlib.sha256(inputs).hexdigest(),source['input_sha256']);self.assertEqual(len(paired['cases']),306);self.assertEqual(len(inputs.splitlines()),306);original=[]
        for line,case in zip(inputs.splitlines(),paired['cases']):
            self.assertEqual(case['id'],line.split(b'\t')[0].decode());values={}
            for name,item in case['files'].items():
                values[name]=bytes.fromhex(item['hex']);self.assertEqual(hashlib.sha256(values[name]).hexdigest(),item['sha256'])
            self.assertEqual(values['input.bin'],line);self.assertEqual(values['original.bin'],values['rust.bin']);self.assertEqual(values['original.stderr.txt'],b'')
            parts=values['original.bin'].rstrip(b'\n').split(b'\t')
            self.assertEqual(len(parts),2);self.assertEqual(len(parts[1].split(b'|')),6)
            original.append(values['original.bin'])
        self.assertEqual(b''.join(original).hex(),source['original_hex'])
        prefix=b''.join(original)[:source['prior_incomplete_prefix_bytes']]
        self.assertEqual(hashlib.sha256(prefix).hexdigest(),source['prior_incomplete_prefix_sha256'])
    def test_original_complete_registry_module_and_reproducible_inputs(self):
        folder=PLAN/'evidence/workspaces-oracle-2026-11-03';p=json.loads((folder/'provenance.json').read_text())
        for name,key in [('workspaces.mjs','bundle_sha256'),('workspaces.mjs.map','source_map_sha256')]:self.assertEqual(hashlib.sha256(gzip.decompress((folder/(name+'.gz')).read_bytes())).hexdigest(),p[key])
        m=json.loads(gzip.decompress((folder/'workspaces.mjs.map.gz').read_bytes()))
        for path in ['src/commands/state.ts','src/plugins/registry.ts']:
            contents=[text for name,text in zip(m['sources'],m['sourcesContent'])if name.endswith('/'+path)]
            original=subprocess.check_output(['git','show',p['reference_sha']+':'+path],cwd=PLAN.parents[1]).decode();self.assertEqual(contents,[original])
        for identity,digest in p['fixture_plugins'].items():
            fixture=(PLAN/'fixtures/workspaces/probes'/(identity+'.ts')).read_bytes()
            self.assertEqual(hashlib.sha256(fixture).hexdigest(),digest)
            contents=[text for name,text in zip(m['sources'],m['sourcesContent'])if name.endswith('/plugins/'+identity+'/plugin.ts')]
            self.assertEqual(contents,[fixture.decode()])
        module=runpy.run_path(str(PLAN/'tools/generate_workspace_cases.py'));self.assertEqual(module['generate'](),(PLAN/'fixtures/workspaces/cases.tsv').read_bytes())
