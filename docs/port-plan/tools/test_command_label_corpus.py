# SPDX-License-Identifier: AGPL-3.0-only
import gzip, hashlib, json, runpy, subprocess, unittest
from pathlib import Path
PLAN = Path(__file__).resolve().parents[1]

def cases(path):
    """Bound memory while retaining full per-case values, not just digests."""
    with gzip.open(path, 'rt') as stream:
        assert stream.read(10) == '{"cases":['
        pending = ''
        decoder = json.JSONDecoder()
        while True:
            pending = pending.lstrip(',\n ')
            if pending.startswith(']}'): return
            try:
                value, end = decoder.raw_decode(pending)
            except json.JSONDecodeError:
                chunk = stream.read(65536)
                if not chunk: raise AssertionError('incomplete case archive')
                pending += chunk
                continue
            yield value
            pending = pending[end:]

class CommandLabelCorpusTests(unittest.TestCase):
    def test_complete_inputs_values_effects_and_retained_source_prefix(self):
        source = PLAN/'evidence/command-label-source-2026-11-03'
        paired = PLAN/'evidence/command-label-paired-2026-11-03'
        report = json.loads((source/'report.json').read_text())
        proof = json.loads((paired/'report.json').read_text())
        inputs = (PLAN/'fixtures/command-label/cases.tsv').read_bytes()
        self.assertEqual(hashlib.sha256(inputs).hexdigest(), report['input_sha256'])
        self.assertEqual(report['cases'], 640)
        self.assertEqual(proof['cases'], 640)
        self.assertTrue(proof['identity'].endswith('dirty: '))
        self.assertEqual(hashlib.sha256((paired/'cases.json.gz').read_bytes()).hexdigest(), proof['case_archive_sha256'])
        digest, prefix, remaining = hashlib.sha256(), hashlib.sha256(), report['prior_prefix_bytes']
        count = 0
        with gzip.open(source/'original.tsv.gz', 'rb') as original:
            for descriptor, case in zip(inputs.splitlines(), cases(paired/'cases.json.gz'), strict=True):
                line = original.readline()
                digest.update(line)
                prefix.update(line[:remaining]); remaining = max(0, remaining-len(line))
                identity, raw, result, effects = line.rstrip(b'\n').split(b'\t')
                self.assertEqual(case['id'].encode(), identity)
                values = {}
                for name, item in case['files'].items():
                    value = bytes.fromhex(item['hex'])
                    self.assertEqual(hashlib.sha256(value).hexdigest(), item['sha256'])
                    values[name] = value
                self.assertEqual(values['descriptor.tsv'], descriptor)
                self.assertEqual(values['input.bin'], bytes.fromhex(raw.decode()))
                output = bytes.fromhex(result[3:].decode()) if result.startswith(b'ok:') else b''
                status = b'ok' if result.startswith(b'ok:') else b'error:same'
                self.assertIn(result[:3], [b'ok:', b'err'])
                for language in ['original', 'rust']:
                    self.assertEqual(values[language+'.bin'], output)
                    self.assertEqual(values[language+'.status.txt'], status)
                    self.assertEqual(values[language+'.effects.txt'], effects)
                count += 1
            self.assertEqual(original.read(), b'')
        self.assertEqual(count, 640)
        self.assertEqual(remaining, 0)
        self.assertEqual(prefix.hexdigest(), report['prior_prefix_sha256'])
        self.assertEqual(digest.hexdigest(), report['original_sha256'])

    def test_pinned_expression_source_map_and_reproducible_descriptors(self):
        folder = PLAN/'evidence/command-label-oracle-2026-11-03'
        p = json.loads((folder/'provenance.json').read_text())
        for name, key in [('registry.mjs', 'bundle_sha256'), ('registry.mjs.map', 'source_map_sha256')]:
            self.assertEqual(hashlib.sha256(gzip.decompress((folder/(name+'.gz')).read_bytes())).hexdigest(), p[key])
        source_map = json.loads(gzip.decompress((folder/'registry.mjs.map.gz').read_bytes()))
        path = 'src/commands/registry.ts'
        contents = [value for name, value in zip(source_map['sources'], source_map['sourcesContent']) if name.endswith('/'+path)]
        original = subprocess.check_output(['git', 'show', p['reference_sha']+':'+path], cwd=PLAN.parents[1]).decode()
        self.assertEqual(original.count(p['label_expression']), 2)
        self.assertEqual(contents, [original+p['appended_source']])
        self.assertEqual(p['node']['unicode'], '17.0')
        generate = runpy.run_path(str(PLAN/'tools/generate_command_label_cases.py'))['generate']
        self.assertEqual(generate(), (PLAN/'fixtures/command-label/cases.tsv').read_bytes())
