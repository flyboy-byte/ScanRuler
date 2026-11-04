// SPDX-License-Identifier: AGPL-3.0-only
// Original prepared-model lifecycle projection; full unprojected records are retained separately.
import {readFileSync,writeFileSync} from 'node:fs'
import {execFileSync} from 'node:child_process'
import {fileURLToPath} from 'node:url'
const root=fileURLToPath(new URL('../../../',import.meta.url)),input=readFileSync(0,'utf8')
const output=execFileSync(process.execPath,[fileURLToPath(new URL('./scan-session-owned.mjs',import.meta.url))],{cwd:root,input,env:{...process.env,SCANRULER_IMPORT_WORKER_STATE:'1'},maxBuffer:256*1024*1024,timeout:180000,encoding:'utf8'})
if(process.env.SCANRULER_WORKER_SOURCE_FULL)writeFileSync(process.env.SCANRULER_WORKER_SOURCE_FULL,output)
function wire(value){const parts=[];const u=n=>{const b=Buffer.alloc(4);b.writeUInt32LE(n);parts.push(b)};const visit=v=>{if(v===null){parts.push(Buffer.from([1]));return}if(typeof v==='boolean'){parts.push(Buffer.from([v?2:3]));return}if(typeof v==='number'){const b=Buffer.alloc(9);b[0]=4;b.writeDoubleLE(v,1);parts.push(b);return}if(typeof v==='string'){parts.push(Buffer.from([5]));u(v.length);const b=Buffer.alloc(v.length*2);for(let i=0;i<v.length;i++)b.writeUInt16LE(v.charCodeAt(i),i*2);parts.push(b);return}if(Array.isArray(v)){parts.push(Buffer.from([6]));u(v.length);v.forEach(visit);return}parts.push(Buffer.from([7]));const entries=Object.entries(v);u(entries.length);for(const[k,v]of entries){visit(k);visit(v)}};visit(value);return Buffer.concat(parts).toString('hex')}
for(const line of output.trim().split('\n')){const v=JSON.parse(line);const loaded=v.events.filter(e=>e[0]==='worker-out'&&e[1].type==='loaded').at(-1)?.[1];const frame={result:v.result,error:v.error,versions:v.versions,nextRequestId:v.nextRequestId,sources:v.sources,stores:v.stores,mesh:loaded?{positions:loaded.positions,indices:loaded.indices}:null,worker:v.worker};console.log(v.id+'\t'+wire(frame))}
