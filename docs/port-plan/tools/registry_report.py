# SPDX-License-Identifier: AGPL-3.0-only
"""Validate/archive every full registry observation; original bytes stay independent."""
import gzip,hashlib,json
from pathlib import Path
from schema_report import Reader
ROOT=Path(__file__).resolve().parents[3];PLAN=ROOT/'docs/port-plan'
def load(p):return json.loads(gzip.decompress(p.read_bytes()))
def fields(v):
 if v[0]!=7:raise ValueError('Expected observed object')
 return dict(v[1])
def frame(b):
 r=Reader(b)
 if r.take(4)!=b'REG3':raise ValueError('Registry frame')
 root=r.value();r.end()
 if root[0]!=6 or not root[1]:raise ValueError('Empty scenario')
 for observation in root[1]:
  if [k for k,v in observation[1]]!=['value','error','afterInput','target','commands','headless']:raise ValueError('Observed key set/order')
  o=fields(observation);c=o['commands'];h=o['headless']
  if c[0]!=6 or h[0]!=6 or len(c[1])!=len(h[1])or any(v[0]not in [2,3]for v in h[1]):raise ValueError('Catalog/headless')
  names=[]
  for command in c[1]:
   names.append(fields(command)['name'][1]);keys=[k for k,v in command[1]]
   if keys[:7]!=['name','title','description','input','readOnly','returnsFile','returnsImage']:raise ValueError('Metadata key set/order')
  if len(set(names))!=len(names):raise ValueError('Duplicate catalog name')
 return root
def compare(original,candidate):
 if original!=candidate:raise ValueError('Complete registry state mismatch')
 frame(original)
def archive():
 run=max((ROOT/'rust/target/registry-diff').glob('run-*'),key=lambda p:int(p.name[4:]));source=load(PLAN/'evidence/2026-11-02-registry-source-350.json.gz');frozen={r['id']:r for r in source['cases']};manifest=json.loads((PLAN/'fixtures/registry/manifest.json').read_text());rows=[];ops=success=refusal=source_fault=0
 for c in manifest['cases']:
  d=run/c['id'];original=(d/'original.bin').read_bytes();rust=(d/'rust.bin').read_bytes();compare(bytes.fromhex(frozen[c['id']]['hex']),original);compare(original,rust)
  observations=frame(rust)[1];ops+=len(observations)
  for o in observations:
   f=fields(o)
   if f['target'][0]==4:success+=1
   e=f['error']
   if e[0]!=0:
    err=fields(e)
    if err['name'][1]=='TypeError':source_fault+=1
    else:refusal+=1
  files={}
  for name in ['input.bin','original.bin','rust.bin','original.stderr.txt']:
   b=(d/name).read_bytes();files[name]=dict(bytes=len(b),sha256=hashlib.sha256(b).hexdigest(),**({'hex':b.hex()}if name in ['rust.bin','original.stderr.txt']else{}))
  if files['input.bin']['sha256']!=c['sha256']or files['original.stderr.txt']['bytes']:raise ValueError('Input/stderr')
  rows.append(dict(id=c['id'],observations=len(observations),files=files))
 if {p.name for p in run.iterdir()if p.is_dir()}!=set(frozen):raise ValueError('Case set')
 report=dict(contract='engine.session.registry.bytes.v1',candidate_sha=(run/'candidate.txt').read_text().strip(),reference_sha=source['reference_sha'],identity=(run/'identity.txt').read_text(),manifest_sha256=hashlib.sha256((PLAN/'fixtures/registry/manifest.json').read_bytes()).hexdigest(),source_capture='2026-11-02-registry-source-350.json.gz',source_capture_sha256=hashlib.sha256((PLAN/'evidence/2026-11-02-registry-source-350.json.gz').read_bytes()).hexdigest(),seeds=manifest['seeds'],scenario_count=len(rows),operation_observations=ops,successful_admissions=success,refusals=refusal,source_faults=source_fault,application_commands_implemented=0,browser_runtime_claimed=False,cases=rows)
 out=PLAN/'evidence/2026-11-02-registry.json.gz';out.write_bytes(gzip.compress((json.dumps(report,indent=2)+'\n').encode(),mtime=0));print(report['candidate_sha'],len(rows),'scenarios',ops,'observations',success,'admissions',refusal,'refusals',source_fault,'source faults')
if __name__=='__main__':archive()
