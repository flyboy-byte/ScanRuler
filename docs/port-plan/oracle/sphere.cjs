// SPDX-License-Identifier: AGPL-3.0-only
// Full frozen TS modules: only TypeScript syntax is removed, no body extraction.
const {execFileSync}=require('node:child_process'),{readFileSync,writeFileSync,mkdirSync}=require('node:fs'),path=require('node:path'),vm=require('node:vm'),ts=require('typescript'),crypto=require('node:crypto')
const {model,packet}=require('./sphere-cases.cjs'),pin='2cd89c6769a10c27ee7754b477ada521c9ef5cb7',sources={},compiled={},maps={},cache={}
for(const file of ['sphere','linalg','clip']){
 const name='src/core/fit/'+file+'.ts',raw=execFileSync('git',['show',pin+':'+name],{encoding:'utf8'});sources[name]=raw
 const result=ts.transpileModule(raw,{fileName:name,compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS,sourceMap:true,inlineSources:true}});compiled[name]=result.outputText;maps[name]=result.sourceMapText
}
function load(name){if(cache[name])return cache[name].exports;const module={exports:{}};cache[name]=module;const run=vm.runInThisContext('(function(exports,require,module){'+compiled[name]+'\n})',{filename:name});run(module.exports,request=>load(path.posix.normalize(path.posix.join(path.posix.dirname(name),request+'.ts'))),module);return module.exports}
const api=load('src/core/fit/sphere.ts')
function output(r){if(r===null)return Buffer.from([0]);const b=Buffer.alloc(1+64+4+r.used.length*4+1+(r.free?32:0));b[0]=1;let at=1;for(const n of [r.sphere.cx,r.sphere.cy,r.sphere.cz,r.sphere.r,r.sigma,r.span,r.min,r.max]){b.writeDoubleLE(n,at);at+=8}b.writeUInt32LE(r.used.length,at);at+=4;for(const n of r.used){b.writeUInt32LE(n,at);at+=4}b[at++]=Number(r.free!==undefined);if(r.free)for(const n of [r.free.cx,r.free.cy,r.free.cz,r.free.r]){b.writeDoubleLE(n,at);at+=8}return b}
for(const line of readFileSync(process.argv[2]??0,'utf8').trim().split('\n')){const m=model(line),before=packet(m),result=api.fitSphereClipped(m.positions,m.idx,m.k,m.held),after=packet(m);process.stdout.write(m.id+'\t'+before.toString('hex')+'\t'+output(result).toString('hex')+'\t'+after.toString('hex')+'\n')}
if(process.argv[3]){const dir=process.argv[3];mkdirSync(dir,{recursive:true});const hash=x=>crypto.createHash('sha256').update(x).digest('hex');writeFileSync(path.join(dir,'modules.json'),JSON.stringify({reference_sha:pin,typescript:ts.version,node:process.versions,sources,compiled,maps,source_sha256:Object.fromEntries(Object.entries(sources).map(([k,v])=>[k,hash(v)])),comparator:{absolute:1e-10,relative:1e-10,used:'exact',optional:'exact',nonfinite:'exact class',signed_zero:'exact sign'}},null,2)+'\n')}
