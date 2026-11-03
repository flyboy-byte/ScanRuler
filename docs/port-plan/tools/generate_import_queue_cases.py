# SPDX-License-Identifier: AGPL-3.0-only
"""Authored controlled jobs/listeners, no original or Rust results."""
from pathlib import Path
import json,hashlib,random,copy
from generate_schema_cases import wire
ROOT=Path(__file__).resolve().parents[3];OUT=ROOT/'docs/port-plan/fixtures/import-queue'
def sub(i,token):return dict(op='subscribe',listener=i,token=token)
def unsub(token):return dict(op='unsubscribe',token=token)
def run(job):return dict(op='run',job=job)
def resolve(job):return dict(op='resolve',job=job)
def generate():
 OUT.mkdir(parents=True,exist_ok=True);rows=[]
 def add(id,ops,listeners=None,jobs=None):
  if listeners is None:listeners=[dict(throwAt=0)for _ in range(3)]
  if jobs is None:jobs=[dict(id=k,value=k+' 🦀 \ud800',fail=k=='B',immediate=k=='C')for k in ['A','B','C','N']]
  ops=ops+[resolve(k)for k in ['A','B','C','N']]
  b=wire(dict(listeners=listeners,jobs=jobs,ops=ops));(OUT/(id+'.bin')).write_bytes(b);rows.append(dict(id=id,bytes=len(b),sha256=hashlib.sha256(b).hexdigest()))
 add('empty',[]);add('fifo',[sub(0,'a'),run('A'),run('B'),run('C'),resolve('B'),resolve('A')]);add('error-next',[run('B'),run('A'),resolve('B'),resolve('A')]);add('immediate',[sub(0,'a'),run('C')])
 add('duplicate-subscription',[sub(0,'a'),sub(0,'b'),run('A'),unsub('a'),resolve('A')])
 add('old-unsubscribe-new-registration',[sub(0,'a'),sub(0,'b'),unsub('a'),sub(0,'c'),unsub('b'),run('C')])
 add('repeat-unsubscribe',[sub(0,'a'),unsub('a'),unsub('a'),run('C')])
 add('registration-order',[sub(2,'a'),sub(0,'b'),sub(1,'c'),run('C')])
 for at in [1,2,3]:
  for job in ['A','B']:add('listener-fault-'+str(at)+'-'+job,[sub(0,'a'),run(job),resolve(job),run('C')],listeners=[dict(throwAt=at),dict(throwAt=0),dict(throwAt=0)])
 add('remove-during-notify',[sub(0,'a'),sub(1,'b'),run('A'),resolve('A')],listeners=[dict(throwAt=0,action=dict(at=1,kind='remove',token='b')),dict(throwAt=0),dict(throwAt=0)])
 add('add-during-notify',[sub(0,'a'),run('A'),resolve('A')],listeners=[dict(throwAt=0,action=dict(at=1,kind='add',listener=1,token='b')),dict(throwAt=0),dict(throwAt=0)])
 add('added-listener-throws',[sub(0,'a'),run('A'),run('C')],listeners=[dict(throwAt=0,action=dict(at=1,kind='add',listener=1,token='b')),dict(throwAt=1),dict(throwAt=0)])
 add('enqueue-during-notify',[sub(0,'a'),sub(1,'b'),run('A'),resolve('A'),resolve('N')],listeners=[dict(throwAt=0,action=dict(at=1,kind='enqueue',job='N')),dict(throwAt=0),dict(throwAt=0)])
 add('enqueue-then-throw',[sub(0,'a'),run('A'),resolve('N'),run('C')],listeners=[dict(throwAt=1,action=dict(at=1,kind='enqueue',job='N')),dict(throwAt=0),dict(throwAt=0)])
 for seed in [1,7,99,1039]:
  rng=random.Random(seed)
  for i in range(12):
   listeners=[dict(throwAt=rng.choice([0,0,1,2,3]))for _ in range(3)];jobs=[dict(id=k,value=k+' \ud800',fail=bool(rng.randrange(2)),immediate=bool(rng.randrange(2)))for k in ['A','B','C','N']]
   order=list(range(3));rng.shuffle(order);ops=[sub(n,'token'+str(n))for n in order];order=['A','B','C'];rng.shuffle(order)
   for k in order:
    ops.append(run(k))
    if rng.randrange(2):ops.append(resolve(k))
   ops.append(unsub('token'+str(rng.randrange(3))));add('seed-'+str(seed)+'-'+str(i),ops,listeners,jobs)
 (OUT/'manifest.json').write_text(json.dumps(dict(reference_sha='2cd89c6769a10c27ee7754b477ada521c9ef5cb7',seeds=[1,7,99,1039],cases=rows),indent=2)+'\n');print(len(rows),'queue cases')
if __name__=='__main__':generate()
