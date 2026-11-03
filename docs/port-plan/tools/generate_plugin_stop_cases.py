# SPDX-License-Identifier: AGPL-3.0-only
"""Independent full truthiness, stop-outcome and mutation matrix."""
from pathlib import Path
def generate():
    return ''.join(f'case-{action*32+phase*16+a*4+b:03}\t{a}\t{b}\t{action}\t{phase}\t{(action+a+b)%5}\t{(phase+a*2+b)%5}\n' for action in range(7) for phase in range(2) for a in range(4) for b in range(4)).encode() + b'case-224\t3\t3\t7\t0\t0\t0\ncase-225\t2\t2\t7\t1\t3\t4\ncase-226\t2\t2\t8\t0\t3\t3\n'
if __name__=='__main__':
    folder=Path(__file__).resolve().parents[1]/'fixtures/plugin-stop';folder.mkdir(exist_ok=True);(folder/'cases.tsv').write_bytes(generate())
