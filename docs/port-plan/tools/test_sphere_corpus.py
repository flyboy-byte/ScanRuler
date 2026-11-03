# SPDX-License-Identifier: AGPL-3.0-only
import gzip,hashlib,json,math,struct,subprocess,unittest
from pathlib import Path
PLAN=Path(__file__).resolve().parents[1]
def load(name):return json.loads(gzip.decompress((PLAN/'evidence'/name).read_bytes()))
def decode(raw):
    if raw==b'\0':return None
    if not raw or raw[0]!=1:raise ValueError('bad result tag')
    fields=struct.unpack_from('<8d',raw,1);n=struct.unpack_from('<I',raw,65)[0];end=69+4*n
    used=struct.unpack_from('<'+str(n)+'I',raw,69);present=raw[end]
    if present not in [0,1] or len(raw)!=end+1+present*32:raise ValueError('bad shape')
    free=struct.unpack_from('<4d',raw,end+1)if present else None
    return fields,used,free

def compare(test,actual,original):
    a,b=decode(actual),decode(original);test.assertEqual(a is None,b is None)
    if b is None:return
    test.assertEqual(a[1],b[1]);test.assertEqual(a[2]is None,b[2]is None)
    for x,y in zip(a[0]+(a[2]or()),b[0]+(b[2]or()),strict=True):
        if math.isnan(y):test.assertTrue(math.isnan(x))
        elif not math.isfinite(y):test.assertEqual(x,y)
        elif x==0 and y==0:test.assertEqual(math.copysign(1,x),math.copysign(1,y))
        else:test.assertTrue(math.isfinite(x)and abs(x-y)<=1e-10+1e-10*abs(y))

class SphereCorpusTests(unittest.TestCase):
    def test_full_inputs_models_statistics_used_free_and_immutability(self):
        source=load('2026-11-03-sphere-source.json.gz');paired=load('2026-11-03-sphere.json.gz');inputs=(PLAN/'fixtures/sphere/cases.tsv').read_bytes()
        self.assertEqual(hashlib.sha256(inputs).hexdigest(),source['input_sha256']);self.assertEqual(source['cases'],1560);self.assertEqual(len(paired['cases']),1560);self.assertTrue(paired['identity'].endswith('dirty: '));source_lines=bytes.fromhex(source['original_hex']).splitlines(keepends=True);self.assertEqual(len(source_lines),1560);success=0
        for descriptor,line,case in zip(inputs.splitlines(),source_lines,paired['cases'],strict=True):
            id,input_hex,output_hex,after_hex=line.rstrip(b'\n').split(b'\t');self.assertEqual(case['id'].encode(),id);self.assertEqual(descriptor.split(b'\t')[0],id);values={}
            for name,item in case['files'].items():
                values[name]=bytes.fromhex(item['hex']);self.assertEqual(hashlib.sha256(values[name]).hexdigest(),item['sha256'])
            self.assertEqual(values['descriptor.tsv'],descriptor);self.assertEqual(values['input.bin'],bytes.fromhex(input_hex.decode()));self.assertEqual(values['original.bin'],bytes.fromhex(output_hex.decode()));self.assertEqual(values['original.after.bin'],bytes.fromhex(after_hex.decode()));self.assertEqual(values['input.bin'],values['original.after.bin']);self.assertEqual(values['input.bin'],values['rust.after.bin']);self.assertEqual(values['original.stderr.txt'],b'');compare(self,values['rust.bin'],values['original.bin']);success+=values['original.bin'][0]
        self.assertEqual(success,624)
        raw=b''.join(source_lines);self.assertEqual(hashlib.sha256(raw[:source['prior_prefix_bytes']]).hexdigest(),source['prior_prefix_sha256'])
        rows=[(m,s,k,h)for m in range(16)for s in [1,7,99,1039]for k in [0,1,3]for h in ['none','10','9.7','0','-10','nan','inf']]+[(m,s,k,h)for m in [5,6,8,9,13,15]for s in [1,7,99,1039]for k in ['-3','nan','inf']for h in ['none','10','-inf']]
        expected=''.join(f'case-{i:04}\t{m}\t{s}\t{k}\t{h}\n'for i,(m,s,k,h)in enumerate(rows)).encode();self.assertEqual(inputs,expected)
        p=paired['comparator'];self.assertEqual((p['absolute'],p['relative']),(1e-10,1e-10))
    def test_full_frozen_modules_source_maps_and_comparator_canaries(self):
        modules=load('2026-11-03-sphere-modules.json.gz');self.assertEqual(len(modules['sources']),3)
        for path,source in modules['sources'].items():
            original=subprocess.check_output(['git','show',modules['reference_sha']+':'+path],cwd=PLAN.parents[1]).decode();self.assertEqual(source,original);self.assertEqual(hashlib.sha256(source.encode()).hexdigest(),modules['source_sha256'][path]);self.assertEqual(json.loads(modules['maps'][path])['sourcesContent'],[original]);self.assertIn('exports.',modules['compiled'][path])
        paired=load('2026-11-03-sphere.json.gz');original=next(bytes.fromhex(c['files']['original.bin']['hex'])for c in paired['cases']if c['files']['original.bin']['hex'].startswith('01'))
        changed=bytearray(original);struct.pack_into('<d',changed,1,float('nan'))
        with self.assertRaises(AssertionError):compare(self,bytes(changed),original)
        changed=bytearray(original);struct.pack_into('<I',changed,69,37)
        with self.assertRaises(AssertionError):compare(self,bytes(changed),original)
