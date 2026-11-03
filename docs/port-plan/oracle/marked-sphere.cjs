// SPDX-License-Identifier: AGPL-3.0-only
// Execute the complete pinned dependency graph, no body extraction or stubs.
const {execFileSync}=require('node:child_process'),{readFileSync,writeFileSync}=require('node:fs'),path=require('node:path'),vm=require('node:vm'),ts=require('typescript'),crypto=require('node:crypto')
const {model,packet}=require('./marked-sphere-cases.cjs'),pin='2cd89c6769a10c27ee7754b477ada521c9ef5cb7',sources={},compiled={},maps={},cache={}
function load(name){
 if(cache[name])return cache[name].exports
 const raw=execFileSync('git',['show',pin+':'+name],{encoding:'utf8'});sources[name]=raw
 const transpiled=ts.transpileModule(raw,{fileName:name,compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS,sourceMap:true,inlineSources:true}});compiled[name]=transpiled.outputText;maps[name]=transpiled.sourceMapText
 const module={exports:{}};cache[name]=module
 vm.runInThisContext('(function(exports,require,module){'+compiled[name]+'\n})',{filename:name})(module.exports,request=>{if(!request.startsWith('.'))throw Error('Unreviewed external oracle dependency '+request);return load(path.posix.normalize(path.posix.join(path.posix.dirname(name),request+(path.posix.extname(request)?'':'.ts'))))},module)
 return module.exports
}
const api=load('src/core/fit/fitSphereFromSeed.ts')
function output(result,m){
 const chunks=[],byte=n=>chunks.push(Buffer.from([n])),word=n=>{const b=Buffer.alloc(4);b.writeUInt32LE(n);chunks.push(b)},text=s=>{const b=Buffer.from(s);word(b.length);chunks.push(b)},number=n=>{const b=Buffer.alloc(8);b.writeDoubleLE(n);chunks.push(b)}
 if(result.ok){const r=result.value;byte(0);text(r.kind);for(const n of [...r.center,r.radius,r.sigma,r.formError,r.residualMin,r.residualMax])number(n);word(r.usedPoints);word(r.regionSize);byte(Number(r.held!==undefined));if(r.held){number(r.held.freeDiameter);number(r.held.shift)}word(r.region.length);for(const v of r.region)word(v);byte(Number(r.region===m.idx))}
 else{const e=result.value;byte(1);text(e.name);byte(Number(e instanceof api.FitError));text(e.constructor.name);text(e.message)}
 return Buffer.concat(chunks)
}
for(const line of readFileSync(process.argv[2]??0,'utf8').trim().split('\n')){
 const m=model(line),input=packet(m);let result
 try{result={ok:true,value:api.fitSphereOnSelection({positions:m.positions},m.idx,{sigma:m.k,...(m.held===undefined?{}:{knownDiameter:m.held})})}}catch(e){if(!(e instanceof api.FitError))throw e;result={ok:false,value:e}}
 process.stdout.write(m.id+'\t'+input.toString('hex')+'\t'+output(result,m).toString('hex')+'\t'+packet(m).toString('hex')+'\n')
}
if(process.argv[3])writeFileSync(process.argv[3],JSON.stringify({reference_sha:pin,typescript:ts.version,node:process.versions,sources,compiled,maps,source_sha256:Object.fromEntries(Object.entries(sources).map(([k,v])=>[k,crypto.createHash('sha256').update(v).digest('hex')])),meaning:'Complete unmodified pinned dependency graph, only TypeScript syntax erased. MeshGraph.positions/owned-settings boundary; other graph properties are not read by selection.'},null,2)+'\n')
