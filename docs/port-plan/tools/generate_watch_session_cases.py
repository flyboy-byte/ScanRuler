# SPDX-License-Identifier: AGPL-3.0-only
"""Fixed watchable protocol inputs, independent of both implementations."""
from pathlib import Path
def generate():
 roles=[([],[],[],[]),([0],[],[],[]),([0,1],[1,2],[2,3],[3,0]),([2,0,2],[1,0],[3,1],[]),([],[0,1,0],[1,2],[2,3,0]),([3,2],[],[1,0],[0,3]),([1],[1],[1],[1]),([0,1],[],[],[])]
 failures=[(-1,-1,0),(1,-1,0),(3,-1,0),(-1,1,1),(-1,1,2),(-1,0,1)]
 rows=[]
 for role in roles:
  for sub,stop,stopmode in failures:
   for immediate,listenerfail in [(0,0),(5,0),(5,1)]:
    n=len(rows);rows.append(f'case-{n:03}\t'+'\t'.join(','.join(map(str,x))for x in role)+f'\t{sub}\t{stop}\t{stopmode}\t{immediate}\t{listenerfail}\t{n%2}')
 return ('\n'.join(rows)+'\n').encode()
if __name__=='__main__':
 folder=Path(__file__).resolve().parents[1]/'fixtures/watch-session';folder.mkdir(exist_ok=True);(folder/'cases.tsv').write_bytes(generate())
