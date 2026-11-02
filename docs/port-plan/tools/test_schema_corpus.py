# SPDX-License-Identifier: AGPL-3.0-only
import gzip,hashlib,json,re,unittest
from pathlib import Path
import schema_report as report
from generate_schema_cases import wire
ROOT=Path(__file__).resolve().parents[3];PLAN=ROOT/'docs/port-plan'
class SchemaCorpusTests(unittest.TestCase):
 def test_complete_raw_frames_match_independent_original_and_mutants_fail(self):
  d=json.loads((PLAN/'evidence/2026-11-02-schema.json').read_text());source=json.loads((PLAN/'evidence/2026-11-02-schema-source-expanded.json').read_text());self.assertEqual(len(d['cases']),697);self.assertEqual(d['counts'],{'accepted':320,'refused':376,'source-fault':1});self.assertEqual(d['working_commands_claimed'],0);self.assertFalse(d['browser_runtime_claimed'])
  for c,s in zip(d['cases'],source['cases']):
   self.assertEqual(c['id'],s['id']);f=c['files'];o=bytes.fromhex(f['original.bin']['hex']);r=bytes.fromhex(f['rust.bin']['hex']);report.compare(bytes.fromhex(s['hex']),o);report.compare(o,r)
   for name,item in f.items():
    b=(PLAN/'fixtures/schema'/(c['id']+'.bin')).read_bytes() if name=='input.bin' else bytes.fromhex(item['hex']);self.assertEqual(len(b),item['bytes']);self.assertEqual(hashlib.sha256(b).hexdigest(),item['sha256'])
   if c['id']=='type-number-text':
    changed=bytearray(r);changed[16]^=1
    with self.assertRaises(ValueError):report.compare(o,bytes(changed))
    with self.assertRaises(ValueError):report.compare(o,r[:-1])
 def test_fixed_inputs_case_ids_and_no_lost_original_observations(self):
  m=json.loads((PLAN/'fixtures/schema/manifest.json').read_text());ids=[c['id'] for c in m['cases']];self.assertEqual(len(ids),697);self.assertEqual(len(set(ids)),697);self.assertEqual(m['seeds'],[1,7,99,1039]);self.assertEqual(m['commands'],140)
  self.assertEqual(set(ids),{p.stem for p in (PLAN/'fixtures/schema').glob('*.bin')})
  table=(ROOT/'rust/crates/port-engine/tests/support/schema_cases.rs').read_text();self.assertEqual(re.findall(r'id: "([^"]+)"',table),ids)
  for c in m['cases']:
   b=(PLAN/'fixtures/schema'/(c['id']+'.bin')).read_bytes();self.assertEqual(len(b),c['bytes']);self.assertEqual(hashlib.sha256(b).hexdigest(),c['sha256']);r=report.Reader(b);r.value();r.value();r.text();r.end()
  first=json.loads((PLAN/'evidence/2026-11-02-schema-source.json').read_text());expanded=json.loads((PLAN/'evidence/2026-11-02-schema-source-expanded.json').read_text());self.assertEqual(first['cases'],expanded['cases'][:659])
 def test_all_real_command_schemas_have_success_and_refusal(self):
  core=json.loads((ROOT/'mcp/src/commands.json').read_text())['commands'];names={c['name'] for c in core};catalog=core+[c for c in json.loads(gzip.decompress((PLAN/'evidence/browser-2026-11-01/all.json.gz').read_bytes()))['commands'] if c['name'] not in names]
  states={c['id']:c['state'] for c in json.loads((PLAN/'evidence/2026-11-02-schema.json').read_text())['cases']};self.assertEqual(len(catalog),140)
  for c in catalog:
   for intent,state in [('accepted','accepted'),('full','accepted'),('refused','refused')]:
    id='command-'+c['name']+'-'+intent;self.assertEqual(states[id],state);self.assertTrue((PLAN/'fixtures/schema'/(id+'.bin')).read_bytes().startswith(wire(c['input'])))
 def test_malformed_frames_cannot_look_like_empty_success(self):
  for b in [b'',b'VLD1',b'VLD1'+b'\xff'*20]:
   with self.assertRaises(ValueError):report.frame(b)
