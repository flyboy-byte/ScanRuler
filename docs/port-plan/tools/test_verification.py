# SPDX-License-Identifier: AGPL-3.0-only
"""Synthetic metadata attacks and positive schema controls, not runtime parity evidence."""
import copy
import hashlib
import io
import json
from contextlib import redirect_stdout
from pathlib import Path
import tempfile
import unittest

import port_plan
import verification as v
import next_task
import ci_report


class Fixture:
    def __init__(self, root):
        self.root=Path(root);self.plan=self.root/'docs/port-plan';self.folder=self.plan/'evidence';self.folder.mkdir(parents=True)
        self.source='rust/crates/example/src/lib.rs';p=self.root/self.source;p.parent.mkdir(parents=True);p.write_text('// synthetic metadata fixture\n')
        self.pin='a'*40;self.sha='c'*40;self.identity='math.example'
        self.unit={'id':self.identity,'title':'Synthetic','scope':'Metadata only','acceptance':['Schema controls'],'status':'implemented','reference_sha':self.pin,'implemented_commit':self.sha,'depends_on':[],'open_items':[],'rust_paths':[self.source],'verification':'fixed-corpus','evidence':[{'kind':'differential','location':'evidence/fixed.json','candidate_sha':self.sha,'reference_sha':self.pin,'case_ids':['fixed-case']}]}
        (self.folder/'fixed.json').write_text('{}')
        self.common={'candidate_sha':self.sha,'reference_sha':self.pin,'scope_units':[self.identity],'source_hashes':{self.source:hashlib.sha256(p.read_bytes()).hexdigest()}}
        records=[]
        for sabotage in [False,True]:
            row={'unit':'example','seed':1,'cases':1,'identical_success':int(not sabotage),'identical_refusal':0,'host_limit':0,'original_timeout':0,'divergences':int(sabotage),'sabotage':sabotage,'log':('canary'if sabotage else'normal')+'.log'}
            self.log(row);records.append(row)
        raw={'input.bin':b'input','original.bin':b'a','rust.bin':b'b','original.status.txt':b'ok=true timeout=false\n','rust.status.txt':b'ok=true timeout=false\n','original.message.txt':b'','rust.message.txt':b'','original.progress.txt':b'','rust.progress.txt':b''}
        files={name:{'hex':data.hex(),'sha256':hashlib.sha256(data).hexdigest()}for name,data in raw.items()}
        self.fuzz={**self.common,'schema':'port.verification.fuzz.v1','unit_families':{self.identity:'example'},'observations':v.OBSERVATIONS,'records':records,'canary_reproductions':[{'unit':'example','files':files}]}
        self.write('fuzz',self.fuzz)
        for name in ['original.json','rust.json']:(self.folder/name).write_text('{"synthetic":true}')
        self.review={**self.common,'schema':'port.verification.review.v1','case_ids':['fixed-case'],'profile':'comparison-only','reviewed_by':'synthetic-metadata-control','unknown_reach':False,'unresolved':[]}
        for name in ['original','rust']:self.review[name+'_coverage']={'location':name+'.json','sha256':hashlib.sha256((self.folder/(name+'.json')).read_bytes()).hexdigest()}
        self.write('review',self.review)
        self.unit['verification_evidence']=[{'kind':'fuzz-campaign','location':'evidence/fuzz.json'},{'kind':'coverage-review','location':'evidence/review.json'}]
        self.meta={'baseline':self.pin,'statuses':['planned','implemented','compared','integrated','accepted']}

    def log(self,row):
        raw=('fuzz unit={unit} seed={seed} cases={cases} identical_success={identical_success} identical_refusal={identical_refusal} host_limit={host_limit} original_timeout={original_timeout} divergences={divergences}\n'.format(**row)).encode()
        (self.folder/row['log']).write_bytes(raw);row['log_sha256']=hashlib.sha256(raw).hexdigest()

    def write(self,name,value):(self.folder/(name+'.json')).write_text(json.dumps(value))
    def errors(self,unit=None):return port_plan.validate(self.meta,[unit or self.unit],self.plan)


class VerificationTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup);self.f=Fixture(self.temp.name)

    def bad(self,fragment=None):
        errors=self.f.errors();self.assertTrue(errors)
        if fragment:self.assertTrue(any(fragment in e for e in errors),errors)

    def test_explicit_none_and_fixed_positive(self):
        self.assertFalse(self.f.errors());self.f.unit['verification']='none';self.f.unit['evidence']=[];self.assertFalse(self.f.errors())

    def test_unknown_and_missing_level(self):
        for value in [None,'compiled','fuzzed ',42]:
            with self.subTest(value=value):self.f.unit['verification']=value;self.bad('verification level')

    def test_empty_fixed_and_wrong_reference(self):
        self.f.unit['evidence'][0]['case_ids']=[];self.bad()
        self.f.unit['evidence'][0]['case_ids']=['case'];self.f.unit['evidence'][0]['reference_sha']='b'*40;self.bad()

    def tooling(self):
        self.f.unit['evidence'] = []
        raw = b'case (suite.case) ... ok\n\nRan 1 test in 0.001s\n\nOK\n'
        (self.f.folder/'tooling.log').write_bytes(raw)
        report = {**self.f.common, 'schema':'port.verification.fixed.v1', 'case_ids':['suite.case'], 'passed':True, 'runner':'python-unittest', 'log':'tooling.log', 'log_sha256':hashlib.sha256(raw).hexdigest()}
        self.f.write('tooling',report)
        self.f.unit['verification_evidence'].append({'kind':'fixed-corpus','location':'evidence/tooling.json'})
        return report

    def test_fixed_tooling_positive_and_log_hash(self):
        report = self.tooling(); self.assertFalse(self.f.errors())
        report['log_sha256']='0'*64; self.f.write('tooling',report); self.bad('tooling log hash')

    def test_fixed_tooling_rejects_zero_failed_or_wrong_case_run(self):
        report = self.tooling()
        for raw in [b'Ran 0 tests in 0.001s\n\nOK\n', b'case (suite.case) ... FAIL\n\nRan 1 test in 0.001s\n\nFAILED (failures=1)\n', b'case (suite.other) ... ok\n\nRan 1 test in 0.001s\n\nOK\n']:
            with self.subTest(raw=raw):
                (self.f.folder/'tooling.log').write_bytes(raw)
                report['log_sha256']=hashlib.sha256(raw).hexdigest();self.f.write('tooling',report);self.bad('positive passing case')

    def test_truncated_gzip_is_validation_error(self):
        self.f.unit['verification']='fuzzed'
        (self.f.folder/'truncated.gz').write_bytes(b'\x1f\x8b\x08')
        self.f.unit['verification_evidence'][0]['location']='evidence/truncated.gz';self.bad('unsupported fuzzed')

    def test_positive_fuzz_and_reviewed_accepted_metadata(self):
        self.f.unit['verification']='fuzzed';self.assertFalse(self.f.errors())
        self.f.unit.update(verification='coverage-reviewed',status='accepted');self.assertFalse(self.f.errors())

    def test_accepted_lower_grades_never_pass(self):
        self.f.unit['status']='accepted'
        for level in ['none','fixed-corpus','fuzzed']:
            with self.subTest(level=level):self.f.unit['verification']=level;self.bad('accepted requires')

    def test_missing_fuzz_link_and_no_fixed_basis(self):
        self.f.unit['verification']='fuzzed';self.f.unit['verification_evidence']=[];self.bad('fuzz-campaign')
        self.f.unit['evidence']=[];self.bad()

    def test_wrong_fuzz_scope_pin_candidate_and_schema(self):
        self.f.unit['verification']='fuzzed'
        for key,value in [('schema','raw-coverage'),('scope_units',[]),('reference_sha','b'*40),('candidate_sha',None)]:
            with self.subTest(key=key):
                record=copy.deepcopy(self.f.fuzz);record[key]=value;self.f.write('fuzz',record);self.bad()

    def test_status_only_fuzz_not_supported(self):
        self.f.unit['verification']='fuzzed';self.f.fuzz['observations']=['success'];self.f.write('fuzz',self.f.fuzz);self.bad('full declared')

    def test_zero_and_nonpartitioning_fuzz_counts(self):
        self.f.unit['verification']='fuzzed';row=self.f.fuzz['records'][0];row['cases']=0;self.f.log(row);self.f.write('fuzz',self.f.fuzz);self.bad()
        row['cases']=2;self.f.log(row);self.f.write('fuzz',self.f.fuzz);self.bad('partition')

    def test_ordinary_divergence_is_not_fuzzed(self):
        self.f.unit['verification']='fuzzed';row=self.f.fuzz['records'][0];row['cases']=2;row['divergences']=1;self.f.log(row);self.f.write('fuzz',self.f.fuzz);self.bad('ordinary fuzz')

    def test_refusals_or_host_limits_alone_are_not_successes(self):
        self.f.unit['verification']='fuzzed';row=self.f.fuzz['records'][0];row['identical_success']=0;row['host_limit']=1;self.f.log(row);self.f.write('fuzz',self.f.fuzz);self.bad('positive successful')

    def test_missing_or_nonfailing_canary(self):
        self.f.unit['verification']='fuzzed';self.f.fuzz['records']=self.f.fuzz['records'][:1];self.f.write('fuzz',self.f.fuzz);self.bad()

    def test_canary_zero_divergences_rejected(self):
        self.f.unit['verification']='fuzzed';row=self.f.fuzz['records'][1];row.update(divergences=0,identical_success=1);self.f.log(row);self.f.write('fuzz',self.f.fuzz);self.bad('canary found no')

    def test_missing_corruption_payload_is_not_canary_evidence(self):
        self.f.unit['verification']='fuzzed';self.f.fuzz['canary_reproductions']=[];self.f.write('fuzz',self.f.fuzz);self.bad('actual corruption')

    def test_identical_canary_payloads_are_rejected(self):
        self.f.unit['verification']='fuzzed';files=self.f.fuzz['canary_reproductions'][0]['files'];files['rust.bin']=copy.deepcopy(files['original.bin']);self.f.write('fuzz',self.f.fuzz);self.bad('no value/state difference')

    def test_canary_payload_hash_and_missing_fields(self):
        self.f.unit['verification']='fuzzed';files=self.f.fuzz['canary_reproductions'][0]['files'];files['rust.bin']['sha256']='0'*64;self.f.write('fuzz',self.f.fuzz);self.bad('artifact hash')
        del files['rust.bin'];self.f.write('fuzz',self.f.fuzz);self.bad('incomplete canary')

    def test_log_hash_and_counter_spoofing(self):
        self.f.unit['verification']='fuzzed';row=self.f.fuzz['records'][0];row['log_sha256']='0'*64;self.f.write('fuzz',self.f.fuzz);self.bad('log hash')
        self.f.log(row);row['seed']=7;self.f.write('fuzz',self.f.fuzz);self.bad('support counters')

    def test_duplicate_campaign_cannot_inflate_evidence(self):
        self.f.unit['verification']='fuzzed';self.f.fuzz['records'].append(copy.deepcopy(self.f.fuzz['records'][0]));self.f.write('fuzz',self.f.fuzz);self.bad('duplicate campaign')

    def test_stale_source_or_generator_invalidates_high_grade(self):
        self.f.unit['verification']='fuzzed';(self.f.root/self.f.source).write_text('changed');self.bad('stale source')

    def test_missing_owned_source_hash_rejected(self):
        self.f.unit['verification']='fuzzed';other=self.f.root/'other.rs';other.write_text('other');self.f.fuzz['source_hashes']={'other.rs':hashlib.sha256(other.read_bytes()).hexdigest()};self.f.write('fuzz',self.f.fuzz);self.bad('omits owned')

    def test_unsafe_evidence_path_rejected(self):
        self.f.unit['verification']='fuzzed';self.f.unit['verification_evidence'][0]['location']='../outside.json';self.bad('unsafe')

    def test_coverage_requires_review_beyond_raw_counts(self):
        self.f.unit['verification']='coverage-reviewed'
        for key,value in [('schema','llvm-lines'),('reviewed_by',''),('unresolved',['zero-count line']),('unknown_reach',True),('profile','ordinary-tests')]:
            with self.subTest(key=key):record=copy.deepcopy(self.f.review);record[key]=value;self.f.write('review',record);self.bad()

    def test_coverage_needs_both_unchanged_engine_profiles(self):
        self.f.unit['verification']='coverage-reviewed';del self.f.review['original_coverage'];self.f.write('review',self.f.review);self.bad('both engine')
        self.f.write('review',{**self.f.review,'original_coverage':{'location':'original.json','sha256':'0'*64}});self.bad('profile missing or changed')

    def test_existing_accepted_open_item_and_dependency_gates_remain(self):
        self.f.unit.update(status='accepted',verification='coverage-reviewed',open_items=['unimplemented success continuation']);self.bad('accepted with open')
        self.f.unit['open_items']=[];self.f.unit['depends_on']=['math.missing'];self.bad('hard dependency')

    def test_task_picker_prints_verification(self):
        unit=copy.deepcopy(self.f.unit);unit['verification']='fuzzed';row=(1,0,unit['id'],unit,{'id':'C','name':'Harness','touches':''});out=io.StringIO()
        with redirect_stdout(out):next_task.show(row,{unit['id']:unit})
        self.assertIn('verification: fuzzed',out.getvalue())

    def test_generated_status_distinguishes_lifecycle_and_evidence_snapshot(self):
        report=ci_report.status();self.assertEqual(sum(report['verification_levels'].values()),len(report['units']))
        self.assertEqual(report['accepted_units'],[])
        by_id={u['id']:u for u in report['units']}
        self.assertEqual(by_id['io.ply.ascii']['verification'],'fuzzed')
        self.assertNotEqual(by_id['io.ply.ascii']['tested_sha'],by_id['io.ply.ascii']['verification_tested_sha'])
        self.assertIn('| Verification |',ci_report.status_markdown(report))
