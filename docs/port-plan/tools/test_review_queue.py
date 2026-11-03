# SPDX-License-Identifier: AGPL-3.0-only
import copy
import json
import unittest
import review_queue as q


class ReviewQueueTests(unittest.TestCase):
    def fixture(self):
        checks=[{'name':name,'id':i,'status':'COMPLETED','conclusion':'SUCCESS','url':'https://example.invalid/check'}for i,name in enumerate(q.CHECKS)]
        def row(number,base,ref):return {'number':number,'title':'Synthetic | row','url':f'https://github.com/{q.REPO}/pull/{number}','isDraft':False,'baseRefName':base,'headRefName':ref,'headRefOid':str(number)*40,'statusCheckRollup':copy.deepcopy(checks)}
        return {'schema':'port.review.queue.v1','reference_sha':q.PIN,'main_sha':'a'*40,'observed_at_utc':'2026-11-03 12:00:00 UTC','excluded_handoff_pr':84,'extra_dependencies':{},'required_checks':list(q.CHECKS),'prs':[row(1,'main','one'),row(2,'one','two')]}

    def test_positive_graph_and_escaped_document(self):
        data=self.fixture();self.assertEqual(q.validate(data),{1:[],2:[1]});text=q.render(data);self.assertIn('Synthetic \\| row',text);self.assertEqual(text.count('11 green'),2)

    def test_snapshot_identity_pin_and_required_checks(self):
        for key,value in [('main_sha','short'),('reference_sha','b'*40),('schema','unknown'),('observed_at_utc','recently'),('required_checks',q.CHECKS[:-1])]:
            with self.subTest(key=key):
                data=self.fixture();data[key]=value
                with self.assertRaises(ValueError):q.validate(data)

    def test_missing_duplicate_and_self_referential_records(self):
        data=self.fixture();data['prs'][1]['baseRefName']='missing'
        with self.assertRaisesRegex(ValueError,'Missing open base'):q.validate(data)
        for field in ['number','headRefName']:
            data=self.fixture();data['prs'][1][field]=data['prs'][0][field]
            with self.assertRaises(ValueError):q.validate(data)
        data=self.fixture();data['excluded_handoff_pr']=1
        with self.assertRaises(ValueError):q.validate(data)

    def test_base_and_extra_dependency_cycles_fail(self):
        data=self.fixture();data['prs'][0]['baseRefName']='two'
        with self.assertRaisesRegex(ValueError,'cycle'):q.validate(data)
        data=self.fixture();data['extra_dependencies']={'1':[2]}
        with self.assertRaisesRegex(ValueError,'cycle'):q.validate(data)
        data=self.fixture();data['extra_dependencies']={'1':[99]}
        with self.assertRaisesRegex(ValueError,'Missing integration parent'):q.validate(data)

    def test_missing_pending_skipped_or_red_check_is_never_green(self):
        for kind in ['missing','pending','skipped','failed']:
            data=self.fixture();p=data['prs'][0]
            if kind=='missing':p['statusCheckRollup'].pop()
            elif kind=='pending':p['statusCheckRollup'][0].update(status='IN_PROGRESS',conclusion='')
            else:p['statusCheckRollup'][0]['conclusion']='SKIPPED'if kind=='skipped'else'FAILURE'
            self.assertNotEqual(q.check_state(p),'11 green');q.validate(data)

    def test_ambiguous_check_or_unfinished_conclusion_is_rejected(self):
        data=self.fixture();data['prs'][0]['statusCheckRollup'].append(data['prs'][0]['statusCheckRollup'][0])
        with self.assertRaisesRegex(ValueError,'duplicate check'):q.validate(data)
        data=self.fixture();data['prs'][0]['statusCheckRollup'][0]['status']='IN_PROGRESS'
        with self.assertRaisesRegex(ValueError,'Unfinished'):q.validate(data)


    def test_merged_parent_needs_exact_identity_and_main_ancestry(self):
        data=self.fixture();data['extra_dependencies']={'2':[3]};data['merged_dependencies']={'3':{'number':3,'head_sha':'c'*40,'merge_sha':'d'*40,'merged_at':'2026-11-03T10:00:00Z','main_contains':True,'url':f'https://github.com/{q.REPO}/pull/3'}}
        q.validate(data);self.assertIn('#3 (merged)',q.render(data))
        for key,value in [('main_contains',False),('merge_sha','short'),('merged_at',None)]:
            with self.subTest(key=key):
                bad=copy.deepcopy(data);bad['merged_dependencies']['3'][key]=value
                with self.assertRaises(ValueError):q.validate(bad)

    def test_actual_snapshot_and_table_preserve_all_heads_and_integration_edges(self):
        data=json.loads(q.SNAPSHOT.read_text());q.validate(data);self.assertEqual(q.DOCUMENT.read_text(),q.render(data));self.assertEqual(data['extra_dependencies']['75'],[74,69,64])
        text=q.DOCUMENT.read_text()
        for p in data['prs']:
            self.assertIn(p['headRefOid'],text);self.assertIn(f"[#{p['number']}]",text)
        bad=copy.deepcopy(data);bad['extra_dependencies'].pop('75')
        with self.assertRaisesRegex(ValueError,'Missing known integration'):q.validate(bad)
