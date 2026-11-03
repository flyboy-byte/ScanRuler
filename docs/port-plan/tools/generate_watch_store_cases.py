# SPDX-License-Identifier: AGPL-3.0-only
"""Authored identity/operation inputs, independent of both engines."""
from pathlib import Path
def generate():
 return ''.join(f'case-{a*20+b*5+mode:03}\t{a}\t{b}\t{mode}\n' for a in range(4)for b in range(4)for mode in range(5)).encode()
if __name__=='__main__':
 folder=Path(__file__).resolve().parents[1]/'fixtures/watch-stores';folder.mkdir(exist_ok=True);(folder/'cases.tsv').write_bytes(generate())
