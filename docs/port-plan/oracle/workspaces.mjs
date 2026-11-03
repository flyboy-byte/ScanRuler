// SPDX-License-Identifier: AGPL-3.0-only
import {readFileSync} from 'node:fs'
function wire(value){const chunks=[],put=b=>chunks.push(b),u32=n=>{const b=Buffer.alloc(4);b.writeUInt32LE(n);put(b)},text=s=>{u32(s.length);const b=Buffer.alloc(s.length*2);for(let i=0;i<s.length;i++)b.writeUInt16LE(s.charCodeAt(i),i*2);put(b)},walk=v=>{if(v===undefined){put(Buffer.from([0]));return}if(v===null){put(Buffer.from([1]));return}if(typeof v==='boolean'){put(Buffer.from([v?3:2]));return}if(typeof v==='number'){put(Buffer.from([4]));const b=Buffer.alloc(8);b.writeDoubleLE(v);put(b);return}if(typeof v==='string'){put(Buffer.from([5]));text(v);return}if(v instanceof Uint8Array){put(Buffer.from([8]));u32(v.length);put(v);return}if(Array.isArray(v)){put(Buffer.from([6]));u32(v.length);for(const x of v)walk(x);return}const keys=Object.keys(v);put(Buffer.from([7]));u32(keys.length);for(const key of keys){text(key);walk(v[key])}};walk(value);return Buffer.concat(chunks).toString('hex')}
const bundle=new URL('../../../rust/target/workspaces-reference/dist-session/workspaces.mjs',import.meta.url)
const decode=s=>{let out='';for(let i=0;i<s.length;i+=4)out+=String.fromCharCode(parseInt(s.slice(i,i+4),16));return out},hex=s=>Array.from({length:s.length},(_,i)=>s.charCodeAt(i).toString(16).padStart(4,'0')).join('')
let count=0,current=-1,api
for(const line of readFileSync(0,'utf8').trimEnd().split('\n')){
 const[id,m,h]=line.split('\t'),mask=+m,input=decode(h),ids=['probe-a','probe-b','probe-c','probe-d'].filter((_,i)=>mask<16&&!!(mask&(1<<i)))
 globalThis.localStorage={getItem:key=>key==='scanruler.beta'?JSON.stringify(ids):null,setItem(){},removeItem(){}}
 globalThis.location={search:mask===16?'?plugins=none':''}
 if(mask!==current){api=await import(bundle.href+'?context='+count++);api.installPlugins();current=mask}
 const installed=api.plugins().map(p=>[p.id,p.workspace?p.workspace.id:null]),names=api.workspaceNames();names[0]='changed';names.push('extra');const second=api.workspaceNames();const first=api.plugins().find(p=>p.workspace),previous=first?.workspace.id;if(first)first.workspace.id='later\ud800';const third=api.workspaceNames();if(first)first.workspace.id=previous
 process.stdout.write(id+'\t'+[hex(api.publicWorkspace(input)),hex(api.internalWorkspace(input)),wire(installed),wire(names),wire(second),wire(third)].join('|')+'\n')
}
