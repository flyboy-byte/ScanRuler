#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
"""Authoring tool for the PNG cases compared against Chrome (no codec, no Rust output).

  python3 generate.py named  OUT            hand-built probes (names in OUT.names.json)
  python3 generate.py sweep  OUT SEED N     every truncation plus N byte flips of six bases
  python3 generate.py struct OUT SEED N     N random images, each with five CRC-fixed mutations

OUT is the batch the oracle reads: u32 count, then u32 length + bytes per case. Python's own
zlib compresses image data, so regenerating with another zlib can change bytes; the committed
batches (hashes in manifest.json) are authoritative.
"""
import json, random, struct, sys, zlib

def chunk(t,d,crc=None):
  c=zlib.crc32(t+d)&0xffffffff if crc is None else crc
  return struct.pack('>I',len(d))+t+d+struct.pack('>I',c)
SIG=b'\x89PNG\r\n\x1a\n'
def ihdr(w,h,bd,ct,il=0,comp=0,filt=0): return struct.pack('>IIBBBBB',w,h,bd,ct,comp,filt,il)
def raw_rows(w,h,bpp_bits,ch,seed=1,filt=0):
  import random; r=random.Random(seed); stride=(w*ch*bpp_bits+7)//8
  return b''.join(bytes([filt])+bytes(r.randrange(256) for _ in range(stride)) for _ in range(h))
def png(w,h,bd,ct,data=None,il=0,extra_before=b'',extra_after=b'',idat=None,iend=True,seed=1,filt=0,ihdr_crc=None,plte=None,trns=None):
  ch={0:1,2:3,3:1,4:2,6:4}[ct]
  if data is None: data=raw_rows(w,h,bd,ch,seed,filt)
  z=zlib.compress(data) if idat is None else idat
  out=SIG+chunk(b'IHDR',ihdr(w,h,bd,ct,il),ihdr_crc)+extra_before
  if plte is not None: out+=chunk(b'PLTE',plte)
  if trns is not None: out+=chunk(b'tRNS',trns)
  out+=chunk(b'IDAT',z)+extra_after
  if iend: out+=chunk(b'IEND',b'')
  return out

def adam7_raw(w,h,ch,bd,pixel):  # pixel(x,y)->bytes per pixel (bd 8 or 16)
  passes=[(0,0,8,8),(4,0,8,8),(0,4,4,8),(2,0,4,4),(0,2,2,4),(1,0,2,2),(0,1,1,2)]
  out=b''
  for x0,y0,dx,dy in passes:
    xs=list(range(x0,w,dx)); ys=list(range(y0,h,dy))
    if not xs or not ys: continue
    for y in ys: out+=b'\0'+b''.join(pixel(x,y) for x in xs)
  return out

def actl(n,plays=0): return chunk(b'acTL',struct.pack('>II',n,plays))
def fctl(seq,w,h,x=0,y=0,dispose=0,blend=0,crc=None): return chunk(b'fcTL',struct.pack('>IIIIIHHBB',seq,w,h,x,y,1,10,dispose,blend),crc)
def rows(w,h,v,a=255): return b''.join(b'\0'+b''.join(bytes([(v+x*20)%256,(v//2+y*30)%256,(255-v)%256,a]) for x in range(w)) for y in range(h))
def fdat(seq,data,crc=None): return chunk(b'fdAT',struct.pack('>I',seq)+data,crc)
H=SIG+chunk(b'IHDR',ihdr(4,3,8,6)); E=chunk(b'IEND',b'')
idat=lambda v=40: chunk(b'IDAT',zlib.compress(rows(4,3,v)))

class W:
  def __init__(s): s.bits=[]
  def put(s,v,n):  # LSB-first value
    for i in range(n): s.bits.append((v>>i)&1)
  def code(s,c,n):  # Huffman code MSB-first
    for i in range(n-1,-1,-1): s.bits.append((c>>i)&1)
  def bytes(s):
    b=s.bits+[0]*((-len(s.bits))%8); return bytes(sum(b[i+j]<<j for j in range(8)) for i in range(0,len(b),8))
def lit(w,v):
  if v<144: w.code(0x30+v,8)
  elif v<256: w.code(0x190+v-144,9)
  elif v<280: w.code(v-256,7)
  else: w.code(0xc0+v-280,8)
def stream(rows,after,final=1):
  w=W(); w.put(final,1); w.put(1,2)
  for b in rows: lit(w,b)
  after(w); return b'\x78\x01'+w.bytes()

def set_basic():
  cases={}
  base=png(5,4,8,6)
  cases['base-rgba']=base
  cases['bad-ihdr-crc']=png(5,4,8,6,ihdr_crc=0)
  b=bytearray(base); i=base.index(b'IDAT'); b[i+4]^=0  # placeholder
  d=zlib.compress(raw_rows(5,4,8,4)); 
  cases['bad-idat-crc']=SIG+chunk(b'IHDR',ihdr(5,4,8,6))+chunk(b'IDAT',d,0)+chunk(b'IEND',b'')
  dd=bytearray(d); dd[-1]^=1
  cases['bad-adler']=png(5,4,8,6,idat=bytes(dd))
  cases['no-iend']=png(5,4,8,6,iend=False)
  cases['trail-after-iend']=base+b'garbage'
  cases['bad-ancillary-crc']=png(5,4,8,6,extra_before=chunk(b'tEXt',b'a\x00b',0))
  cases['unknown-critical']=png(5,4,8,6,extra_before=chunk(b'ABCD',b'xx'))
  cases['unknown-ancillary']=png(5,4,8,6,extra_before=chunk(b'abcd',b'xx'))
  cases['filter5']=png(5,4,8,6,filt=5)
  cases['short-data']=png(5,4,8,6,data=raw_rows(5,4,8,4)[:-7])
  cases['long-data']=png(5,4,8,6,data=raw_rows(5,4,8,4)+b'\0'*30)
  cases['zlib-truncated']=png(5,4,8,6,idat=d[:len(d)//2])
  cases['split-idat']=SIG+chunk(b'IHDR',ihdr(5,4,8,6))+chunk(b'IDAT',d[:5])+chunk(b'IDAT',d[5:])+chunk(b'IEND',b'')
  cases['empty-idat-between']=SIG+chunk(b'IHDR',ihdr(5,4,8,6))+chunk(b'IDAT',d[:5])+chunk(b'tEXt',b'a\0b')+chunk(b'IDAT',d[5:])+chunk(b'IEND',b'')
  cases['interlace']=png(9,7,8,6,il=1,data=None)  # data would be wrong layout; regenerate below
  for bd in (1,2,4,8,16):
    cases[f'gray{bd}']=png(7,3,bd,0,seed=bd)
  for bd in (8,16):
    cases[f'rgb{bd}']=png(7,3,bd,2,seed=bd); cases[f'rgba{bd}']=png(7,3,bd,6,seed=bd); cases[f'ga{bd}']=png(7,3,bd,4,seed=bd)
  for bd in (1,2,4,8):
    pal=bytes(range(3*(1<<bd))) if bd<8 else bytes((i*7)%256 for i in range(768))
    cases[f'pal{bd}']=png(7,3,bd,3,seed=bd,plte=pal[:3*(1<<bd)])
  cases['pal-short']=png(7,3,8,3,plte=bytes(30))
  cases['pal-trns']=png(7,3,8,3,plte=bytes((i*5)%256 for i in range(768)),trns=bytes(range(0,200,3)))
  cases['gray-trns']=png(7,3,8,0,trns=struct.pack('>H',37))
  cases['rgb-trns']=png(7,3,8,2,data=b''.join(b'\0'+bytes([10,20,30]*7) for _ in range(3)),trns=struct.pack('>HHH',10,20,30))
  cases['gray16-trns']=png(7,3,16,0,trns=struct.pack('>H',0x1234))
  cases['bad-bitdepth']=png(7,3,3,0)
  cases['zero-width']=png(0,3,8,0,data=b'\0'*3)
  cases['gama']=png(5,4,8,6,extra_before=chunk(b'gAMA',struct.pack('>I',100000)))
  cases['srgb']=png(5,4,8,6,extra_before=chunk(b'sRGB',b'\0'))
  cases['plte-for-rgb']=png(5,4,8,2,plte=bytes(range(30)))
  cases['no-idat']=SIG+chunk(b'IHDR',ihdr(5,4,8,6))+chunk(b'IEND',b'')
  return cases

def set_layout():
  cases={}
  pix=lambda x,y: bytes([(x*37+y*11)%256,(x*5+y*90)%256,(x*y*3)%256,(200+x+y)%256])
  cases['interlace-9x7']=png(9,7,8,6,il=1,data=adam7_raw(9,7,4,8,pix))
  cases['interlace-1x1']=png(1,1,8,6,il=1,data=adam7_raw(1,1,4,8,pix))
  cases['interlace-3x2']=png(3,2,8,6,il=1,data=adam7_raw(3,2,4,8,pix))
  cases['noninterlace-9x7']=png(9,7,8,6,data=b''.join(b'\0'+b''.join(pix(x,y) for x in range(9)) for y in range(7)))
  vals=[0x0000,0x00ff,0x0080,0x007f,0x0100,0x7f7f,0x7f80,0x7fff,0x8000,0x80ff,0xfe80,0xff00,0xff7f,0xff80,0xffff,0x0181]
  cases['gray16-vals']=png(16,1,16,0,data=b'\0'+b''.join(struct.pack('>H',v) for v in vals))
  # 16-bit RGBA with alpha values exercising premul
  cases['rgba16-vals']=png(16,1,16,6,data=b'\0'+b''.join(struct.pack('>HHHH',v,v,v,0x8080) for v in vals))
  cases['iend-bad-crc']=SIG+chunk(b'IHDR',ihdr(5,4,8,6))+chunk(b'IDAT',zlib.compress(b'\0'*21*4))+chunk(b'IEND',b'',0)
  cases['iend-with-data']=SIG+chunk(b'IHDR',ihdr(5,4,8,6))+chunk(b'IDAT',zlib.compress(b'\0'*21*4))+chunk(b'IEND',b'xx')
  z=zlib.compress(b'\0'*21*4)
  cases['two-ihdr']=SIG+chunk(b'IHDR',ihdr(5,4,8,6))+chunk(b'IHDR',ihdr(5,4,8,6))+chunk(b'IDAT',z)+chunk(b'IEND',b'')
  cases['text-first']=SIG+chunk(b'tEXt',b'a\0b')+chunk(b'IHDR',ihdr(5,4,8,6))+chunk(b'IDAT',z)+chunk(b'IEND',b'')
  cases['text-after-idat']=SIG+chunk(b'IHDR',ihdr(5,4,8,6))+chunk(b'IDAT',z)+chunk(b'tEXt',b'a\0b')+chunk(b'IEND',b'')
  cases['badcrc-anc-after-idat']=SIG+chunk(b'IHDR',ihdr(5,4,8,6))+chunk(b'IDAT',z)+chunk(b'tEXt',b'a\0b',0)+chunk(b'IEND',b'')
  cases['idat-after-iend-junk']=SIG+chunk(b'IHDR',ihdr(5,4,8,6))+chunk(b'IDAT',z)+chunk(b'IEND',b'')+chunk(b'IDAT',b'zz',0)
  cases['huge-chunk-len']=SIG+chunk(b'IHDR',ihdr(5,4,8,6))+struct.pack('>I',0x7fffffff)+b'IDAT'+z
  cases['chunklen-gt-2^31']=SIG+chunk(b'IHDR',ihdr(5,4,8,6))+struct.pack('>I',0x80000000)+b'tEXt'+b'ab'+chunk(b'IDAT',z)+chunk(b'IEND',b'')
  cases['zlib-bad-fcheck']=png(5,4,8,6,idat=bytes([0x78,0x00])+z[2:])
  cases['zlib-method-7']=png(5,4,8,6,idat=bytes([0x77,0x9c&0xff])+z[2:])
  cases['zlib-fdict']=png(5,4,8,6,idat=bytes([0x78,0xbb])+b'\0\0\0\1'+z[2:])
  cases['raw-deflate-no-header']=png(5,4,8,6,idat=z[2:])
  cases['zlib-trailing']=png(5,4,8,6,idat=z+b'junkjunk')
  cases['zlib-no-adler']=png(5,4,8,6,idat=z[:-4])
  cases['pal-no-plte']=png(7,3,8,3)
  cases['plte-after-idat']=SIG+chunk(b'IHDR',ihdr(7,3,8,3))+chunk(b'IDAT',zlib.compress(b'\0'*8*3))+chunk(b'PLTE',bytes(range(30)))+chunk(b'IEND',b'')
  cases['trns-before-plte']=SIG+chunk(b'IHDR',ihdr(7,3,8,3))+chunk(b'tRNS',b'\x10\x20')+chunk(b'PLTE',bytes(range(30)))+chunk(b'IDAT',zlib.compress(b'\0'+bytes(range(7))+(b'\0'+bytes(range(7)))*2))+chunk(b'IEND',b'')
  cases['trns-too-long']=png(7,3,8,3,plte=bytes(range(30)),trns=bytes(range(20)),data=(b'\0'+bytes(range(7)))*3)
  cases['plte-len-not-3']=png(7,3,8,3,plte=bytes(range(31)),data=(b'\0'+bytes(range(7)))*3)
  cases['plte-256plus']=png(7,3,8,3,plte=bytes(range(256))*3+b'\1\2\3',data=(b'\0'+bytes(range(7)))*3)
  cases['gray-trns-bad-len']=png(7,3,8,0,trns=b'\x00')
  cases['rgba-with-trns']=png(5,4,8,6,trns=b'\0\1\0\2\0\3')
  cases['ihdr-filter1']=SIG+chunk(b'IHDR',struct.pack('>IIBBBBB',5,4,8,6,0,1,0))+chunk(b'IDAT',z)+chunk(b'IEND',b'')
  cases['ihdr-comp1']=SIG+chunk(b'IHDR',struct.pack('>IIBBBBB',5,4,8,6,1,0,0))+chunk(b'IDAT',z)+chunk(b'IEND',b'')
  cases['ihdr-interlace2']=SIG+chunk(b'IHDR',struct.pack('>IIBBBBB',5,4,8,6,0,0,2))+chunk(b'IDAT',z)+chunk(b'IEND',b'')
  cases['ihdr-short']=SIG+chunk(b'IHDR',ihdr(5,4,8,6)[:12])+chunk(b'IDAT',z)+chunk(b'IEND',b'')
  cases['ct1']=SIG+chunk(b'IHDR',struct.pack('>IIBBBBB',5,4,8,1,0,0,0))+chunk(b'IDAT',z)+chunk(b'IEND',b'')
  cases['rgb4']=SIG+chunk(b'IHDR',struct.pack('>IIBBBBB',5,4,4,2,0,0,0))+chunk(b'IDAT',z)+chunk(b'IEND',b'')
  cases['w-2^31']=SIG+chunk(b'IHDR',ihdr(0x80000000,1,8,0))+chunk(b'IDAT',z)+chunk(b'IEND',b'')
  cases['big-dims-small-data']=SIG+chunk(b'IHDR',ihdr(30000,30000,8,0))+chunk(b'IDAT',z)+chunk(b'IEND',b'')
  cases['sig-only']=SIG
  cases['bad-sig-byte']=b'\x89PNG\r\n\x1a\x0b'+png(5,4,8,6)[8:]
  cases['filters-all']=png(6,5,8,2,data=b''.join(bytes([f])+bytes(((x*31+f*7+y)%256) for x in range(18)) for y,f in enumerate([0,1,2,3,4])))
  cases['filters-sub-first-row-paeth']=png(6,2,8,0,data=bytes([4])+bytes(range(10,16))+bytes([3])+bytes(range(20,26)))
  # APNG: acTL + fcTL before IDAT (IDAT is frame 0), second frame fdAT
  def actl(n,plays=0): return chunk(b'acTL',struct.pack('>II',n,plays))
  def fctl(seq,w,h,x=0,y=0): return chunk(b'fcTL',struct.pack('>IIIIIHHBB',seq,w,h,x,y,1,10,0,0))
  rows=lambda w,h,v: b''.join(b'\0'+bytes([v,v//2,255-v,255])*w for _ in range(h))
  cases['apng-default-frame']=SIG+chunk(b'IHDR',ihdr(4,3,8,6))+actl(2)+fctl(0,4,3)+chunk(b'IDAT',zlib.compress(rows(4,3,40)))+fctl(1,4,3)+chunk(b'fdAT',struct.pack('>I',2)+zlib.compress(rows(4,3,200)))+chunk(b'IEND',b'')
  cases['apng-hidden-default']=SIG+chunk(b'IHDR',ihdr(4,3,8,6))+actl(1)+chunk(b'IDAT',zlib.compress(rows(4,3,40)))+fctl(0,4,3)+chunk(b'fdAT',struct.pack('>I',1)+zlib.compress(rows(4,3,200)))+chunk(b'IEND',b'')
  cases['apng-bad-fdat']=SIG+chunk(b'IHDR',ihdr(4,3,8,6))+actl(2)+fctl(0,4,3)+chunk(b'IDAT',zlib.compress(rows(4,3,40)))+fctl(1,4,3)+chunk(b'fdAT',struct.pack('>I',2)+b'garbage')+chunk(b'IEND',b'')
  return cases

def set_apng():
  C={}
  C['A-default']=H+actl(2)+fctl(0,4,3)+idat()+fctl(1,4,3)+fdat(2,zlib.compress(rows(4,3,200)))+E
  C['B-hidden']=H+actl(2)+idat()+fctl(0,4,3)+fdat(1,zlib.compress(rows(4,3,200)))+E
  C['B-hidden-offset']=H+actl(2)+idat()+fctl(0,2,1,1,1)+fdat(1,zlib.compress(rows(2,1,200)))+E
  C['B-hidden-offset-blend1']=H+actl(2)+idat()+fctl(0,2,1,1,1,blend=1)+fdat(1,zlib.compress(rows(2,1,200,128)))+E
  C['B-hidden-no-frame']=H+actl(1)+idat()+E
  C['B-hidden-fdat-bad']=H+actl(2)+idat()+fctl(0,4,3)+fdat(1,b'garbage')+E
  C['B-hidden-fdat-badcrc']=H+actl(2)+idat()+fctl(0,4,3)+fdat(1,zlib.compress(rows(4,3,200)),0)+E
  C['B-hidden-seq-wrong']=H+actl(2)+idat()+fctl(0,4,3)+fdat(5,zlib.compress(rows(4,3,200)))+E
  C['B-hidden-fctl-seq-wrong']=H+actl(2)+idat()+fctl(3,4,3)+fdat(4,zlib.compress(rows(4,3,200)))+E
  C['B-hidden-region-out']=H+actl(2)+idat()+fctl(0,4,3,1,0)+fdat(1,zlib.compress(rows(4,3,200)))+E
  C['B-hidden-fctl-zero']=H+actl(2)+idat()+fctl(0,0,3)+fdat(1,zlib.compress(rows(4,3,200)))+E
  C['B-hidden-no-iend']=H+actl(2)+idat()+fctl(0,4,3)+fdat(1,zlib.compress(rows(4,3,200)))
  C['B-hidden-trunc-after-fdat-hdr']=(H+actl(2)+idat()+fctl(0,4,3)+fdat(1,zlib.compress(rows(4,3,200)))+E)[:-4]
  C['B-hidden-2fdat']=H+actl(2)+idat()+fctl(0,4,3)+fdat(1,zlib.compress(rows(4,3,200))[:6])+fdat(2,zlib.compress(rows(4,3,200))[6:])+E
  C['C-actl0']=H+actl(0)+idat()+E
  C['C-actl0-hidden']=H+actl(0)+idat()+fctl(0,4,3)+fdat(1,zlib.compress(rows(4,3,200)))+E
  C['D-actl-after-idat']=H+idat()+actl(2)+fctl(0,4,3)+fdat(1,zlib.compress(rows(4,3,200)))+E
  C['G-fctl-size-mismatch']=H+actl(2)+fctl(0,2,2)+idat()+E
  C['G-fctl-offset-default']=H+actl(2)+fctl(0,4,3,1,0)+idat()+E
  C['J-fctl-badcrc-before-idat']=H+actl(2)+fctl(0,4,3,crc=0)+idat()+fctl(1,4,3)+fdat(2,zlib.compress(rows(4,3,200)))+E
  C['M-two-actl']=H+actl(2)+actl(3)+idat()+fctl(0,4,3)+fdat(1,zlib.compress(rows(4,3,200)))+E
  C['N-actl-badcrc']=H+chunk(b'acTL',struct.pack('>II',2,0),0)+idat()+fctl(0,4,3)+fdat(1,zlib.compress(rows(4,3,200)))+E
  C['O-actl-len5']=H+chunk(b'acTL',b'\0\0\0\2\0')+idat()+fctl(0,4,3)+fdat(1,zlib.compress(rows(4,3,200)))+E
  C['P-hidden-chunk-between']=H+actl(2)+idat()+chunk(b'tEXt',b'a\0b')+fctl(0,4,3)+fdat(1,zlib.compress(rows(4,3,200)))+E
  C['Q-hidden-idat-broken']=H+actl(2)+chunk(b'IDAT',b'xx')+fctl(0,4,3)+fdat(1,zlib.compress(rows(4,3,200)))+E
  C['R-default-dispose2']=H+actl(2)+fctl(0,4,3,dispose=2)+idat()+E
  C['S-hidden-dispose2']=H+actl(2)+idat()+fctl(0,4,3,dispose=2)+fdat(1,zlib.compress(rows(4,3,200)))+E
  C['T-fctl-len-bad']=H+actl(2)+idat()+chunk(b'fcTL',struct.pack('>IIIIIHHBB',0,4,3,0,0,1,10,0,0)[:20])+fdat(1,zlib.compress(rows(4,3,200)))+E
  C['U-actl-after-fctl']=H+fctl(0,4,3)+actl(2)+idat()+E
  C['V-pal-hidden']=SIG+chunk(b'IHDR',ihdr(4,3,8,3))+chunk(b'PLTE',bytes(range(30)))+actl(2)+chunk(b'IDAT',zlib.compress((b'\0'+bytes([0,1,2,3]))*3))+fctl(0,4,3)+fdat(1,zlib.compress((b'\0'+bytes([4,5,6,7]))*3))+E
  C['W-hidden-interlaced']=SIG+chunk(b'IHDR',ihdr(4,3,8,6,1))+actl(2)+chunk(b'IDAT',b'zz')+fctl(0,4,3)+fdat(1,zlib.compress(rows(4,3,200)))+E
  return C

def set_apng_edges():
  z=lambda w,h,v=200: zlib.compress(rows(w,h,v))
  C={}
  C['default-seq1']=H+actl(2)+fctl(1,4,3)+idat()+E
  C['default-seq5-then-fdat']=H+actl(2)+fctl(5,4,3)+idat()+fctl(6,4,3)+fdat(7,z(4,3))+E
  C['default-dispose3']=H+actl(2)+fctl(0,4,3,dispose=3)+idat()+E
  C['default-blend2']=H+actl(2)+fctl(0,4,3,blend=2)+idat()+E
  C['hidden-dispose3']=H+actl(2)+idat()+fctl(0,4,3,dispose=3)+fdat(1,z(4,3))+E
  C['hidden-blend2']=H+actl(2)+idat()+fctl(0,4,3,blend=2)+fdat(1,z(4,3))+E
  C['fctl-no-actl']=H+fctl(0,4,3)+idat()+E
  C['fctl-no-actl-mismatch']=H+fctl(0,2,2)+idat()+E
  C['hidden-no-actl']=H+idat()+fctl(0,4,3)+fdat(1,z(4,3))+E
  C['actl-huge-frames']=H+actl(0x80000000)+idat()+fctl(0,4,3)+fdat(1,z(4,3))+E
  C['actl-huge-plays']=H+actl(2,0xffffffff)+idat()+fctl(0,4,3)+fdat(1,z(4,3))+E
  C['hidden-fctl-then-iend']=H+actl(2)+idat()+fctl(0,4,3)+E
  C['hidden-fdat-first']=H+actl(2)+idat()+fdat(0,z(4,3))+fctl(1,4,3)+fdat(2,z(4,3))+E
  C['hidden-two-fctl']=H+actl(2)+idat()+fctl(0,4,3)+fctl(1,4,3)+fdat(2,z(4,3))+E
  C['hidden-fdat-short']=H+actl(2)+idat()+fctl(0,4,3)+chunk(b'fdAT',b'\0\0')+E
  C['hidden-fdat-seq-only']=H+actl(2)+idat()+fctl(0,4,3)+fdat(1,b'')+fdat(2,z(4,3))+E
  C['hidden-fctl-badcrc']=H+actl(2)+idat()+fctl(0,4,3,crc=0)+fdat(1,z(4,3))+E
  C['hidden-fdat-then-idat']=H+actl(2)+idat()+fctl(0,4,3)+fdat(1,z(4,3))+idat()+E
  C['hidden-idat-fdat-interleave-text']=H+actl(2)+idat()+fctl(0,4,3)+chunk(b'tEXt',b'a\0b')+fdat(1,z(4,3))+E
  C['hidden-big-delay0']=H+actl(2)+idat()+chunk(b'fcTL',struct.pack('>IIIIIHHBB',0,4,3,0,0,0,0,0,0))+fdat(1,z(4,3))+E
  C['hidden-frame-1x1-corner']=H+actl(2)+idat()+fctl(0,1,1,3,2)+fdat(1,z(1,1))+E
  C['hidden-x-overflow']=H+actl(2)+idat()+fctl(0,4,3,0xffffffff,0)+fdat(1,z(4,3))+E
  C['actl-between-idat']=H+chunk(b'IDAT',zlib.compress(rows(4,3,40))[:5])+actl(2)+chunk(b'IDAT',zlib.compress(rows(4,3,40))[5:])+E
  C['default-fctl-after-actl-badcrc']=H+chunk(b'acTL',struct.pack('>II',2,0),0)+fctl(0,2,2)+idat()+E
  C['hidden-critical-unknown-between']=H+actl(2)+idat()+chunk(b'ABCD',b'x')+fctl(0,4,3)+fdat(1,z(4,3))+E
  C['hidden-plte-between']=H+actl(2)+idat()+chunk(b'PLTE',bytes(9))+fctl(0,4,3)+fdat(1,z(4,3))+E
  C['hidden-text-badcrc-between']=H+actl(2)+idat()+chunk(b'tEXt',b'a\0b',0)+fctl(0,4,3)+fdat(1,z(4,3))+E
  C['hidden-trns-between']=SIG+chunk(b'IHDR',ihdr(4,3,8,2))+actl(2)+chunk(b'IDAT',zlib.compress(b'\0'+bytes(12)*1+b'\0'+bytes(12)+b'\0'+bytes(12)))+chunk(b'tRNS',struct.pack('>HHH',0,0,0))+fctl(0,4,3)+fdat(1,zlib.compress((b'\0'+bytes(12))*3))+E
  return C

def set_trns():
  C={}
  g8=b''.join(b'\0'+bytes([5,6,7,8]) for _ in range(1))  # 4x1 gray8 values 5..8
  for n,t in [('g8-len1',b'\x00'),('g8-len2',b'\x00\x06'),('g8-len3',b'\x00\x06\x07'),('g8-len4',b'\x00\x06\x00\x07'),('g8-len5',b'\x00\x06\x00\x07\x08'),('g8-hi',b'\x01\x06'),('g8-len0',b'')]:
    C[n]=png(4,1,8,0,data=g8,trns=t)
  g4=b'\0'+bytes([0x56,0x78])  # 4 px values 5,6,7,8
  for n,t in [('g4-5',b'\x00\x05'),('g4-0105',b'\x01\x05'),('g4-0015',b'\x00\x15'),('g4-len3',b'\x00\x05\x00')]:
    C[n]=png(4,1,4,0,data=g4,trns=t)
  g1=b'\0'+bytes([0b10100000])
  for n,t in [('g1-1',b'\x00\x01'),('g1-0101',b'\x01\x01'),('g1-0003',b'\x00\x03')]:
    C[n]=png(4,1,1,0,data=g1,trns=t)
  rgb=b'\0'+bytes([10,20,30, 11,21,31, 10,20,30, 1,2,3])
  for n,t in [('rgb8-ok',struct.pack('>HHH',10,20,30)),('rgb8-hi',struct.pack('>HHH',0x10a,20,30)),('rgb8-len5',struct.pack('>HHH',10,20,30)[:5]),('rgb8-len7',struct.pack('>HHH',10,20,30)+b'\0'),('rgb8-len8',struct.pack('>HHHH',10,20,30,0)),('rgb8-len2',b'\x00\x0a')]:
    C[n]=png(4,1,8,2,data=rgb,trns=t)
  rgb16=b'\0'+b''.join(struct.pack('>HHH',*v) for v in [(0x0a0b,1,2),(0x0a00,1,2),(0x0b0b,1,2),(5,5,5)])
  for n,t in [('rgb16-full',struct.pack('>HHH',0x0a0b,1,2)),('rgb16-low',struct.pack('>HHH',0x0b0b,1,2))]:
    C[n]=png(4,1,16,2,data=rgb16,trns=t)
  g16=b'\0'+b''.join(struct.pack('>H',v) for v in [0x1234,0x0034,0x1200,0x3434])
  C['g16-1234']=png(4,1,16,0,data=g16,trns=b'\x12\x34'); C['g16-len3']=png(4,1,16,0,data=g16,trns=b'\x12\x34\x00')
  pal=bytes(range(12))
  for n,t in [('pal-trns0',b''),('pal-trns-len5',b'\x10\x20\x30\x40\x50')]:
    C[n]=png(4,1,8,3,plte=pal,data=b'\0'+bytes([0,1,2,3]),trns=t)
  C['ga8-trns']=png(2,1,8,4,data=b'\0'+bytes([5,200,6,201]),trns=b'\x00\x05')
  return C

def set_deflate_tail():
  rows=b'\0\x10\x20\x30'*2  # 3x2 gray8: filter0 + 3 bytes, two rows
  mk=lambda z: SIG+chunk(b'IHDR',ihdr(3,2,8,0))+chunk(b'IDAT',z)+chunk(b'IEND',b'')
  C={}
  def eob(w): lit(w,256)
  C['ok-eob']=mk(stream(rows,eob))
  def bad286(w): lit(w,286)
  def junk(n): return lambda w: [lit(w,65) for _ in range(n)]
  for n in [0,1,2,4,8,16,64]:
    C[f'lit286-then-{n}junk']=mk(stream(rows,lambda w,n=n:(lit(w,286),[lit(w,65) for _ in range(n)],lit(w,256))))
    C[f'{n}lits-then-lit286']=mk(stream(rows,lambda w,n=n:([lit(w,65) for _ in range(n)],lit(w,286))))
  def far(w): lit(w,257); w.code(29,5); w.put(0,13)  # len 3, dist code 29 base 24577
  for n in [0,8,64]:
    C[f'far-then-{n}']=mk(stream(rows,lambda w,n=n:(far(w),[lit(w,65) for _ in range(n)],lit(w,256))))
  def dist30(w): lit(w,257); w.code(30,5)
  for n in [0,8,64]:
    C[f'dist30-then-{n}']=mk(stream(rows,lambda w,n=n:(dist30(w),[lit(w,65) for _ in range(n)],lit(w,256))))
  # second block with bad type 3 after eob of non-final block
  def blocktype3(n): return lambda w:(lit(w,256),w.put(1,1),w.put(3,2),[w.put(0x55,8) for _ in range(n)])
  for n in [0,8,64]:
    C[f'blocktype3-then-{n}']=mk(stream(rows,blocktype3(n),final=0))
  return C

def set_deflate_mid():
  def st(body):
    w=W(); w.put(1,1); w.put(1,2); body(w); return b'\x78\x01'+w.bytes()
  mk=lambda z,wd=3,h=2: SIG+chunk(b'IHDR',ihdr(wd,h,8,0))+chunk(b'IDAT',z)+chunk(b'IEND',b'')
  C={}
  lits=lambda w,bs:[lit(w,b) for b in bs]
  C['ref']=mk(st(lambda w:(lits(w,b'\0\x10\x20\x30\0\x11\x21\x31'),lit(w,256))))
  C['far-first']=mk(st(lambda w:(lit(w,257),w.code(29,5),w.put(0,13),lits(w,b'\x10\x20\x30\0\x11'),lit(w,256))))  # 3 bytes from far + 5 lits = 8
  C['far-mid']=mk(st(lambda w:(lits(w,b'\0\x10\x20\x30\0'),lit(w,257),w.code(4,5),w.put(0,1),lit(w,256))))  # dist 5..6 within 5 bytes? dist code4 base5 extra1 -> 5
  C['far-mid-6']=mk(st(lambda w:(lits(w,b'\0\x10\x20\x30\0'),lit(w,257),w.code(4,5),w.put(1,1),lit(w,256))))  # dist 6 > 5 produced
  C['lit286-mid']=mk(st(lambda w:(lits(w,b'\0\x10\x20'),lit(w,286),lits(w,b'\x30\0\x11\x21\x31'),lit(w,256))))
  C['lit287-mid']=mk(st(lambda w:(lits(w,b'\0\x10\x20'),lit(w,287),lits(w,b'\x30\0\x11\x21\x31'),lit(w,256))))
  C['dist30-mid']=mk(st(lambda w:(lits(w,b'\0\x10\x20\x30\0'),lit(w,257),w.code(30,5),lit(w,256))))
  C['dist31-mid']=mk(st(lambda w:(lits(w,b'\0\x10\x20\x30\0'),lit(w,257),w.code(31,5),lit(w,256))))
  C['eob-early']=mk(st(lambda w:(lits(w,b'\0\x10\x20\x30\0'),lit(w,256))))
  # exact: rows complete then EOB missing, stream ends (no adler)
  C['no-eob-end']=mk(st(lambda w:lits(w,b'\0\x10\x20\x30\0\x11\x21\x31')))
  # rows complete exactly at end of block 1 (nonfinal) and then type3 header in next byte(s)
  def nf(body):
    w=W(); w.put(0,1); w.put(1,2); body(w); return w
  for n in [0,1,2,3,4,6,8]:
    def body(w,n=n):
      lits(w,b'\0\x10\x20\x30\0\x11\x21\x31'); lit(w,256); w.put(1,1); w.put(3,2)
      for _ in range(n): w.put(0xaa,8)
    w=nf(body); C[f'type3-after-{n}']=mk(b'\x78\x01'+w.bytes())
  for n in [0,2,4,8]:
    def body(w,n=n):
      lits(w,b'\0\x10\x20\x30\0\x11\x21\x31'); lit(w,256); w.put(0,1); w.put(0,2)
    w=nf(body); z=b'\x78\x01'+w.bytes()+b'\x05\x00\x00\x00'+bytes(n)  # stored with bad nlen
    C[f'storedbad-after-{n}']=mk(z)
  return C

def set_deflate_chunks():
  hdr=lambda: SIG+chunk(b'IHDR',ihdr(3,2,8,0))
  E=chunk(b'IEND',b'')
  C={}
  rows=b'\0\x10\x20\x30\0\x11\x21\x31'
  # A: rows in chunk1 (byte aligned end?), then chunk2 with EOB + type3 + junk
  w=W(); w.put(0,1); w.put(1,2); [lit(w,b) for b in rows]
  b1=w.bytes(); nb=len(w.bits)
  # build full stream then split at byte boundary after rows
  w2=W(); w2.put(0,1); w2.put(1,2); [lit(w2,b) for b in rows]; lit(w2,256); w2.put(1,1); w2.put(3,2); [w2.put(0x55,8) for _ in range(8)]
  z=b'\x78\x01'+w2.bytes()
  cut=2+(nb+7)//8
  C['split-after-rows']=hdr()+chunk(b'IDAT',z[:cut])+chunk(b'IDAT',z[cut:])+E
  C['same-chunk']=hdr()+chunk(b'IDAT',z)+E
  C['split-late']=hdr()+chunk(b'IDAT',z[:cut+2])+chunk(b'IDAT',z[cut+2:])+E
  # threshold: vary trailing padding so bits available after rows = k
  for pre in range(0,8):
    for extra in [0,1]:
      w=W(); w.put(0,1); w.put(1,2)
      # shift bit position with 'pre' leading bits by using... use different first-literal codes: rows fixed; add extra literal before? keep rows; instead pad via image width
      [lit(w,b) for b in rows]
      pos=len(w.bits)
      lit(w,256); w.put(1,1); w.put(3,2)
      for _ in range(pre): w.bits.append(1)
      zz=b'\x78\x01'+w.bytes()+bytes(extra)
      avail=len(zz)*8-16-pos
      C[f'thr-avail{avail}']=hdr()+chunk(b'IDAT',zz)+E
  return C


def named():
  out = {}
  for prefix, f in [('basic', set_basic), ('layout', set_layout), ('apng', set_apng), ('apng-edge', set_apng_edges),
                    ('trns', set_trns), ('deflate-tail', set_deflate_tail), ('deflate-mid', set_deflate_mid),
                    ('deflate-chunks', set_deflate_chunks)]:
    for k, v in f().items():
      out[f'{prefix}/{k}'] = v
  return out

def sweep(seed, n):
  r=random.Random(seed)
  bases=[]
  pix=lambda x,y: bytes([(x*37+y*11)%256,(x*5+y*90)%256,(x*y*3)%256,(200+x*7+y)%256])
  rows=b''.join(bytes([y%5])+b''.join(pix(x,y) for x in range(6)) for y in range(5))
  z=zlib.compress(rows,9)
  bases.append(SIG+chunk(b'IHDR',ihdr(6,5,8,6))+chunk(b'tEXt',b'k\0v')+chunk(b'IDAT',z[:10])+chunk(b'IDAT',z[10:])+chunk(b'tEXt',b'q\0w')+chunk(b'IEND',b''))
  bases.append(png(9,7,8,6,il=1,data=adam7_raw(9,7,4,8,pix)))
  pal=bytes(r.randrange(256) for _ in range(48))
  bases.append(png(7,4,4,3,plte=pal,trns=bytes([0,128,255,7]),seed=3))
  bases.append(png(5,3,16,2,trns=struct.pack('>HHH',1,2,3),seed=4))
  bases.append(png(8,3,2,0,seed=5))
  bases.append(png(6,4,8,4,idat=zlib.compress(b''.join(bytes([1])+bytes(r.randrange(256) for _ in range(12)) for _ in range(4)),0)))
  cases=[]
  for b in bases:
    for cut in range(len(b)+1): cases.append(b[:cut])
    for _ in range(n):
      m=bytearray(b); k=r.randrange(1,4)
      for _ in range(k):
        i=r.randrange(8,len(m)); m[i]=r.randrange(256) if r.random()<0.5 else m[i]^(1<<r.randrange(8))
      cases.append(bytes(m))
  return cases

def structural(seed, n):
  r=random.Random(seed); N=n
  def comp(data):
    lvl=r.choice([0,1,6,9]); st=r.choice([zlib.Z_DEFAULT_STRATEGY,zlib.Z_FILTERED,zlib.Z_HUFFMAN_ONLY,zlib.Z_RLE,zlib.Z_FIXED])
    c=zlib.compressobj(lvl,zlib.DEFLATED,r.choice([9,12,15]),9,st); return c.compress(data)+c.flush()
  def img():
    ct=r.choice([0,2,3,4,6]); bd=r.choice({0:[1,2,4,8,16],2:[8,16],3:[1,2,4,8],4:[8,16],6:[8,16]}[ct])
    w,h=r.randrange(1,24),r.randrange(1,12); ch={0:1,2:3,3:1,4:2,6:4}[ct]; stride=(w*ch*bd+7)//8
    rows=b''
    for y in range(h):
      if r.random()<0.5: line=bytes(r.randrange(256) for _ in range(stride))
      else: v=r.randrange(256); line=bytes((v+i*r.randrange(3))%256 for i in range(stride))
      rows+=bytes([r.randrange(5)])+line
    out=SIG+chunk(b'IHDR',ihdr(w,h,bd,ct))
    if ct==3: out+=chunk(b'PLTE',bytes(r.randrange(256) for _ in range(3*r.randrange(1,1<<bd + 0 if bd<8 else 256))))
    if r.random()<0.3 and ct in (0,2,3): out+=chunk(b'tRNS',bytes(r.randrange(256) for _ in range({0:2,2:6,3:r.randrange(1,20)}[ct])))
    z=comp(rows); k=r.randrange(1,4); cuts=sorted(r.randrange(len(z)+1) for _ in range(k-1)); parts=[z[a:b] for a,b in zip([0]+cuts,cuts+[len(z)])]
    for p in parts: out+=chunk(b'IDAT',p)
    return out+chunk(b'IEND',b'')
  def chunks(b):
    at=8; out=[]
    while at+8<=len(b):
      L,t=struct.unpack_from('>I4s',b,at); out.append([t,b[at+8:at+8+L]]); at+=12+L
    return out
  def build(cs): return SIG+b''.join(chunk(t,d) for t,d in cs)
  cases=[]
  for _ in range(N):
    b=img(); cases.append(b)
    cs=chunks(b)
    # mutate IDAT payload with CRC fixed
    m=[list(x) for x in cs]; idx=[i for i,(t,_) in enumerate(m) if t==b'IDAT']
    i=r.choice(idx); d=bytearray(m[i][1])
    if d:
      for _ in range(r.randrange(1,3)):
        j=r.randrange(len(d)); d[j]=r.randrange(256) if r.random()<0.3 else d[j]^(1<<r.randrange(8))
    m[i][1]=bytes(d); cases.append(build(m))
    # truncate zlib stream with valid CRC
    m=[list(x) for x in cs]; i=idx[-1]; m[i][1]=m[i][1][:r.randrange(len(m[i][1])+1)]; cases.append(build(m))
    # header field mutation with CRC fixed
    m=[list(x) for x in cs]; d=bytearray(m[0][1]); j=r.randrange(13); d[j]=r.randrange(256) if r.random()<0.5 else d[j]^(1<<r.randrange(8)); m[0][1]=bytes(d); cases.append(build(m))
    # palette / trns length mutations
    m=[list(x) for x in cs]
    for x in m:
      if x[0] in (b'PLTE',b'tRNS') and r.random()<0.7: x[1]=x[1][:r.randrange(len(x[1])+1)] if r.random()<0.5 else x[1]+bytes(r.randrange(256) for _ in range(r.randrange(1,6)))
    cases.append(build(m))
    # chunk drop/dup/swap
    m=[list(x) for x in cs]; op=r.randrange(3); i=r.randrange(1,len(m))
    if op==0: del m[i]
    elif op==1: m.insert(i,list(m[i]))
    else:
      j=r.randrange(1,len(m)); m[i],m[j]=m[j],m[i]
    cases.append(build(m))
  return cases


def write(path, cases):
  out = struct.pack('<I', len(cases))
  for c in cases:
    out += struct.pack('<I', len(c)) + c
  open(path, 'wb').write(out)

if __name__ == '__main__':
  kind, path = sys.argv[1], sys.argv[2]
  if kind == 'named':
    cases = named()
    json.dump(list(cases), open(path + '.names.json', 'w'), indent=0)
    write(path, list(cases.values()))
  elif kind == 'sweep':
    write(path, sweep(int(sys.argv[3]), int(sys.argv[4])))
  else:
    write(path, structural(int(sys.argv[3]), int(sys.argv[4])))
