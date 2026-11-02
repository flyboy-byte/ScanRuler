// SPDX-License-Identifier: AGPL-3.0-only
// Build the entire immutable original registry graph; no behavioral stubs.
import {execFileSync} from 'node:child_process'
import {mkdirSync,readFileSync,writeFileSync,symlinkSync,existsSync} from 'node:fs'
import {resolve} from 'node:path'
import {fileURLToPath} from 'node:url'
const root=fileURLToPath(new URL('../../../',import.meta.url)),pin='2cd89c6769a10c27ee7754b477ada521c9ef5cb7'
if(Number(process.versions.node.split('.')[0])!==22)throw Error('Original registry requires Node22')
for(const p of ['package-lock.json','package.json']){
 const original=execFileSync('git',['rev-parse',pin+':'+p],{cwd:root,encoding:'utf8'}).trim(),actual=execFileSync('git',['hash-object',p],{cwd:root,encoding:'utf8'}).trim();if(original!==actual)throw Error('Original dependency/version input changed: '+p)
}
const dir=resolve(root,'rust/target/registry-reference');mkdirSync(dir,{recursive:true})
execFileSync('git',['archive','--format=tar','--output',dir+'/source.tar',pin],{cwd:root})
execFileSync('tar',['-xf',dir+'/source.tar','-C',dir])
if(!existsSync(dir+'/node_modules'))symlinkSync(root+'/node_modules',dir+'/node_modules','dir')
writeFileSync(dir+'/.port-session-entry.ts',"export { registerCommands, listCommands, runsHeadless, runCommand } from './src/commands/registry'\n")
writeFileSync(dir+'/.port-session-build.mjs',`import { build } from 'vite'\nawait build({root:process.cwd(),configFile:process.cwd()+'/vite.config.ts',logLevel:'warn',publicDir:false,build:{ssr:'.port-session-entry.ts',outDir:'dist-session',emptyOutDir:true,target:'node22',sourcemap:true,rollupOptions:{output:{entryFileNames:'registry.mjs',format:'es'}}},ssr:{target:'node'}})\n`)
execFileSync(process.execPath,['.port-session-build.mjs'],{cwd:dir,stdio:'inherit',timeout:120000})
console.log(dir+'/dist-session/registry.mjs')
