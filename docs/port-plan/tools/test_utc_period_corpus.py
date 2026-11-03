# SPDX-License-Identifier: AGPL-3.0-only
import gzip,hashlib,json,runpy,unittest
from pathlib import Path
PLAN=Path(__file__).resolve().parents[1]
def load(name):return json.loads(gzip.decompress((PLAN/'evidence'/name).read_bytes()))
class UtcPeriodCorpusTests(unittest.TestCase):
    def test_source_native_complete_time_and_state(self):
        source=load('2026-11-03-utc-period-source.json.gz');paired=load('2026-11-03-utc-period.json.gz');inputs=(PLAN/'fixtures/clock-utc/cases.tsv').read_bytes()
        self.assertEqual(hashlib.sha256(inputs).hexdigest(),source['input_sha256']);self.assertEqual(source['seeds'],[1,7,99,1039])
        self.assertEqual(len(inputs.splitlines()),1911);self.assertEqual(len(paired['cases']),1911);original=[]
        for line,case in zip(inputs.splitlines(),paired['cases']):
            self.assertEqual(case['id'],line.split(b'\t')[0].decode());values={}
            for name,item in case['files'].items():
                values[name]=bytes.fromhex(item['hex']);self.assertEqual(hashlib.sha256(values[name]).hexdigest(),item['sha256'])
            self.assertEqual(values['input.bin'],line);self.assertEqual(values['original.bin'],values['rust.bin']);self.assertEqual(values['original.stderr.txt'],b'')
            fields=values['original.bin'].rstrip(b'\n').split(b'\t');self.assertEqual(len(fields),3)
            frames=fields[2].split(b';');self.assertEqual(len(frames),6)
            for frame in frames:self.assertEqual(len(frame.split(b',')),3)
            original.append(values['original.bin'])
        self.assertEqual(b''.join(original).hex(),source['original_hex'])
        self.assertTrue(any(b'\tthrow:RangeError:' in v for v in original));self.assertTrue(any(b'\tok:' in v for v in original))
    def test_immutable_seed_inputs_reproduce_independently(self):
        module=runpy.run_path(str(PLAN/'tools/generate_utc_cases.py'))
        self.assertEqual(module['generate_period'](),(PLAN/'fixtures/clock-utc/cases.tsv').read_bytes())
    def test_prior_input_source_and_rust_prefix_is_byte_exact(self):
        paired=load('2026-11-03-utc-period.json.gz');old=load('2026-11-02-utc.json.gz')
        self.assertEqual(paired['cases'][:311],old['cases'])
        source=load('2026-11-03-utc-period-source.json.gz');old_source=load('2026-11-02-utc-source.json.gz')
        self.assertTrue(bytes.fromhex(source['original_hex']).startswith(bytes.fromhex(old_source['original_hex'])))
        inputs=(PLAN/'fixtures/clock-utc/cases.tsv').read_bytes();prior=(PLAN/'fixtures/clock-utc/cases-311.tsv').read_bytes()
        self.assertTrue(inputs.startswith(prior));self.assertEqual(len(prior.splitlines()),311)
        self.assertEqual(paired['added_period_cases'],1600);self.assertEqual(len(paired['cases']),1911)
