# SPDX-License-Identifier: AGPL-3.0-only
"""Independent strict-discard/raw-busy/deferred/wait/body matrix."""
from pathlib import Path
def generate():
    rows=[]
    for discard in range(5):
      for busy in range(4):
       for deferred in range(2):
        for wait in range(6):
         for body in range(5):
          rows.append(f'case-{len(rows):04}\t{discard}\t{busy}\t{deferred}\t{wait}\t{body}\n')
    return ''.join(rows).encode()
if __name__=='__main__':
    folder=Path(__file__).resolve().parents[1]/'fixtures/execute';folder.mkdir(exist_ok=True);(folder/'cases.tsv').write_bytes(generate())
