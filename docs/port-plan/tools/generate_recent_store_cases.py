# SPDX-License-Identifier: AGPL-3.0-only
"""Independent clocks, completion reactions, copy and eviction schedules."""
from pathlib import Path

def generate():
    rows=[]
    for mode in [0,1,2,3,4,5,8,9,10]:
        for start in range(10):
            for end in range(4):rows.append(f'case-{len(rows):04}\t{mode}\t{start}\t{end}\n')
    for mode in [6,7]:rows.append(f'case-{len(rows):04}\t{mode}\t0\t0\n')
    return ''.join(rows).encode()

if __name__=='__main__':
    folder=Path(__file__).resolve().parents[1]/'fixtures/recent-store';folder.mkdir(exist_ok=True);(folder/'cases.tsv').write_bytes(generate())
