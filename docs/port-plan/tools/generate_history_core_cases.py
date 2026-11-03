# SPDX-License-Identifier: AGPL-3.0-only
"""Authored opaque owned data inputs only; no original or Rust answers."""
from pathlib import Path
import json,hashlib,copy,random
from generate_schema_cases import wire,UNDEFINED
ROOT=Path(__file__).resolve().parents[3];OUT=ROOT/'docs/port-plan/fixtures/history-core'
FIELDS={'measure': ['elements', 'dimensions', 'sections', 'nextId', 'nextNumber', 'nextOfKind', 'nextDimensionId', 'nextOfDimGroup', 'nextSectionNumber', 'appliedAlignment', 'modelCenter'], 'flat': ['subject', 'sheets', 'pxPerMm', 'calSource', 'elements', 'nextId', 'nameCounts', 'dimensions', 'nextDimId', 'dimCounts', 'datum', 'counts', 'nextCountId', 'notes', 'nextNoteId', 'turns', 'mirror', 'splitAxes', 'edgeSensitivity', 'showEdges', 'snapToEdge', 'showGrid'], 'deviation': ['source', 'align', 'globalAlign', 'pairs', 'localMaxDistance', 'targetId', 'targetSide', 'targetFacingDeg', 'mapFacingDeg', 'targetScope', 'scopeCount', 'showElement', 'range', 'rangeAuto', 'maxDistance', 'maxDistanceAuto', 'bands', 'tolerance', 'showHistogram', 'showNominal', 'showScan', 'showMap', 'mapOn', 'split', 'probes', 'nextProbeId'], 'thickness': ['method', 'maxThickness', 'maxThicknessAuto', 'coneRays', 'coneAngleDeg', 'normalDeviationDeg', 'low', 'high', 'scaleAuto', 'bands', 'limit', 'showHistogram', 'probes', 'nextProbeId']}
def ref(n):return {'ref':n}
def generate():
 OUT.mkdir(parents=True,exist_ok=True);rows=[]
 def add(id,**options):
  nodes=[]
  def node(kind,**data):nodes.append(dict(kind=kind,**data));return ref(len(nodes)-1)
  empty=node('array',items=[]);obj=node('object',fields=[]);filled=node('array',items=[0,1,2]);refs=node('array',items=[0]);nullrefs=node('array',items=[None]);emptydraft=node('object',fields=[['picks',empty],['refs',nullrefs]]);pickdraft=node('object',fields=[['picks',filled],['refs',empty]]);editdraft=node('object',fields=[['editId',0],['picks',empty],['refs',empty]]);dimdraft=node('object',fields=[['refs',refs]]);tool=node('object',fields=[['kind','datum'],['opaque',filled]]);scope=node('words',values=[1,7,99]);scan=node('object',fields=[['name','scan 🦀']])
  drafts=[None,emptydraft,pickdraft,editdraft];dims=[None,emptydraft,dimdraft,editdraft]
  def value(k,i,changed=False):
   if k in ['elements','dimensions','sections','pairs','probes','counts','notes','modelCenter']:return filled if changed else empty
   if k in ['sheets','nameCounts','dimCounts','nextOfKind','nextOfDimGroup']:return obj
   if k.startswith('show')or k.endswith('Auto')or k in ['mirror','splitAxes','snapToEdge','mapOn','split']:return changed
   if k in ['subject','appliedAlignment','datum','align','globalAlign']:return None
   if k in ['source','method','calSource','targetSide','targetScope']:return 'changed' if changed else 'initial'
   return i+100 if changed else i+0.5
  stores=[[[k,value(k,i)]for i,k in enumerate(keys)]for keys in FIELDS.values()]
  stores[0]+=[['opaque',filled]]
  stores[1]+=[['draft',drafts[options.get('draft',0)]],['dimDraft',dims[options.get('dim',0)]],['tool',tool],['subjectVersion',options.get('version',5)],['opaque',scan]]
  stores[2]+=[['picking',True],['pendingScan',scan],['marking',True],['scopeVersion',options.get('scope_version',9)],['opaque',filled]]
  stores[3]+=[['opaque','keep']]
  patch=[]
  for ix,key in enumerate(FIELDS):
   if options.get('families',15)&(1<<ix):
    fields=[[k,value(k,i,True)]for i,k in enumerate(FIELDS[key])]+[['added',filled]]
    if key=='flat':fields+=[['draft',editdraft],['dimDraft',dimdraft],['tool',tool],['subjectVersion',999]]
    if key=='deviation':fields+=[['picking',True],['pendingScan',scan],['marking',True]]
    patch.append([key,node('object',fields=fields)])
  if options.get('scope',True):patch.append(['scope',options.get('scope_value',scope)])
  patch.append(['alpha',filled]);patch.append(['scan',scan])
  if options.get('null_roots'):patch=[[k,None if k in FIELDS else v]for k,v in patch]
  data=dict(nodes=nodes,stores=stores,scope=scope,scan=scan,participants=options.get('participants',['alpha','missing','flat','alpha']),fault=options.get('fault','none'),ops=[{'op':'capture'},{'op':'apply','patch':patch},{'op':'capture'},{'op':'apply','patch':patch},{'op':'capture'}])
  b=wire(data);(OUT/(id+'.bin')).write_bytes(b);rows.append(dict(id=id,bytes=len(b),sha256=hashlib.sha256(b).hexdigest()))
 for mask in range(16):add('families-'+str(mask),families=mask)
 for d in range(4):
  for dim in range(4):add('draft-'+str(d)+'-'+str(dim),draft=d,dim=dim)
 for fault in ['scope-get','scan-get','scope-set','scope-after','participant-0','participant-1','participant-2','participant-3']:add('fault-'+fault,fault=fault)
 add('scope-absent',scope=False);add('scope-null',scope_value=None);add('scope-undefined',scope_value=UNDEFINED);add('scope-zero',scope_value=0);add('null-roots',null_roots=True)
 for version in [0,-0.,1e21,float('nan'),float('inf')]:add('version-'+str(len(rows)),version=version,scope_version=version)
 for seed in [1,7,99,1039]:
  rng=random.Random(seed)
  for i in range(12):add('seed-'+str(seed)+'-'+str(i),families=rng.randrange(16),draft=rng.randrange(4),dim=rng.randrange(4),scope=bool(rng.randrange(2)),fault=rng.choice(['none','scope-set','participant-0','participant-2']))
 (OUT/'manifest.json').write_text(json.dumps(dict(reference_sha='2cd89c6769a10c27ee7754b477ada521c9ef5cb7',seeds=[1,7,99,1039],fields=FIELDS,cases=rows),indent=2)+'\n');print(len(rows),'core capture/apply cases')
if __name__=='__main__':generate()
