// SPDX-License-Identifier: AGPL-3.0-only
// Independent controls drive actual pinned runCommand, not a replacement registry.
import {readFileSync} from 'node:fs'
function wire(value){const chunks=[],put=b=>chunks.push(b),u32=n=>{const b=Buffer.alloc(4);b.writeUInt32LE(n);put(b)},text=s=>{u32(s.length);const b=Buffer.alloc(s.length*2);for(let i=0;i<s.length;i++)b.writeUInt16LE(s.charCodeAt(i),i*2);put(b)},walk=v=>{if(v===undefined){put(Buffer.from([0]));return}if(v===null){put(Buffer.from([1]));return}if(typeof v==='boolean'){put(Buffer.from([v?3:2]));return}if(typeof v==='number'){put(Buffer.from([4]));const b=Buffer.alloc(8);b.writeDoubleLE(v);put(b);return}if(typeof v==='string'){put(Buffer.from([5]));text(v);return}if(v instanceof Uint8Array){put(Buffer.from([8]));u32(v.length);put(v);return}if(Array.isArray(v)){put(Buffer.from([6]));u32(v.length);for(const x of v)walk(x);return}const keys=Object.keys(v);put(Buffer.from([7]));u32(keys.length);for(const key of keys){text(key);walk(v[key])}};walk(value);return Buffer.concat(chunks).toString('hex')}
const url=new URL('../../../rust/target/readonly-reference/dist-session/registry.mjs',import.meta.url)
const turn=()=>new Promise(setImmediate),saved={set:setTimeout,clear:clearTimeout,date:Date};let serial=0
for(const line of readFileSync(0,'utf8').trimEnd().split('\n')){
 const [id,...fields]=line.split('\t'),[route,deferred,settleMode,bodyMode,jobMode,busy,extra=0]=fields.map(Number)
 const api=await import(url.href+'?case='+serial++),log=[],frames=[],timers=[],settles=[],bodies=[],blocks=[],unhandled=[]
 let now=1234,doc=0,captures=0,caller='pending',reason
 const typed=new api.CommandError('invalid_state','body failed \ud800'),err=e=>[e.name,e.code,e.message,e===typed,e===reason]
 const onUnhandled=e=>unhandled.push(err(e));process.on('unhandledRejection',onUnhandled)
 globalThis.Date=class extends saved.date {constructor(...args){super(...(args.length?args:[now]))}static now(){if(extra===3||extra===5&&now!==1234)throw Error("clock");return extra===4?NaN:now}}
 globalThis.setTimeout=(fn,ms)=>{if(ms!==10)throw Error('unexpected timer');timers.push({fn,active:true});log.push('timer:set');return timers.length}
 globalThis.clearTimeout=id=>{timers[id-1].active=false;log.push('timer:clear')}
 api.configureCommands({busy(){log.push('busy');if(busy===2)throw Error('busy fault');return busy===1?'working':null}})
 api.registerSettle(next=>{log.push('settle:'+Number(next.readOnly));if(settleMode===0)return;if(settleMode===1)return Promise.resolve();if(settleMode===3)throw Error('ignored settle');return new Promise((resolve,reject)=>settles.push(()=>settleMode===4?reject(Error("ignored async settle")):resolve()))})
 api.registerHistory({id:'unit',sections:[{key:'unit',compare:'identity',capture(){captures++;return doc},apply(v){doc=v}}]})
 api.useCommandActivity.subscribe(s=>log.push(s.running?'activity:start':'activity:clear'))
 const run=(_input,ctx)=>{log.push('body:'+Number(ctx.deferred===true));ctx.signal.addEventListener('abort',()=>{reason=ctx.signal.reason;log.push('abort')});doc++;ctx.progress('first \ud800');if(bodyMode===2)throw typed;if(bodyMode===1)return Promise.reject(Error('body failed'));if(bodyMode===9)return Promise.reject(Error('Failed to fetch dynamically imported module: fixture'));if(bodyMode===3||bodyMode===4)return new Promise(resolve=>bodies.push(()=>{doc++;ctx.progress('late');resolve('late')}));return Promise.resolve(bodyMode===5?undefined:bodyMode===7?new Uint8Array(23):bodyMode===8?'x'.repeat(16000):bodyMode===11?doc:'ok')}
 api.registerCommands('unit',[{name:'run',title:'Read',description:'fixture',input:{type:'object',properties:{}},readOnly:route<4,interrupts:route===1||route===3||route===5,scene:route===2||route===3,history:false,timeoutMs:10,run},{name:'block',title:'Block',description:'scene blocker',input:{type:'object',properties:{}},readOnly:true,scene:true,timeoutMs:10,run(){log.push('block:start');return new Promise(resolve=>blocks.push(()=>{log.push('block:end');resolve('block')}))}}])
 const job=jobMode===0?undefined:jobMode===1?'':jobMode===2?'job':'job\ud800'
 const frame=()=>{const a=api.useCommandActivity.getState(),h=api.useHistory.getState();frames.push(wire([caller,doc,captures,a.running,a.progress,api.runningChange(),h.past.length,h.future.length,h.revision,api.__historyPaused(),timers.filter(t=>t.active).length,log,api.recentOutcomes(),api.recentOutcomes(''),api.recentOutcomes('job'),api.pageStale(),unhandled]))}
 try{
  if(extra===1)api.runCommand('unit.block',{},undefined,undefined,true).then(()=>log.push('reply:block-ok'),()=>log.push('reply:block-error'))
  for(let i=0;i<(extra===2?101:1);i++)try{api.runCommand(route===6?'missing':'unit.run',route===7?42:{},s=>log.push('progress:'+s),job,!!deferred).then(v=>{caller=['ok',v];log.push(extra===2?'reply:'+i+':ok:'+v:'reply:ok')},e=>{caller=['err',...err(e)];log.push(extra===2?'reply:'+i+':error':'reply:error')})}catch(e){caller=['sync',...err(e)]}
  frame();await turn();frame();now=1251.5
  if(bodyMode===4||settleMode===2)for(const t of timers)if(t.active){t.active=false;log.push('timer:fire');t.fn()}
  await turn();frame();for(const finish of settles)finish();await turn();frame();now=1220;for(const finish of blocks)finish();await turn();for(const finish of bodies)finish();await turn();frame()
  if(caller==='pending')throw Error('unfinished case '+id)
  process.stdout.write(id+'\t'+frames.join(';')+'\n')
 }finally{globalThis.Date=saved.date;globalThis.setTimeout=saved.set;globalThis.clearTimeout=saved.clear;process.removeListener('unhandledRejection',onUnhandled)}
}
