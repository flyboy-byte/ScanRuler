# SPDX-License-Identifier: AGPL-3.0-only
import hashlib,json,re,struct,unittest
from pathlib import Path

class ObjMeshCorpusTests(unittest.TestCase):
    def test_complete_committed_source_state_and_progress(self):
        root=Path(__file__).resolve().parents[3]
        report=json.loads((root/'docs/port-plan/evidence/2026-11-02-obj-mesh.json').read_text())
        m=json.loads((root/'docs/port-plan/fixtures/obj-mesh/manifest.json').read_text())
        self.assertEqual([c['id'] for c in report['cases']],[c['id'] for c in m['cases']]);self.assertEqual(report['complete_outputs'],53);self.assertEqual(report['source_refusals'],28)
        for c in report['cases']:
            files=c['files'];self.assertEqual(files['input.bin']['sha256'],c['sha256']);self.assertEqual(files['original.bin']['sha256'],files['rust.bin']['sha256'])
            for name,item in files.items():
                if name=='input.bin':continue
                b=bytes.fromhex(item['hex']);self.assertEqual(len(b),item['bytes']);self.assertEqual(hashlib.sha256(b).hexdigest(),item['sha256'])
            b=bytes.fromhex(files['original.bin']['hex']);self.assertEqual(b[:4],b'OBJ1');self.assertEqual(b[4],0 if c['success'] else 2);self.assertEqual(files['original.stderr.txt']['bytes'],0)
        progress=bytes.fromhex(next(c for c in report['cases'] if c['id']=='line-progress')['files']['original.bin']['hex'])
        skipped=bytes.fromhex(next(c for c in report['cases'] if c['id']=='short-unicode-line-skips-progress')['files']['original.bin']['hex'])
        self.assertIn('Reading OBJ… 100%'.encode(),progress);self.assertNotIn('Reading OBJ… 100%'.encode(),skipped)

    def test_fixed_inputs_and_full_progress_boundaries(self):
        root=Path(__file__).resolve().parents[3];folder=root/'docs/port-plan/fixtures/obj-mesh'
        m=json.loads((folder/'manifest.json').read_text());rows=m['cases'];ids=[r['id'] for r in rows]
        self.assertEqual(m['reference_sha'],'2cd89c6769a10c27ee7754b477ada521c9ef5cb7');self.assertEqual(m['license'],'AGPL-3.0-only');self.assertEqual(m['seeds'],[1,7,99,1039])
        self.assertEqual(len(ids),81);self.assertEqual(len(set(ids)),81);self.assertEqual(sum(r['success'] for r in rows),53)
        self.assertEqual({p.stem for p in folder.glob('*.bin')},set(ids))
        table=(root/'rust/crates/port-io/tests/support/obj_mesh_cases.rs').read_text();self.assertEqual(re.findall(r'id: "([a-z0-9-]+)"',table),ids)
        for r in rows:
            b=(folder/(r['id']+'.bin')).read_bytes();self.assertEqual(hashlib.sha256(b).hexdigest(),r['sha256'],r['id']);self.assertEqual(len(b),r['bytes'])
        for id,expected in [('line-progress','noop'),('short-unicode-line-skips-progress','🦀')]:
            b=(folder/(id+'.bin')).read_bytes();n=struct.unpack('<I',b[:4])[0];text=b[4:4+n].decode('utf8');self.assertEqual(text.split('\n')[1<<20],expected)
            self.assertGreater(n,5_000_000)
