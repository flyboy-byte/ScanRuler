// SPDX-License-Identifier: AGPL-3.0-only
// Full pinned limited + inScene bodies, not a substitute promise/queue model.
import {readFileSync} from 'node:fs'
const bundle=new URL('../../../rust/target/reactions-reference/dist-session/registry.mjs',import.meta.url)
const hex=s=>Array.from({length:s.length},(_,i)=>s.charCodeAt(i).toString(16).padStart(4,'0')).join('')
const turn=()=>new Promise(setImmediate), realSet=globalThis.setTimeout, realClear=globalThis.clearTimeout
let count=0
for(const line of readFileSync(0,'utf8').trimEnd().split('\n')){
 const [id,policy,late,drop]=line.split('\t'),api=await import(bundle.href+'?case='+count++),log=[],frames=[],states=['pending','pending','pending']
 let body='pending',doc=0,reason,active=false,timer,resolveBody,rejectBody,unhandled=0
 const bodyPromise=new Promise((resolve,reject)=>{resolveBody=resolve;rejectBody=reject})
 const error=e=>'error:'+hex(e.name)+':'+hex(e.code??'')+':'+hex(e.message)+':'+Number(e===reason)
 const resolve=value=>{if(body!=='pending')return;body='ok:'+value;doc=42;log.push('body:ok');resolveBody(value)}
 const reject=e=>{if(body!=='pending')return;body=error(e);doc=-1;log.push('body:error');rejectBody(e)}
 const onUnhandled=()=>unhandled++;process.on('unhandledRejection',onUnhandled)
 globalThis.setTimeout=(f,ms)=>{if(ms!==10)throw Error('duration');active=true;timer=f;log.push('timer:set');return 1}
 globalThis.clearTimeout=timerId=>{if(timerId!==1)throw Error('timer identity');active=false;log.push('timer:clear')}
 const fire=()=>{if(!active){log.push('timer:ignored');return}active=false;log.push('timer:fire');timer()}
 const frame=()=>frames.push([states.join(','),body,doc,Number(reason!==undefined),Number(active),log.join(','),unhandled].join('|'))
 try{
 for(let i=0;i<3;i++){
  const result=api.inScene({scene:true},()=>{
   if(i===1){log.push('second:start');throw Error('second failed')}
   if(i===2){log.push('third:start');return Promise.resolve('third')}
   log.push('first:start');const abort=new AbortController()
   abort.signal.addEventListener('abort',()=>{reason=abort.signal.reason;log.push('abort');if(+policy===1)resolve('fallback');else if(+policy===2)reject(reason)})
   return api.limited('first',10,abort,()=>{log.push('body:start');return bodyPromise})
  })
  // Native drop controls only its powerless Ticket; the captured result receiver stays owned.
  void drop
  result.then(v=>{states[i]='ok:'+v;log.push('reply:'+i+':ok')},e=>{states[i]=error(e);log.push('reply:'+i+':error')})
 }
 frame();await turn();frame();fire();await turn();frame()
 if(+late)reject(Error('late failed'));else resolve('late')
 await turn();frame();fire();await turn();frame()
 if(states.includes('pending')||body==='pending'||active||unhandled)throw Error('incomplete/unhandled reaction case')
 process.stdout.write(id+'\t'+frames.join(';')+'\n')
 }finally{globalThis.setTimeout=realSet;globalThis.clearTimeout=realClear;process.removeListener('unhandledRejection',onUnhandled)}
}
