# SPDX-License-Identifier: AGPL-3.0-only
import hashlib,json,re,unittest
from pathlib import Path
from collections import Counter

class ObjTextureCorpusTests(unittest.TestCase):
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
