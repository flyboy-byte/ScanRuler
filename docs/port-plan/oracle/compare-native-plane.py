# SPDX-License-Identifier: AGPL-3.0-only
# Exact typed command state comparison, retaining the existing marked-plane fit tolerance.
import struct,sys,json
from pathlib import Path
def read(path):
 data=Path(path).read_bytes();offset=0
 def take(n):
  nonlocal offset
  b=data[offset:offset+n];offset+=n;assert len(b)==n;return b
 def u():return struct.unpack('<I',take(4))[0]
 def value():
  t=take(1)[0]
  if t==0:return {'$wireUndefined':True}
  if t==1:return None
  if t in [2,3]:return t==2
  if t==4:return struct.unpack('<d',take(8))[0]
  if t==5:return take(u()*2).decode('utf-16-le',errors='surrogatepass')
  if t==6:return [value() for _ in range(u())]
  if t==7:return {value():value() for _ in range(u())}
  raise AssertionError(t)
 v=value();assert offset==len(data);return v
FLOATS={'center','normal','basisU','basisV','extentU','extentV','sigma','formError','residualMin','residualMax'}
def compare(a,b,path=()):
 if type(a)!=type(b):return [(path,a,b)]
 if isinstance(a,dict):
  if list(a)!=list(b):return [(path+('keys',),list(a),list(b))]
  return [d for k in a for d in compare(a[k],b[k],path+(k,))]
 if isinstance(a,list):
  if len(a)!=len(b):return [(path+('length',),len(a),len(b))]
  return [d for i,(x,y) in enumerate(zip(a,b))for d in compare(x,y,path+(i,))]
 if isinstance(a,float) and any(anchor in path and any(k in FLOATS for k in path[path.index(anchor)+1:]) for anchor in ('fit','measured')):
  if abs(a-b)<=1e-12*(1+abs(a)):return []
 return [] if a==b else [(path,a,b)]
if __name__=='__main__':
 diffs=compare(read(sys.argv[1]),read(sys.argv[2]));
 for p,a,b in diffs[:8]:print(repr(p),repr(a)[:600],repr(b)[:600])
 sys.exit(bool(diffs))
