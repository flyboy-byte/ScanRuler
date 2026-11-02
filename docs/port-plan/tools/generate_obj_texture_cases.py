# SPDX-License-Identifier: AGPL-3.0-only
# Authored inputs only; original/Rust outputs are never read.
from pathlib import Path
import json,struct,hashlib,math,copy
OUT=Path(__file__).resolve().parents[1]/'fixtures/obj-texture';OUT.mkdir(parents=True,exist_ok=True)
rows=[]
def word(n):return struct.pack('<I',n)
def string(s):
 b=s.encode();return word(len(b))+b
def strings(v):return word(len(v))+b''.join(string(x) for x in v)
def floats(v):return word(len(v))+b''.join(struct.pack('<d',x) for x in v)
def ints(v):return word(len(v))+b''.join(struct.pack('<i',x) for x in v)
def image(i):return string(i['name'])+word(i['width'])+word(i['height'])+word(len(i['rgba']))+bytes(i['rgba'])
def add(id,mode,payload):
 b=bytes([mode])+payload;(OUT/(id+'.bin')).write_bytes(b)
 rows.append({'id':id,'mode':mode,'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()})
for i,p in enumerate(['',' foo/BAR.PNG ','C:\\scanner\\TEX.png','/','a\\','../A/B.mtl','a/b\\C.JPG','  İΣΟΣ.PNG  ','ΟΣ/ΟΣ.PNG','Σ.Σ/ΑΣΑ.PNG','e\u0301/É.PNG','\ufeffX\u00a0.PNG\ufeff','a/\u2003FOO.PNG\u2003','a/b#Tag.PNG','FOO.PNG','foo.png','🦀/CRAB.JPG']):add('basename-'+str(i),1,string(p))
mtls=['','map_Kd ignored.png','newmtl a\nmap_Kd first.png\nnewmtl b\nmap_Kd second.png\nnewmtl a\nmap_Kd third.png',
 'newmtl\nmap_Kd empty-name.png','newmtlPREFIX\nMAP_KD Mixed Case.PNG # comment','newmtl a\nKd 1 1 1',
 'newmtl a\nmap_Kd\nmap_Kd \n','newmtl a\nmap_Kd --unknown 1 file.png','newmtl a\nmap_Kd -s 1e2 image.png',
 'newmtl a\nmap_Kd -s +1 image.png','newmtl a\nmap_Kd -o -1 . .. image.png','newmtl a\nmap_Kd -s 1 -t 2 3 image.png',
 'newmtl a\nmap_Kd -clamp on','newmtl a\nmap_Kd -s','newmtl A\nmap_Kd one.png\nnewmtl a\nmap_Kd two.png',
 '\ufeffnewmtl 名\r\nmap_Kd\u00a0\"quoted name.png\"\r\n','newmtl a\nmap_Kd first.png\nmap_Kd -s',
 'newmtl a\nmap_Kd -o -. 1 2 image.png','newmtl a\nmap_Kd -MM 0 1 map with spaces.jpg']
for opt,takes in [('-blendu',1),('-blendv',1),('-cc',1),('-clamp',1),('-imfchan',1),('-texres',1),('-bm',1),('-boost',1),('-mm',2),('-o',3),('-s',3),('-t',3),('-type',1)]:mtls.append('newmtl skin\nmap_Kd '+opt+' '+' '.join(['1']*takes)+' texture.png')
for i,s in enumerate(mtls):add('mtl-'+str(i),2,string(s))
for i,s in enumerate(['','   ','a.mtl','a.mtl b.MTL','a.mtl b.png','my material.mtl','my material','"quoted name.mtl"','one.mtl\t two.mtl','\ufeffa.mtl\u00a0B.mtl\ufeff','a.mtl b.mtl ', 'a.MTL\nb.MtL']):add('libraries-'+str(i),3,string(s))
quad={'name':'tex.png','width':2,'height':2,'rgba':[255,0,0,0,0,255,0,255,0,0,255,127,255,255,255,64]}
samples=[(.25,.75),(.75,.75),(.25,.25),(.75,.25),(.5,.75),(1.25,.25),(-.75,-.25),(0,0),(1,1),(-0.,-0.),
 (.5,math.nextafter(.75,0)),(math.nextafter(.5,0),.75),(math.nextafter(.5,1),.75),
 (1e308,-1e308),(5e-324,-5e-324),(float('nan'),.5),(.5,float('nan')),(float('inf'),0),(0,float('-inf'))]
for i,(u,v) in enumerate(samples):add('sample-'+str(i),4,image(quad)+struct.pack('<dd',u,v)+word(1)+word(6)+bytes([11,22,33,44,55,66]))
for name,w,h,rgba in [('single',1,1,[7,19,251,0]),('wide',3,1,[5,12,34,255,10,99,0,0,255,8,9,255]),('tall',1,3,[1,2,3,255,7,8,9,0,250,251,252,255]),('short',2,2,[255,12]),('zero-width',0,2,[]),('zero-height',2,0,[])]:
 for j,(u,v) in enumerate([(0,0),(.5,.5),(1.25,-.25)]):
  img={'name':name,'width':w,'height':h,'rgba':rgba};add('sample-'+name+'-'+str(j),4,image(img)+struct.pack('<dd',u,v)+word(1)+word(6)+bytes([11,22,33,44,55,66]))
def bake(id,refs,given):
 b=floats(refs.get('uvs',[]))+ints(refs.get('vertexUv',[]))+ints(refs.get('vertexMaterial',[]))+strings(refs.get('materialNames',[]))+strings(refs.get('libraries',[]))+bytes([given is not None])
 if given is not None:
  mats=given.get('materials',[]);imgs=given.get('images',[]);b+=word(len(mats))+b''.join(string(m['name'])+string(m['text']) for m in mats)+word(len(imgs))+b''.join(image(i) for i in imgs)
 add('bake-'+id,5,b)
refs={'uvs':[.25,.25,.75,.25,.75,.75,.25,.75],'vertexUv':[0,1,2,3],'vertexMaterial':[0,0,0,0],'materialNames':['skin'],'libraries':['part.mtl']}
mtl={'name':'part.mtl','text':'newmtl skin\nmap_Kd C:\\scanner\\out\\TEX.png\n'}
bake('applied',refs,{'materials':[mtl],'images':[quad]})
bake('absent',refs,None);bake('empty-companions',refs,{});bake('image-missing',refs,{'materials':[mtl]});bake('only-image',refs,{'images':[dict(quad,name='anything.jpg')]})
for name,patch in [('no-uv',{'uvs':[]}),('no-library',{'libraries':[]}),('no-material',{'materialNames':[],'vertexMaterial':[-1]*4}),('unused',{'vertexUv':[-1]*4}),('short-material-array',{'vertexMaterial':[]}),('out-of-range-uv',{'vertexUv':[100,1,-1,0]}),('out-of-range-material',{'vertexMaterial':[100,0,-1,0]}),('nan-uv',{'uvs':[float('nan'),float('inf'),.5,.5]}),('empty-vertices',{'vertexUv':[]}),('duplicate-libraries',{'libraries':['part.mtl','part.mtl','other.mtl']}),('empty-library-name',{'libraries':['/']})]:
 r=copy.deepcopy(refs);r.update(patch);bake(name,r,{'materials':[mtl],'images':[quad]})
red={'name':'red.png','width':1,'height':1,'rgba':[255,0,0,255]};green={'name':'green.png','width':1,'height':1,'rgba':[0,255,0,255]}
r=copy.deepcopy(refs);r['materialNames']=['skin','other'];r['vertexMaterial']=[1,0,1,-1]
bake('two-materials',r,{'materials':[{'name':'part.mtl','text':'newmtl skin\nmap_Kd red.png\nnewmtl other\nmap_Kd green.png'}],'images':[red,green]})
bake('missing-in-order',r,{'materials':[{'name':'part.mtl','text':'newmtl skin\nmap_Kd absent1.png\nnewmtl other\nmap_Kd absent2.png'}],'images':[red,green]})
bake('unlisted-library',refs,{'materials':[{'name':'unlisted.mtl','text':mtl['text']}],'images':[quad,red]})
bake('first-material-definition',refs,{'materials':[{'name':'part.mtl','text':'newmtl skin\nmap_Kd red.png'},{'name':'extra.mtl','text':'newmtl skin\nmap_Kd green.png'}],'images':[red,green]})
bake('duplicate-library-last-text',refs,{'materials':[mtl,{'name':'PART.MTL','text':'newmtl skin\nmap_Kd red.png'}],'images':[quad,red]})
bake('duplicate-image-last-value',refs,{'materials':[mtl],'images':[quad,dict(red,name='TEX.PNG')]})
bake('single-map-loose',dict(refs,vertexMaterial=[-1]*4),{'materials':[mtl],'images':[quad,red]})
bake('no-map-no-note',dict(refs,libraries=[]),{'images':[quad,red]})
bake('multiple-missing-maps-no-material',dict(refs,materialNames=[],vertexMaterial=[-1]*4),{'materials':[{'name':'part.mtl','text':'newmtl a\nmap_Kd Foo.PNG\nnewmtl b\nmap_Kd Bar.PNG'}]})
bake('partial-missing-applied',dict(r,libraries=['missing.mtl','part.mtl']),{'materials':[{'name':'part.mtl','text':'newmtl skin\nmap_Kd red.png\nnewmtl other\nmap_Kd missing.png'}],'images':[red,green]})
for seed in [1,7,99,1039]:
 s=seed
 def rnd():
  global s;s=(1664525*s+1013904223)&0xffffffff;return s/4294967296
 rgba=[int(rnd()*256) for _ in range(4*3*2)];img={'name':'seed.png','width':3,'height':2,'rgba':rgba};uv=[rnd()*8-4 for _ in range(32)]
 bake('seed-'+str(seed),{'uvs':uv,'vertexUv':list(range(16)),'vertexMaterial':[-1]*16,'materialNames':[],'libraries':[]},{'images':[img]})
 for i in range(4):add('sample-seed-'+str(seed)+'-'+str(i),4,image(img)+struct.pack('<dd',uv[2*i],uv[2*i+1])+word(1)+word(6)+bytes([11,22,33,44,55,66]))
(OUT/'manifest.json').write_text(json.dumps({'schema':1,'reference_sha':'2cd89c6769a10c27ee7754b477ada521c9ef5cb7','license':'AGPL-3.0-only','seeds':[1,7,99,1039],'comparison':'exact OTX1 strings, ordered maps/notes, colors and complete sampler output state; no tolerance','cases':rows},indent=2)+'\n')
print(len(rows),'fixed OBJ texture/helper inputs')
