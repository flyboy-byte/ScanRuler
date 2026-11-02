# SPDX-License-Identifier: AGPL-3.0-only
import hashlib,json,re,struct,unittest
from pathlib import Path

class ErrorCorpusTests(unittest.TestCase):
    def test_fixed_hashes_all_codes_and_compiled_schedule(self):
        root=Path(__file__).resolve().parents[3];folder=root/'docs/port-plan/fixtures/engine-errors'
        manifest=json.loads((folder/'manifest.json').read_text());rows=manifest['cases'];ids=[c['id'] for c in rows]
        self.assertEqual(manifest['seeds'],[1,7,99,1039]);self.assertEqual(manifest['license'],'AGPL-3.0-only')
        self.assertEqual(manifest['reference_sha'],'2cd89c6769a10c27ee7754b477ada521c9ef5cb7')
        self.assertEqual(len(ids),52);self.assertEqual(len(set(ids)),52)
        self.assertEqual({p.stem for p in folder.glob('*.bin')},set(ids))
        table=(root/'rust/crates/port-engine/tests/support/error_cases.rs').read_text()
        self.assertEqual(re.findall(r'id: "([a-z0-9-]+)"',table),ids)
        for code in range(12):
            for kind in ['empty','escaping','unicode','surrogates']:self.assertIn(f'code-{code:02d}-{kind}',ids)
        for c in rows:
            b=(folder/(c['id']+'.bin')).read_bytes();self.assertEqual(hashlib.sha256(b).hexdigest(),c['sha256'])
            self.assertEqual(len(b),c['bytes']);self.assertLess(b[0],12)
            self.assertEqual(len(b),5+2*struct.unpack('<I',b[1:5])[0])
