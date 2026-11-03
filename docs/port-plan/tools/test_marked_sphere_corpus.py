# SPDX-License-Identifier: AGPL-3.0-only
import gzip,hashlib,json,math,runpy,struct,subprocess,unittest
from pathlib import Path
PLAN=Path(__file__).resolve().parents[1]
def load(name):return json.loads(gzip.decompress((PLAN/'evidence'/name).read_bytes()))
def decode(raw):
    at=1
    def text():
        nonlocal at
        n=struct.unpack_from('<I',raw,at)[0];at+=4;value=raw[at:at+n].decode();at+=n;return value
    if raw[0]==1:
        name=text();brand=raw[at];at+=1;constructor=text();message=text();assert at==len(raw);return 'error',(name,brand,constructor,message)
    assert raw[0]==0
    kind=text();numbers=struct.unpack_from('<8d',raw,at);at+=64;used,size=struct.unpack_from('<II',raw,at);at+=8;held=raw[at];at+=1;assert held in [0,1]
    if held:numbers+=struct.unpack_from('<2d',raw,at);at+=16
    n=struct.unpack_from('<I',raw,at)[0];at+=4;region=struct.unpack_from('<'+str(n)+'I',raw,at);at+=4*n;identity=raw[at];at+=1;assert at==len(raw);assert n==size;assert identity==1;return 'ok',(kind,numbers,used,size,held,region,identity)
class MarkedSphereCorpusTests(unittest.TestCase):
    def test_full_outputs_held_region_identity_exact_fiterrors_and_immutable_inputs(self):
        source=load('2026-11-03-marked-sphere-source.json.gz');paired=load('2026-11-03-marked-sphere.json.gz');inputs=(PLAN/'fixtures/marked-sphere/cases.tsv').read_bytes();lines=bytes.fromhex(source['original_hex']).splitlines()
        self.assertEqual(hashlib.sha256(inputs).hexdigest(),source['input_sha256']);self.assertEqual(source['cases'],2220);self.assertEqual(len(paired['cases']),2220);self.assertTrue(paired['identity'].endswith('dirty: '));success=0;clipped=0
        for descriptor,line,case in zip(inputs.splitlines(),lines,paired['cases'],strict=True):
            id,input_hex,output_hex,after_hex=line.split(b'\t');self.assertEqual(case['id'].encode(),id);values={}
            for name,item in case['files'].items():
                values[name]=bytes.fromhex(item['hex']);self.assertEqual(hashlib.sha256(values[name]).hexdigest(),item['sha256'])
            self.assertEqual(values['descriptor.tsv'],descriptor);self.assertEqual(descriptor.split(b'\t')[0],id);self.assertEqual(values['input.bin'],bytes.fromhex(input_hex.decode()));self.assertEqual(values['original.bin'],bytes.fromhex(output_hex.decode()));self.assertEqual(values['input.bin'],bytes.fromhex(after_hex.decode()));self.assertEqual(values['input.bin'],values['original.after.bin']);self.assertEqual(values['input.bin'],values['rust.after.bin']);self.assertEqual(values['original.stderr.txt'],b'');a,b=decode(values['rust.bin']),decode(values['original.bin']);self.assertEqual(a[0],b[0])
            if b[0]=='error':self.assertEqual(a,b);self.assertEqual(b[1][:3],('Error',1,'FitError'));continue
            success+=1;clipped+=b[1][2]<b[1][3];self.assertEqual(a[1][0],b[1][0]);self.assertEqual(a[1][2:],b[1][2:])
            for x,y in zip(a[1][1],b[1][1],strict=True):
                if math.isnan(y):self.assertTrue(math.isnan(x))
                elif not math.isfinite(y):self.assertEqual(x,y)
                elif x==0 and y==0:self.assertEqual(math.copysign(1,x),math.copysign(1,y))
                else:self.assertTrue(math.isfinite(x)and abs(x-y)<=1e-10+1e-10*abs(y))
        self.assertEqual(success,759);self.assertGreater(clipped,0);self.assertEqual(len(lines),2220)
    def test_complete_unmodified_pinned_dependency_graph_and_input_reproduction(self):
        modules=load('2026-11-03-marked-sphere-modules.json.gz');self.assertEqual(len(modules['sources']),18)
        for path,source in modules['sources'].items():
            original=subprocess.check_output(['git','show',modules['reference_sha']+':'+path],cwd=PLAN.parents[1]).decode();self.assertEqual(source,original);self.assertEqual(hashlib.sha256(source.encode()).hexdigest(),modules['source_sha256'][path]);self.assertEqual(json.loads(modules['maps'][path])['sourcesContent'],[original])
        generate=runpy.run_path(str(PLAN/'tools/generate_marked_sphere_cases.py'))['generate'];self.assertEqual(generate(),(PLAN/'fixtures/marked-sphere/cases.tsv').read_bytes())
