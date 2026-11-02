# SPDX-License-Identifier: AGPL-3.0-only
import gzip,hashlib,json,re,struct,unittest
from pathlib import Path
import registry_report as rr
from generate_registry_cases import untag
from generate_schema_cases import wire
ROOT=Path(__file__).resolve().parents[3];PLAN=ROOT/'docs/port-plan'
class RegistryCorpusTests(unittest.TestCase):
 def test_complete_independent_source_rust_frames_and_refused_coercion(self):
  report=rr.load(PLAN/'evidence/2026-11-02-registry.json.gz');source=rr.load(PLAN/'evidence/2026-11-02-registry-source-350.json.gz');self.assertEqual(report['scenario_count'],350);self.assertEqual(report['application_commands_implemented'],0);self.assertFalse(report['browser_runtime_claimed'])
  for c,s in zip(report['cases'],source['cases']):
   self.assertEqual(c['id'],s['id']);o=bytes.fromhex(s['hex']);r=bytes.fromhex(c['files']['rust.bin']['hex']);rr.compare(o,r)
   self.assertEqual(c['files']['original.bin']['sha256'],hashlib.sha256(o).hexdigest())
   for name,item in c['files'].items():
    b=(PLAN/'fixtures/registry'/(c['id']+'.bin')).read_bytes()if name=='input.bin'else o if name=='original.bin'else bytes.fromhex(item['hex']);self.assertEqual(len(b),item['bytes']);self.assertEqual(hashlib.sha256(b).hexdigest(),item['sha256'])
   if c['id']=='refused-coercion':
    states=rr.frame(r)[1]
    for i,tag in [(1,3),(2,2)]:
     f=rr.fields(states[i]);self.assertEqual(rr.fields(f['error'])['code'][1],'invalid_input');self.assertEqual(rr.fields(f['afterInput'])['flag'][0],tag);self.assertEqual(f['target'][0],0)
 def test_no_old_observation_removed_when_fields_and_case_added(self):
  first=rr.load(PLAN/'evidence/2026-11-02-registry-source.json.gz');second=rr.load(PLAN/'evidence/2026-11-02-registry-source-v2.json.gz');third=rr.load(PLAN/'evidence/2026-11-02-registry-source-v3.json.gz');now=rr.load(PLAN/'evidence/2026-11-02-registry-source-350.json.gz');self.assertEqual(third['cases'],now['cases'][:209]);previous=rr.load(PLAN/'evidence/2026-11-02-registry-source-210.json.gz');self.assertEqual(previous['cases'],now['cases'][:210])
  for a,b,c in zip(first['cases'],second['cases'],third['cases']):
   root=rr.frame(bytes.fromhex(c['hex']));old1=(6,[(7,[(k,v)for k,v in row[1]if k not in ['afterInput','target']])for row in root[1]]);old2=(6,[(7,[(k,v)for k,v in row[1]if k!='target'])for row in root[1]])
   self.assertEqual(b'REG1'+wire(untag(old1)),bytes.fromhex(a['hex']));self.assertEqual(b'REG2'+wire(untag(old2)),bytes.fromhex(b['hex']))
 def test_fixed_inputs_all_definitions_hashes_and_compiled_schedule(self):
  manifest=json.loads((PLAN/'fixtures/registry/manifest.json').read_text());ids=[c['id']for c in manifest['cases']];self.assertEqual(len(ids),350);self.assertEqual(len(set(ids)),350);self.assertEqual(manifest['seeds'],[1,7,99,1039]);self.assertEqual(set(ids),{p.stem for p in (PLAN/'fixtures/registry').glob('*.bin')});self.assertEqual(re.findall(r'id: "([^"]+)"',(ROOT/'rust/crates/port-engine/tests/support/registry_cases.rs').read_text()),ids)
  catalog=json.loads(gzip.decompress((PLAN/'evidence/browser-2026-11-01/all.json.gz').read_bytes()));names={c['name']for c in catalog['commands']};self.assertEqual({i[8:]for i in ids if i.startswith('command-')},names)
  for c in manifest['cases']:
   b=(PLAN/'fixtures/registry'/(c['id']+'.bin')).read_bytes();self.assertEqual(len(b),c['bytes']);self.assertEqual(hashlib.sha256(b).hexdigest(),c['sha256']);r=rr.Reader(b);r.value();r.end()
  for name,item in json.loads((PLAN/'evidence/2026-11-02-registry-source-manifest.json').read_text()).items():
   b=(PLAN/'evidence'/name).read_bytes();raw=gzip.decompress(b);self.assertEqual(len(b),item['bytes']);self.assertEqual(hashlib.sha256(b).hexdigest(),item['sha256']);self.assertEqual(len(raw),item['raw_bytes']);self.assertEqual(hashlib.sha256(raw).hexdigest(),item['raw_sha256'])
  folder=PLAN/'evidence/registry-oracle-2026-11-02';provenance=json.loads((folder/'provenance.json').read_text());source=rr.load(PLAN/'evidence/2026-11-02-registry-source-350.json.gz')
  for name,key in [('registry.mjs','bundle_sha256'),('registry.mjs.map','source_map_sha256')]:self.assertEqual(hashlib.sha256(gzip.decompress((folder/(name+'.gz')).read_bytes())).hexdigest(),provenance[key])
  self.assertEqual(source['bundle_sha256'],provenance['bundle_sha256']);self.assertEqual(source['package_lock_sha256'],provenance['package_lock_sha256']);self.assertEqual(provenance['collator']['locale'],'en-US');self.assertEqual(provenance['reference_sha'],source['reference_sha'])
 def test_changed_real_metadata_error_boolean_and_missing_tail_rejected(self):
  d=rr.load(PLAN/'evidence/2026-11-02-registry-source-350.json.gz');b=bytes.fromhex(next(c['hex']for c in d['cases']if c['id']=='unknown-input'))
  for text in ['Probe 🦀','There is no command']:
   changed=bytearray(b);at=changed.index(text.encode('utf-16-le'));changed[at]^=1;rr.frame(bytes(changed))
   with self.assertRaises(ValueError):rr.compare(b,bytes(changed))
  key='flag'.encode('utf-16-le')+bytes([3]);changed=bytearray(b);at=changed.index(key)+len(key)-1;changed[at]=2;rr.frame(bytes(changed))
  with self.assertRaises(ValueError):rr.compare(b,bytes(changed))
  for bad in [b'',b'REG3',b[:-1],b+bytes([0])]:
   with self.assertRaises(ValueError):rr.frame(bad)
