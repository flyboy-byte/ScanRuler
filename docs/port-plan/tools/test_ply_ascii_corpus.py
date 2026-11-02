# SPDX-License-Identifier: AGPL-3.0-only
"""Reject changed/missing inputs and incomplete compiled schedules, not bless outputs."""
import hashlib
import json
from pathlib import Path
import re
import unittest


class PlyCorpusTests(unittest.TestCase):
    def test_input_hashes_and_compiled_schedule_are_complete(self):
        root = Path(__file__).resolve().parents[3]
        directory = root / 'docs/port-plan/fixtures/ply-ascii'
        manifest = json.loads((directory / 'manifest.json').read_text())
        self.assertEqual(manifest['reference_sha'], '2cd89c6769a10c27ee7754b477ada521c9ef5cb7')
        self.assertEqual(manifest['license'], 'AGPL-3.0-only')
        self.assertEqual(manifest['seeds'], [1, 7, 99, 1039])
        rows = manifest['cases']
        ids = [r['id'] for r in rows]
        self.assertEqual(len(ids), 60)
        self.assertEqual(len(set(ids)), 60)
        self.assertEqual(sum(r['success'] for r in rows), 46)
        self.assertEqual({p.stem for p in directory.glob('*.ply')}, set(ids))
        table = (root / 'rust/crates/port-io/tests/support/ply_ascii_cases.rs').read_text()
        compiled = re.findall(r'id: "([a-z0-9-]+)"', table)
        self.assertEqual(compiled, ids)
        for r in rows:
            self.assertRegex(r['id'], r'^[a-z0-9-]+$')
            data = (directory / (r['id'] + '.ply')).read_bytes()
            self.assertEqual(hashlib.sha256(data).hexdigest(), r['sha256'], r['id'])
            self.assertEqual(len(data), r['bytes'], r['id'])
