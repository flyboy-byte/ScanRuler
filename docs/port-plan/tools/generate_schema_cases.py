# SPDX-License-Identifier: AGPL-3.0-only
"""Authored/seeded inputs only; never reads original or Rust results."""
import gzip,hashlib,json,struct
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3]; OUT=ROOT/'docs/port-plan/fixtures/schema'
PIN='2cd89c6769a10c27ee7754b477ada521c9ef5cb7'
class Special:
 def __init__(self,tag,value=None):self.tag,self.value=tag,value
UNDEFINED=Special(0)
def text(s):
 b=s.encode('utf-16-le','surrogatepass');return struct.pack('<I',len(b)//2)+b
def wire(v):
 if isinstance(v,Special):return bytes([v.tag])+(struct.pack('<I',len(v.value))+v.value if v.tag==8 else b'')
 if v is None:return b'\1'
 if isinstance(v,bool):return bytes([3 if v else 2])
 if isinstance(v,(int,float)):return b'\4'+struct.pack('<d',v)
 if isinstance(v,str):return b'\5'+text(v)
 if isinstance(v,list):return b'\6'+struct.pack('<I',len(v))+b''.join(map(wire,v))
 return b'\7'+struct.pack('<I',len(v))+b''.join(text(k)+wire(x) for k,x in v.items())
def generate():
 OUT.mkdir(parents=True,exist_ok=True);rows=[]
 def add(id,s,v,path='input',**meta):
  b=wire(s)+wire(v)+text(path);(OUT/(id+'.bin')).write_bytes(b)
  rows.append(dict(id=id,bytes=len(b),sha256=hashlib.sha256(b).hexdigest(),**meta))
 def obj(p,required=[],**extra):return dict(type='object',properties=p,required=required,additionalProperties=False,**extra)
 for typ in ['null','boolean','number','integer','string','array','object']:
  for label,v in [('null',None),('undefined',UNDEFINED),('bool',True),('float',1.25),('text','true'),('list',[]),('object',{}),('binary',Special(8,b'abc'))]:add(f'type-{typ}-{label}',{'type':typ},v)
 for label,n in [('nan',float('nan')),('pos-inf',float('inf')),('neg-inf',-float('inf')),('negative-zero',-0.),('fraction',2.5)]:add('number-'+label,{'type':'integer','minimum':1,'exclusiveMinimum':2,'maximum':3},n)
 for i,(s,v) in enumerate([
  ({'type':'string','format':'binary'},Special(8,b'\0\xff')),
  ({'type':'string','minLength':2},'🦀'),({'type':'string','minLength':2},'\ud800'),
  ({'type':'string','minLength':1},''),({'type':'array','minItems':1},[]),({'type':'array','maxItems':1},[1,2]),
  ({'type':'array','items':{'type':'boolean'}},['true','false','True',False]),
  (obj({'flag':{'type':'boolean'}},['flag']),{'flag':'true'}),
  (obj({'flag':{'type':'boolean','enum':[False]}},['flag']),{'flag':'true'}),
  (obj({'flag':{'oneOf':[{'type':'string'},{'type':'boolean'}]}}),{'flag':'false'}),
  ({'oneOf':[{'title':'first','type':'boolean'},{'title':'second','type':'boolean'}]},True),
  ({'oneOf':[{'type':'integer'},{'type':'object','required':['x'],'properties':{'x':{'type':'number'}}}]},{}),
  ({'oneOf':[{'type':'number'}]},'bad'),({'oneOf':[]},0),
  ({'enum':[None,False,0,'x']},'no'),({'enum':[0]},-0.),
  (obj({'x':{'type':'number'}},['x']),{'x':UNDEFINED,'extra':UNDEFINED}),
  (obj({}),{'b':2,'10':1,'2':3,'a':4}),
  (obj({'id':{'type':'number'},'name':{'type':'string'}}),{'id':'Key top'}),
  (obj({'id':{'type':'number'},'name':{'type':'string'}}),{'id':' 0x10 '}),
  (obj({'id':{'type':'number'},'name':{'type':'string'}}),{'id':'1e309'}),
  (obj({'id':{'type':'number'},'name':{'type':'string'}}),{'id':'\ufeff  '}),
  (obj({'id':{'type':'number'},'name':{'type':'string'}}),{'id':'0b101'}),
  (obj({'id':{'type':'number'},'name':{'type':'string'}}),{'id':'\u0085'}),
  (obj({'radius':{'type':'number'}},['radius'],names={'field':'field','slots':['radius']}),{'radius':'r + 1'}),
  (obj({'extent':obj({'to':{'type':'number'}})},names={'field':'expr','slots':['extent.to']}),{'extent':{'to':'end'}}),
  ({'oneOf':[obj({'flag':{'type':'boolean'},'x':{'type':'number'}},['x']),obj({'flag':{'type':'string'}})]},{'flag':'true'}),
  (obj({'nested':{'oneOf':[obj({'b':{'type':'boolean'}},['b']),obj({'b':{'type':'boolean'}},['b'])]}}),{'nested':{'b':'false'}}),
  (obj({'z':{'type':'number'},'2':{},'1':{}}),{'extra':1}),
  ({'description':'🦀\ud800\n','default':UNDEFINED},{}),
  ({'default':{'list':[UNDEFINED,float('nan'),float('inf'),-0.],'bytes':Special(8,b'abc')}},0),
 ]):add(f'edge-{i:03d}',s,v,path='at.🦀' if i%5==0 else 'input')
 for n in [0.,1.,-1.,1e-7,1e-6,1e20,1e21,5e-324,1.7976931348623157e308,1000000000000000128.]:
  add('bound-'+str(len(rows)),{'type':'number','minimum':n},-float('inf') if n<0 else n-abs(n)-1)
 # Registration budget boundary lengths and depth, not just schemas used by commands.
 for n in [7979,7980,7981,7999,8000]:add(f'weight-{n}',{'description':'x'*n},None)
 for n in [10,11,12,13]:
  s={'type':'number'}
  for _ in range(n):s={'items':s,'type':'array'}
  add(f'depth-{n}',s,[])
 def sample(s,all_fields=False):
  if 'enum' in s:return s['enum'][0]
  if 'oneOf' in s:return sample(s['oneOf'][0],all_fields)
  t=s.get('type')
  if t=='object':return {k:sample(v,all_fields) for k,v in s.get('properties',{}).items() if all_fields or k in s.get('required',[])}
  if t=='array':return [sample(s.get('items',{}),all_fields) for _ in range(s.get('minItems',0))]
  if t=='string':return Special(8,b'abc') if s.get('format')=='binary' else 'x'*max(1,s.get('minLength',0))
  if t in ['number','integer']:return max(0,s.get('minimum',0),s.get('exclusiveMinimum',-1)+1)
  if t=='boolean':return True
  return None
 catalog=json.loads((ROOT/'mcp/src/commands.json').read_text())['commands']
 core_names={c['name'] for c in catalog}
 catalog += [c for c in json.loads(gzip.decompress((ROOT/'docs/port-plan/evidence/browser-2026-11-01/all.json.gz').read_bytes()))['commands'] if c['name'] not in core_names]
 for cmd in catalog:
  for kind,v in [('accepted',sample(cmd['input'])),('full',sample(cmd['input'],True)),('refused',None)]:add('command-'+cmd['name']+'-'+kind,cmd['input'],v,command=cmd['name'],intent=kind)
 for seed in [1,7,99,1039]:
  state=seed
  def rand(n):
   nonlocal state
   state=(1664525*state+1013904223)&0xffffffff;return state%n
  for i in range(32):
   p={};v={}
   for j in range(1+rand(6)):
    key=['flag','id','name','radius','2','10','🦀'][j];typ=['boolean','number','integer','string','null'][rand(5)]
    s={'type':typ}
    if rand(3)==0:s={'oneOf':[s,{'type':'boolean'}]}
    if rand(3)==0:s={'type':'array','items':s,'minItems':1,'maxItems':3}
    p[key]=s;value=[UNDEFINED,None,'true','false','r',-0.,float('nan'),rand(10),True,'\ud800'][rand(10)]
    v[key]=[value,'false'] if s.get('type')=='array' else value
   s=obj(p,list(p)[:rand(len(p)+1)],names={'field':'field','slots':['radius','id']})
   add(f'seed-{seed}-{i:02d}',s,v,seed=seed)
 for key in ['constructor','__proto__','toString','valueOf','hasOwnProperty','__defineGetter__','toLocaleString']:
  add('prototype-'+key.replace('_','z'),obj({},[key]),{key:'true'})
  add('inherited-'+key.replace('_','z'),obj({},[key]),{})
 for i,name in enumerate(['0o77','-0x10','+0b1','0b2','0x','Infinity','-Infinity','inf','NaN','1_000','1e-2','.5','+1','1.',' 1 \ufeff','\u2028','\u200b','1e309','\ud800','false']):
  add(f'id-number-{i:02d}',obj({'id':{'type':'number'},'name':{'type':'string'}}),{'id':name})
 for seed in [1,7,99,1039]:
  state=seed;props={};value={}
  for i in range(64):
   state=(6364136223846793005*state+1442695040888963407)&((1<<64)-1)
   x=struct.unpack('<d',struct.pack('<Q',state))[0]
   if not __import__('math').isfinite(x):x=1e-7
   props[str(i)]={'type':'number',('minimum' if x>0 else 'maximum'):x,'default':x};value[str(i)]=0
  add(f'number-seed-{seed}',obj(props),value,seed=seed)
 manifest=dict(schema=1,reference_sha=PIN,license='AGPL-3.0-only',seeds=[1,7,99,1039],commands=len(catalog),comparison='exact VLD1 errors UTF16, coerced typed value, schema UTF16 JSON length and depth, budget and original faults',cases=rows)
 (OUT/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
 table='// SPDX-License-Identifier: AGPL-3.0-only\nvec![\n'+''.join('    Case { id: "'+r['id']+'", input: include_bytes!("../../../../../docs/port-plan/fixtures/schema/'+r['id']+'.bin") },\n' for r in rows)+']\n'
 (ROOT/'rust/crates/port-engine/tests/support/schema_cases.rs').write_text(table)
 print(len(rows),'cases;',len(catalog),'command schemas')
if __name__=='__main__':generate()
