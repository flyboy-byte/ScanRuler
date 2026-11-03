# SPDX-License-Identifier: AGPL-3.0-only
"""Fixed Date inputs; no reference or Rust answers are read."""
from pathlib import Path
import datetime as dt
import math
import struct

def generate():
 values=[math.nan,math.inf,-math.inf,-8640000000000001.,-8640000000000000.,-8639999999999999.,8640000000000001.,8640000000000000.,8639999999999999.,-1000.001,-999.999,-1.999,-1.,-.999,-0.,0.,.999,1.,1.999,999.999,1000.001,86400000.,-86400000.]
 epoch=dt.date(1970,1,1)
 for year in [1,4,100,400,1600,1700,1900,1969,1970,1999,2000,2004,2100,2400,9999]:
  for month,day in [(1,1),(2,28),(3,1),(12,31)]:
   ms=(dt.date(year,month,day)-epoch).days*86400000
   values.extend([float(ms-1),float(ms),float(ms+1)])
 for seed in [1,7,99,1039]:
  state=seed
  for _ in range(16):
   state=(6364136223846793005*state+1442695040888963407)&((1<<64)-1)
   values.append(float(state%17280000000000001-8640000000000000))
  for _ in range(8):
   state=(6364136223846793005*state+1442695040888963407)&((1<<64)-1)
   values.append(struct.unpack('>d',struct.pack('>Q',state))[0])
 for start in [(dt.date(1,1,1)-epoch).days*86400000-366*86400000,((dt.date(9999,12,31)-epoch).days+1)*86400000]:
  values.extend(float(start+delta) for delta in [-1,0,1,31*86400000,59*86400000,60*86400000])
 return ''.join(f'case-{i:03}\t'+struct.pack('>d',v).hex()+'\n' for i,v in enumerate(values)).encode()
def generate_period():
 raw=generate();rows=[];start=len(raw.splitlines());epoch=dt.date(1970,1,1)
 for year in range(1600,2000):
  for month,day in [(1,1),(3,1)]:
   for delta in [-1,0]:
    ms=(dt.date(year,month,day)-epoch).days*86400000-5*146097*86400000+delta
    rows.append(f'case-{start+len(rows):03}\t'+struct.pack('>d',float(ms)).hex()+'\n')
 return raw+''.join(rows).encode()
if __name__=='__main__':
 folder=Path(__file__).resolve().parents[1]/'fixtures/clock-utc';folder.mkdir(exist_ok=True)
 (folder/'cases.tsv').write_bytes(generate_period())
