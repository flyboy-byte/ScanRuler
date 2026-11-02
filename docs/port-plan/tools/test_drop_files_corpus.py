# SPDX-License-Identifier: AGPL-3.0-only
import hashlib,json,re,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3];BASE=ROOT/'docs/port-plan'
class DropFilesCorpusTests(unittest.TestCase):
 def test_fixed_inputs_and_complete_original_rust_state(self):
  folder=BASE/'fixtures/drop-files';m=json.loads((folder/'manifest.json').read_text());s=json.loads((BASE/'evidence/2026-11-02-drop-files-source.json').read_text());r=json.loads((BASE/'evidence/2026-11-02-drop-files.json').read_text())
  ids=[c['id']for c in m['cases']];self.assertEqual(len(ids),40);self.assertEqual(len(set(ids)),40);self.assertEqual(m['seeds'],[1,7,99,1039]);self.assertEqual(m['license'],'AGPL-3.0-only')
  self.assertEqual(ids,[c['id']for c in s['cases']]);self.assertEqual(ids,[c['id']for c in r['cases']]);self.assertEqual({p.stem for p in folder.glob('*.bin')},set(ids))
  schedule=(ROOT/'rust/crates/port-io/tests/support/drop_files_cases.rs').read_text();self.assertEqual(re.findall(r'id: "([a-z0-9-]+)"',schedule),ids)
  self.assertEqual(r['complete_state_cases'],40);self.assertEqual(r['working_import_commands_claimed'],0);self.assertEqual(r['successful_results']+r['platform_errors'],40)
  for c,source,proof in zip(m['cases'],s['cases'],r['cases']):
   raw=(folder/(c['id']+'.bin')).read_bytes();self.assertEqual(len(raw),c['bytes']);self.assertEqual(hashlib.sha256(raw).hexdigest(),c['sha256']);self.assertEqual(source['input_sha256'],c['sha256']);files=proof['files'];self.assertEqual(files['input.bin']['sha256'],c['sha256']);self.assertEqual(files['original.bin']['hex'],source['original_hex']);self.assertEqual(files['original.bin']['hex'],files['rust.bin']['hex']);self.assertEqual(files['original.stderr.txt']['bytes'],0)
   for name,item in files.items():
    if name=='input.bin':continue
    b=bytes.fromhex(item['hex']);self.assertEqual(len(b),item['bytes']);self.assertEqual(hashlib.sha256(b).hexdigest(),item['sha256'])
   self.assertTrue(bytes.fromhex(source['original_hex']).startswith(b'DRP1'))
  for seed in m['seeds']:
   for i in range(4):self.assertIn(f'seed-{seed}-{i}',ids)
