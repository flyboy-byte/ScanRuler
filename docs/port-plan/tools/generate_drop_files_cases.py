#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
"""Independent callback snapshots, not source/Rust results."""
import hashlib,json,random,struct
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3];OUT=ROOT/'docs/port-plan/fixtures/drop-files';OUT.mkdir(exist_ok=True);rows=[]
w=lambda n:struct.pack('<I',n&0xffffffff)
t=lambda s:w(len(s.encode()))+s.encode()
def err(e):return b'\0'if e is None else b'\1'+t(e[0])+t(e[1])
def ids(v):return w(len(v))+b''.join(w(i)for i in v)
def entry(flags=1,file=0,batches=None,fe=None,re=None):return dict(flags=flags,file=file,batches=batches or [],fe=fe,re=re)
def item(e,kind=True,api=True,error=None):return dict(entry=e,kind=kind,api=api,error=error)
def batch(v,error=None):return dict(ids=v,error=error)
E=('NotReadableError','authored failure: café')
def add(name,entries,items,fallback=None,present=True):
 b=bytes([present])+w(8)+ids([7,6]if fallback is None else fallback)+w(len(items))
 for i in items:b+=bytes([i['kind'],i['api']])+w(-1 if i['entry']is None else i['entry'])+err(i['error'])
 b+=w(len(entries))
 for e in entries:
  b+=bytes([e['flags']])+w(e['file'])+err(e['fe'])+err(e['re'])+w(len(e['batches']))
  for x in e['batches']:b+=err(x['error'])+ids(x['ids'])
 (OUT/(name+'.bin')).write_bytes(b);rows.append(dict(id=name,bytes=len(b),sha256=hashlib.sha256(b).hexdigest()))
f=lambda n:entry(file=n)
d=lambda batches,**kw:entry(flags=2,batches=batches,**kw)
add('null',[],[],present=False);add('empty',[],[],fallback=[])
add('fallback',[f(0),f(1)],[item(0),item(1)])
add('no-entries',[],[item(None),item(None)])
add('non-file-items',[d([])],[item(0,kind=False)])
add('one-directory',[d([batch([1,2])]),f(1),f(2)],[item(0)])
add('multiple-batches',[d([batch([1]),batch([2]),batch([])]),f(1),f(2)],[item(0)])
add('batch-error-before-file-reads',[d([batch([1]),batch([],E)]),f(1)],[item(0)])
add('one-level-only',[d([batch([1,2])]),d([batch([3])]),f(2),f(3)],[item(0)])
add('duplicate-directory-readers',[d([batch([1])]),f(1)],[item(0),item(0)])
add('both-flags',[entry(flags=3,file=3),d([batch([2])]),f(2)],[item(0),item(1)])
add('no-entry-flags',[entry(flags=0),d([batch([0,2])]),f(2)],[item(0),item(1)])
add('mixed-top-order',[f(0),d([batch([3])]),f(2),f(3)],[item(0),item(1),item(2)])
add('file-error',[entry(file=0,fe=E),d([])],[item(0),item(1)])
add('entry-error',[d([])],[item(0,error=E),item(0)])
add('reader-error',[d([],re=E)],[item(0)])
add('batch-error',[d([batch([],E)])],[item(0)])
add('child-file-error',[d([batch([1,2])]),entry(file=1,fe=E),f(2)],[item(0)])
add('missing-getter',[d([])],[item(0,api=False)])
add('missing-entry-with-directory',[d([batch([1])]),f(1)],[item(None),item(0)])
add('after-empty-ignored',[d([batch([]),batch([1],E)]),f(1)],[item(0)])
add('duplicate-child-identities',[d([batch([1,1,2])]),f(3),f(3)],[item(0)])
add('getters-before-traversal',[d([batch([2])]),f(1),f(2)],[item(0),item(1,error=E)])
add('unicode-error-fields',[d([batch([],('TypeError','零\0é'))])],[item(0)])
for seed in [1,7,99,1039]:
 rng=random.Random(seed)
 for j in range(4):
  entries=[d([batch([2,3]),batch([4,2])]),d([batch([5,3])])]+[f(rng.randrange(8))for _ in range(4)]
  if j==2:entries[3]['fe']=('NotFoundError',f'seed {seed}')
  items=[item(i)for i in rng.sample(range(6),6)]
  if j==3:items[2]['error']=('SecurityError',f'seed {seed}')
  add(f'seed-{seed}-{j}',entries,items)
(OUT/'manifest.json').write_text(json.dumps(dict(contract='drop.files.input.v1',license='AGPL-3.0-only',reference_sha='2cd89c6769a10c27ee7754b477ada521c9ef5cb7',seeds=[1,7,99,1039],cases=rows),indent=2)+'\n')
lines=['// SPDX-License-Identifier: AGPL-3.0-only','vec![']+[f' Case {{ id: "{r["id"]}", input: include_bytes!("../../../../../docs/port-plan/fixtures/drop-files/{r["id"]}.bin") }},'for r in rows]+[']']
(ROOT/'rust/crates/port-io/tests/support/drop_files_cases.rs').write_text('\n'.join(lines)+'\n');print(len(rows),'fixed callback traces')
