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
 const [id,shape,units]=input[0].split('\t')
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
 const body=shape==='malformed'?'not an STL':`solid ${id}\n`+facet('0 0 0','1 0 0','0 1 0')+(shape==='quad'?facet('1 0 0','1 1 0','0 1 0'):'')+'endsolid\n'
 const bytes=Uint8Array.from(Buffer.from(body)),before=pack(Object.fromEntries(stores.map(([name,s])=>[name,s.getState()])))
 let result,error
 try{result=await source.runCommand('scan.open',{bytes,name:id+'.stl',units},text=>events.push(['progress',text]))}catch(e){error={code:e.code,message:e.message,name:e.name}}
 const record={id,input:input[0],bytes:pack(bytes),before,result:pack(result),error:pack(error),versions:{scan:client.scanVersion,nominal:client.nominalVersion},sources:pack(session.sources.current),stores:pack(Object.fromEntries(stores.map(([name,s])=>[name,s.getState()]))),events}
 for(const stop of stops)stop();unwatch();unhost();client.rpc.dispose()
 console.log(JSON.stringify(record))
}
