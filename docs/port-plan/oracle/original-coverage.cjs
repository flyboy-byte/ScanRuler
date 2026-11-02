// SPDX-License-Identifier: AGPL-3.0-only
// Oracle-only instrumentation. Full reference bodies, no changed formulas.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),inspector=require('node:inspector'),crypto=require('node:crypto')
const {execFileSync}=require('node:child_process'),{createRequire}=require('node:module')
const root=process.cwd(),pin='2cd89c6769a10c27ee7754b477ada521c9ef5cb7',ts=require(root+'/node_modules/typescript')
const target=process.argv[2],nativeCandidate=process.argv[3];if(!target||!/^([a-f0-9]{40})$/.test(nativeCandidate||''))throw Error('usage: original-coverage.cjs RUN_DIRECTORY NATIVE_CHECKPOINT_SHA')
const suiteSpecs=[['stl-diff','binary-stl.cjs'],['stl-dispatch-diff','stl-dispatch.cjs'],['ply-ascii-diff','ply-ascii.cjs'],['ply-binary-diff','ply-ascii.cjs'],['obj-mesh-diff','obj-mesh.cjs'],['obj-texture-diff','obj-texture.cjs'],['obj-files-diff','obj-files.cjs'],['drop-files-diff','drop-files.cjs'],['engine-error-diff','engine-errors.cjs'],['schema-diff','schema.cjs']]
const sha=()=>execFileSync('git',['rev-parse','HEAD'],{encoding:'utf8'}).trim(),hash=b=>crypto.createHash('sha256').update(b).digest('hex')
const unitFiles=fs.readdirSync(root+'/docs/port-plan/plan/units').filter(p=>p.endsWith('.json')).map(p=>JSON.parse(fs.readFileSync(root+'/docs/port-plan/plan/units/'+p)))
const paths=[...new Set(unitFiles.filter(u=>u.status==='implemented').flatMap(u=>u.source_paths||[]))].filter(p=>p.endsWith('.ts'))
const sources=new Map(),compiled=new Map(),byCode=new Map(),modules=new Map()
for(const p of paths){const s=execFileSync('git',['show',pin+':'+p],{encoding:'utf8'});sources.set(p,s);byCode.set(s,p)}
function compile(source){
 const p=byCode.get(source);if(!p)throw Error('Unmapped original TS module; never silently map transformed code')
 if(compiled.has(p))return compiled.get(p)
 const r=ts.transpileModule(source,{fileName:p,compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS,sourceMap:true,inlineSources:true}})
 const info={path:p,url:'scanruler-reference/'+p+'.js',source,js:r.outputText,map:JSON.parse(r.sourceMapText)}
 compiled.set(p,info);modules.set(info.js,info);return info
}
const tsProxy=new Proxy(ts,{get(t,p){return p==='transpileModule'?(s,_options)=>({outputText:compile(s).js}):t[p]}})
// Existing STL adapters erase annotations manually. Coverage uses full compiler
// erasure at the same entrypoint; the coordinator compares every output/sidecar.
const stl=sources.get('src/core/parsers/stl.ts');const dispatch=stl.replace("import type { ParsedMesh } from '../types'",'').replace('export function parseSTL(buffer: ArrayBuffer, onProgress?: (text: string) => void): ParsedMesh {','function parseSTL(buffer, onProgress) {').replace('function parseBinary(dv: DataView, triCount: number, onProgress?: (text: string) => void): ParsedMesh {','function parseBinary(dv, triCount, onProgress) {').replace('function parseAscii(buffer: ArrayBuffer, onProgress?: (text: string) => void): ParsedMesh {','function parseAscii(buffer, onProgress) {').replace('const coords: number[] = []','const coords = []').replace('let m: RegExpExecArray | null','let m')+'\nparseSTL'
const sig='function parseBinary(dv: DataView, triCount: number, onProgress?: (text: string) => void): ParsedMesh {'
const helper=stl.slice(stl.indexOf(sig),stl.indexOf('\nfunction parseAscii',stl.indexOf(sig))).replace(sig,'function parseBinary(dv, triCount, onProgress) {')+'\nparseBinary'
const wrappedVm={...vm,runInNewContext(code,context,options){
 let info=modules.get(code)
 if(code===dispatch||code===helper){info=compile(stl);code=info.js+'\n'+(code===dispatch?'exports.parseSTL':'parseBinary');context={...context,exports:{}}}
 if(!info)throw Error('Missing source map at reference execution boundary')
 return vm.runInNewContext(code,context,{...options,filename:info.url})
}}
const gitCache=new Map();for(const[p,s]of sources)gitCache.set(pin+':'+p,s)
async function execute(adapter,input,args=[]){
 const stdout=[],stderr=[],proc={argv:['node',adapter,...args],cwd:()=>root,exitCode:0,stdout:{write:b=>{stdout.push(Buffer.from(b));return true}},stderr:{write:b=>{stderr.push(Buffer.from(b));return true}}}
 const realRequire=createRequire(path.join(root,'docs/port-plan/oracle',adapter));const require_=name=>{
  if(name==='node:fs')return {...fs,readFileSync:(p,...rest)=>p===0?input:fs.readFileSync(p,...rest)}
  if(name==='node:child_process')return {execFileSync:(program,args,options)=>{
   if(program!=='git'||args[0]!=='show'||!gitCache.has(args[1]))throw Error('Unexpected original process boundary')
   const s=gitCache.get(args[1]);return options?.encoding?s:Buffer.from(s)
  }}
  if(name==='node:vm')return wrappedVm
  if(name===root+'/node_modules/typescript')return tsProxy
  return realRequire(name)
 }
 const code=fs.readFileSync(path.join(root,'docs/port-plan/oracle',adapter),'utf8')
 const result=vm.runInNewContext(code,{require:require_,process:proc,Buffer,TextDecoder,TextEncoder,console:{error:e=>proc.stderr.write(String(e)+'\n')}},{filename:'coverage-adapter/'+adapter})
 if(result&&typeof result.then==='function')await result
 return {stdout:Buffer.concat(stdout),stderr:Buffer.concat(stderr),status:proc.exitCode}
}
const session=new inspector.Session();session.connect();const post=(method,params={})=>new Promise((resolve,reject)=>session.post(method,params,(e,r)=>e?reject(e):resolve(r)))
;(async()=>{
 fs.mkdirSync(target,{recursive:true});execFileSync('git',['diff','--exit-code',nativeCandidate,'--','rust'],{stdio:'pipe'});const candidate=sha(),roster=JSON.parse(fs.readFileSync(root+'/docs/port-plan/ci/native-suites.json'));const runs=[],observations=[]
 await post('Profiler.enable');await post('Profiler.startPreciseCoverage',{callCount:true,detailed:true})
 for(const[directory,adapter]of suiteSpecs){
  const spec=roster.suites.find(s=>s.directory===directory);if(!spec)throw Error('Missing native roster suite')
  const base=path.join(root,'rust/target',directory),latest=fs.readdirSync(base).filter(p=>/^run-\d+$/.test(p)).sort((a,b)=>BigInt(a.slice(4))<BigInt(b.slice(4))?-1:1).at(-1),run=path.join(base,latest)
  // Same-candidate complete native evidence is required before coverage replay.
  const identity=fs.existsSync(run+'/candidate.txt')?fs.readFileSync(run+'/candidate.txt','utf8').trim():fs.readFileSync(run+'/identity.txt','utf8').match(/^candidate: ([a-f0-9]{40})$/m)?.[1]
  if(identity!==nativeCandidate)throw Error('Stale native evidence '+directory)
  const u=unitFiles.find(u=>u.id===spec.unit),ids=u.evidence.find(e=>e.kind==='differential').case_ids
  if(JSON.stringify(fs.readdirSync(run).filter(p=>fs.statSync(path.join(run,p)).isDirectory()).sort())!==JSON.stringify([...ids].sort()))throw Error('Case roster mismatch')
  for(const id of ids){
   const d=path.join(run,id),input=fs.readFileSync(d+'/input.bin');let payload=input,args=[]
   if(directory==='stl-diff')args=[fs.readFileSync(d+'/count.txt','utf8').trim()]
   if(directory==='schema-diff'){const len=Buffer.alloc(4);len.writeUInt32LE(input.length);payload=Buffer.concat([len,input])}
   const result=await execute(adapter,Buffer.from(payload),args);let observed=result.stdout
   if(directory==='schema-diff'){if(observed.readUInt32LE(0)!==observed.length-4)throw Error('Bad schema batch');observed=observed.subarray(4)}
   const expected=fs.readFileSync(d+'/original.bin'),side=fs.readFileSync(d+'/'+(fs.existsSync(d+'/original.stderr.txt')?'original.stderr.txt':'original.progress-or-error.txt'))
   if(!observed.equals(expected)||!result.stderr.equals(side))throw Error('Instrumented reference mismatch '+spec.unit+'/'+id)
   // Both success and ordinary source refusal are allowed; other statuses fail.
   const expectedStatus=(['stl-diff','stl-dispatch-diff','ply-ascii-diff','ply-binary-diff'].includes(directory)&&expected.length===0&&side.length>0)?2:0;
   if(result.status!==expectedStatus)throw Error('Reference harness status '+result.status)
   observations.push({unit:spec.unit,id,inputSha256:hash(input),originalSha256:hash(expected),sidecarSha256:hash(side),status:result.status})
  }
  runs.push({unit:spec.unit,run:latest,cases:ids.length});console.log(spec.unit,ids.length,'complete output/sidecar replays')
 }
 const coverage=await post('Profiler.takePreciseCoverage');await post('Profiler.stopPreciseCoverage');session.disconnect()
 const result={contract:'original.coverage.raw.v1',candidateSha:candidate,nativeCandidateSha:nativeCandidate,referenceSha:pin,node:process.version,typescript:ts.version,dirty:execFileSync('git',['status','--porcelain'],{encoding:'utf8'}).trim(),sourceMaps:[...compiled.values()],coverage:coverage.result.filter(s=>s.url.startsWith('scanruler-reference/')),runs,observations,unmappedSources:paths.filter(p=>!compiled.has(p))}
 if(!result.coverage.length||result.observations.length===0)throw Error('Empty coverage')
 fs.writeFileSync(path.join(target,'raw.json'),JSON.stringify(result));console.log('mapped',compiled.size,'sources;',observations.length,'observations')
})().catch(e=>{console.error(e);process.exitCode=1;session.disconnect()})
