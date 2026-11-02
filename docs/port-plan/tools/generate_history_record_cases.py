# SPDX-License-Identifier: AGPL-3.0-only
"""Authored immutable graph/operation inputs; no source or Rust answers."""
from pathlib import Path
import json,hashlib
from generate_schema_cases import wire,UNDEFINED
ROOT=Path(__file__).resolve().parents[3];OUT=ROOT/'docs/port-plan/fixtures/history-record'
def ref(n):return {'ref':n}
def obj(fields):return {'kind':'object','fields':[[k,v]for k,v in fields.items()]}
def set_(node,section=0):return {'op':'set','section':section,'value':ref(node)}
def sync(label,steps):return {'op':'sync','label':label,'steps':steps}
def start(id,label,enter=[],leave=[]):return {'op':'start_async','id':id,'label':label,'enter':enter,'leave':leave}
def finish(id):return {'op':'finish_async','id':id}
def generate():
 OUT.mkdir(parents=True,exist_ok=True);rows=[]
 def add(id,ops,nodes=None,sections=None,**meta):
  if nodes is None:nodes=[obj({'value':n})for n in range(75)]
  if sections is None:sections=[{'key':'probe','value':ref(0)}]
  b=wire(dict(nodes=nodes,sections=sections,ops=ops));(OUT/(id+'.bin')).write_bytes(b);rows.append(dict(id=id,bytes=len(b),sha256=hashlib.sha256(b).hexdigest(),**meta))
 add('simple',[sync('first',[set_(1)]),sync('second',[set_(2)])])
 add('noop',[sync('nothing',[]),sync('same ref',[set_(0)])])
 add('body-throw',[sync('throws',[set_(1),{'op':'throw'}]),sync('after',[set_(2)])])
 add('nested-sync',[sync('outer',[set_(1),sync('inner',[set_(2)]),set_(3)])])
 add('captured-workspace',[sync('move',[set_(1),{'op':'workspace','value':'reverse'}]),sync('new workspace',[set_(2)])])
 add('utf16-label',[sync('É 測定 🦀 \ud800',[set_(1)])])
 for guard in ['busy','hook','restoring','importing','participant']:
  add('sync-guard-'+guard,[{'op':'flags','value':{guard:True}},sync('guarded',[set_(1)]),{'op':'flags','value':{guard:False}},sync('free',[set_(2)])])
  add('async-guard-'+guard,[{'op':'flags','value':{guard:True}},start(1,'async',[set_(1)],[set_(2)]),finish(1),{'op':'flags','value':{guard:False}}])
 add('async-basic',[start(1,'first',[set_(1)],[set_(2)]),finish(1)])
 add('async-overlap-inner-first',[start(1,'outer',[set_(1)],[set_(4)]),start(2,'inner',[set_(2)],[set_(3)]),finish(2),finish(1)])
 add('async-overlap-outer-first',[start(1,'outer',[set_(1)],[set_(3)]),start(2,'inner',[set_(2)],[set_(4)]),finish(1),finish(2)])
 add('sync-while-async',[start(1,'outer',[set_(1)],[set_(3)]),sync('nested sync',[set_(2)]),finish(1)])
 add('async-in-sync',[sync('outer sync',[set_(1),start(1,'inner async',[set_(2)],[set_(3)])]),finish(1)])
 add('async-leave-throw',[start(1,'throws',[],[set_(1),{'op':'throw'}]),finish(1)])
 add('epoch-sync',[sync('lost',[set_(1),{'op':'clear'},set_(2)]),sync('new document',[set_(3)])])
 add('epoch-async',[start(1,'late',[set_(1)],[set_(2)]),{'op':'clear'},finish(1),sync('new',[set_(3)])])
 add('clear-while-paused',[start(1,'late',[],[set_(2)]),{'op':'clear'},sync('not recorded',[set_(1)]),finish(1)])
 add('group-basic',[{'op':'group'},sync('first',[set_(1)]),sync('second',[set_(2)]),{'op':'end_group'},sync('third',[set_(3)])])
 add('group-workspace',[{'op':'group'},sync('first',[set_(1)]),{'op':'workspace','value':'flat'},sync('second',[set_(2)])])
 add('group-reverse-keeps-entry',[{'op':'group'},sync('first',[set_(1)]),sync('back',[set_(0)])])
 add('group-repeat-begin',[{'op':'group'},sync('first',[set_(1)]),{'op':'group'},sync('second',[set_(2)]),{'op':'clear'},{'op':'group'},sync('third',[set_(3)])])
 add('group-active-at-finish',[start(1,'async',[],[set_(1)]),{'op':'group'},finish(1),sync('merged',[set_(2)])])
 add('limit64',[sync('step '+str(i),[set_(i)])for i in range(1,70)])
 add('redo-noop-retains',[sync('first',[set_(1)]),{'op':'seed_future'},sync('noop',[])])
 add('redo-new-clears',[sync('first',[set_(1)]),{'op':'seed_future'},sync('new',[set_(2)])])
 add('group-redo-no-merge',[{'op':'group'},sync('first',[set_(1)]),{'op':'seed_future'},sync('new',[set_(2)])])
 for offset in [1,2]:
  add('capture-sync-'+str(offset),[{'op':'capture_fail','offset':offset},sync('attempt',[set_(1)]),sync('retry',[set_(2)])])
  add('capture-async-'+str(offset),[{'op':'capture_fail','offset':offset},start(1,'attempt',[],[set_(1)]),finish(1),sync('retry',[set_(2)])])
 add('finally-capture-overrides-body',[{'op':'capture_fail','offset':2},sync('both fail',[set_(1),{'op':'throw','message':'body error'}])])
 for identity in [False,True]:
  add('section-equality-'+str(identity).lower(),[sync('same fields new ref',[set_(1)]),sync('changed',[set_(2)])],nodes=[obj({'x':1}),obj({'x':1}),obj({'x':2})],sections=[{'key':'probe','value':ref(0),'identity':identity}])
 add('nan-noop-recorded',[sync('NaN same root',[])],nodes=[obj({'x':float('nan')})])
 add('signed-zero-equal',[sync('minus zero',[set_(1)])],nodes=[obj({'x':0.}),obj({'x':-0.})])
 add('infinite-equal',[sync('same infinity',[set_(1)])],nodes=[obj({'x':float('inf'),'n':-float('inf')}),obj({'x':float('inf'),'n':-float('inf')})])
 add('fields-added-ignored',[sync('extra',[set_(1)]),sync('removed',[set_(0)])],nodes=[obj({'x':1}),obj({'x':1,'y':2})])
 add('undefined-and-prototype',[sync('missing',[set_(1)]),sync('added ignored',[set_(0)]),sync('prototype fallback',[set_(2)]),sync('missing constructor',[set_(1)])],nodes=[obj({'x':UNDEFINED}),obj({}),obj({'constructor':UNDEFINED})])
 arrays=[{'kind':'array','items':[1,2]},{'kind':'array','items':[1,2]},{'kind':'array','items':[1,3]},obj({'a':ref(0)}),obj({'a':ref(1)}),obj({'a':ref(2)})]
 add('array-elements',[sync('equal array',[set_(4)]),sync('different array',[set_(5)])],nodes=arrays,sections=[{'key':'probe','value':ref(3)}])
 nodes=[obj({'x':1}),obj({'x':1}),{'kind':'array','items':[ref(0)]},{'kind':'array','items':[ref(0)]},{'kind':'array','items':[ref(1)]},obj({'a':ref(2)}),obj({'a':ref(3)}),obj({'a':ref(4)})]
 add('array-object-identities',[sync('same item ref',[set_(6)]),sync('new item ref',[set_(7)])],nodes=nodes,sections=[{'key':'probe','value':ref(5)}])
 add('object-field-identities',[sync('different object',[set_(3)])],nodes=[obj({'x':1}),obj({'x':1}),obj({'a':ref(0)}),obj({'a':ref(1)})],sections=[{'key':'probe','value':ref(2)}])
 add('byte-field-identities',[sync('different bytes',[set_(3)])],nodes=[{'kind':'bytes','values':[0,255]},{'kind':'bytes','values':[0,255]},obj({'a':ref(0)}),obj({'a':ref(1)})],sections=[{'key':'probe','value':ref(2)}])
 add('scope-and-scan-identities',[sync('new scope',[set_(1,2)]),sync('new scan',[set_(3,1)]),sync('plugin',[set_(5,0)])],nodes=[{'kind':'words','values':[0,2,4294967295]},{'kind':'words','values':[0,2,4294967295]},obj({'file':'scan.stl'}),obj({'file':'scan.stl'}),obj({'x':0}),obj({'x':1})],sections=[{'key':'probe','value':ref(4)},{'key':'scan','value':ref(2)},{'key':'scope','value':ref(0)}])
 add('group-patch-key-order',[{'op':'group'},sync('a',[set_(1,0)]),sync('b',[set_(2,1)]),sync('a later',[set_(3,0)])],sections=[{'key':'left','value':ref(0)},{'key':'right','value':ref(0)}])
 add('numeric-section-order',[sync('all',[set_(1,0),set_(2,1),set_(3,2)])],sections=[{'key':'10','value':ref(0)},{'key':'2','value':ref(0)},{'key':'probe','value':ref(0)}])
 add('current-policy-identity',[sync('changed policy',[{'op':'mode','section':0,'identity':True},set_(1)])],nodes=[obj({'x':1}),obj({'x':1})])
 add('current-policy-fields',[sync('changed policy',[{'op':'mode','section':0,'identity':False},set_(1)])],nodes=[obj({'x':1}),obj({'x':1})],sections=[{'key':'probe','value':ref(0),'identity':True}])
 for seed in [1,7,99,1039]:
  state=seed
  def rand(n):
   nonlocal state
   state=(1664525*state+1013904223)&0xffffffff;return state%n
  for case in range(12):
   ops=[];nodes=[obj({'x':n%5,'a':n//5})for n in range(12)]
   for j in range(12):
    typ=rand(8)
    if typ==0:ops.append({'op':'group'})
    elif typ==1:ops.append({'op':'end_group'})
    elif typ==2:ops.append({'op':'clear'})
    elif typ==3:ops.append({'op':'workspace','value':['elements','flat','reverse'][rand(3)]})
    elif typ==4:ops.extend([start(j,'async-'+str(j),[set_(rand(12),rand(2))],[set_(rand(12),rand(2))]),finish(j)])
    elif typ==5:ops.append(sync('nested-'+str(j),[sync('inner',[set_(rand(12),rand(2))]),set_(rand(12),rand(2))]))
    else:ops.append(sync('step-'+str(j),[set_(rand(12),rand(2))]))
   add(f'seed-{seed}-{case:02d}',ops,nodes=nodes,sections=[{'key':'left','value':ref(0)},{'key':'right','value':ref(1),'identity':bool(case%2)}],seed=seed)
 add('identity-scalar-nan',[sync('NaN',[])],nodes=[],sections=[{'key':'probe','value':float('nan'),'identity':True}])
 add('array-nan-identity',[sync('same ref',[]),sync('new ref',[set_(3)])],nodes=[{'kind':'array','items':[float('nan')]},{'kind':'array','items':[float('nan')]},obj({'a':ref(0)}),obj({'a':ref(1)})],sections=[{'key':'probe','value':ref(2)}])
 add('numeric-field-order',[sync('new fields',[set_(1)])],nodes=[obj({'10':1,'2':2,'x':3}),obj({'10':2,'2':3,'x':4})])
 add('surrogate-section-key',[sync('unicode',[set_(1)])],sections=[{'key':'測定\ud800','value':ref(0)}])
 manifest=dict(schema=1,license='AGPL-3.0-only',reference_sha='2cd89c6769a10c27ee7754b477ada521c9ef5cb7',seeds=[1,7,99,1039],comparison='HRC1 complete patches and actual graph identities/payloads, workspace/revision/held/current/pending/capture log after every operation',cases=rows);(OUT/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
 table='// SPDX-License-Identifier: AGPL-3.0-only\nvec![\n'+''.join('    Case { id: "'+c['id']+'", input: include_bytes!("../../../../../docs/port-plan/fixtures/history-record/'+c['id']+'.bin") },\n'for c in rows)+']\n';(ROOT/'rust/crates/port-engine/tests/support/history_record_cases.rs').write_text(table);print(len(rows),'history recording scenarios')
if __name__=='__main__':generate()
