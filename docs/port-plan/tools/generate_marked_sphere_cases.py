# SPDX-License-Identifier: AGPL-3.0-only
"""Authored marked-sphere settings/selection profiles, no expected outputs."""
from pathlib import Path
import struct

def generate():
    rows=[]
    def add(mode,seed,k,known,target='default'):rows.append(f'case-{len(rows):05}\t{mode}\t{seed}\t{k}\t{known}\t{target}\n')
    known=['none','20','19.4','0.0625','2','4','0','-1','nan','inf','1e21']
    for mode in [0,1,2,3,4,5,6,7,8,9,13,14,15,16,17,18]:
        for seed in [1,7,99,1039]:
            for k in [0,1,3]:
                for d in known:add(mode,seed,k,d,24 if mode in [5,16,17,18]else'default')
    for target in [0,1,19,20,21,24]:
        for d in known+['5e-324','-0','-inf']:add(16,1,3,d,target)
    for d in [20/1.1,20/0.9]:
        raw=struct.unpack('>Q',struct.pack('>d',d))[0]
        for delta in [-1,0,1]:
            neighbor=struct.unpack('>d',struct.pack('>Q',raw+delta))[0]
            for seed in [1,7,99,1039]:add(16,seed,3,format(neighbor,'.17g'),24)
    return ''.join(rows).encode()

if __name__=='__main__':
    folder=Path(__file__).resolve().parents[1]/'fixtures/marked-sphere';folder.mkdir(exist_ok=True);(folder/'cases.tsv').write_bytes(generate())
