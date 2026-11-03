// SPDX-License-Identifier: AGPL-3.0-only
// Transpile the entire pinned class without reached stubs or annotation rewrites.
import {execFileSync} from 'node:child_process'
import {readFileSync,writeFileSync,mkdirSync} from 'node:fs'
import {createRequire} from 'node:module'
import {createHash} from 'node:crypto'
import {fileURLToPath} from 'node:url'
const root=fileURLToPath(new URL('../../../',import.meta.url)),pin='2cd89c6769a10c27ee7754b477ada521c9ef5cb7',require=createRequire(root+'package.json'),ts=require('typescript')
if(Number(process.versions.node.split('.')[0])!==22)throw Error('Original queue requires Node22')
const source=execFileSync('git',['show',pin+':src/app/importQueue.ts'],{cwd:root}),sha=b=>createHash('sha256').update(b).digest('hex')
const result=ts.transpileModule(source.toString(),{fileName:'src/app/importQueue.ts',compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS,sourceMap:true,inlineSources:true}}),out=root+'rust/target/import-queue-reference'
mkdirSync(out,{recursive:true});writeFileSync(out+'/queue.cjs',result.outputText);writeFileSync(out+'/queue.cjs.map',result.sourceMapText)
writeFileSync(out+'/provenance.json',JSON.stringify({reference_sha:pin,node:process.versions,typescript:ts.version,source_sha256:sha(source),bundle_sha256:sha(result.outputText),source_map_sha256:sha(result.sourceMapText),package_lock_sha256:sha(readFileSync(root+'package-lock.json'))},null,2)+'\n')
