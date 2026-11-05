# SPDX-License-Identifier: AGPL-3.0-only
"""One fixed project corpus. Exact state/JSON values; existing plane budget for fit geometry only."""
import json,sys,math
from pathlib import Path
p=Path(sys.argv[1]);old=[json.loads(x)for x in(p/'original.jsonl').read_text().splitlines()];new=[json.loads(x)for x in(p/'rust.jsonl').read_text().splitlines()];ids=[x.split('\t')[0]for x in(p/'cases.tsv').read_text().splitlines()]
assert len(old)==len(new)==len(ids)==21 and [x['id']for x in old]==[x['id']for x in new]==ids
errors=[]
def compare(a,b,path,geometry=False):
 if type(a)!=type(b) and not isinstance(a,(int,float)) and not isinstance(b,(int,float)):return path+': types'
 if isinstance(a,dict):
  if not isinstance(b,dict)or a.keys()!=b.keys():return path+': keys '+str(a.keys()-b.keys())+' / '+str(b.keys()-a.keys())
  for k in a:
   e=compare(a[k],b[k],path+'.'+k,geometry or k in ('fit','measured'))
   if e:return e
 elif isinstance(a,list):
  if not isinstance(b,list)or len(a)!=len(b):return path+': lengths'
  for i,(x,y)in enumerate(zip(a,b)):
   e=compare(x,y,path+f'[{i}]',geometry)
   if e:return e
 elif isinstance(a,(int,float))and not isinstance(a,bool)and isinstance(b,(int,float))and not isinstance(b,bool):
  if geometry and math.isfinite(a)and math.isfinite(b):
   if abs(a-b)>1e-12*(1+abs(a)):return path+f': {a} != {b}'
  elif a!=b:return path+f': {a} != {b}'
 elif a!=b:return path+f': {a!r} != {b!r}'
for a,b in zip(old,new):
 e=compare(a,b,a['id'])
 if e:errors.append(e);print(e)
(p/'comparison.json').write_text(json.dumps({'cases':ids,'total':len(ids),'mismatches':len(errors),'errors':errors},indent=2)+'\n')
print(f'M3d original/Rust: {len(ids)} cases; mismatches={len(errors)}');sys.exit(bool(errors))
