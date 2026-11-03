// SPDX-License-Identifier: AGPL-3.0-only
import {readFileSync} from 'node:fs'
const bundle=new URL('../../../rust/target/settle-registry-reference/dist-session/registry.mjs',import.meta.url),turn=()=>new Promise(setImmediate)
const hex=s=>Array.from({length:s.length},(_,i)=>s.charCodeAt(i).toString(16).padStart(4,'0')).join('');let count=0
for(const line of readFileSync(0,'utf8').trimEnd().split('\n')){
 const [id,a,b,c,d,after]=line.split('\t'),initial=a.split(',').map(Number),modes=b.split(',').map(Number),actions=c.split(',').map(Number),targets=d.split(',').map(Number),api=await import(bundle.href+'?case='+count++),log=[],jobs=[],frames=[],status=[],mutated=new Set(),subscriptions=new Map()
 let unhandled=0;const onUnhandled=()=>unhandled++;process.on('unhandledRejection',onUnhandled)
 try{
 const remove=i=>{const undo=api.registerSettle(callbacks[i]);undo()}
 const callbacks=modes.map((mode,i)=>next=>{
  log.push('call:'+i+':'+hex(next.name)+':'+Number(next.readOnly))
  if(!mutated.has(i)){mutated.add(i);const target=targets[i];switch(actions[i]){
   case 1:remove(i);break;case 2:remove(target);break;case 3:api.registerSettle(callbacks[target]);break;
   case 4:remove(i);api.registerSettle(callbacks[i]);break;
   case 5:remove(target);api.registerSettle(callbacks[target]);break;
   case 6:api.registerSettle(callbacks[target]);api.registerSettle(callbacks[target]);break;
  }}
  if(mode===0)return;if(mode===1)throw Error('sync-'+i)
  if(mode===2)return Promise.resolve();if(mode===3)return Promise.reject(Error('ready-'+i))
  return new Promise((resolve,reject)=>jobs.push({done:false,finish:()=>mode===4?resolve():reject(Error('late-'+i))}))
 })
 for(const i of initial)subscriptions.set(i,api.registerSettle(callbacks[i]))
 const frame=()=>frames.push([status.join(','),log.join(','),unhandled].join('|'))
 for(let round=0;round<2;round++){
  const wait=api.settle(round?'mesh.write\ud800':'mesh.read',{readOnly:!!round});status.push(wait===null?'null':'pending');if(wait)wait.then(()=>{status[round]='done'})
  frame();await turn();frame()
  for(let i=jobs.length-1;i>=0;i--)if(!jobs[i].done){jobs[i].done=true;jobs[i].finish();log.push('finish:'+i);frame();await turn();frame()}
  await turn();frame();if(status[round]==='pending')throw Error('Unfinished work')
  if(round===0){if(after==='reregister0')api.registerSettle(callbacks[0]);if(after!=='none')subscriptions.get(0)?.();log.push('after:'+after);frame()}
 }
 process.stdout.write(id+'\t'+frames.join(';')+'\n')
 }finally{process.removeListener('unhandledRejection',onUnhandled)}
}
