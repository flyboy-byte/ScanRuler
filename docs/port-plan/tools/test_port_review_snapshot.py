# SPDX-License-Identifier: AGPL-3.0-only
import json,re,unittest
from pathlib import Path
PLAN=Path(__file__).resolve().parents[1]
class PortReviewSnapshotTests(unittest.TestCase):
    def test_heads_dependencies_and_document_rows_are_lossless(self):
        data=json.loads((PLAN/'evidence/2026-11-03-pr-review-snapshot.json').read_text());prs=data['prs'];text=(PLAN/'REVIEW-HANDOFF-2026-11-03.md').read_text()
        self.assertEqual(len(data['required_checks']),11);self.assertEqual(len(set(data['required_checks'])),11)
        self.assertRegex(data['main_sha'],r'^[0-9a-f]{40}$');self.assertIn(data['observed_at_utc'],text)
        refs={p['headRefName']:p for p in prs};self.assertEqual(len(refs),len(prs));self.assertEqual(len({p['number']for p in prs}),len(prs))
        for p in prs:
            self.assertRegex(p['headRefOid'],r'^[0-9a-f]{40}$');self.assertIn(f"| [#{p['number']}]",text);self.assertIn(p['headRefOid'][:8],text)
            seen=set();current=p
            while current['baseRefName']!='main':
                self.assertNotIn(current['headRefName'],seen);seen.add(current['headRefName']);self.assertIn(current['baseRefName'],refs);current=refs[current['baseRefName']]
        for p in prs:
            checks={c.get('name'):c.get('conclusion')for c in p['statusCheckRollup']}
            row=next(line for line in text.splitlines()if line.startswith(f"| [#{p['number']}]"))
            if '11 green' in row:self.assertTrue(all(checks.get(n)=='SUCCESS'for n in data['required_checks']))
