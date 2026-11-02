# SPDX-License-Identifier: AGPL-3.0-only
# Independent inputs only. No original/Rust outputs are read.
from pathlib import Path
import hashlib,json,struct
out=Path(__file__).resolve().parents[1]/'fixtures/engine-errors';out.mkdir(parents=True,exist_ok=True)
rows=[]
def units(s):return list(struct.unpack('<'+'H'*(len(s.encode('utf-16-le'))//2),s.encode('utf-16-le')))
def add(id,code,msg):
 b=bytes([code])+struct.pack('<I',len(msg))+b''.join(struct.pack('<H',u) for u in msg)
 (out/(id+'.bin')).write_bytes(b);rows.append({'id':id,'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()})
messages=[('empty',[]),('escaping',units('A "quote" \\ /\b\f\n\r\t\x00\x01\x1f\u2028\u2029')),
 ('unicode',units('Μικρό 測定 🦀 e\u0301 é')),
 ('surrogates',[0xd800,65,0xdc00,0xd83e,0xdd80,0xd800,0xd800,0xdc00,0])]
for code in range(12):
 for name,msg in messages:add(f'code-{code:02d}-{name}',code,msg)
for seed in [1,7,99,1039]:
 s=seed;msg=[]
 for _ in range(256):s=(1664525*s+1013904223)&0xffffffff;msg.append(s>>16)
 add('seed-'+str(seed),seed%12,msg)
(out/'manifest.json').write_text(json.dumps({'schema':1,'reference_sha':'2cd89c6769a10c27ee7754b477ada521c9ef5cb7','license':'AGPL-3.0-only','seeds':[1,7,99,1039],'comparison':'exact ERR1 field frame: code UTF8, name/message/display UTF16, original JSON UTF8; every code and message unit','cases':rows},indent=2)+'\n')
print(len(rows),'command-error construction/serialization cases')
