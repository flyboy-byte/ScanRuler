# SPDX-License-Identifier: AGPL-3.0-only
"""Independent complete code-point blocks and authored UTF16/callback inputs."""
from pathlib import Path

def generate():
    rows=[]
    def add(kind,a=0,b=0):rows.append(f'case-{len(rows):04}\t{kind}\t{a}\t{b}\n')
    for kind in [0,1]:
        for start in range(0,0x110000,4096):add(kind,start,min(start+4096,0x110000))
    for pattern in range(12):
        for mode in range(6):add(2,pattern,mode)
    for pattern in range(12):add(2,pattern,6)
    return ''.join(rows).encode()

if __name__=='__main__':
    folder=Path(__file__).resolve().parents[1]/'fixtures/command-label';folder.mkdir(exist_ok=True);(folder/'cases.tsv').write_bytes(generate())
