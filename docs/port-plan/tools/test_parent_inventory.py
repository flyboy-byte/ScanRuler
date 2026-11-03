# SPDX-License-Identifier: AGPL-3.0-only
import copy,gzip,hashlib,json,unittest
from pathlib import Path
import ci_report,parent_inventory
PLAN=Path(__file__).resolve().parents[1]
class ParentInventoryTests(unittest.TestCase):
    def setUp(self):
        compressed=(PLAN/'ci/nov03-parent-inventory.json.gz').read_bytes();raw=gzip.decompress(compressed);manifest=json.loads((PLAN/'ci/nov03-parent-inventory.manifest.json').read_text());self.assertEqual(hashlib.sha256(compressed).hexdigest(),manifest['gzip_sha256']);self.assertEqual(hashlib.sha256(raw).hexdigest(),manifest['snapshot_sha256']);self.old=json.loads(raw)['suites'];self.current=ci_report.roster();self.assertEqual(len(self.old),57);self.assertEqual(sum(len(s['case_ids'])for s in self.old),22938)
    def test_current_preserves_every_parent_and_allows_additive_cases(self):
        self.assertEqual(parent_inventory.validate(self.old,self.current),[]);new=copy.deepcopy(self.current);new[0]['case_ids'].append('additional-source-case');self.assertEqual(parent_inventory.validate(self.old,new),[])
    def test_removed_suite_fails(self):self.assertTrue(parent_inventory.validate(self.old,self.current[1:]))
    def test_equal_count_case_replacement_fails(self):
        new=copy.deepcopy(self.current);new[0]['case_ids'][0]='replacement-with-same-count';self.assertTrue(parent_inventory.validate(self.old,new))
    def test_dropped_raw_file_contract_fails(self):
        new=copy.deepcopy(self.current);suite=next(s for s in new if set(s['required_files'])-{'input.bin','original.bin'});extra=next(iter(set(suite['required_files'])-{'input.bin','original.bin'}));suite['required_files'].remove(extra);self.assertTrue(parent_inventory.validate(self.old,new))
    def test_old_case_cannot_gain_missing_rust_output_exception(self):
        new=copy.deepcopy(self.current);suite=next(s for s in new if set(s['case_ids'])-set(s['refusals_without_rust_bin']));id=next(iter(set(suite['case_ids'])-set(suite['refusals_without_rust_bin'])));suite['refusals_without_rust_bin'].append(id);self.assertTrue(parent_inventory.validate(self.old,new))
    def test_changed_directory_or_entrypoint_fails(self):
        for key in ['test','directory']:
            new=copy.deepcopy(self.current);new[0][key]+='-changed';self.assertTrue(parent_inventory.validate(self.old,new))
