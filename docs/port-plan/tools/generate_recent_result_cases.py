# SPDX-License-Identifier: AGPL-3.0-only
"""Independent owned-data, binary64, UTF16, property-order and size inputs."""
from pathlib import Path
import struct

def generate():
    rows=[]
    def add(kind,arg=0,bits=0): rows.append(f'case-{len(rows):04}\t{kind}\t{arg}\t{bits:016x}\n')
    add(0);add(1);add(2,0);add(2,1)
    for bits in [0,1,0x8000000000000000,0x0010000000000000,0x7fefffffffffffff,0x7ff0000000000000,0xfff0000000000000,0x7ff8000000000001,0x3ff0000000000000,0x3fb999999999999a]:add(3,bits=bits)
    for seed in [1,7,99,1039]:
        state=seed
        for _ in range(24):
            state=(6364136223846793005*state+1442695040888963407)&((1<<64)-1)
            add(3,bits=state)
    for i in range(10):add(4,i)
    for n in [0,1,255,4096,16000,1048576]:add(5,n)
    for i in range(12):add(6,i,0x3fb999999999999a+i);add(7,i,0x8000000000000000 if i%2 else 0x7ff0000000000000)
    for depth in [0,1,2,12,32]:add(8,depth)
    for n in range(15995,16003):add(9,n)
    for mode in range(4):
        for n in ([2665,2666,2667,2668] if mode in [0,2] else [7998,7999,8000,8001]):add(10,n,mode)
    for mode in range(4):
        for n in range(15980,16006):add(11,n,mode)
    numbers=[line.split('\t')[3].strip() for line in rows if line.split('\t')[1]=='3']
    for bits in numbers:add(12,16000,int(bits,16))
    extra=[struct.unpack('<Q',struct.pack('<d',n))[0] for n in [1e-7,1e-6,1e20,1e21,1e22,9007199254740991.0,9007199254740992.0,1000000000000000128.0,0.0000012345,0.0000009999999999999999]]
    tiny=struct.unpack('<Q',struct.pack('<d',1e-6))[0]
    extra += [tiny-1,tiny+1]
    for bits in extra:add(3,bits=bits);add(12,16000,bits)
    return ''.join(rows).encode()

if __name__=='__main__':
    folder=Path(__file__).resolve().parents[1]/'fixtures/recent-result';folder.mkdir(exist_ok=True);(folder/'cases.tsv').write_bytes(generate())
