# SPDX-License-Identifier: AGPL-3.0-only
from pathlib import Path
import tempfile
import unittest
import json
import baseline_capture as baseline


class BaselineTests(unittest.TestCase):
    def test_repository_guard_inventory_matches_original(self):
        root = Path(__file__).resolve().parents[3]
        contract = json.loads((root / 'docs/port-plan/ci/original-exclusions.json').read_text())
        self.assertEqual(contract['reference_sha'], baseline.REFERENCE)
        baseline.check_sites(root, contract)

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / 'tests').mkdir()
        self.guard = "describe.skipIf(!FILE.exists)('private scan', () => {"
        (self.root / 'tests/a.test.ts').write_text(self.guard)
        self.contract = {'sites': [{'path': 'tests/a.test.ts', 'guard': self.guard, 'reason': 'private scan absent'}]}
        self.data = {'numTotalTests': 2, 'testResults': [{'name': str(self.root / 'tests/a.test.ts'),
            'assertionResults': [{'status': 'passed', 'fullName': 'generated plane'},
                                 {'status': 'pending', 'fullName': 'private scan plane'}]}]}

    def test_records_individual_exclusions_and_values_separately(self):
        baseline.check_sites(self.root, self.contract)
        r = baseline.unit_results(self.root, self.data, self.contract)
        self.assertEqual((r['total'], r['passed'], len(r['excluded'])), (2, 1, 1))

    def test_new_guard_cannot_silently_hide_cases(self):
        (self.root / 'tests/a.test.ts').write_text(self.guard + "\nit.skip('hidden', () => {})")
        with self.assertRaises(ValueError): baseline.check_sites(self.root, self.contract)

    def test_removed_guard_or_duplicate_contract_fails(self):
        for sites in [[], self.contract['sites'] * 2]:
            with self.assertRaises(ValueError): baseline.check_sites(self.root, {'sites': sites})

    def test_unmapped_pending_test_fails_even_in_known_file(self):
        self.data['testResults'][0]['assertionResults'][1]['fullName'] = 'new optional behavior'
        with self.assertRaises(ValueError): baseline.unit_results(self.root, self.data, self.contract)

    def test_empty_or_inconsistent_results_fail(self):
        for data in [{'numTotalTests': 0, 'testResults': []}, dict(self.data, numTotalTests=9)]:
            with self.assertRaises(ValueError): baseline.unit_results(self.root, data, self.contract)

    def test_failure_remains_failure(self):
        self.data['testResults'][0]['assertionResults'][0]['status'] = 'failed'
        r = baseline.unit_results(self.root, self.data, self.contract)
        self.assertEqual(r['failed'], 1)
        self.assertEqual(r['failures'][0]['name'], 'generated plane')

    def test_complete_run_records_require_nonempty_logs(self):
        log = self.root / 'run.log'
        log.write_text('actual output')
        records = [{'id': i, 'command': ['example'], 'exit_code': 0, 'log': str(log)} for i in baseline.REQUIRED]
        self.assertEqual(len(baseline.run_records(records)), 6)
        for bad in [records[:-1], records + [records[0]]]:
            with self.assertRaises(ValueError): baseline.run_records(bad)
        log.write_text('')
        with self.assertRaises(ValueError): baseline.run_records(records)


if __name__ == '__main__':
    unittest.main()
