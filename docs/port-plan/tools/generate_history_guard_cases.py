# SPDX-License-Identifier: AGPL-3.0-only
import random
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
rows=[]
masks=[0,4095]+[1<<i for i in range(12)]+[4095^(1<<i)for i in range(12)]+[(1<<i)|(1<<j)for i in range(12)for j in range(i)]
for mask in masks:rows.append((mask,0,0,0,0,0,-1,0))
for slot in range(4):
 for code in range(9):
  row=[0,0,0,0,0,0,-1,0];row[slot+1]=code;rows.append(tuple(row))
for mask in range(8):
 for fault in [-1,0,1,2]:rows.append((0,0,0,0,0,mask,fault,0))
for mask in [0,1,2,4]:rows.append((mask,0,0,0,0,0,-1,1))
for seed in [1,7,99,1039]:
 rng=random.Random(seed)
 for _ in range(12):rows.append((rng.randrange(4096),*(rng.randrange(9)for _ in range(4)),rng.randrange(8),rng.choice([-1,0,1,2]),rng.randrange(2)))
(ROOT/'fixtures/history-guards/cases.tsv').open('w').write(''.join(f'case-{i:03d}\t'+'\t'.join(map(str,r))+'\n'for i,r in enumerate(rows)))
print(len(rows))
