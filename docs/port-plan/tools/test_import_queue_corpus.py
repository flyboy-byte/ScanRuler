# SPDX-License-Identifier: AGPL-3.0-only
import gzip,hashlib,json,re,subprocess,unittest
from pathlib import Path
from schema_report import Reader
PLAN=Path(__file__).resolve().parents[1];ROOT=PLAN.parents[1]
def load(name):return json.loads(gzip.decompress((PLAN/'evidence'/name).read_bytes()))
def frame(data):
 r=Reader(data)
 if r.take(4)!=b'IQU1':raise ValueError('Queue frame magic')
 value=r.value();r.end()
 if value[0]!=6 or not value[1]:raise ValueError('Empty queue trace')
 for row in value[1]:
  if row[0]!=7 or [k for k,_ in row[1]]!=['error','busy','pending','log','results']:raise ValueError('Queue observation fields/order')
 return value
class ImportQueueCorpusTests(unittest.TestCase):
 def test_complete_independent_class_and_rust_results(self):
  s=load('2026-11-02-import-queue-source.json.gz');p=load('2026-11-02-import-queue.json.gz');m=json.loads((PLAN/'fixtures/import-queue/manifest.json').read_text());self.assertEqual(len(s['cases']),67);self.assertEqual(p['operation_observations'],755);self.assertEqual((p['successful_jobs'],p['failed_jobs']),(59,57))
  self.assertEqual(hashlib.sha256((PLAN/'fixtures/import-queue/manifest.json').read_bytes()).hexdigest(),s['manifest_sha256']);self.assertEqual(m['seeds'],[1,7,99,1039])
  for case,source,fixture in zip(p['cases'],s['cases'],m['cases']):
   self.assertEqual(case['id'],source['id']);self.assertEqual(case['id'],fixture['id']);values={}
   for name,item in case['files'].items():
    values[name]=bytes.fromhex(item['hex']);self.assertEqual(hashlib.sha256(values[name]).hexdigest(),item['sha256'])
   self.assertEqual(values['original.bin'],bytes.fromhex(source['hex']));self.assertEqual(values['rust.bin'],values['original.bin']);self.assertEqual(values['original.stderr.txt'],b'');self.assertEqual(values['input.bin'],(PLAN/'fixtures/import-queue'/(case['id']+'.bin')).read_bytes());self.assertEqual(hashlib.sha256(values['input.bin']).hexdigest(),fixture['sha256'])
   last=dict(frame(values['rust.bin'])[1][-1][1]);self.assertTrue(all(dict(result[1])['status'][1]!='pending'for result in last['results'][1]))
  ids=[c['id']for c in m['cases']];self.assertEqual(len(set(ids)),67);self.assertEqual(set(ids),{f.stem for f in (PLAN/'fixtures/import-queue').glob('*.bin')});self.assertEqual(ids,re.findall(r'id: "([^"]+)"',(ROOT/'rust/crates/port-engine/tests/support/import_queue_cases.rs').read_text()))
 def test_full_original_source_map_and_compiler_identity(self):
  folder=PLAN/'evidence/import-queue-oracle-2026-11-02';p=json.loads((folder/'provenance.json').read_text());s=load('2026-11-02-import-queue-source.json.gz');self.assertEqual(p,s['provenance'])
  for name,key in [('queue.cjs','bundle_sha256'),('queue.cjs.map','source_map_sha256')]:self.assertEqual(hashlib.sha256(gzip.decompress((folder/(name+'.gz')).read_bytes())).hexdigest(),p[key])
  original=subprocess.check_output(['git','show',s['reference_sha']+':src/app/importQueue.ts'],cwd=ROOT);self.assertEqual(hashlib.sha256(original).hexdigest(),p['source_sha256']);m=json.loads(gzip.decompress((folder/'queue.cjs.map.gz').read_bytes()));self.assertEqual(m['sourcesContent'],[original.decode()])
 def test_missing_result_frames_cannot_pass(self):
  s=load('2026-11-02-import-queue-source.json.gz');b=bytes.fromhex(s['cases'][0]['hex'])
  for bad in [b'',b'IQU1',b[:-1],b+b'\0']:
   with self.assertRaises(ValueError):frame(bad)
