// SPDX-License-Identifier: AGPL-3.0-only
// Compile both complete pinned modules. No stubs, private exports or rewritten source.
import {execFileSync} from 'node:child_process'
import {readFileSync,writeFileSync,mkdirSync,rmSync} from 'node:fs'
import {createRequire} from 'node:module'
import {fileURLToPath} from 'node:url'
import {createHash} from 'node:crypto'
const root=fileURLToPath(new URL('../../../',import.meta.url)),pin='2cd89c6769a10c27ee7754b477ada521c9ef5cb7'
if(Number(process.versions.node.split('.')[0])!==22)throw Error('Original requires Node22')
for(const p of ['package-lock.json','package.json'])if(execFileSync('git',['rev-parse',pin+':'+p],{cwd:root,encoding:'utf8'}).trim()!==execFileSync('git',['hash-object',p],{cwd:root,encoding:'utf8'}).trim())throw Error('Changed dependency input '+p)
const req=createRequire(root+'/package.json'),ts=req('typescript'),out=root+'/rust/target/stale-detect-reference'
rmSync(out,{recursive:true,force:true});mkdirSync(out,{recursive:true})
const hash=b=>createHash('sha256').update(b).digest('hex'),files={}
for(const name of ['types','stalePage']){
 const path='src/commands/'+name+'.ts',source=execFileSync('git',['show',pin+':'+path],{cwd:root}),output=ts.transpileModule(source.toString(),{fileName:path,compilerOptions:{module:ts.ModuleKind.CommonJS,target:ts.ScriptTarget.ES2022,sourceMap:true,inlineSources:true}})
 for(const [suffix,data] of [['.ts',source],['.cjs',output.outputText],['.cjs.map',output.sourceMapText]]){writeFileSync(out+'/'+name+suffix,data);files[name+suffix]=hash(data)}
}
writeFileSync(out+'/oracle-provenance.json',JSON.stringify({reference_sha:pin,node:process.versions,typescript:ts.version,files},null,2)+'\n')
