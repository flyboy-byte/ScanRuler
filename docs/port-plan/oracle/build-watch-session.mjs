// SPDX-License-Identifier: AGPL-3.0-only
// Build the entire immutable original registry graph; no behavioral stubs.
import {execFileSync} from 'node:child_process'
import {mkdirSync,readFileSync,writeFileSync,symlinkSync,existsSync,rmSync,mkdtempSync,cpSync} from 'node:fs'
import {resolve} from 'node:path'
import {tmpdir} from 'node:os'
import {createHash} from 'node:crypto'
import {createRequire} from 'node:module'
import {fileURLToPath} from 'node:url'
const root=fileURLToPath(new URL('../../../',import.meta.url)),pin='2cd89c6769a10c27ee7754b477ada521c9ef5cb7'
if(Number(process.versions.node.split('.')[0])!==22)throw Error('Original registry requires Node22')
for(const p of ['package-lock.json','package.json']){
 const original=execFileSync('git',['rev-parse',pin+':'+p],{cwd:root,encoding:'utf8'}).trim(),actual=execFileSync('git',['hash-object',p],{cwd:root,encoding:'utf8'}).trim();if(original!==actual)throw Error('Original dependency/version input changed: '+p)
}
const output=resolve(root,'rust/target/watch-session-reference');rmSync(output,{recursive:true,force:true});mkdirSync(output,{recursive:true});symlinkSync(root+'/node_modules',output+'/node_modules','dir')
const dir=mkdtempSync(resolve(tmpdir(),'scanruler-registry-'))
execFileSync('git',['archive','--format=tar','--output',dir+'/source.tar',pin],{cwd:root})
execFileSync('tar',['-xf',dir+'/source.tar','-C',dir])
if(!existsSync(dir+'/node_modules'))symlinkSync(root+'/node_modules',dir+'/node_modules','dir')
writeFileSync(dir+'/.port-session-entry.ts',"export { watchSession, watchSessionStores, registerBusy } from './src/commands/activity'\n")
writeFileSync(dir+'/.port-session-build.mjs',`import { build } from 'vite'\nawait build({root:process.cwd(),configFile:process.cwd()+'/vite.config.ts',logLevel:'warn',publicDir:false,build:{ssr:'.port-session-entry.ts',outDir:'dist-session',emptyOutDir:true,target:'node22',sourcemap:true,rollupOptions:{output:{entryFileNames:'activity.mjs',format:'es'}}},ssr:{target:'node'}})\n`)
execFileSync(process.execPath,['.port-session-build.mjs'],{cwd:dir,stdio:'inherit',timeout:120000})
const hash=b=>createHash('sha256').update(b).digest('hex'),req=createRequire(root+'/package.json')
writeFileSync(output+'/oracle-provenance.json',JSON.stringify({reference_sha:pin,node:process.versions,collator:new Intl.Collator().resolvedOptions(),vite:req('vite/package.json').version,bundle_sha256:hash(readFileSync(dir+'/dist-session/activity.mjs')),source_map_sha256:hash(readFileSync(dir+'/dist-session/activity.mjs.map')),package_lock_sha256:hash(readFileSync(root+'/package-lock.json'))},null,2)+'\n')
cpSync(dir+'/dist-session',output+'/dist-session',{recursive:true})
rmSync(dir,{recursive:true,force:true})
console.log(output+'/dist-session/activity.mjs')
