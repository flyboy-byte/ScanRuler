// SPDX-License-Identifier: AGPL-3.0-only
import {readFileSync} from 'node:fs'
function wire(value){const chunks=[],put=b=>chunks.push(b),u32=n=>{const b=Buffer.alloc(4);b.writeUInt32LE(n);put(b)},text=s=>{u32(s.length);const b=Buffer.alloc(s.length*2);for(let i=0;i<s.length;i++)b.writeUInt16LE(s.charCodeAt(i),i*2);put(b)},walk=v=>{if(v===undefined){put(Buffer.from([0]));return}if(v===null){put(Buffer.from([1]));return}if(typeof v==='boolean'){put(Buffer.from([v?3:2]));return}if(typeof v==='number'){put(Buffer.from([4]));const b=Buffer.alloc(8);b.writeDoubleLE(v);put(b);return}if(typeof v==='string'){put(Buffer.from([5]));text(v);return}if(v instanceof Uint8Array){put(Buffer.from([8]));u32(v.length);put(v);return}if(Array.isArray(v)){put(Buffer.from([6]));u32(v.length);for(const x of v)walk(x);return}const keys=Object.keys(v);put(Buffer.from([7]));u32(keys.length);for(const key of keys){text(key);walk(v[key])}};walk(value);return Buffer.concat(chunks).toString('hex')}
const bundle=new URL('../../../rust/target/recent-store-reference/dist-session/registry.mjs',import.meta.url),turn=()=>new Promise(setImmediate)
const codes=['unknown_command','invalid_input','busy','no_scan','not_found','invalid_state','failed','unavailable','not_implemented','timeout','internal','stale_page'],times=[0,-0.9,0.9,-1.9,1.9,8640000000000000,-8640000000000000,8640000000000001,NaN,Infinity]
let count=0
for(const line of readFileSync(0,'utf8').trimEnd().split('\n')){
 const[id,mm,ss,ee]=line.split('\t'),mode=+mm,start=+ss,end=+ee,api=await import(bundle.href+'?case='+count++),oldNow=Date.now,log=[],frames=[],tasks=[],identities=[],callers=[];let now=times[start],phase='start',unhandled=0,frozen,edited,firstRef,throwClock=false
 const onUnhandled=()=>unhandled++;process.on('unhandledRejection',onUnhandled)
 try{
 Date.now=()=>{log.push('clock:'+phase);if(throwClock)throw Error('clock');return now}
 const value=i=>mode===3?undefined:mode===4?'x'.repeat(16000):mode===5?new Uint8Array(23):mode===8?[i,{bytes:new Uint8Array(3)}]:mode===9?null:mode===10?-0:{n:i,nested:{u:undefined,bytes:new Uint8Array(3)}}
 const frame=()=>{const all=api.recentOutcomes(),filtered=['jobA','','missing'].map(j=>api.recentOutcomes(j).map(o=>o.name)),current=all[0];const alias=edited&&current?Number(edited.result===current.result&&edited.error===current.error):'-';frames.push([wire(all),wire(filtered),frozen?wire(frozen):'-',edited?wire(edited):'-',identities.join(','),callers.join(','),log.join(','),unhandled,alias,firstRef?wire(firstRef):'-'].join('|'))}
 const add=i=>{let resolve,reject;const run=new Promise((a,b)=>{resolve=a;reject=b}),job=i%3===0?'jobA':i%3===1?'':'job\ud800',name='cmd'+i+'💩\ud800';try{identities.push(Number(api.remember(job,name,run)===run))}catch(e){identities.push(e.name+':'+e.message)}callers.push('pending');run.then(()=>callers[i]='ok',()=>callers[i]='err');tasks.push({resolve,reject,value:value(i)})}
 const total=mode===6||mode===7?105:1
 for(let i=0;i<total;i++){add(i);if(i===0){frozen=api.recentOutcomes()[0];firstRef=api.recent[0];frame()}}
 frame();await turn();frame()
 now=end===0?times[start]+17.5:end===1?times[start]-1:NaN;throwClock=end===2;phase='end'
 const finish=async(i,failed)=>{if(failed){const error=new api.CommandError(codes[i%12],'fail \ud800');const json=error.toJSON.bind(error);error.toJSON=()=>{log.push('normalize:'+i);if(mode===2)throw Error('normalize');return json()};tasks[i].reject(error)}else tasks[i].resolve(tasks[i].value);log.push('finish:'+i);frame();await turn();frame()}
 if(total===1)await finish(0,mode===1||mode===2);else{await finish(0,true);await finish(104,mode===7);await finish(99,false)}
 const all=api.recentOutcomes();if(all.length){edited=all[0];edited.job='copy';edited.state='failed';edited.ms=-123;if(edited.result&&typeof edited.result==='object'){if(Array.isArray(edited.result))edited.result.push(999);else edited.result.n=999}else if(Object.hasOwn(edited,'result'))edited.result='copy primitive';if(edited.error)edited.error.message='changed';frame()}
 await turn();frame();process.stdout.write(id+'\t'+frames.join(';')+'\n')
 }finally{Date.now=oldNow;process.removeListener('unhandledRejection',onUnhandled)}
}
