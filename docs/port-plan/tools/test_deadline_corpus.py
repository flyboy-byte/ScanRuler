# SPDX-License-Identifier: AGPL-3.0-only
import gzip,hashlib,json,re,struct,subprocess,unittest
from pathlib import Path
from schema_report import Reader
PLAN=Path(__file__).resolve().parents[1];ROOT=PLAN.parents[1]
def load(name):return json.loads(gzip.decompress((PLAN/'evidence'/name).read_bytes()))
def frame(data):
 r=Reader(data)
 if r.take(4)!=b'CDL1':raise ValueError('Deadline frame magic')
 root=r.value();r.end()
 if root[0]!=7 or [k for k,_ in root[1]]!=['defaultTimeoutMs','frames']:raise ValueError('Deadline root fields')
 root=dict(root[1]);frames=root['frames']
 if frames[0]!=6 or not frames[1]:raise ValueError('Empty deadline trace')
 for row in frames[1]:
  if row[0]!=7 or [k for k,_ in row[1]]!=['caller','body','doc','aborted','reason','timerActive','log','unhandled']:raise ValueError('Deadline state shape/order')
 return root
class DeadlineCorpusTests(unittest.TestCase):
 def test_full_independent_source_and_rust_state(self):
  s=load('2026-11-02-deadline-source-291.json.gz');p=load('2026-11-02-deadline.json.gz');m=json.loads((PLAN/'fixtures/command-deadline/manifest.json').read_text());self.assertEqual(len(s['cases']),291);self.assertEqual(p['operation_observations'],2037);self.assertEqual((p['successful_callers'],p['failed_callers'],p['aborted_successful_callers']),(104,187,69));self.assertEqual(m['seeds'],[1,7,99,1039])
  self.assertEqual(hashlib.sha256((PLAN/'fixtures/command-deadline/manifest.json').read_bytes()).hexdigest(),s['manifest_sha256'])
  for case,source,fixture in zip(p['cases'],s['cases'],m['cases']):
   self.assertEqual(case['id'],source['id']);self.assertEqual(case['id'],fixture['id']);values={}
   for name,item in case['files'].items():
    values[name]=bytes.fromhex(item['hex']);self.assertEqual(hashlib.sha256(values[name]).hexdigest(),item['sha256'])
   self.assertEqual(values['original.bin'],bytes.fromhex(source['hex']));self.assertEqual(values['rust.bin'],values['original.bin']);self.assertEqual(values['original.stderr.txt'],b'');self.assertEqual(values['input.bin'],(PLAN/'fixtures/command-deadline'/(case['id']+'.bin')).read_bytes());self.assertEqual(hashlib.sha256(values['input.bin']).hexdigest(),fixture['sha256'])
   root=frame(values['rust.bin']);self.assertEqual(root['defaultTimeoutMs'],(4,struct.pack('<d',600000).hex()))
   for row in root['frames'][1]:self.assertEqual(dict(row[1])['unhandled'],(6,[]))
   last=dict(root['frames'][1][-1][1]);self.assertNotEqual(dict(last['caller'][1])['status'][1],'pending');self.assertFalse(last['timerActive'][0]==3)
  ids=[c['id']for c in m['cases']];self.assertEqual(len(set(ids)),291);self.assertEqual(set(ids),{f.stem for f in (PLAN/'fixtures/command-deadline').glob('*.bin')});self.assertEqual(ids,re.findall(r'id: "([^"]+)"',(ROOT/'rust/crates/port-engine/tests/support/deadline_cases.rs').read_text()))
 def test_all_earlier_inputs_and_passing_values_retained(self):
  old=load('2026-11-02-deadline-source.json.gz');now=load('2026-11-02-deadline-source-291.json.gz');self.assertEqual(old['cases'],now['cases'][:288]);first=load('2026-11-02-deadline-288.json.gz');paired=load('2026-11-02-deadline.json.gz')
  for before,after in zip(first['cases'],paired['cases']):self.assertEqual(before['id'],after['id']);self.assertEqual(before['rust_hex'],after['files']['rust.bin']['hex'])
  before=json.loads((PLAN/'fixtures/command-deadline/manifest-288.json').read_text());after=json.loads((PLAN/'fixtures/command-deadline/manifest.json').read_text());self.assertEqual(before['cases'],after['cases'][:288]);self.assertEqual(before['seeds'],after['seeds'])
 def test_full_original_body_and_archive_identity(self):
  folder=PLAN/'evidence/deadline-oracle-2026-11-02';p=json.loads((folder/'provenance.json').read_text());s=load('2026-11-02-deadline-source-291.json.gz');self.assertEqual(p,s['provenance'])
  for name,key in [('registry.mjs','bundle_sha256'),('registry.mjs.map','source_map_sha256')]:self.assertEqual(hashlib.sha256(gzip.decompress((folder/(name+'.gz')).read_bytes())).hexdigest(),p[key])
  m=json.loads(gzip.decompress((folder/'registry.mjs.map.gz').read_bytes()));contents=[text for name,text in zip(m['sources'],m['sourcesContent'])if name.endswith('/src/commands/registry.ts')];original=subprocess.check_output(['git','show',s['reference_sha']+':src/commands/registry.ts'],cwd=ROOT).decode();self.assertEqual(contents,[original+'\nexport { limited }\n'])
 def test_missing_state_or_results_cannot_pass(self):
  s=load('2026-11-02-deadline-source-291.json.gz');b=bytes.fromhex(s['cases'][0]['hex'])
  for bad in [b'',b'CDL1',b[:-1],b+b'\0']:
   with self.assertRaises(ValueError):frame(bad)
