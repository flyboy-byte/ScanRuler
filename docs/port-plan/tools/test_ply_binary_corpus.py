# SPDX-License-Identifier: AGPL-3.0-only
import hashlib
import json
from pathlib import Path
import re
import unittest
import gzip


class BinaryPlyCorpusTests(unittest.TestCase):
    def test_committed_output_bytes_hashes_and_progress_are_complete(self):
        root = Path(__file__).resolve().parents[3]
        folder = root / 'docs/port-plan/evidence/ply-binary-2026-11-02'
        report = json.loads((folder / 'report.json').read_text())
        manifest = json.loads((root / 'docs/port-plan/fixtures/ply-binary/manifest.json').read_text())
        self.assertEqual([c['id'] for c in report['cases']], [c['id'] for c in manifest['cases']])
        for case in report['cases']:
            self.assertEqual(case['files']['input.bin']['sha256'], case['sha256'])
            for name, item in case['files'].items():
                if name == 'input.bin':
                    continue
                if 'gzip' in item:
                    packed = (folder / item['gzip']).read_bytes()
                    self.assertEqual(hashlib.sha256(packed).hexdigest(), item['gzip_sha256'])
                    raw = gzip.decompress(packed)
                else:
                    raw = bytes.fromhex(item['hex'])
                self.assertEqual(len(raw), item['bytes'])
                self.assertEqual(hashlib.sha256(raw).hexdigest(), item['sha256'])
            for a, b in [('original.bin', 'rust.bin'), ('original.progress-or-error.txt', 'rust.progress-or-error.txt')]:
                self.assertEqual(case['files'][a]['sha256'], case['files'][b]['sha256'])
        for id, message in [('le-vertex-progress', 'Reading PLY vertices… 100%\n'),
                            ('le-face-progress', 'Reading PLY faces… 100%\n')]:
            case = next(c for c in report['cases'] if c['id'] == id)
            self.assertEqual(bytes.fromhex(case['files']['original.progress-or-error.txt']['hex']).decode(), message)

    def test_all_alias_roles_endians_and_progress_inputs_are_retained(self):
        root = Path(__file__).resolve().parents[3]
        directory = root / 'docs/port-plan/fixtures/ply-binary'
        manifest = json.loads((directory / 'manifest.json').read_text())
        self.assertEqual(manifest['reference_sha'], '2cd89c6769a10c27ee7754b477ada521c9ef5cb7')
        self.assertEqual(manifest['license'], 'AGPL-3.0-only')
        self.assertEqual(manifest['seeds'], [1, 7, 99, 1039])
        rows = manifest['cases']
        ids = [r['id'] for r in rows]
        self.assertEqual(len(ids), 168)
        self.assertEqual(len(set(ids)), 168)
        self.assertEqual(sum(r['success'] for r in rows), 152)
        self.assertEqual({p.stem for p in directory.glob('*.ply')}, set(ids))
        table = (root / 'rust/crates/port-io/tests/support/ply_binary_cases.rs').read_text()
        self.assertEqual(re.findall(r'id: "([a-z0-9-]+)"', table), ids)
        for endian in ['le', 'be']:
            for role in ['vertex', 'count', 'index']:
                for scalar in ['char','int8','uchar','uint8','short','int16','ushort','uint16',
                               'int','int32','uint','uint32','float','float32','double','float64']:
                    self.assertIn(f'{endian}-{role}-{scalar}', ids)
        for r in rows:
            self.assertRegex(r['id'], r'^[a-z0-9-]+$')
            data = (directory / (r['id'] + '.ply')).read_bytes()
            self.assertEqual(hashlib.sha256(data).hexdigest(), r['sha256'], r['id'])
            self.assertEqual(len(data), r['bytes'], r['id'])
        n = (1 << 20) + 1
        vertex = (directory / 'le-vertex-progress.ply').read_bytes()
        face = (directory / 'le-face-progress.ply').read_bytes()
        self.assertIn(f'element vertex {n}\n'.encode(), vertex)
        self.assertEqual(len(vertex.split(b'end_header\n', 1)[1]), n * 3 + 13)
        self.assertIn(f'element face {n}\n'.encode(), face)
        self.assertEqual(len(face.split(b'end_header\n', 1)[1]), 36)
