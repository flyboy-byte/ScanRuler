# SPDX-License-Identifier: AGPL-3.0-only
# Independent binary PLY inputs, never expected Rust/original results.
from pathlib import Path
import hashlib
import json
import struct

OUT = Path(__file__).resolve().parents[1] / 'fixtures/ply-binary'
OUT.mkdir(parents=True, exist_ok=True)
TYPES = {'char':'b','int8':'b','uchar':'B','uint8':'B','short':'h','int16':'h',
         'ushort':'H','uint16':'H','int':'i','int32':'i','uint':'I','uint32':'I',
         'float':'f','float32':'f','double':'d','float64':'d'}
CASES = []
VERTS = [(0,0,0),(1,0,0),(0,1,0)]

def pack(t, values, endian):
    return b''.join(struct.pack(endian + TYPES[t], v) for v in values)

def add(id, props, body, endian='<', success=True, comment=''):
    fmt = 'binary_little_endian' if endian == '<' else 'binary_big_endian'
    data = ('ply\nformat '+fmt+' 1.0\n'+comment+'\n'.join(props)+'\nend_header\n').encode() + body
    (OUT / (id+'.ply')).write_bytes(data)
    CASES.append({'id':id,'success':success,'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()})

def vertex(t='float', n=3):
    return [f'element vertex {n}', f'property {t} x', f'property {t} y', f'property {t} z']

def face(ct='uchar', it='int', name='vertex_indices', n=1):
    return [f'element face {n}', f'property list {ct} {it} {name}']

for e, prefix in [('<','le'),('>','be')]:
    xyz = pack('float', [v for p in VERTS for v in p], e)
    tri = pack('uchar',[3],e) + pack('int',[0,1,2],e)
    for t in TYPES:
        add(prefix+'-vertex-'+t, vertex(t)+face(), pack(t,[v for p in VERTS for v in p],e)+tri,e)
        add(prefix+'-count-'+t, vertex()+face(t), xyz+pack(t,[3],e)+pack('int',[0,1,2],e),e)
        add(prefix+'-index-'+t, vertex()+face(it=t), xyz+pack('uchar',[3],e)+pack(t,[0,1,2],e),e)
    add(prefix+'-utf8-winding',vertex()+face(),xyz+tri,e,comment='comment Créé 🦀\ncomment ScanRuler: keep winding\n')
    add(prefix+'-mixed-properties', ['element vertex 3','property double z','property uchar red','property short x','property float y']+face(name='vertex_index'), b''.join(pack('double',[p[2]],e)+pack('uchar',[255],e)+pack('short',[p[0]],e)+pack('float',[p[1]],e) for p in VERTS)+tri,e)
    add(prefix+'-list-vertices',['element vertex 3','property list uchar short extras','property float x','property float y','property float z']+face(),b''.join(pack('uchar',[i],e)+pack('short',range(i),e)+pack('float',p,e) for i,p in enumerate(VERTS))+tri,e)
    add(prefix+'-list-coordinate-zero',['element vertex 3','property list uchar short x','property float y','property float z']+face(),b''.join(pack('uchar',[1],e)+pack('short',[99],e)+pack('float',p[1:],e) for p in VERTS)+tri,e)
    add(prefix+'-quad-fan',vertex(n=4)+face(),xyz+pack('float',[1,1,0],e)+pack('uchar',[4],e)+pack('int',[0,1,3,2],e),e)
    add(prefix+'-face-scalars-and-extra-list',vertex()+['element face 1','property ushort material','property list uchar float extra','property list uchar int vertex_indices','property double ignored'],xyz+pack('ushort',[12],e)+pack('uchar',[2],e)+pack('float',[.2,.3],e)+tri+pack('double',[17],e),e)
    add(prefix+'-unknown-fixed-element',['element junk 2','property ushort tag']+vertex()+face(),pack('ushort',[17,18],e)+xyz+tri,e)
    add(prefix+'-unknown-list-refusal',['element junk 0','property list uchar int values']+vertex()+face(),xyz+tri,e,False)
    add(prefix+'-face-before-vertex',face()+vertex(),tri+xyz,e)
    add(prefix+'-repeated-vertices',vertex(n=1)+vertex()+face(),pack('float',[9,9,9],e)+xyz+tri,e)
    add(prefix+'-repeated-faces-replace',vertex()+face()+face(),xyz+tri+pack('uchar',[3],e)+pack('int',[2,1,0],e),e)
    for n in [0,1,2]:
        # Source unconditionally reads first/previous even for short lists.
        add(prefix+'-short-padded-'+str(n),vertex()+face(),xyz+pack('uchar',[n],e)+pack('int',[0,1],e),e)
        add(prefix+'-short-truncated-'+str(n),vertex()+face(),xyz+pack('uchar',[n],e)+pack('int',range(n),e),e,n==2)
    for n,label in [(3.5,'fraction'),(float('nan'),'nan'),(-1,'negative')]:
        add(prefix+'-float-face-count-'+label,vertex()+face('float'),xyz+pack('float',[n],e)+pack('int',[0,1,2,2],e),e)
    add(prefix+'-unsigned-conversion',vertex()+face(it='double'),xyz+pack('uchar',[3],e)+pack('double',[-1,4294967297,float('nan')],e),e)
    add(prefix+'-truncated-vertex',vertex()+face(),xyz[:7],e,False)
    add(prefix+'-truncated-face',vertex()+face(),xyz+tri[:-1],e,False)
    add(prefix+'-missing-coordinate',['element vertex 1','property float x','property float y']+face(),xyz+tri,e,False)
    add(prefix+'-unknown-coordinate-type',['element vertex 1','property mystery x','property float y','property float z']+face(),xyz+tri,e,False)
    add(prefix+'-unknown-unused-stride',vertex()+['property mystery ignored']+face(),xyz+tri,e)
    add(prefix+'-unknown-skipped-stride',['element junk 1','property mystery tag']+vertex()+face(),xyz+tri,e)
    add(prefix+'-list-unknown-count',['element vertex 1','property list mystery int extra','property float x','property float y','property float z']+face(),xyz+tri,e,False)
    add(prefix+'-scalar-bounds',['element vertex 3','property char x','property uint y','property double z']+face(),b''.join(pack('char',[-128+i],e)+pack('uint',[4294967295-i],e)+pack('double',[-0.0 if i==0 else 1/3],e) for i in range(3))+tri,e)
    # Source Float32Array stores quiet signaling NaNs after Number widening.
    add(prefix+'-float-special-bits',vertex()+face(),b''.join(struct.pack(e+'I',v) for v in [0x80000000,0x7f800000,0xff800000,0x7fc12345,0xffc54321,0x7f812345,1,0x7f7fffff,0x00800000])+tri,e)
    for seed in [1,7,99,1039]:
        s=seed;values=[]
        for _ in range(72):
            s=(1664525*s+1013904223)&0xffffffff;values.append((s/4294967296-.5)*100)
        faces=b''.join(pack('uchar',[3],e)+pack('int',[i,i+1,i+2],e) for i in range(0,24,3))
        add(prefix+'-seed-'+str(seed),vertex('double',24)+face(n=8),pack('double',values,e)+faces,e)

# Both source progress boundaries, not manufactured callbacks or reduced counts.
N=(1<<20)+1
add('le-vertex-progress',vertex('char',N)+face(),bytes(N*3)+b'\x03'+struct.pack('<iii',0,1,2))
add('le-face-progress',vertex()+['element face '+str(N)],pack('float',[v for p in VERTS for v in p],'<'))
manifest={'schema':1,'reference_sha':'2cd89c6769a10c27ee7754b477ada521c9ef5cb7','license':'AGPL-3.0-only','seeds':[1,7,99,1039],
          'comparison':'exact full PLY1 float32/index/count/winding bytes and progress/refusals; no tolerance','cases':CASES}
(OUT/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(len(CASES),'binary PLY cases;',sum(c['success'] for c in CASES),'declared successes')
