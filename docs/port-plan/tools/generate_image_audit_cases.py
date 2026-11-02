#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
"""Independent authored image inputs; Pillow is reference fixture tooling only."""
import hashlib,io,json,random,struct,zlib
from pathlib import Path
from PIL import Image,features,__version__
ROOT=Path(__file__).resolve().parents[3];OUT=ROOT/'docs/port-plan/fixtures/image-audit';OUT.mkdir(exist_ok=True)
rows=[]
def add(identity,data,kind):
    (OUT/(identity+'.bin')).write_bytes(data)
    rows.append(dict(id=identity,format=kind,bytes=len(data),sha256=hashlib.sha256(data).hexdigest()))
def encoded(img,kind,**kwargs):
    out=io.BytesIO();img.save(out,format=kind,**kwargs);return out.getvalue()
def pixels(w,h,seed,alpha=True):
    rng=random.Random(seed);im=Image.new('RGBA',(w,h));im.putdata([tuple(rng.randrange(256) for _ in range(3))+(rng.randrange(256) if alpha else 255,) for _ in range(w*h)]);return im
atlas=Image.new('RGBA',(256,256));atlas.putdata([(v,(v*31)%256,(v*73)%256,a) for a in range(256) for v in range(256)])
add('png-alpha-atlas',encoded(atlas,'PNG'),'png')
base=pixels(17,13,1039);opaque=base.convert('RGB')
add('png-rgba',encoded(base,'PNG'),'png');add('png-rgb',encoded(opaque,'PNG'),'png')
add('png-gray',encoded(base.convert('L'),'PNG'),'png');add('png-gray-alpha',encoded(base.convert('LA'),'PNG'),'png')
gray16=Image.new('I;16',(17,13));gray16.putdata([i*293%65536 for i in range(221)]);add('png-gray16',encoded(gray16,'PNG'),'png')
palette=Image.new('P',(17,13));palette.putpalette([v for i in range(256) for v in (i,255-i,(i*7)%256)]);palette.putdata([i%16 for i in range(221)]);palette.info['transparency']=bytes(range(256));add('png-palette-alpha',encoded(palette,'PNG'),'png')
png=encoded(base,'PNG');chunk=b'gAMA'+struct.pack('>I',100000);gamma=struct.pack('>I',4)+chunk+struct.pack('>I',zlib.crc32(chunk));add('png-gamma',png[:33]+gamma+png[33:],'png')
for quality in [75,95]:
 for sub in [0,2]:add(f'jpeg-q{quality}-sub{sub}',encoded(opaque,'JPEG',quality=quality,subsampling=sub),'jpeg')
add('jpeg-progressive',encoded(opaque,'JPEG',quality=90,progressive=True),'jpeg')
add('jpeg-gray',encoded(base.convert('L'),'JPEG',quality=90),'jpeg')
add('jpeg-cmyk',encoded(opaque.convert('CMYK'),'JPEG',quality=90),'jpeg')
for lossless in [True,False]:add('webp-'+('lossless' if lossless else 'lossy'),encoded(base,'WEBP',lossless=lossless,quality=90,exact=True),'webp')
add('bmp-rgb',encoded(opaque,'BMP'),'bmp');add('bmp-rgba',encoded(base,'BMP'),'bmp')
for seed in [1,7,99,1039]:
 for kind in ['PNG','JPEG','BMP','WEBP']:
  im=pixels(7+seed%5,5+seed%7,seed)
  if kind in ['JPEG','BMP']:im=im.convert('RGB')
  add(f'seed-{seed}-{kind.lower()}',encoded(im,kind),kind.lower())
for kind in ['PNG','JPEG','BMP','WEBP']:
 im=base.convert('RGB') if kind in ['JPEG','BMP'] else base
 data=encoded(im,kind)
 for tag,n in [('empty',0),('signature',4),('half',len(data)//2),('last',len(data)-1)]:add(kind.lower()+'-truncated-'+tag,data[:n],kind.lower())
 bad=bytearray(data);bad[0]^=255;add(kind.lower()+'-bad-signature',bytes(bad),kind.lower())
add('junk',b'not an image\0\xff','unknown')
manifest=dict(contract='image.audit.input.v1',license='AGPL-3.0-only',reference_sha='2cd89c6769a10c27ee7754b477ada521c9ef5cb7',seeds=[1,7,99,1039],authoring_tool=dict(pillow=__version__,jpeg=features.version_codec('jpg'),webp=features.version_module('webp')),cases=rows)
(OUT/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n');print(len(rows),'independent encoded inputs')
