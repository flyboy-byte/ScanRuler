// SPDX-License-Identifier: AGPL-3.0-only
// Full original prepared-scan consumer and real stores; providers are independent controls.
import {readFileSync} from 'node:fs'
import {execFileSync} from 'node:child_process'
import {once} from 'node:events'
function wire(value){const chunks=[],put=b=>chunks.push(b),u32=n=>{const b=Buffer.alloc(4);b.writeUInt32LE(n);put(b)},text=s=>{u32(s.length);const b=Buffer.alloc(s.length*2);for(let i=0;i<s.length;i++)b.writeUInt16LE(s.charCodeAt(i),i*2);put(b)},walk=v=>{if(v===undefined){put(Buffer.from([0]));return}if(v===null){put(Buffer.from([1]));return}if(typeof v==='boolean'){put(Buffer.from([v?3:2]));return}if(typeof v==='number'){put(Buffer.from([4]));const b=Buffer.alloc(8);b.writeDoubleLE(v);put(b);return}if(typeof v==='string'){put(Buffer.from([5]));text(v);return}if(v instanceof Uint8Array){put(Buffer.from([8]));u32(v.length);put(v);return}if(Array.isArray(v)){put(Buffer.from([6]));u32(v.length);for(const x of v)walk(x);return}const keys=Object.keys(v);put(Buffer.from([7]));u32(keys.length);for(const key of keys){text(key);walk(v[key])}};walk(value);return Buffer.concat(chunks).toString('hex')}
const url=new URL('../../../rust/target/scan-session-commit-reference/dist-session/registry.mjs',import.meta.url)
const plain=v=>{if(typeof v==='function')return undefined;if(v===null||typeof v!=='object'||v instanceof Uint8Array)return v;if(ArrayBuffer.isView(v))return Array.from(v);if(Array.isArray(v))return v.map(plain);return Object.fromEntries(Object.entries(v).filter(([,x])=>typeof x!=='function').map(([k,x])=>[k,plain(x)]))}
const input=readFileSync(0,'utf8').trimEnd().split('\n')
if(process.env.SCANRULER_SCAN_SESSION_CHILD!=='1'){
 for(let start=0;start<input.length;start+=64){
  const output=execFileSync(process.execPath,[new URL(import.meta.url).pathname],{input:input.slice(start,start+64).join('\n')+'\n',env:{...process.env,SCANRULER_SCAN_SESSION_CHILD:'1'},maxBuffer:256*1024*1024})
  if(!process.stdout.write(output))await once(process.stdout,'drain')
 }
}else{
let serial=0
for(const line of input){
 const [id,...f]=line.split('\t'),[meshMode,nominal,dirty,manual,withScene,colours,fault,sequence,seed]=f.map(Number)
 const api=await import(url.href+'?case='+serial++),events=[],frames=[];let enabled=false,failed=false,error=null,initialApp,reentered=false,mutated=false,stopSecond=()=>{}
 const fail=(n,label)=>{if(enabled&&fault===n&&!failed){failed=true;throw Error(label+' fault')}}
 const scene={setPreviewRegion(){events.push(['preview-region']);fail(1,'preview')},setPreview(){events.push(['preview'])},clearNominal(){events.push(['clear-nominal']);fail(2,'nominal')},clearScan(){events.push(['clear-scan'])},modelSize(){events.push(['model-size']);fail(4,'size');return 7+meshMode},modelCenter(){events.push(['model-center']);fail(5,'center');return [4,5,6]}}
 const session=api.createSession({clientRef:{current:null},sceneRef:{current:withScene?scene:null}})
 const point={id:41,kind:'point',name:'Held point',color:'#123456',source:{type:'picked'},status:'done',visible:true,fit:{kind:'point',center:[1,2,3],sigma:0,usedPoints:1,regionSize:1}}
 if(dirty){
  api.useStore.setState({projectName:'kept project',fileName:'old.ply',modelSize:42,modelCenter:[1,2,3],statusText:'old status',progress:'old progress',errorText:'old error',working:'old work',nextId:101,nextDimensionId:202,nextNumber:8,nextOfKind:{point:3,line:3,plane:3,sphere:3,cylinder:3,cone:3,circle:3,torus:3},nextOfDimGroup:{distance:6,angle:7},nextSectionNumber:9,scanHidden:true,elementsHidden:true,settings:{method:'gaussian',sigma:2},showLabels:false,showBackfaces:false,translucent:true,wireframe:true,scanColors:true,showScanColors:false,imprintOpen:true})
  api.historyAction('Agent: seed',()=>api.useStore.setState({elements:[point]}))
  api.useDeviation.setState({source:'element',alignStatus:'done',align:{rotation:[1,0,0,0,1,0,0,0,1],translation:[1,2,3]},alignMessage:'held',alignStopping:true,globalAlign:{held:true},mapStatus:'ready',referenceStatus:'ready',volume:{held:true},volumeStatus:'ready',volumeMessage:'held',volumeHover:7,stats:{held:true},histogram:{held:true},probes:[{id:9,point:[1,2,3],value:0.25}],nextProbeId:99,picking:true,pairs:[{scan:[1,2,3],nominal:[4,5,6]}],pendingScan:[1,2,3],marking:true,maxDistanceAuto:false,animate:true,targetId:41,targetSide:-1,elementStatus:'ready',mapVersion:11,referenceVersion:12,elementVersion:13,targetScope:'marked',scopeCount:8,scopeVersion:4,range:9,rangeAuto:false,maxDistance:7,tolerance:0.4,showHistogram:true,showNominal:false,showScan:false,showMap:false,split:true,localMaxDistance:0.5})
  api.useThickness.setState({status:'ready',message:'held',stats:{held:true},histogram:{held:true},probes:[{id:9,point:[1,2,3],value:0.25}],nextProbeId:99,mapVersion:7,low:0.2,high:4,scaleAuto:false,limit:0.75,showHistogram:false})
  api.useMark.setState({gesture:'brush',erase:true,backfaces:true,diameter:5,count:11,inverts:6})
  api.rememberSurface(41,new Uint32Array([0,1]));session.measure.clearPreview()
  session.sources.current.scan={name:'old.ply',bytes:new Uint8Array([79,76,68]),units:'in'};session.sources.current.image={name:'image.png',bytes:new Uint8Array([1,2,3])}
  for(const [key,ref] of Object.entries(session.maps))ref.current=key==='elementScope'?new Uint32Array([1,2]):key.endsWith('Rgb')?new Uint8Array([7,8]):key==='referenceMap'?{held:new Float32Array([1.25,2.5])}:new Float32Array([1.25,2.5])
 }
 api.useDeviation.setState({nominalName:nominal?'reference.ply':null,nominalVertices:nominal?4:0,nominalTriangles:nominal?4:0,nominalFromSession:nominal===2,nominalBusy:nominal===2,nominalStep:nominal?{held:true}:null})
 session.sources.current.reference=nominal?{name:'reference.ply',bytes:new Uint8Array([82,nominal]),units:'mm'}:null
 api.useThickness.setState({maxThicknessAuto:!manual,maxThickness:manual?17:10})
 initialApp=api.useStore.getState();events.length=0
 const watched=[['app',api.useStore,8],['deviation',api.useDeviation,9],['thickness',api.useThickness,10],['mark',api.useMark,11],['surfaces',api.useSurfaces,12]]
 for(const [name,store,n] of watched)store.subscribe((s,prev)=>{events.push(['state',name,plain(s),plain(prev),s===store.getState(),name==='app'?[s.elements===prev.elements,s.settings===prev.settings,s===initialApp]:null,plain(session.sources.current),plain(Object.fromEntries(Object.entries(session.maps).map(([k,r])=>[k,r.current]))),session.measure.draftSeq.current,plain(api.__portRegions())]);if(name==='app'&&sequence===4&&!reentered){reentered=true;stopSecond();api.useStore.subscribe((current,previous)=>events.push(['app-extra','C',plain(current),plain(previous),current===api.useStore.getState()]));api.useStore.setState({progress:'listener'})}fail(n,name)})
 if(sequence===4)stopSecond=api.useStore.subscribe((current,previous)=>events.push(['app-extra','B',plain(current),plain(previous),current===api.useStore.getState()]))
 api.useHistory.subscribe(s=>{events.push(['history',plain(s)]);fail(7,'history')})
 api.onScan({loaded(scan){events.push(['loaded',plain(scan)]);fail(6,'loaded')}})
 if(sequence===3){
  const late={loaded(){events.push(['loaded-extra','L3'])}},duplicate={loaded(){events.push(['loaded-extra','duplicate'])}};let off=()=>{}
  api.onScan({loaded(){events.push(['loaded-extra','L1']);if(!mutated){mutated=true;off();api.onScan(late)}}})
  off=api.onScan(duplicate);api.onScan(duplicate)
 }
 const prepared=mode=>{
  if(!mode)return null
  let positions=mode===1?[0,0,0,2,0,0,0,3,0]:mode===2?[-1,-2,-3,1,-2,-3,1,2,-3,-1,2,3]:[0,0,0]
  let state=seed>>>0;if(seed)positions=positions.map(()=>{state=(Math.imul(state,1664525)+1013904223)>>>0;return Math.fround((state%2001-1000)/7)})
  positions=new Float32Array(positions);const indices=new Uint32Array(mode===1?[0,1,2]:mode===2?[0,1,2,0,3,1,0,2,3,1,3,2]:[]),vertexCount=positions.length/3,triangleCount=indices.length/3
  const body='ply\nformat ascii 1.0\nelement vertex '+vertexCount+'\nproperty float x\nproperty float y\nproperty float z\nelement face '+triangleCount+'\nproperty list uchar int vertex_indices\nend_header\n'+Array.from({length:vertexCount},(_,i)=>Array.from(positions.slice(i*3,i*3+3)).join(' ')).join('\n')+'\n'+Array.from({length:triangleCount},(_,i)=>'3 '+Array.from(indices.slice(i*3,i*3+3)).join(' ')).join('\n')+'\n'
  return {id:1,source:{name:'scan'+mode+'\ud800.ply',bytes:new TextEncoder().encode(body),units:'mm'},mesh:{positions,indices,vertexCount,triangleCount,...(colours?{colors:new Uint8Array(colours===1?0:vertexCount*3).fill(23)}:{})},view:{commit(){events.push(['view-commit']);fail(3,'view');api.__portNoView.commit()},dispose(){events.push(['view-dispose']);api.__portNoView.dispose()}}}
 }
 const capture=()=>frames.push(wire([error,plain(api.useStore.getState()),plain(api.useDeviation.getState()),plain(api.useThickness.getState()),plain(api.useMark.getState()),plain(api.useSurfaces.getState()),plain(session.sources.current),plain(Object.fromEntries(Object.entries(session.maps).map(([k,r])=>[k,r.current]))),plain(api.__portRegions()),plain(api.useHistory.getState()),session.measure.draftSeq.current,plain(session.measure.draftRegion.current),events]))
 capture();enabled=true
 const commit=mode=>{try{session.scan.commitScan(prepared(mode));error=null}catch(e){error=[e.name,e.message]}capture()}
 commit(meshMode);if(sequence===1)commit(0);if(sequence===2){commit(0);commit(meshMode)}if(sequence===3)commit(meshMode)
 process.stdout.write(id+'\t'+frames.join(';')+'\n')
}

}
