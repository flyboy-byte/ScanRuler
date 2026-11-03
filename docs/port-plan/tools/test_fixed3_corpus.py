# SPDX-License-Identifier: AGPL-3.0-only
import gzip,hashlib,json,runpy,subprocess,unittest
from pathlib import Path
PLAN=Path(__file__).resolve().parents[1]
def load(name):return json.loads(gzip.decompress((PLAN/'evidence'/name).read_bytes()))
class Fixed3CorpusTests(unittest.TestCase):
    def test_exact_full_primitive_category_text_and_immutability(self):
        source=load('2026-11-03-fixed3-source.json.gz');paired=load('2026-11-03-fixed3.json.gz');inputs=(PLAN/'fixtures/fixed3/cases.tsv').read_bytes();lines=bytes.fromhex(source['original_hex']).splitlines()
        self.assertEqual(hashlib.sha256(inputs).hexdigest(),source['input_sha256']);self.assertEqual(source['cases'],8492);self.assertEqual(len(paired['cases']),8492);self.assertTrue(paired['identity'].endswith('dirty: '));self.assertEqual(len(lines),8492)
        for descriptor,line,case in zip(inputs.splitlines(),lines,paired['cases'],strict=True):
            id,raw,kind,text,after=line.split(b'\t');self.assertEqual(case['id'].encode(),id);self.assertEqual(descriptor.split(b'\t')[0],id);values={}
            for name,item in case['files'].items():
                values[name]=bytes.fromhex(item['hex']);self.assertEqual(hashlib.sha256(values[name]).hexdigest(),item['sha256'])
            self.assertEqual(values['descriptor.tsv'],descriptor);self.assertEqual(values['input.bin'],bytes.fromhex(raw.decode()));self.assertEqual(values['input.bin'],bytes.fromhex(after.decode()));self.assertEqual(values['input.bin'],values['original.after.bin']);self.assertEqual(values['input.bin'],values['rust.after.bin']);self.assertEqual(values['original.bin'],bytes.fromhex(text.decode()));self.assertEqual(values['original.bin'],values['rust.bin']);self.assertEqual(values['original.category.txt'],kind);self.assertEqual(values['rust.category.txt'],kind);self.assertEqual(values['original.stderr.txt'],b'');self.assertTrue(values['rust.bin'].isascii());self.assertLessEqual(len(values['rust.bin']),26)
    def test_pinned_call_sites_and_reproducible_input_bits(self):
        p=load('2026-11-03-fixed3-provenance.json.gz');original=subprocess.check_output(['git','show',p['reference_sha']+':'+p['source_path']],cwd=PLAN.parents[1]).decode();self.assertEqual(p['source'],original);self.assertEqual(hashlib.sha256(original.encode()).hexdigest(),p['source_sha256'])
        for expression in p['expressions']:self.assertEqual(original.count(expression),1)
        generate=runpy.run_path(str(PLAN/'tools/generate_fixed3_cases.py'))['generate'];self.assertEqual(generate(),(PLAN/'fixtures/fixed3/cases.tsv').read_bytes())
