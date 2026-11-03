# SPDX-License-Identifier: AGPL-3.0-only
"""Fixed operation/type inputs; no source or Rust answers."""
from pathlib import Path
import itertools
def generate():
 rows=[]
 for modes in itertools.product(range(4),repeat=3):
  for action in range(3):
   text=['busy','',"Failed to fetch dynamically imported module",'busy \ud800'][len(rows)%4]
   raw=text.encode('utf-16-le','surrogatepass');h=''.join(f'{int.from_bytes(raw[i:i+2],"little"):04x}' for i in range(0,len(raw),2))
   rows.append(f'case-{len(rows):03}\t'+','.join(map(str,modes))+f'\t{action}\t'+h)
 return ('\n'.join(rows)+'\n').encode()
if __name__=='__main__':
 folder=Path(__file__).resolve().parents[1]/'fixtures/activity-guard';folder.mkdir(exist_ok=True);(folder/'cases.tsv').write_bytes(generate())
