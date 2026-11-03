# SPDX-License-Identifier: AGPL-3.0-only
"""Immutable authored/seeded inputs; never reads source or Rust answers."""
from pathlib import Path
def generate():
 rows=[]
 def add(initial,modes,actions,targets,after):rows.append(f'case-{len(rows):03}\t'+','.join(map(str,initial))+'\t'+','.join(map(str,modes))+'\t'+','.join(map(str,actions))+'\t'+','.join(map(str,targets))+'\t'+after)
 for initial in [[0],[0,1],[0,2],[0,1,2]]:
  for action in range(7):
   for after in ['none','old0','reregister0']:
    n=len(rows);add(initial,[(n+i)%6 for i in range(3)],[action,0,0],[1,2,0],after)
 for seed in [1,7,99,1039]:
  state=seed
  def rand(n):
   global state
   state=(1664525*state+1013904223)&0xffffffff;return state%n
  for i in range(8):add([0,1,2],[rand(6)for _ in range(3)],[rand(7)for _ in range(3)],[rand(3)for _ in range(3)],['none','old0','reregister0'][rand(3)])
 return ('\n'.join(rows)+'\n').encode()
if __name__=="__main__":
 (Path(__file__).resolve().parents[1]/"fixtures/settle-registry/cases.tsv").write_bytes(generate())
