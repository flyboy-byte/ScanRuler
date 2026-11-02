# SPDX-License-Identifier: AGPL-3.0-only
from pathlib import Path
import hashlib,json,struct,copy
OUT=Path(__file__).resolve().parents[1]/'fixtures/obj-mesh';OUT.mkdir(parents=True,exist_ok=True)
rows=[]
def w(n):return struct.pack('<I',n)
def s(x):
 b=x.encode();return w(len(b))+b
def img(i):return s(i['name'])+w(i['width'])+w(i['height'])+w(len(i['rgba']))+bytes(i['rgba'])
def add(id,text,given=None,success=True):
 raw=text if isinstance(text,bytes) else text.encode();b=w(len(raw))+raw+bytes([given is not None])
 if given is not None:
  mats=given.get('materials',[]);images=given.get('images',[]);b+=w(len(mats))+b''.join(s(m['name'])+s(m['text']) for m in mats)+w(len(images))+b''.join(img(i) for i in images)
 (OUT/(id+'.bin')).write_bytes(b);rows.append({'id':id,'success':success,'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()})
V='v 0 0 0\nv 1 0 0\nv 0 1 0\n';T=V+'f 1 2 3\n'
for id,text,ok in [('triangle',T,1),('quad',V+'v 1 1 0\nf 1 2 4 3\n',1),('negative',V+'f -3 -2 -1\n',1),('negative-leading-zero',V+'f -03 -02 -01\n',1),('forward-face','f 1 2 3\n'+V,1),('future-vertex',V+'f 1 2 4\nv 1 1 0\n',1),('huge-forward-index',V+'f 1 2 4294967295\n',1),('ignored-normals-groups','o part\ng first\ns off\nvn 0 0 1\n'+V+'f 1//9 2//9 3//9\n',1),('short-faces',V+'f 1\nf 1 2\nf 1 2 3',1),('empty','',0),('vertices-only',V,0),('short-only',V+'f 1 2\n',0),('no-vertices','f 1 2 3',0),('short-directive','v \nvt \nf \n',0),('bad-vertex','v x 0 0\n'+T,0),('missing-coordinate','v 0 0\n'+T,0),('nonfinite-vertex','v Infinity 0 0\n'+T,0),('prefix-not-number','v 1junk 0 0\n'+T,0),('extra-vertex-fields',V+'v 1 1 0 2 ignored\nf 1 2 4',1),('homogeneous-ignored','v 0 0 0 2\nv 1 0 0 2\nv 0 1 0 2\nf 1 2 3',1),('zero-signs','v -0 -0 -0\nv 0 -0 0\nv 0 0 -0\nf 1 2 3',1),('finite-to-f32-infinity','v 1e308 -1e308 1e-320\nv 0 0 0\nv 1 1 1\nf 1 2 3',1),('unicode-separator-between','v 0\u00a00\u20030\nv 1 0 0\nv 0 1 0\nf 1\u00a02\u20033',1),('non-ascii-directive-separator','v\u00a00 0 0\n',0),('crlf',T.replace('\n','\r\n'),1),('bom',b'\xef\xbb\xbf'+T.encode(),1),('invalid-utf8-comment',b'# \xf0\x80\xff\n'+T.encode(),1),('inline-comment',V+'f 1 2 3# tail',1),('unicode-mtl-name','usemtl 名\n'+T,1),('number-hex','v 0x10 0Xf 0b11\nv 0o7 0O10 0B101\nv +.5 1. 1.e2\nf 1 2 3',1),('signed-hex-refusal','v -0x1 0 0\n'+T,0),('hex-big-rounding','v 0x20000000000001 0x20000000000003 0x1000000000000080\nv 0 0 0\nv 1 1 1\nf 1 2 3',1),('binary-big-rounding','v 0b1'+('0'*52)+'1 0o400000000000000003 0\nv 0 0 0\nv 1 1 1\nf 1 2 3',1),('invalid-number-token','v 0x 0 0\n'+T,0),('non-js-space','v 0\u00850 0 0\n'+T,0)]:add(id,text,success=bool(ok))
for i,t in enumerate(['0','-0','+1','1.0','1e0','0x1','-4','4294967296','9007199254740992','x','1x','--1','-','１２']):add('bad-face-'+str(i),V+'f '+t+' 2 3\n',success=False)
quad={'name':'tex.png','width':2,'height':2,'rgba':[255,0,0,0,0,255,0,255,0,0,255,127,255,255,255,64]}
M={'name':'part.mtl','text':'newmtl skin\nmap_Kd C:\\scanner\\out\\TEX.png\n'}
Q='mtllib part.mtl\n'+V+'v 1 1 0\nvt .25 .25\nvt .75 .25\nvt .75 .75\nvt .25 .75\nusemtl skin\nf 1/1 2/2 4/3 3/4\n'
add('texture-applied',Q,{'materials':[M],'images':[quad]});add('texture-missing',Q);add('texture-image-missing',Q,{'materials':[M]});add('texture-only-image',Q,{'images':[quad]});add('texture-empty-companions',Q,{})
for id,transform in [('uv-negative',lambda q:q.replace('1/1 2/2 4/3 3/4','1/-4 2/-3 4/-2 3/-1')),('uv-zero',lambda q:q.replace('1/1','1/0')),('uv-hex',lambda q:q.replace('1/1','1/0x1')),('uv-plus',lambda q:q.replace('1/1','1/+1')),('uv-fraction-invalid',lambda q:q.replace('1/1','1/1.5')),('uv-unavailable',lambda q:q.replace('1/1','1/99')),('uv-double-slash',lambda q:q.replace('1/1','1//3')),('uv-nonfinite-sanitized',lambda q:q.replace('vt .25 .25','vt NaN Infinity')),('uv-prefix-invalid-sanitized',lambda q:q.replace('vt .25 .25','vt .25junk 1e+')),('uv-missing-v',lambda q:q.replace('vt .25 .25','vt .25')),('uv-huge-finite',lambda q:q.replace('vt .25 .25','vt 1e308 -1e308')),('uv-only-empty-faces',lambda q:q.replace('f 1/1 2/2 4/3 3/4','f 1 2 4 3')),('sparse-forward-uv',lambda q:q.replace('f 1/1 2/2 4/3 3/4','f 1/1 2/2 4294967295/3'))]:add(id,transform(Q),{'materials':[M],'images':[quad]})
red={'name':'red.png','width':1,'height':1,'rgba':[255,0,0,0]};green={'name':'green.png','width':1,'height':1,'rgba':[0,255,0,255]};G={'materials':[{'name':'part.mtl','text':'newmtl red\nmap_Kd red.png\nnewmtl green\nmap_Kd green.png'}],'images':[red,green]}
base='mtllib part.mtl\n'+V+'vt .2 .3\nvt .7 .8\n'
add('first-textured-corner-wins',base+'usemtl red\nf 1/1 2/1 3/1\nusemtl green\nf 1/2 2/2 3/2\n',G)
add('first-invalid-uv-does-not-bind',base+'usemtl red\nf 1/99 2/1 3/1\nusemtl green\nf 1/2 2/2 3/2\n',G)
add('first-no-material-stays',base+'f 1/1 2/1 3/1\nusemtl green\nf 1/2 2/2 3/2\n',G)
add('used-note-vertex-order',base+'usemtl green\nf 3/1 3/1 3/1\nusemtl red\nf 1/1 2/1 1/1\n',G)
add('future-uv-not-retroactive','mtllib part.mtl\n'+V+'usemtl skin\nf 1/1 2/1 3/1\nvt .25 .25\n',{'materials':[M],'images':[quad]})
add('empty-material-name','mtllib part.mtl\n'+V+'vt 0 0\nusemtl\nf 1/1 2/1 3/1\n',{'materials':[{'name':'part.mtl','text':'newmtl\nmap_Kd tex.png'}],'images':[quad]})
add('keyword-prefixes','mtllibpart.mtl\n'+V+'vt 0 0\nusemtlskin\nf 1/1 2/1 3/1\n',{'materials':[M],'images':[quad]})
add('error-before-color-progress','mtllib part.mtl\nvt .2 .3\nv x 0 0\nf 1/1 1/1 1/1',{'materials':[M],'images':[quad]},False)
for seed in [1,7,99,1039]:
 s0=seed;verts=[]
 for i in range(24):
  xyz=[]
  for _ in range(3):s0=(1664525*s0+1013904223)&0xffffffff;xyz.append(format((s0/4294967296-.5)*100,'.17g'))
  verts.append('v '+' '.join(xyz))
 text='\n'.join(verts)+'\n'+''.join('f '+str(i+1)+' '+str(i+2)+' '+str(i+3)+'\n' for i in range(0,24,3));add('seed-'+str(seed),text)
# Exact original line/progress boundaries, no reduced input or injected callback.
N=1<<20
prefix=V+'noop\n'*(N-3)
add('line-progress',prefix+'noop\nf 1 2 3\n')
add('short-unicode-line-skips-progress',prefix+'🦀\nf 1 2 3\n')
(OUT/'manifest.json').write_text(json.dumps({'schema':1,'reference_sha':'2cd89c6769a10c27ee7754b477ada521c9ef5cb7','license':'AGPL-3.0-only','seeds':[1,7,99,1039],'comparison':'exact complete OBJ1 positions/indices/colors/texture note, progress and source refusal; no tolerance','cases':rows},indent=2)+'\n')
print(len(rows),'fixed OBJ parser inputs;',sum(c['success'] for c in rows),'declared successful')
