# SPDX-License-Identifier: AGPL-3.0-only
"""Independent operation inputs, never source/Rust answers."""
import gzip,hashlib,json,struct
from pathlib import Path
from generate_schema_cases import wire,Special,UNDEFINED
from schema_report import Reader
ROOT=Path(__file__).resolve().parents[3];OUT=ROOT/'docs/port-plan/fixtures/registry'
def untag(v):
 t,x=v
 if t==0:return UNDEFINED
 if t==1:return None
 if t in [2,3]:return t==3
 if t==4:return struct.unpack('<d',bytes.fromhex(x))[0]
 if t==5:return x
 if t==6:return list(map(untag,x))
 if t==7:return {k:untag(v)for k,v in x}
 return Special(8,bytes.fromhex(x))
def generate():
 OUT.mkdir(parents=True,exist_ok=True);rows=[]
 def add(id,defs,ops,**meta):
  b=wire(dict(defs=defs,ops=ops));(OUT/(id+'.bin')).write_bytes(b);rows.append(dict(id=id,bytes=len(b),sha256=hashlib.sha256(b).hexdigest(),**meta))
 def command(name='read',**kw):return dict(name=name,title='Probe 🦀',description='Authored callback echoes admitted arguments.',input=dict(type='object',properties={'flag':{'type':'boolean'}},additionalProperties=False),**kw)
 def reg(indices=[0],namespace='probe'):return dict(op='register',namespace=namespace,indices=indices)
 core=json.loads((ROOT/'mcp/src/commands.json').read_text())['commands'];names={c['name']for c in core};all_=json.loads(gzip.decompress((ROOT/'docs/port-plan/evidence/browser-2026-11-01/all.json.gz').read_bytes()));catalog=core+[c for c in all_['commands']if c['name']not in names];defs=[];groups={}
 for c in catalog:
  namespace,verb=c['name'].split('.',1);d=dict(c,name=verb,headless=c['name']in all_['headlessFlagNames']);i=len(defs);defs.append(d);groups.setdefault(namespace,[]).append(i);ops=[reg(namespace=namespace)]
  for kind in ['accepted','full','refused']:
   r=Reader((ROOT/('docs/port-plan/fixtures/schema/command-'+c['name']+'-'+kind+'.bin')).read_bytes());r.value();value=untag(r.value());r.text();r.end();ops.append(dict(op='admit',name=c['name'],input=value))
  ops+=[dict(op='headless',name=c['name']),dict(op='cleanup',handle=0),dict(op='headless',name=c['name'])]
  add('command-'+c['name'],[d],ops,command=c['name'])
 ops=[reg(indices,namespace)for namespace,indices in groups.items()]+[dict(op='list')]
 for i in range(len(groups)):ops.append(dict(op='cleanup',handle=i))
 add('all-real-catalog',defs,ops)
 for i,ns in enumerate(['','Upper','a..b','a-','a\n','a.9','a_','a0','a.b']):add(f'namespace-{i}',[command()],[reg(namespace=ns)])
 for i,name in enumerate(['','Upper','a..b','a-','a\n','a\r','a\u2028','a.9','a_','a0','a.b','🦀','\ud800']):add(f'name-{i}',[command(name)],[reg()])
 add('partial-duplicate',[command(),command('next')],[reg([0,1,0]),dict(op='list'),reg([1]),reg([0])])
 add('partial-invalid',[command(),command('Invalid')],[reg([0,1]),dict(op='list'),dict(op='admit',name='probe.read',input={'flag':'false'})])
 add('cleanup-old-handle',[command()],[reg(),dict(op='cleanup',handle=0),reg(),dict(op='cleanup',handle=0),dict(op='admit',name='probe.read',input={}),dict(op='cleanup',handle=1)])
 add('empty-registration',[],[reg([]),dict(op='cleanup',handle=0),dict(op='cleanup',handle=0)])
 add('unknown-input',[command()],[reg(),dict(op='admit',name='probe.missing',input={}),dict(op='admit',name='probe.read',input=None),dict(op='admit',name='probe.read',input=UNDEFINED),dict(op='admit',name='probe.read',input={'flag':'true'}),dict(op='admit',name='probe.read',input={'flag':'bad','extra':1})])
 add('metadata-presence',[command('first',exact=False,long=False,headless=False,askForLess=''),command('second',exact=True,long=True,headless=True,names={'space':'parts','at':['items[].name']},askForLess='one item')],[reg([0,1])])
 for i,input in enumerate([{'type':'number'},{},dict(type='object',oneOf=[])]):add(f'input-{i}',[dict(command(),input=input)],[reg(),dict(op='admit',name='probe.read',input={})])
 for budget in [8000,8001]:
  base={'type':'object','description':''};n=budget-len(json.dumps(base,separators=(',',':')));base['description']='x'*n;add('budget-'+str(budget),[dict(command(),input=base)],[reg()])
 for depth in [12,13]:
  v=0
  for _ in range(depth-1):v=[v]
  add('depth-'+str(depth),[dict(command(),input={'type':'object','default':v})],[reg()])
 alphabet=['a','a_a','aa','a0','a.a','a.a_a','a.aa','a10','a2','a_0','a_1','a.a0','a_','a.aaaa']
 add('locale-order',[command(s)for s in alphabet],[reg(list(range(len(alphabet))))])
 for seed in [1,7,99,1039]:
  state=seed
  for case in range(8):
   ds=[]
   for i in range(8):
    state=(1664525*state+1013904223)&0xffffffff;ds.append(command(alphabet[state%len(alphabet)],readOnly=bool(state&1),headless=bool(state&2)))
   ops=[reg([i])for i in range(len(ds))]+[dict(op='list')];add(f'seed-{seed}-{case}',ds,ops,seed=seed)
 add('refused-coercion',[command()],[reg(),dict(op='admit',name='probe.read',input={'flag':'true','extra':1}),dict(op='admit',name='probe.read',input={'flag':'false','extra':2})])
 manifest=dict(schema=1,license='AGPL-3.0-only',reference_sha='2cd89c6769a10c27ee7754b477ada521c9ef5cb7',seeds=[1,7,99,1039],commands=140,comparison='exact REG3 full result/error/afterInput mutation/body target/coerced args/catalog/headless state after every operation',cases=rows);(OUT/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
 table='// SPDX-License-Identifier: AGPL-3.0-only\nvec![\n'+''.join('    Case { id: "'+c['id']+'", input: include_bytes!("../../../../../docs/port-plan/fixtures/registry/'+c['id']+'.bin") },\n'for c in rows)+']\n';(ROOT/'rust/crates/port-engine/tests/support/registry_cases.rs').write_text(table);print(len(rows),'registry scenarios')
if __name__=='__main__':generate()
