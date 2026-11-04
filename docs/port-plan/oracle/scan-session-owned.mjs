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
 if(shape.startsWith('grid-')){faces='';for(let y=0;y<4;y++)for(let x=0;x<4;x++){faces+=facet(`${x} ${y} 0`,`${x+1} ${y} 0`,`${x} ${y+1} 0`);faces+=facet(`${x+1} ${y} 0`,`${x+1} ${y+1} 0`,`${x} ${y+1} 0`)}}
 if(shape==='grid-staging'&&seedText!==undefined){
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
 try{if(shape!=='grid-staging')result=await source.runCommand('scan.open',{bytes,name:filename,units,...(shape==='held-discard'?{discard:true}:{})},text=>events.push(['progress',text]))}catch(e){error={code:e.code,message:e.message,name:e.name}}
 const steps=[]
 const step=async(name,args)=>{const start=events.length;let result,error;try{result=await source.runCommand(name,args,text=>events.push(['progress',text]))}catch(e){error={code:e.code,message:e.message,name:e.name}}steps.push({name,input:pack(args),result:pack(result),error:pack(error),versions:{scan:client.scanVersion,nominal:client.nominalVersion},stores:pack(Object.fromEntries(stores.map(([name,s])=>[name,s.getState()]))),eventStart:start,eventEnd:events.length,sources:pack(session.sources.current)})}
 const workerStep=async(name,args,body)=>{const start=events.length;let result,error;try{result=await body()}catch(e){error={code:e.code,message:e.message,name:e.name}}steps.push({name,input:pack(args),result:pack(result),error:pack(error),versions:{scan:client.scanVersion,nominal:client.nominalVersion},nextRequestId:client.nextId,worker:pack(workerSnapshot()),stores:pack(Object.fromEntries(stores.map(([name,s])=>[name,s.getState()]))),eventStart:start,eventEnd:events.length,sources:pack(session.sources.current)})}
 if(shape==='grid-staging'){
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
