// SPDX-License-Identifier: AGPL-3.0-only
// Full original command/session/worker observations, independent input controls.
import {readFileSync} from 'node:fs'
import {execFileSync} from 'node:child_process'
import {fileURLToPath} from 'node:url'
const root=fileURLToPath(new URL('../../../',import.meta.url))
if(Number(process.versions.node.split('.')[0])!==22)throw Error('Original requires Node22')
const input=readFileSync(0,'utf8').trim().split('\n').filter(Boolean)
if(!process.argv.includes('--case')){
 for(const row of input)process.stdout.write(execFileSync(process.execPath,[fileURLToPath(import.meta.url),'--case'],{cwd:root,input:row+'\n',timeout:60000,maxBuffer:64*1024*1024}))
}else{
 if(input.length!==1)throw Error('Exactly one isolated case')
 const [id,shape,units,seedText,nText]=input[0].split('\t')
 const source=await import(new URL('../../../rust/target/scan-session-owned-reference/dist-session/registry.mjs',import.meta.url))
 const pack=v=>{
  if(v===undefined)return {$undefined:true}
  if(typeof v==='number'&&(!Number.isFinite(v)||Object.is(v,-0)))return {$number:Object.is(v,-0)?'-0':String(v)}
  if(v===null||typeof v!=='object')return v
  if(v instanceof ArrayBuffer)return {$type:'ArrayBuffer',hex:Buffer.from(v).toString('hex')}
  if(ArrayBuffer.isView(v))return {$type:v.constructor.name,hex:Buffer.from(v.buffer,v.byteOffset,v.byteLength).toString('hex')}
  if(Array.isArray(v))return v.map(pack)
  return Object.fromEntries(Object.entries(v).filter(([,x])=>typeof x!=='function').map(([k,x])=>[k,pack(x)]))
 }
 const events=[]
 await source.__portWorkerInThisThread()
 const workerSnapshot=await source.__portWorkerSnapshotProvider()
 const Original=globalThis.Worker
 globalThis.Worker=class extends Original{
  constructor(...args){super(...args);let listener=null;Object.defineProperty(this,'onmessage',{configurable:true,get:()=>listener?event=>{events.push(['worker-out',pack(event.data)]);listener(event)}:null,set:f=>{listener=f}})}
  postMessage(message,...args){events.push(['worker-in',pack(message)]);return super.postMessage(message,...args)}
 }
 const client=new source.MeshWorkerClient(),session=source.createSession({clientRef:{current:client},sceneRef:{current:null}})
 const stores=[['app',source.useStore],['deviation',source.useDeviation],['thickness',source.useThickness],['mark',source.useMark],['history',source.useHistory]]
 const stops=stores.map(([name,store])=>store.subscribe((current,previous)=>events.push(['store',name,pack(current),pack(previous)])))
 const unwatch=session.watch(),unhost=source.installCommandHost({session});source.registerCoreCommands()
 const facet=(a,b,c)=>`facet normal 0 0 1\nouter loop\nvertex ${a}\nvertex ${b}\nvertex ${c}\nendloop\nendfacet\n`
 let faces=facet('0 0 0','1 0 0','0 1 0')+(shape==='quad'?facet('1 0 0','1 1 0','0 1 0'):'')
 if(shape.startsWith('grid-')||shape.startsWith('plane-')){faces='';for(let y=0;y<4;y++)for(let x=0;x<4;x++){faces+=facet(`${x} ${y} 0`,`${x+1} ${y} 0`,`${x} ${y+1} 0`);faces+=facet(`${x+1} ${y} 0`,`${x+1} ${y+1} 0`,`${x} ${y+1} 0`)}}
 if((shape==='grid-staging'||shape.startsWith('plane-'))&&seedText!==undefined&&seedText!=='flat'){
  const n=Number(nText),points=[];let seed=Number(seedText)>>>0;for(let y=0;y<n;y++)for(let x=0;x<n;x++){seed=(Math.imul(seed,1664525)+1013904223)>>>0;points.push([x,y,Math.fround((seed/4294967296-.5)*.1)].join(' '))}
  faces='';for(let y=0;y<n-1;y++)for(let x=0;x<n-1;x++){const a=y*n+x;faces+=facet(points[a],points[a+1],points[a+n]);faces+=facet(points[a+1],points[a+n+1],points[a+n])}
 }
 const body=shape==='malformed'?'not an STL':`solid ${id}\n`+faces+'endsolid\n'
 let inputBytes=Buffer.from(body),filename=id+'.stl'
 if(shape.startsWith('ply-')){
  filename=id+'.ply';const encoding=shape==='ply-le'?'binary_little_endian':shape==='ply-be'?'binary_big_endian':'ascii'
  const header=Buffer.from(`ply\nformat ${encoding} 1.0\nelement vertex 4\nproperty float x\nproperty float y\nproperty float z\nelement face 2\nproperty list uchar int vertex_indices\nend_header\n`)
  if(shape==='ply-malformed')inputBytes=Buffer.from('ply\nnot a header\n')
  else if(encoding==='ascii')inputBytes=Buffer.concat([header,Buffer.from('0 0 0\n1 0 0\n0 1 0\n1 1 0\n3 0 1 2\n3 1 3 2\n')])
  else {const data=Buffer.alloc(48+26),le=shape==='ply-le';let offset=0;for(const v of [0,0,0,1,0,0,0,1,0,1,1,0]){le?data.writeFloatLE(v,offset):data.writeFloatBE(v,offset);offset+=4}for(const tri of [[0,1,2],[1,3,2]]){data[offset++]=3;for(const v of tri){le?data.writeInt32LE(v,offset):data.writeInt32BE(v,offset);offset+=4}}inputBytes=Buffer.concat([header,data])}
 }
 if(shape.startsWith('obj-')){filename=id+'.obj';inputBytes=readFileSync(new URL('../fixtures/native-import/obj/'+shape.slice(4)+'.obj',import.meta.url))}
 if(shape.startsWith('held-')) {
  const held={}
  if(['held-mixed','held-project','held-discard'].includes(shape))held.elements=[{},{}]
  if(shape!=='held-sections')held.dimensions=[{}]
  if(shape!=='held-dimensions')held.sections=Array.from({length:3},(_,i)=>({id:i+1,name:`Section ${i+1}`,color:'#ffffff',visible:true,frame:{origin:[0,0,0],normal:[0,0,1],basisU:[1,0,0],basisV:[0,1,0]},message:'No scan is loaded.'}))
  if(shape==='held-project')held.projectName='Bench “A”'
  source.useStore.setState(held)
 }
 const bytes=Uint8Array.from(inputBytes),before=pack(Object.fromEntries(stores.map(([name,s])=>[name,s.getState()])))
 let result,error
 try{if(shape!=='grid-staging'&&!shape.endsWith('no-scan'))result=await source.runCommand('scan.open',{bytes,name:filename,units,...(shape==='held-discard'?{discard:true}:{})},text=>events.push(['progress',text]))}catch(e){error={code:e.code,message:e.message,name:e.name}}
 const steps=[]
 const step=async(name,args)=>{const start=events.length;let result,error;try{result=await source.runCommand(name,args,text=>events.push(['progress',text]))}catch(e){error={code:e.code,message:e.message,name:e.name}}steps.push({name,input:pack(args),result:pack(result),error:pack(error),versions:{scan:client.scanVersion,nominal:client.nominalVersion},stores:pack(Object.fromEntries(stores.map(([name,s])=>[name,s.getState()]))),eventStart:start,eventEnd:events.length,sources:pack(session.sources.current)})}
 const workerStep=async(name,args,body)=>{const start=events.length;let result,error;try{result=await body()}catch(e){error={code:e.code,message:e.message,name:e.name}}steps.push({name,input:pack(args),result:pack(result),error:pack(error),versions:{scan:client.scanVersion,nominal:client.nominalVersion},nextRequestId:client.nextId,worker:pack(workerSnapshot()),stores:pack(Object.fromEntries(stores.map(([name,s])=>[name,s.getState()]))),eventStart:start,eventEnd:events.length,sources:pack(session.sources.current)})}
 if(shape.startsWith('plane-')) {
  let vertices=Array.from({length:Number(nText??5)**2},(_,i)=>i)
  if(shape==='plane-short')vertices=vertices.slice(0,3)
  if(shape==='plane-reverse')vertices.reverse()
  if(shape==='plane-duplicate')vertices=[...vertices,...vertices]
  if(shape==='plane-invalid')vertices.push(vertices.length)
  const args={kind:'plane',vertices,sigma:Number(input[0].split('\t')[5]??0)}
  if(shape==='plane-known')args.knownDiameter=5
  if(shape==='plane-assumed')args.assumed=5
  if(shape==='plane-name')args.name='  Bench plane  '
  if(shape==='plane-name-surrogate')args.name='\ud800Bench'
  if(shape==='plane-name-bom')args.name='\ufeff Bench plane \ufeff'
  if(shape==='plane-name-blank')args.name=' \t '
  if(shape==='plane-name-blank-bom')args.name='\ufeff\ufeff'
  if(shape==='plane-missing')delete args.vertices
  if(shape.startsWith('plane-undefined-')) {
   const key=shape.slice('plane-undefined-'.length)
   args[key==='vertices-box'?'vertices':key]=undefined
   if(key==='vertices-box')args.box={min:[-100000,-100000,-100000],max:[100000,100000,100000]}
  }
  if(shape.startsWith('plane-extend-')){
   args.extend=shape==='plane-extend-grow'?{uMin:1,uMax:2,vMin:3,vMax:4}:shape==='plane-extend-trim'?{uMin:-100,uMax:-100,vMin:-100,vMax:-100}:shape==='plane-extend-reverse'?{uMax:-100,uMin:-100,vMax:-100,vMin:-100}:shape==='plane-extend-zero'?{uMin:0,uMax:0,vMin:0,vMax:0}:shape==='plane-extend-empty'?{}:shape==='plane-extend-bad-side'?{start:2}:shape==='plane-extend-partial-bad'?{uMin:3,start:2}:shape==='plane-extend-index-key'?{uMin:3,'0':2}:{uMin:undefined}
  }
  if(shape.startsWith('plane-orient-section')) {
   source.useStore.setState({sections:[{id:1,name:'Section ref',color:'#1877c0',visible:true,frame:{origin:[10,0,0],normal:[1,0,0],basisU:[0,1,0],basisV:[0,0,1]},message:'No scan is loaded.'}],nextId:2,nextSectionNumber:2})
  }
  if(shape.startsWith('plane-orient-')){
   const to=shape==='plane-orient-xy'||shape==='plane-orient-inplane-xy'?'XY':shape==='plane-orient-yz'||shape==='plane-orient-inplane-yz'?'YZ':shape==='plane-orient-xz'?'XZ':shape==='plane-orient-alias'?' zx_plane ':shape==='plane-orient-bad-name'?'Missing':shape==='plane-orient-bad-id'?999:shape==='plane-orient-bad-surrogate'?'\ud800Missing':shape==='plane-orient-reference-name'?' plane 1 ':shape==='plane-orient-reference-string-id'?'1':shape==='plane-orient-reference-unicode'?'ος':shape==='plane-orient-section-name'?'section ref':1
   args.orient={to,...(shape.includes('inplane')?{relation:'inPlane'}:{})}
  }

  if(shape.startsWith('plane-box-')){
   delete args.vertices;args.box={min:[-100000,-100000,-100000],max:[100000,100000,100000]}
   if(shape==='plane-box-empty')args.box={min:[10000,10000,10000],max:[10001,10001,10001]}
   if(shape==='plane-box-short')args.box={min:[0,0,-1],max:[1,1,1]}
   if(shape==='plane-box-inverted')args.box={min:[1,0,0],max:[0,1,1]}
   if(shape==='plane-box-facing')args.normal={dir:[0,0,10]}
   if(shape==='plane-box-away')args.normal={dir:[0,0,-10]}
   if(shape==='plane-box-zero-normal')args.normal={dir:[0,0,0]}
   if(shape==='plane-box-facing-180')args.normal={dir:[0,0,-1],withinDeg:180}
   if(shape==='plane-box-both')args.vertices=vertices
  }
  if(shape==='plane-normal-only')args.normal={dir:[0,0,1]}

  if(shape==='plane-marked-missing'){delete args.vertices;args.marked=true}
  if(shape.startsWith('plane-query-')) {
   const cols=input[0].split('\t'),limit=Number(cols[6]??3),fields=cols[7]??'all'
   const query={box:{min:[-100000,-100000,-100000],max:[100000,100000,100000]},limit}
   if(fields!=='default')query.fields=fields
   if(shape==='plane-query-ball'){delete query.box;query.near={at:[2,2,0],radius:2.5}}
   if(shape==='plane-query-facing')query.normal={dir:[0,0,10]}
   if(shape==='plane-query-zero-normal')query.normal={dir:[0,0,0]}
   if(shape==='plane-query-empty'||shape==='plane-query-empty-marked-fit')query.box={min:[10000,10000,10000],max:[10001,10001,10001]}
   if(shape==='plane-query-inverted')query.box={min:[1,0,0],max:[0,1,1]}
   if(shape.includes('marked'))query.mark=true
   if(shape.startsWith('plane-query-undefined-')) {
    const key=shape.slice('plane-query-undefined-'.length)
    query[key]=undefined
    if(key==='box')query.near={at:[2,2,0],radius:100}
   }
   await step('scan.query',query)
   if(shape==='plane-query-marked-stale')await step('scan.open',{bytes,name:'replacement.stl',units})
   if(shape.includes('marked'))await step('element.fit_marked',{kind:'plane',marked:true,sigma:args.sigma})
  } else if(shape.startsWith('plane-orient-reference-')) {
   const first={...args};delete first.orient
   if(shape==='plane-orient-reference-unicode')first.name='ΟΣ'
   await step('element.fit_marked',first)
   await step('element.fit_marked',args)
  } else if(!shape.startsWith('plane-empty-'))await step('element.fit_marked',args)
  if(shape==='plane-two'||shape==='plane-two-undo')await step('element.fit_marked',args)
  if(['plane-undo','plane-redo','plane-two-undo','plane-undo-new','plane-undo-redo-twice','plane-empty-undo'].includes(shape))await step('history.undo',{})
  if(shape==='plane-two-undo')await step('history.undo',{})
  if(['plane-redo','plane-undo-redo-twice','plane-empty-redo'].includes(shape))await step('history.redo',{})
  if(shape==='plane-undo-new')await step('element.fit_marked',args)
  if(shape==='plane-undo-redo-twice'){await step('history.undo',{});await step('history.redo',{})}
 } else if(shape==='grid-staging'){
  let first,second
  await workerStep('prepareScan',{units:'mm'},async()=>first=await client.prepareScan('first.stl',Uint8Array.from(bytes).buffer,{mode:'on',angleDeg:30},undefined,'mm'))
  await workerStep('prepareScan',{units:'cm'},async()=>second=await client.prepareScan('second.stl',Uint8Array.from(bytes).buffer,{mode:'on',angleDeg:30},undefined,'cm'))
  await workerStep('commitImport',{scan:first.id,nominal:999999},()=>client.commitImport({scan:first.id,nominal:999999}))
  await workerStep('commitImport',{scan:first.id},()=>client.commitImport({scan:first.id}))
  await workerStep('commitImport',{scan:first.id},()=>client.commitImport({scan:first.id}))
  await workerStep('discardImport',{ids:[second.id]},()=>client.discardImport([second.id]))
  await workerStep('commitImport',{scan:null,nominal:null},()=>client.commitImport({scan:null,nominal:null}))
  await workerStep('commitImport',{},()=>client.commitImport({}))
  await workerStep('discardImport',{ids:[]},()=>client.discardImport([]))
 }else if(shape==='grid-fit-undo'||shape==='grid-fit-held'||shape==='grid-fit-discard'){
  await step('element.fit_marked',{kind:'plane',vertices:Array.from({length:25},(_,i)=>i),sigma:0})
  if(shape==='grid-fit-undo')await step('history.undo',{})
  else await step('scan.open',{bytes,name:'replacement.stl',units,...(shape==='grid-fit-discard'?{discard:true}:{})})
 }else if(shape==='grid-bad-replacement')await step('scan.open',{bytes:Uint8Array.from(Buffer.from('not an STL')),name:'broken.stl',units})
 else if(shape==='grid-replacement')await step('scan.open',{bytes,name:'replacement.stl',units})
 const inspection=process.env.SCANRULER_IMPORT_WORKER_STATE?{worker:pack(workerSnapshot()),nextRequestId:client.nextId}:{}
 const record={id,input:input[0],bytes:pack(bytes),before,result:pack(result),error:pack(error),...(steps.length?{steps}:{}),versions:{scan:client.scanVersion,nominal:client.nominalVersion},sources:pack(session.sources.current),stores:pack(Object.fromEntries(stores.map(([name,s])=>[name,s.getState()]))),events,...inspection}
 for(const stop of stops)stop();unwatch();unhost();client.rpc.dispose()
 console.log(JSON.stringify(record))
}
