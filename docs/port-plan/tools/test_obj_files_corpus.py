# SPDX-License-Identifier: AGPL-3.0-only
import hashlib
import json
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[3]
BASE = ROOT / 'docs/port-plan'
class ObjFilesCorpusTests(unittest.TestCase):
    def test_fixed_independent_input_schedule(self):
        folder = BASE / 'fixtures/obj-files'
        m = json.loads((folder/'manifest.json').read_text())
        self.assertEqual(m['license'],'AGPL-3.0-only')
        self.assertEqual(m['reference_sha'],'2cd89c6769a10c27ee7754b477ada521c9ef5cb7')
        self.assertEqual(m['seeds'],[1,7,99,1039])
        ids = [r['id'] for r in m['cases']]
        self.assertEqual(len(ids),87);self.assertEqual(len(set(ids)),87)
        self.assertEqual({p.stem for p in folder.glob('*.bin')},set(ids))
        table = (ROOT/'rust/crates/port-io/tests/support/obj_files_cases.rs').read_text()
        self.assertEqual(re.findall(r'id: "([a-z0-9-]+)"',table),ids)
        for c in m['cases']:
            b = (folder/(c['id']+'.bin')).read_bytes()
            self.assertEqual(len(b),c['bytes']);self.assertEqual(b[0],c['mode'])
            self.assertEqual(hashlib.sha256(b).hexdigest(),c['sha256'])
        for seed in m['seeds']:
            for i in range(8):self.assertIn(f'seed-{seed}-{i}',ids)

    def test_complete_proof_preserves_preimplementation_original_capture(self):
        m = json.loads((BASE/'fixtures/obj-files/manifest.json').read_text())
        source = json.loads((BASE/'evidence/2026-11-02-obj-files-source.json').read_text())
        proof = json.loads((BASE/'evidence/2026-11-02-obj-files.json').read_text())
        self.assertEqual(proof['complete_state_comparisons'],87)
        self.assertEqual(proof['working_import_commands_claimed'],0)
        self.assertEqual([r['id'] for r in proof['cases']],[r['id'] for r in m['cases']])
        self.assertEqual([r['id'] for r in source['cases']],[r['id'] for r in m['cases']])
        for c,s,fixture in zip(proof['cases'],source['cases'],m['cases']):
            files=c['files'];self.assertEqual(files['input.bin']['sha256'],fixture['sha256'])
            self.assertEqual(s['input_sha256'],fixture['sha256'])
            self.assertEqual(files['original.bin']['hex'],s['original_hex'])
            self.assertEqual(files['original.bin']['hex'],files['rust.bin']['hex'])
            self.assertEqual(files['original.stderr.txt']['bytes'],0)
            for name,item in files.items():
                if name=='input.bin':continue
                raw=bytes.fromhex(item['hex']);self.assertEqual(len(raw),item['bytes'])
                self.assertEqual(hashlib.sha256(raw).hexdigest(),item['sha256'])
            self.assertTrue(bytes.fromhex(files['original.bin']['hex']).startswith(b'FLS1'))
