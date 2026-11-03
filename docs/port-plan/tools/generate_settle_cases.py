# SPDX-License-Identifier: AGPL-3.0-only
"""Immutable authored/seeded inputs; never reads source or Rust answers."""
from pathlib import Path
def generate():
 rows=[]
 def add(kinds,readonly,order):
  name='element.fit\ud800' if len(rows)%2 else 'session.state';raw=name.encode('utf-16-le','surrogatepass');h=''.join(f'{int.from_bytes(raw[i:i+2],"little"):04x}' for i in range(0,len(raw),2));rows.append(f'case-{len(rows):03}\t'+','.join(map(str,kinds))+f'\t{readonly}\t{order}\t{h}')
 for kinds in [[],[0],[1],[2],[3],[4],[5],[0,1,0],[2,3],[4,5],[1,4,3],[5,4,2],[4,4,4],[5,5,5],[0,2,1,5],[5,0,3,4]]:
  for readonly in [0,1]:add(kinds,readonly,'reverse' if readonly else 'forward')
 for seed in [1,7,99,1039]:
  state=seed
  for i in range(8):
   kinds=[]
   for j in range(4):state=(1664525*state+1013904223)&0xffffffff;kinds.append(state%6)
   add(kinds,i%2,'all' if i%3==0 else 'reverse' if i%3==1 else 'forward')
 return ('\n'.join(rows)+'\n').encode()
if __name__=="__main__":
 (Path(__file__).resolve().parents[1]/"fixtures/settle-run/cases.tsv").write_bytes(generate())
