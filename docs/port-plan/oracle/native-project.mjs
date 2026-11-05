// SPDX-License-Identifier: AGPL-3.0-only
// Real frozen useProject/save/load body under SSR. Project worker transport
// calls the same frozen archive functions directly; mesh work still
// uses the original in-thread worker. No fitting or archive behavior is stubbed.
import {readFileSync,writeFileSync,existsSync} from 'node:fs'
import {execFileSync} from 'node:child_process'
import {fileURLToPath} from 'node:url'
import {zipSync,strToU8} from 'fflate'
const root=fileURLToPath(new URL('../../../',import.meta.url)),rows=readFileSync(0,'utf8').trim().split('\n')
if(!process.argv.includes('--case')){
 for(const row of rows)process.stdout.write(execFileSync(process.execPath,[fileURLToPath(import.meta.url),'--case',process.argv[2]??''],{cwd:root,input:row+'\n',encoding:'utf8',timeout:60000,maxBuffer:16*1024*1024}))
}else{
 const [id,units]=rows[0].split('\t'),s=await import(new URL('../../../rust/target/native-project-reference/dist-session/registry.mjs',import.meta.url))
 await s.__portWorkerInThisThread()
 const worker=await s.__portWorkerSnapshotProvider(),client=new s.MeshWorkerClient(),session=s.createSession({clientRef:{current:client},sceneRef:{current:null}})
 const MeshWorker=globalThis.Worker
 globalThis.Worker=class {
  constructor(url,...args){if(!String(url).includes('projectWorker'))return new MeshWorker(url,...args)}
  postMessage(msg){setTimeout(()=>{try{let result;if(msg.type==='pack')result={type:'pack-ok',requestId:msg.requestId,bytes:s.packProject(msg.manifest,msg.members)};else{const p=s.unpackProject(msg.bytes);result={type:'unpack-ok',requestId:msg.requestId,manifest:p.manifest,members:[...p.members]}}this.onmessage?.({data:result})}catch(e){this.onmessage?.({data:{type:'error',requestId:msg.requestId,message:e.message}})}},0)}
  terminate(){}
 }
 let project
 s.renderToString(s.React.createElement(function Mount(){project=s.useProject({sources:session.sources,clientRef:session.clientRef,sceneRef:session.sceneRef,elementScope:session.maps.elementScope,imports:session.imports,commitScan:session.scan.commitScan,commitNominal:session.deviation.commitNominal,commitImage:image=>{if(image!==null)throw Error('Image reconstruction outside M3');s.useFlat.setState(s.useFlat.getInitialState())},runEdgeDetect:async()=>{if(s.useFlat.getState().imageName!==null)throw Error('Image detection outside M3')},runFit:session.measure.runFit,runDeviation:session.deviation.runDeviation,runThickness:session.thickness.runThickness});return null}))
 const unhost=s.installCommandHost({session,project:{save:project.packProject,open:(file,settled)=>project.openProject(file,false,settled)}}),unwatch=session.watch();s.registerCoreCommands()
 const scan=Uint8Array.from(readFileSync(new URL('../fixtures/native-project/scan.ply',import.meta.url)))
 await s.runCommand('scan.open',{bytes:scan,name:'scan.ply',units})
 if(id!=='scan')await s.runCommand('element.fit_marked',{kind:'plane',vertices:Array.from({length:25},(_,i)=>i),sigma:0})
 if(id==='undo'||id==='redo')await s.runCommand('history.undo',{})
 if(id==='redo')await s.runCommand('history.redo',{})
 const initial=await s.runCommand('project.save',{}),p=s.unpackProject(initial.file.bytes),m=p.manifest,files=[...p.members].map(([name,bytes])=>({name,bytes}))
 if(['unknown','empty-unknown'].includes(id)){m.futurePlugin={text:'kept\ud800',nested:{answer:42}};files.push({name:'plugin.bin',bytes:Uint8Array.from([0,255,17,0])});m.name='  Bench / one  ';m.workspace='future-workspace'}
 if(id==='legacy'){delete m.scan.elements[0].source.settings;delete m.scan.showBackfaces;delete m.scan.sections;delete m.scan.nextSectionNumber;delete m.deviation.mapFacingDeg;for(const k of ['notes','nextNoteId','turns','mirror','sheets','subject'])delete m.flat[k]}
 if(id==='pose')m.scan.appliedAlignment={r:[0,-1,0,1,0,0,0,0,1],t:[10,20,30]}
 if(id==='read-as')m.scan.fileName='previous.stl'
 if(id==='empty'||id==='empty-unknown'){m.scan=null;files.splice(0,1)}
 if(id==='newer')m.schemaVersion=2
 if(id==='reserved')m.futurePlugin={nested:{constructor:1}}
 if(id==='missing')files.splice(0,1)
 if(id==='bad-mesh')files[0].bytes=strToU8('ply\nnot a header\n')
 if(id==='path')files.push({name:'bad/file',bytes:Uint8Array.of(1)})
 if(id==='duplicate')files.push({name:'another.json',bytes:Uint8Array.of(1)})
 let bytes=s.packProject(m,files)
 if(id==='not-json')bytes=zipSync({'project.json':strToU8('{')})
 if(id==='corrupt')bytes=Uint8Array.from([0,1,2,3])
 if(id==='duplicate'){const a=Buffer.from('another.json'),b=Buffer.from('project.json');for(let i=0;i<=bytes.length-a.length;i++)if(Buffer.from(bytes.subarray(i,i+a.length)).equals(a))bytes.set(b,i)}
 if(id==='big-manifest'){for(let i=0;i<bytes.length-28;i++)if(new DataView(bytes.buffer,bytes.byteOffset+i,4).getUint32(0,true)===0x02014b50){new DataView(bytes.buffer).setUint32(i+24,64*1024*1024+1,true);break}}
 let result=null,error=null
 try{result=await s.runCommand('project.load',{bytes,name:'project.scanruler',discard:id!=='held'})}catch(e){error={code:e.code,message:e.message,name:e.name}}
 // Only independent wall-clock readings are excluded; all result/state data remain.
 if(result){delete result.runningSince;delete result.runningMs}
 const snapshot=()=>({sources:plain(session.sources.current),app:plain(s.useStore.getState()),deviation:plain(s.useDeviation.getState()),thickness:plain(s.useThickness.getState()),mark:plain(s.useMark.getState()),graph:worker().graph?Object.fromEntries(["positions","indices","normals","adjOffsets","adjList","vertexCount","triangleCount","bboxDiag"].map(k=>[k,plain(worker().graph[k])])):null,history:{past:s.useHistory.getState().past.map(e=>e.label),future:s.useHistory.getState().future.map(e=>e.label)}})
 const state=snapshot();let saved=null,saveError=null
 try{const out=await s.runCommand('project.save',{}),p=s.unpackProject(out.file.bytes);saved={name:out.file.name,holds:out.holds,aligned:out.aligned,manifest:p.manifest,members:[...p.members].map(([name,bytes])=>[name,Buffer.from(bytes).toString('hex')])}}catch(e){saveError={code:e.code,message:e.message,name:e.name}}
 const actual=process.argv[3]+'/'+id+'/saved.scanruler';if(existsSync(actual))s.unpackProject(Uint8Array.from(readFileSync(actual)))
 function plain(v){if(v===undefined)return {$undefined:true};if(typeof v==='function')return undefined;if(v===null||typeof v!=='object')return v;if(ArrayBuffer.isView(v))return Array.from(v);if(Array.isArray(v))return v.map(plain);return Object.fromEntries(Object.entries(v).filter(([,x])=>typeof x!=='function').map(([k,v])=>[k,plain(v)]))}
 console.log(JSON.stringify({id,result,error,state,saved,saveError}));unwatch();unhost();client.rpc.dispose()
}
