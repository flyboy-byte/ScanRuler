# SPDX-License-Identifier: AGPL-3.0-only
import gzip,hashlib,json,re,subprocess,unittest
from pathlib import Path
PLAN=Path(__file__).resolve().parents[1]
class SourceSchedulingEvidenceTests(unittest.TestCase):
    def test_two_complete_original_runs_at_the_same_configuration(self):
        report=json.loads(gzip.decompress((PLAN/'evidence/2026-11-02-source-scheduling.json.gz').read_bytes()))
        records=report['records'];self.assertEqual(len(records),2)
        self.assertEqual({r['event'] for r in records},{'pull_request','workflow_dispatch'})
        self.assertEqual(len({r['run_id'] for r in records}),2)
        self.assertEqual({r['head_sha'] for r in records},{report['candidate_sha']})
        workflow=bytes.fromhex(report['workflow_hex']);self.assertEqual(hashlib.sha256(workflow).hexdigest(),report['workflow_sha256'])
        self.assertIn(report['configuration'].encode(),workflow);self.assertIn(b'timeout-minutes: 20',workflow)
        source=subprocess.check_output(['git','show',report['reference_sha']+':plugins/reverse/tests/reNetworkCreases.test.ts'],cwd=PLAN.parents[1])
        self.assertEqual(hashlib.sha256(source).hexdigest(),report['source_test_sha256']);self.assertIn(b'toBeLessThan(1500)',source)
        for record in records:
            self.assertEqual(record['conclusion'],'success');self.assertEqual(record['file_counts'],[371,13,384]);self.assertGreater(record['seconds'],0);self.assertLess(record['seconds'],1200)
            raw=gzip.decompress((PLAN/'evidence/source-scheduling-2026-11-02'/record['log']).read_bytes());self.assertEqual(hashlib.sha256(raw).hexdigest(),record['log_sha256'])
            clean=re.sub(rb'(?:\x1b|\^\[)\[[0-?]*[ -/]*[@-~]',b'',raw)
            self.assertIn(report['configuration'].encode(),clean)
            match=re.search(rb'Test Files\s+(\d+) passed\s*\|\s*(\d+) skipped\s*\((\d+)\)',clean)
            self.assertIsNotNone(match);self.assertEqual(list(map(int,match.groups())),[371,13,384])
