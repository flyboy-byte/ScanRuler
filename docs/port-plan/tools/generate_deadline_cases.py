# SPDX-License-Identifier: AGPL-3.0-only
"""Authored timer/body schedules only; never reads source/Rust results."""
import json,hashlib,random,struct,math
from pathlib import Path
from generate_schema_cases import wire
ROOT=Path(__file__).resolve().parents[3];OUT=ROOT/'docs/port-plan/fixtures/command-deadline'
def generate():
 OUT.mkdir(parents=True,exist_ok=True);rows=[]
 patterns=[['fire','resolve','reject'],['resolve','fire','reject'],['reject','fire','resolve'],['resolve-fire','reject','fire'],['fire-resolve','reject','fire'],['reject-fire','resolve','fire'],['fire-reject','resolve','fire']]
 def add(id,mode,abort,ms,ops,kind='plain',name='probe.run'):
  b=wire(dict(name=name,ms=ms,mode=mode,abort=abort,body_kind=kind,value='done 🦀 \ud800',fallback='fallback \udfff',ops=ops+['resolve','reject','fire']));(OUT/(id+'.bin')).write_bytes(b);rows.append(dict(id=id,bytes=len(b),sha256=hashlib.sha256(b).hexdigest()))
 modes=['pending','resolved','rejected','throw','fire-resolve','resolve-fire','fire-reject','reject-fire'];durations=[0,1,1234,600000,float('nan'),float('inf'),1e21,5e-324,-1000,0.1]
 for mode in modes:
  for abort in ['ignore','resolve','reject-reason']:
   for ms in durations:
    i=len(rows);add('case-'+str(i).zfill(3),mode,abort,ms,patterns[i%len(patterns)],['plain','typed','typeerror'][i%3],name='probe.測定\ud800'if i%7==0 else 'probe.run')
 for seed in [1,7,99,1039]:
  rng=random.Random(seed)
  for i in range(12):
   while True:
    ms=struct.unpack('<d',rng.getrandbits(64).to_bytes(8,'little'))[0]
    if math.isfinite(ms):break
   add('seed-'+str(seed)+'-'+str(i),'pending','ignore',ms,['fire','reject','resolve'],kind=['plain','typed','typeerror'][i%3])
 for policy in ['ignore','resolve','reject-reason']:
  add('factory-timer-'+policy,'fire-only',policy,1000,['reject','resolve','fire'],kind='typed')
 (OUT/'manifest.json').write_text(json.dumps(dict(reference_sha='2cd89c6769a10c27ee7754b477ada521c9ef5cb7',seeds=[1,7,99,1039],cases=rows),indent=2)+'\n');print(len(rows),'deadline cases')
if __name__=='__main__':generate()
