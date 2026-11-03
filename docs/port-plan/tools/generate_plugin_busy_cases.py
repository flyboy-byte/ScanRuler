# SPDX-License-Identifier: AGPL-3.0-only
"""Independent authored matrix; no original or Rust outputs enter generation."""
from pathlib import Path
def generate():
    return ''.join(f'case-{a*24+b*6+mode:03}\t{a}\t{b}\t{mode}\n' for a in range(4) for b in range(4) for mode in range(6)).encode()
if __name__=='__main__':
    folder=Path(__file__).resolve().parents[1]/'fixtures/plugin-busy';folder.mkdir(exist_ok=True);(folder/'cases.tsv').write_bytes(generate())
