# SPDX-License-Identifier: AGPL-3.0-only
import hashlib,json,re,unittest
from pathlib import Path
from collections import Counter

class ObjTextureCorpusTests(unittest.TestCase):
    def test_committed_source_and_rust_states_are_complete(self):
        root=Path(__file__).resolve().parents[3]
        report=json.loads((root/'docs/port-plan/evidence/2026-11-02-obj-texture.json').read_text())
        manifest=json.loads((root/'docs/port-plan/fixtures/obj-texture/manifest.json').read_text())
        self.assertEqual([c['id'] for c in report['cases']],[c['id'] for c in manifest['cases']])
        self.assertEqual(report['compared_helper_states'],144);self.assertEqual(report['working_obj_import_commands_claimed'],0)
        for c in report['cases']:
            files=c['files'];self.assertEqual(files['input.bin']['sha256'],c['sha256'])
            self.assertEqual(files['original.bin']['sha256'],files['rust.bin']['sha256'])
            for name,item in files.items():
                if name=='input.bin':continue
                b=bytes.fromhex(item['hex']);self.assertEqual(len(b),item['bytes']);self.assertEqual(hashlib.sha256(b).hexdigest(),item['sha256'])
            b=bytes.fromhex(files['original.bin']['hex']);self.assertEqual(b[:4],b'OTX1');self.assertEqual(b[4],c['mode'])
            self.assertEqual(files['original.stderr.txt']['bytes'],0)

    def test_fixed_inputs_modes_seeds_and_compiled_schedule(self):
        root=Path(__file__).resolve().parents[3];folder=root/'docs/port-plan/fixtures/obj-texture'
        m=json.loads((folder/'manifest.json').read_text());rows=m['cases'];ids=[r['id'] for r in rows]
        self.assertEqual(m['reference_sha'],'2cd89c6769a10c27ee7754b477ada521c9ef5cb7')
        self.assertEqual(m['license'],'AGPL-3.0-only');self.assertEqual(m['seeds'],[1,7,99,1039])
        self.assertEqual(len(ids),144);self.assertEqual(len(set(ids)),144)
        self.assertEqual({p.stem for p in folder.glob('*.bin')},set(ids))
        table=(root/'rust/crates/port-io/tests/support/obj_texture_cases.rs').read_text()
        self.assertEqual(re.findall(r'id: "([a-z0-9-]+)"',table),ids)
        self.assertEqual(set(Counter(r['mode'] for r in rows)),{1,2,3,4,5})
        for r in rows:
            data=(folder/(r['id']+'.bin')).read_bytes()
            self.assertEqual(hashlib.sha256(data).hexdigest(),r['sha256'],r['id'])
            self.assertEqual(len(data),r['bytes']);self.assertEqual(data[0],r['mode'])
        for seed in m['seeds']:
            self.assertIn('bake-seed-'+str(seed),ids)
            for i in range(4):self.assertIn('sample-seed-'+str(seed)+'-'+str(i),ids)
