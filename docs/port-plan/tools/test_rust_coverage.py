# SPDX-License-Identifier: AGPL-3.0-only
import copy,gzip,hashlib,json,unittest
from pathlib import Path
import rust_coverage as cov
ROOT=Path(__file__).resolve().parents[3];BUNDLE=ROOT/'docs/port-plan/evidence/rust-coverage-2026-11-02'
def load(name):
 p=BUNDLE/name
 return gzip.decompress(p.read_bytes()).decode() if name.endswith('.gz')else p.read_text()
class RustCoverageTests(unittest.TestCase):
 def test_exact_lcov_line_counts_and_invalid_artifacts(self):
  text='SF:/repo/rust/crates/port-core/src/example.rs\nDA:1,0\nDA:3,7\nLF:2\nLH:1\nend_of_record\n'
  f=cov.parse_lcov(text);self.assertEqual(next(iter(f.values()))['lines'],{1:0,3:7})
  for bad in ['',text.replace('LF:2','LF:1'),text.replace('LH:1','LH:3'),text.replace('DA:1,0','DA:1,-1'),text.replace('DA:3,7','DA:1,7'),text.replace('end_of_record','')]:
   with self.assertRaises(ValueError):cov.parse_lcov(bad)
 def test_regenerate_entire_committed_report_and_match_llvm_summary(self):
  meta=json.loads(load('run.json'));sources=json.loads(load('sources.json.gz'));lcov=load('coverage.lcov.gz');actual=cov.summarize(lcov,meta,sources);stored=json.loads(load('report.json'));self.assertEqual(actual,stored);self.assertEqual(cov.markdown(actual),load('report.md'));self.assertFalse(actual['ordinary_tests_included']);self.assertFalse(actual['acceptance_claimed']);self.assertEqual(actual['native_suites'],26);self.assertEqual(actual['native_cases'],2015)
  raw=json.loads(load('llvm-export.json.gz'));by_path={}
  for block in raw['data']:
   for f in block['files']:
    if '/rust/crates/'in f['filename']:by_path['rust/crates/'+f['filename'].split('/rust/crates/',1)[1]]=f['summary']['lines']
  for f in actual['files']:
   self.assertEqual(f['sha256'],hashlib.sha256(next(s['text']for s in sources if s['path']==f['path']).encode()).hexdigest())
   if f['instrumented_lines']:
    self.assertEqual(f['llvm_summary_lines'],by_path[f['path']]['count']);self.assertEqual(f['llvm_summary_hits'],by_path[f['path']]['covered']);self.assertEqual(f['line_summary_disagrees_with_explicit_records'],f['instrumented_lines']!=f['llvm_summary_lines'] or len(f['reached'])!=f['llvm_summary_hits'])
   self.assertFalse(set(f['reached'])&set(f['unreached']));self.assertFalse(set(f['uninstrumented'])&set(f['reached']+f['unreached']))
  self.assertTrue(any(f['unreached']for f in actual['files']));self.assertIn('Unavailable',actual['branch_metric'])
 def test_changed_counts_and_missing_comparisons_cannot_pass(self):
  meta=json.loads(load('run.json'));sources=json.loads(load('sources.json.gz'));lcov=load('coverage.lcov.gz');old=cov.summarize(lcov,meta,sources)
  # Force all owned line counts to zero with internally consistent LCOV totals.
  out=[]
  for line in lcov.splitlines():
   if line.startswith('DA:'):line=line.split(',')[0]+',0'
   if line.startswith('LH:'):line='LH:0'
   out.append(line)
  changed=cov.summarize('\n'.join(out),meta,sources);self.assertNotEqual(old,changed);self.assertTrue(all(not f['reached']for f in changed['files']))
  bad=copy.deepcopy(meta);bad['native_cases']=0
  with self.assertRaises(ValueError):cov.summarize(lcov,bad,sources)
  bad=copy.deepcopy(sources);bad[0]['text']=''
  if next(f for f in old['files']if f['path']==bad[0]['path'])['instrumented_lines']:
   with self.assertRaises(ValueError):cov.summarize(lcov,meta,bad)
 def test_manifest_and_exact_native_evidence_scope(self):
  m=json.loads(load('manifest.json'))
  for name,item in m['files'].items():
   b=(BUNDLE/name).read_bytes();self.assertEqual(len(b),item['bytes']);self.assertEqual(hashlib.sha256(b).hexdigest(),item['sha256'])
  e=json.loads(load('native-evidence.json.gz'));run=json.loads(load('run.json'));self.assertEqual(e['candidate_sha'],run['candidate_sha']);self.assertEqual(e['reference_sha'],run['reference_sha']);self.assertEqual(len(e['suites']),26);self.assertEqual(sum(len(s['cases'])for s in e['suites']),2015)
  for s in e['suites']:
   self.assertTrue(s['cases']);self.assertEqual(len(s['cases']),len(set(c['id']for c in s['cases'])))
   for c in s['cases']:self.assertIn('input.bin',c['files']);self.assertIn('original.bin',c['files'])
