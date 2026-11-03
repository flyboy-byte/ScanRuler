# SPDX-License-Identifier: AGPL-3.0-only
"""Authored primitive Number bits; no expected strings or Rust-derived outputs."""
from pathlib import Path
import struct

def generate():
    values=[]
    def add(bits):values.append(bits&((1<<64)-1))
    def bits(x):return struct.unpack('>Q',struct.pack('>d',x))[0]
    for n in [0,1,2,0x000fffffffffffff,0x0010000000000000,0x7fefffffffffffff,0x7ff0000000000000,0x7ff8000000000000,0x7ff0000000000001,0x7ffabcdef1234567]:
        add(n);add(n|(1<<63))
    for x in [0.0005,0.0015,0.0625,0.1875,0.3125,0.4375,0.5625,0.6875,0.8125,0.9375,1.0625,9.9995,99.9995,999.9995,1e-9,1e-7,1e-6,1,10,20,100,1e18,1e20,1e21,1e22,1e23,1e100,1e308]:
        for delta in [-2,-1,0,1,2]:
            add(bits(x)+delta);add((bits(x)+delta)|(1<<63))
    for seed in [1,7,99,1039]:
        state=seed
        for _ in range(2048):
            state=(state*6364136223846793005+1442695040888963407)&((1<<64)-1);add(state)
    return ''.join(f'case-{i:05}\t{value:016x}\n'for i,value in enumerate(values)).encode()

if __name__=='__main__':
    (Path(__file__).resolve().parents[1]/'fixtures/fixed3/cases.tsv').write_bytes(generate())
