# SPDX-License-Identifier: AGPL-3.0-only
"""Validate complete schema observations; archive live raw output, never bless Rust."""
import hashlib,json,struct,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3];PLAN=ROOT/'docs/port-plan'
class Reader:
 def __init__(self,b):self.b=b;self.at=0
 def take(self,n):
  if n<0 or self.at+n>len(self.b):raise ValueError('Truncated frame')
  b=self.b[self.at:self.at+n];self.at+=n;return b
 def n(self):return struct.unpack('<I',self.take(4))[0]
 def text(self):return self.take(self.n()*2).decode('utf-16-le','surrogatepass')
 def value(self):
  tag=self.take(1)[0]
  if tag<=3:return (tag,None)
  if tag==4:return (tag,self.take(8).hex())
  if tag==5:return (tag,self.text())
  if tag==6:return (tag,[self.value() for _ in range(self.n())])
  if tag==7:
   pairs=[(self.text(),self.value()) for _ in range(self.n())]
   if len(set(k for k,v in pairs))!=len(pairs):raise ValueError('Duplicate output key')
   return(tag,pairs)
  if tag==8:return(tag,self.take(self.n()).hex())
  raise ValueError('Unknown value tag')
 def end(self):
  if self.at!=len(self.b):raise ValueError('Trailing frame')
def frame(b):
 r=Reader(b)
 if r.take(4)!=b'VLD1':raise ValueError('Header')
 count=r.n()
 if count not in [0,2]:raise ValueError('Fault shape')
 fault=[r.text() for _ in range(count)];errors=[r.text() for _ in range(r.n())];v=r.value();w=[r.n() for _ in range(4)];r.end()
 if w[2:]!=[8000,12] or (fault and (fault!=['TypeError','Reduce of empty array with no initial value'] or errors)):raise ValueError('Fault/budget')
 return dict(fault=fault,errors=errors,value=v,weight=w[:2])
def compare(original,candidate):
 frame(original);frame(candidate)
 if original!=candidate:raise ValueError('Complete schema state mismatch')
def archive():
 run=max((ROOT/'rust/target/schema-diff').glob('run-*'),key=lambda p:int(p.name[4:]));candidate=(run/'candidate.txt').read_text().strip()
 source=json.loads((PLAN/'evidence/2026-11-02-schema-source-expanded.json').read_text());frozen={c['id']:bytes.fromhex(c['hex']) for c in source['cases']}
 manifest=json.loads((PLAN/'fixtures/schema/manifest.json').read_text());rows=[];counts={};intent={}
 for c in manifest['cases']:
  d=run/c['id'];o=(d/'original.bin').read_bytes();r=(d/'rust.bin').read_bytes();compare(frozen[c['id']],o);compare(o,r);obs=frame(o)
  state='source-fault' if obs['fault'] else 'refused' if obs['errors'] else 'accepted';counts[state]=counts.get(state,0)+1
  if c.get('intent'):intent.setdefault(c['intent'],{});intent[c['intent']][state]=intent[c['intent']].get(state,0)+1
  files={}
  for name in ['input.bin','original.bin','rust.bin','original.stderr.txt']:
   b=(d/name).read_bytes();files[name]=dict(bytes=len(b),sha256=hashlib.sha256(b).hexdigest(),**({'hex':b.hex()} if name!='input.bin' else {}))
  if files['input.bin']['sha256']!=c['sha256'] or files['original.stderr.txt']['bytes']:raise ValueError('Input/stderr')
  rows.append(dict(id=c['id'],state=state,files=files))
 if {p.name for p in run.iterdir() if p.is_dir()}!=set(frozen):raise ValueError('Case inventory')
 report=dict(contract='engine.schema.validate.bytes.v1',reference_sha=manifest['reference_sha'],candidate_sha=candidate,identity=(run/'identity.txt').read_text(),manifest_sha256=source['manifest_sha256'],seeds=manifest['seeds'],counts=counts,commands=140,command_intents=intent,working_commands_claimed=0,browser_runtime_claimed=False,comparator='exact VLD1 complete errors/coerced typed values/weight/budget/source fault; changed error line and missing tail rejected',cases=rows)
 (PLAN/'evidence/2026-11-02-schema.json').write_text(json.dumps(report,indent=2)+'\n');print(candidate,counts,intent)
if __name__=='__main__':archive()
