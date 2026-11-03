# SPDX-License-Identifier: AGPL-3.0-only
import gzip
import hashlib
import json
import re
import subprocess
import unittest
from pathlib import Path
from schema_report import Reader

PLAN=Path(__file__).resolve().parents[1]
ROOT=PLAN.parents[1]
def load(name):return json.loads(gzip.decompress((PLAN/'evidence'/name).read_bytes()))
def frame(data):
 r=Reader(data)
 if r.take(4)!=b'HCP1':raise ValueError('Core history frame')
 value=r.value();r.end()
 if value[0]!=6 or len(value[1])!=5:raise ValueError('Five full observations required')
 for row in value[1]:
  if row[0]!=7 or [k for k,_ in row[1]]!=['error','snapshot','stores','scope','scan','participants','log']:raise ValueError('Observation keys/order')
 return value
class HistoryCoreCorpusTests(unittest.TestCase):
 def test_complete_independent_source_rust_fields(self):
  source=load('2026-11-02-history-core-source.json.gz');paired=load('2026-11-02-history-core.json.gz');manifest=json.loads((PLAN/'fixtures/history-core/manifest.json').read_text())
  self.assertEqual(len(source['cases']),98);self.assertEqual(paired['operation_observations'],490)
  self.assertEqual(hashlib.sha256((PLAN/'fixtures/history-core/manifest.json').read_bytes()).hexdigest(),source['manifest_sha256'])
  for case,frozen,entry in zip(paired['cases'],source['cases'],manifest['cases']):
   self.assertEqual(case['id'],frozen['id']);self.assertEqual(case['id'],entry['id']);values={}
   for name,item in case['files'].items():
    values[name]=bytes.fromhex(item['hex']);self.assertEqual(hashlib.sha256(values[name]).hexdigest(),item['sha256'])
   self.assertEqual(values['original.bin'],bytes.fromhex(frozen['hex']));self.assertEqual(values['rust.bin'],values['original.bin']);self.assertEqual(values['original.stderr.txt'],b'')
   self.assertEqual(values['input.bin'],(PLAN/'fixtures/history-core'/(case['id']+'.bin')).read_bytes());self.assertEqual(hashlib.sha256(values['input.bin']).hexdigest(),entry['sha256'])
   for row in frame(values['rust.bin'])[1]:
    values_=dict(row[1]);self.assertEqual(len(values_['stores'][1]),4)
    snapshot=values_['snapshot']
    if snapshot[0]==0:continue
    self.assertEqual([item[1][0][1]for item in snapshot[1]],['measure','flat','deviation','thickness','scope','scan'])
    for item,(family,keys)in zip(snapshot[1],manifest['fields'].items()):
     root=dict(item[1][1][1]);self.assertEqual([pair[1][0][1]for pair in root['fields'][1]],keys)
  ids=[c['id']for c in manifest['cases']];self.assertEqual(len(set(ids)),98);self.assertEqual(manifest['seeds'],[1,7,99,1039]);self.assertEqual(set(ids),{f.stem for f in (PLAN/'fixtures/history-core').glob('*.bin')})
  self.assertEqual(ids,re.findall(r'id: "([^"]+)"',(ROOT/'rust/crates/port-engine/tests/support/history_core_cases.rs').read_text()))
 def test_compiled_private_adapter_preserves_original_body(self):
  folder=PLAN/'evidence/history-core-oracle-2026-11-02';p=json.loads((folder/'provenance.json').read_text());source=load('2026-11-02-history-core-source.json.gz');self.assertEqual(p,source['provenance'])
  for name,key in [('history.mjs','bundle_sha256'),('history.mjs.map','source_map_sha256')]:self.assertEqual(hashlib.sha256(gzip.decompress((folder/(name+'.gz')).read_bytes())).hexdigest(),p[key])
  m=json.loads(gzip.decompress((folder/'history.mjs.map.gz').read_bytes()));contents=[text for name,text in zip(m['sources'],m['sourcesContent'])if name.endswith('/src/state/historyStore.ts')];self.assertEqual(len(contents),1)
  original=subprocess.check_output(['git','show',source['reference_sha']+':src/state/historyStore.ts'],cwd=ROOT).decode();self.assertEqual(contents[0],original+'\nexport { captureCore, apply }\n')
 def test_truncated_observations_cannot_pass(self):
  source=load('2026-11-02-history-core-source.json.gz');b=bytes.fromhex(source['cases'][0]['hex'])
  for broken in [b'',b'HCP1',b[:-1],b+b'\0']:
   with self.assertRaises(ValueError):frame(broken)
