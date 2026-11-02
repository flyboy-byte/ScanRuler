# SPDX-License-Identifier: AGPL-3.0-only
import collections,hashlib,json,struct,tomllib,unittest
from pathlib import Path
from compare_image_audit import frame,verdict
BASE=Path(__file__).resolve().parents[1]
def read(name):return json.loads((BASE/name).read_text())
def sha(b):return hashlib.sha256(b).hexdigest()
class ImageAuditTests(unittest.TestCase):
    def test_fixed_independent_inputs_and_complete_source_pixels(self):
        m=read('fixtures/image-audit/manifest.json');s=read('evidence/2026-11-02-image-source.json')
        self.assertEqual(m['seeds'],[1,7,99,1039]);self.assertEqual(m['license'],'AGPL-3.0-only')
        self.assertEqual(s['status'],'captured-source-only');self.assertEqual(s['reference_sha'],m['reference_sha'])
        self.assertEqual(s['page_errors'],[]);self.assertRegex(s['browser'],r'^Chrome/');self.assertRegex(s['node'],r'^v22\.')
        ids=[c['id']for c in m['cases']];self.assertEqual(len(ids),56);self.assertEqual(len(set(ids)),56)
        self.assertEqual(ids,[c['id']for c in s['cases']]);self.assertEqual({p.stem for p in (BASE/'fixtures/image-audit').glob('*.bin')},set(ids))
        self.assertEqual(sum(c['source']is not None for c in s['cases']),36)
        for c,source in zip(m['cases'],s['cases']):
            b=(BASE/'fixtures/image-audit'/(c['id']+'.bin')).read_bytes();self.assertEqual(len(b),c['bytes']);self.assertEqual(sha(b),c['sha256']);self.assertEqual(source['input_sha256'],c['sha256'])
            if source['source'] is not None:
                image=source['source'];b=bytes.fromhex(image['rgba_hex']);self.assertEqual(len(b),image['width']*image['height']*4);self.assertEqual(sha(b),image['rgba_sha256'])
        for seed in m['seeds']:
            for kind in ['png','jpeg','bmp','webp']:self.assertIn(f'seed-{seed}-{kind}',ids)

    def test_every_exact_mismatch_and_omission_is_retained(self):
        source=read('evidence/2026-11-02-image-source.json');r=read('evidence/2026-11-02-image-candidate.json')
        self.assertEqual(r['status'],'completed-candidate-audit-not-a-port')
        self.assertEqual(r['source_capture_sha256'],sha((BASE/'evidence/2026-11-02-image-source.json').read_bytes()))
        self.assertEqual([c['id']for c in source['cases']],[c['id']for c in r['cases']])
        counts=collections.Counter()
        for c,s in zip(r['cases'],source['cases']):
            self.assertEqual(c['source'],s['source']);self.assertEqual(c['input_sha256'],s['input_sha256'])
            raw=bytes.fromhex(c['candidate_frame_hex']);self.assertEqual(sha(raw),c['candidate_frame_sha256'])
            parsed=frame(raw);self.assertEqual(parsed,c['candidate']);self.assertEqual(verdict(c['source'],parsed),c['verdict']);counts[c['verdict']]+=1
            if c['verdict']=='pixel_mismatch':
                a=bytes.fromhex(c['source']['rgba_hex']);b=bytes.fromhex(parsed['rgba_hex']);bad=[i for i,(x,y)in enumerate(zip(a,b))if x!=y]
                self.assertEqual(c['different_bytes'],len(bad));self.assertEqual(c['first_difference'],dict(offset=bad[0],source=a[bad[0]],candidate=b[bad[0]]));self.assertEqual(c['max_abs_byte_difference'],max(abs(a[i]-b[i])for i in bad))
        self.assertEqual(dict(counts),r['counts']);self.assertEqual(dict(counts),dict(exact_pixels=8,pixel_mismatch=28,both_omitted=18,presence_mismatch=2))
        # A real complete-pixel observation must change verdict after a bit flip.
        c=next(c for c in r['cases']if c['id']=='png-rgb');changed=bytearray.fromhex(c['candidate_frame_hex']);changed[17]^=1
        self.assertEqual(verdict(c['source'],frame(changed)),'pixel_mismatch')
        with self.assertRaises(ValueError):frame(bytes(changed[:-1]))
        self.assertEqual(verdict(c['source'],dict(kind='omission',message='test')),'presence_mismatch')
        self.assertEqual(verdict(None,dict(kind='host-refusal',message='limit')),'host_refusal')
        for bad in [b'',b'IMG1',b'IMG1\x03'+b'\0'*4,b'IMG1\x01'+struct.pack('<III',1,1,0)]:
            with self.assertRaises(ValueError):frame(bad)

    def test_exact_dependency_closure_and_linked_wasm_evidence(self):
        graph=read('evidence/2026-11-02-image-dependencies.json');lock_path=BASE/'spikes/image-audit/candidate.lock';lock=tomllib.loads(lock_path.read_text())
        self.assertEqual(sha(lock_path.read_bytes()),graph['lock_sha256']);self.assertEqual(graph['root'],'image@0.25.10');self.assertFalse(graph['default_features'])
        self.assertEqual(graph['root_features'],['bmp','jpeg','png','webp']);self.assertEqual(graph['native_graph'],graph['wasm_graph'])
        locked={p['name']+'@'+p['version']:p['checksum']for p in lock['package']if p.get('source')}
        self.assertEqual(len(locked),22);self.assertEqual({p['package']for p in graph['packages']},set(locked))
        for p in graph['packages']:
            self.assertEqual(p['checksum'],locked[p['package']]);review=p['selected_license_review'];self.assertIn(review['option'],['MIT','BSD-3-Clause','Zlib']);self.assertIn(dict(path=review['path'],sha256=review['sha256']),[dict(path=f['path'],sha256=f['sha256'])for f in p['license_files']]);self.assertEqual(p['foreign_code_files'],[]);self.assertTrue(p['license_files'])
            for item in p['license_files']:self.assertGreater(item['bytes'],0);self.assertRegex(item['sha256'],r'^[0-9a-f]{64}$')
            self.assertRegex(p['source_tree_sha256'],r'^[0-9a-f]{64}$');self.assertTrue(set(p['dependencies'])<=set(locked))
            self.assertNotIn('any_c_zlib',p['features'])
        wasm=read('evidence/2026-11-02-image-wasm-build.json');self.assertEqual(wasm['imports'],[]);self.assertGreater(wasm['bytes'],0);self.assertRegex(wasm['module_sha256'],r'^[0-9a-f]{64}$')
