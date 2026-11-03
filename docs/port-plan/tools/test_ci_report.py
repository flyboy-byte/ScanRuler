# SPDX-License-Identifier: AGPL-3.0-only
"""Negative tests for artifact completeness and generated inventory integrity."""
import copy
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
import ci_report as report

SHA = 'a' * 40
REF = 'b' * 40


class ArtifactTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.target = Path(self.temp.name)
        self.suite = {'unit': 'example.unit', 'test': 'pinned_example', 'directory': 'example-diff',
                      'case_ids': ['empty', 'value'], 'refusals_without_rust_bin': [],
                      'required_files': ['input.bin', 'original.bin']}
        self.run = self.target / 'example-diff/run-1'
        self.run.mkdir(parents=True)
        (self.run / 'candidate.txt').write_text(SHA)
        for case in self.suite['case_ids']:
            directory = self.run / case
            directory.mkdir()
            for name in ['input.bin', 'original.bin', 'rust.bin']:
                (directory / name).write_bytes(b'' if case == 'empty' else b'value')

    def check(self):
        return report.evidence(self.target, [self.suite], SHA, REF)

    def test_valid_including_legitimate_empty_values(self):
        value = self.check()
        self.assertEqual(value['candidate_sha'], SHA)
        self.assertEqual(value['suites'][0]['cases'][0]['files']['rust.bin']['bytes'], 0)

    def test_missing_suite(self):
        shutil.rmtree(self.run)
        with self.assertRaises(ValueError): self.check()

    def test_missing_case(self):
        shutil.rmtree(self.run / 'value')
        with self.assertRaises(ValueError): self.check()

    def test_extra_case(self):
        (self.run / 'unexpected').mkdir()
        with self.assertRaises(ValueError): self.check()

    def test_missing_original_or_rust_even_for_empty_case(self):
        for name in ['original.bin', 'rust.bin']:
            path = self.run / 'empty' / name
            path.unlink()
            with self.assertRaises(ValueError): self.check()
            path.write_bytes(b'')

    def test_successful_progress_and_count_files_cannot_disappear(self):
        extra = ['original.progress.txt', 'rust.progress.txt', 'count.txt']
        self.suite['required_files'] += extra
        for case in self.suite['case_ids']:
            for name in extra:
                (self.run / case / name).write_bytes(b'')
        self.check()
        for name in extra:
            path = self.run / 'value' / name
            path.unlink()
            with self.assertRaises(ValueError): self.check()
            path.write_bytes(b'')

    def test_stale_newest_run_cannot_use_older_green(self):
        shutil.copytree(self.run, self.run.with_name('run-2'))
        (self.run.with_name('run-2') / 'candidate.txt').write_text('c' * 40)
        with self.assertRaises(ValueError): self.check()

    def test_linked_case_rejected(self):
        shutil.rmtree(self.run / 'empty')
        (self.run / 'empty').symlink_to(self.run / 'value', target_is_directory=True)
        with self.assertRaises(ValueError): self.check()

    def test_declared_refusal_still_requires_both_error_records(self):
        self.suite['directory'] = 'stl-diff'
        self.suite['refusals_without_rust_bin'] = ['empty']
        destination = self.target / 'stl-diff/run-1'
        destination.parent.mkdir()
        self.run.rename(destination)
        self.run = destination
        (self.run / 'empty/rust.bin').unlink()
        with self.assertRaises(ValueError): self.check()
        for name in ['original.progress-or-error.txt', 'rust.progress.txt']:
            (self.run / 'empty' / name).write_text('refused')
        self.check()


class InventoryTests(unittest.TestCase):
    def test_current_roster_and_status_are_deterministic(self):
        value = report.status()
        self.assertEqual(value, report.status())
        self.assertEqual(value['native_suites'], 48)
        self.assertEqual(value['native_case_identities_including_refusals'], 8497)

    def test_roster_rejects_missing_duplicate_or_unknown_suite(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for relative in ['docs/port-plan/ci', 'docs/port-plan/plan/units', 'rust/xtask/src']:
                shutil.copytree(report.ROOT / relative, root / relative)
            path = root / 'docs/port-plan/ci/native-suites.json'
            original = json.loads(path.read_text())
            for mutate in ['missing', 'duplicate', 'unknown']:
                data = copy.deepcopy(original)
                if mutate == 'missing': data['suites'].pop()
                elif mutate == 'duplicate': data['suites'].append(data['suites'][0])
                else: data['suites'][0]['unit'] = 'unknown.unit'
                path.write_text(json.dumps(data))
                with self.assertRaises((ValueError, KeyError)): report.roster(root)

    def test_stale_generated_backlog_fails(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            shutil.copytree(report.PLAN / 'plan/units', root / 'plan/units')
            # Isolate generated-backlog freshness from unrelated local evidence files.
            for path in (root / 'plan/units').glob('*.json'):
                unit = json.loads(path.read_text()); unit['verification'] = 'none'
                path.write_text(json.dumps(unit))
            metadata = json.loads((report.PLAN / 'backlog.json').read_text())
            metadata['tasks'] = []
            (root / 'backlog.json').write_text(json.dumps(metadata))
            result = subprocess.run(['python3', str(report.PLAN / 'tools/port_plan.py'), '--root', str(root), 'check-generated'], capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('Generated backlog is stale', result.stderr)


if __name__ == '__main__':
    unittest.main()
