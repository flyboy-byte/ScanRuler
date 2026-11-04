// SPDX-License-Identifier: AGPL-3.0-only
// Actual pinned registration/start/execute/guarded/interrupt/history/activity graph.
import {readFileSync} from 'node:fs'
const url=new URL('../../../rust/target/guarded-reference/dist-session/registry.mjs',import.meta.url)
const hex=s=>Array.from({length:s.length},(_,i)=>s.charCodeAt(i).toString(16).padStart(4,'0')).join('')
const text=s=>s===null?'-':hex(s),turn=()=>new Promise(setImmediate)
const saved={set:globalThis.setTimeout,clear:globalThis.clearTimeout,date:globalThis.Date}
let scenario=0
for(const line of readFileSync(0,'utf8').trimEnd().split('\n')) {
 const [id,...fields]=line.split('\t'),[interrupt,history,busy,bodyMode,label,deferred,fault]=fields.map(Number)
 const api=await import(url.href+'?case='+scenario++),log=[],frames=[],timers=new Map(),jobs=[]
 let stopped=false,doneSeen=0,child='-',caller='pending',doc=0,captures=0,reason,unhandled=0,nextTimer=0,ctx
 const bodyError=new api.CommandError('invalid_state','body failed \ud800')
 const plain=message=>Error(message),error=e=>'e:'+hex(e.name)+':'+hex(e.code??'')+':'+hex(e.message)+':'+Number(e===bodyError)+':'+Number(e===reason)
 const onUnhandled=()=>unhandled++;process.on('unhandledRejection',onUnhandled)
 globalThis.Date=class extends saved.date {constructor(...args){super(...(args.length?args:[1234]))}static now(){return 1234}}
 globalThis.setTimeout=(fn,ms)=>{if(ms!==10)throw Error('unexpected timeout');const id=++nextTimer;timers.set(id,{fn,active:true});log.push('timer:set');return id}
 globalThis.clearTimeout=id=>{timers.get(id).active=false;log.push('timer:clear')}
 api.configureCommands({busy(){log.push('busy');if(stopped)return null;if(busy===3)throw plain('busy fault');return busy===0?null:busy===1?'':'working'}})
 api.registerSettle(()=>{log.push('settle')})
 api.registerBusy('plugin',()=>{log.push('plugin-check');return 'plugin'},[],done=>{log.push('stop');stopped=true;done.then(()=>doneSeen++);return Promise.resolve()})
 api.registerHistory({id:'unit',sections:[{key:'unit',compare:'identity',capture(){captures++;log.push('capture:'+captures);if(bodyMode===14)log.push('capture-paused:'+Number(api.__historyPaused()));if(fault===5&&captures===1||fault===6&&captures===2)throw plain('capture fault');return doc},apply(v){doc=v}}]})
 api.useCommandActivity.subscribe(s=>{log.push(s.running?'activity:'+(s.progress===null?'start':text(s.progress)):'activity:clear');if(fault===1&&s.running&&s.progress===null||fault===2&&s.running&&s.progress!==null||fault===3&&!s.running)throw plain('activity fault')})
 const progress=value=>{log.push('caller-progress:'+text(value));if(fault===4)throw plain('progress fault')}
 const run=(_args,context)=>{
  ctx=context;log.push('body:'+Number(context.deferred===true));
  context.signal.addEventListener('abort',()=>{reason=context.signal.reason;log.push('abort')})
  if(bodyMode===22){const extra=()=>log.push('abort-extra');context.signal.addEventListener('abort',extra);context.signal.addEventListener('abort',extra)}
  doc=1;context.progress('first \ud800')
  if(bodyMode===19){const ignored=Promise.resolve('ignored');void ignored;throw bodyError}
  if(bodyMode===0||bodyMode>=16&&bodyMode<=21)return Promise.resolve('ok')
  if(bodyMode===1)return Promise.reject(plain('body failed'))
  if(bodyMode===15){api.clearHistory();return Promise.resolve('ok')}
  if(bodyMode===2)throw bodyError
  if(bodyMode===7)throw 'thrown \ud800'
  if(bodyMode===8)return Promise.reject(plain('Failed to fetch dynamically imported module: fixture'))
  if(bodyMode===9)throw {toString(){throw new TypeError('String fault')}}
  if(bodyMode===10)throw false
  if(bodyMode===13)throw {toString(){log.push('convert-stale:'+text(api.pageStale()));return 'Failed to fetch dynamically imported module: converted'}}
  return new Promise((resolve,reject)=>jobs.push(()=>{doc=2;context.progress('late');if(bodyMode===4||bodyMode===6)reject(bodyError);else resolve('late')}))
 }
 const command={name:'run',title:'ΟΣ 🦀',description:'fixture',input:{type:'object',properties:{}},timeoutMs:10,run,history:!!history,...(interrupt?{interrupts:true}:{})}
 if(bodyMode>=16&&bodyMode<=18)command.discards=()=>{log.push('discards');if(bodyMode===18)throw plain('discard fault');return bodyMode===16?true:'truthy'}
 if(bodyMode===21)command.scene=true
 if(label)command.label=()=>{log.push('label');if(label===3)throw plain('label fault');if(label===5)throw {toString(){log.push('convert-label');return 'label object'}};return label===1?'Custom \ud800':label===2?'':null}
 api.registerCommands('unit',[command,{name:'stop',title:'Stop',description:'nested interrupt',input:{type:'object',properties:{}},interrupts:true,timeoutMs:10,run(_args,context){log.push('stop:'+Number(context.deferred===true));doc=3;context.progress('stop');return Promise.resolve('stop')}}])
 const state=()=>{
  const activity=api.useCommandActivity.getState(),current=api.runningChange(),h=api.useHistory.getState()
  const past=h.past.map(e=>[text(e.label),text(e.workspace),e.group,Object.keys(e.before).join(','),e.before.unit,e.after.unit].join(':')).join(',')
  return [caller,doc,captures,activity.running?[text(activity.running.name),text(activity.running.title),activity.running.since].join(':'):'-',text(activity.progress),current?[text(current.name),Number(current.deferred)].join(':'):'-',past,h.future.length,Number(h.restoring),Number(h.importing),h.revision,[...timers.values()].filter(t=>t.active).length,log.join(','),unhandled,Number(api.__historyPaused()),text(api.pageStale()),child,doneSeen].join('|')
 }
 try {
  const result=api.runCommand('unit.run',{},progress,undefined,!!deferred)
  result.then(v=>{caller='v:'+hex(v);log.push('reply:ok')},e=>{caller=error(e);log.push('reply:error')})
  frames.push(state());await turn();frames.push(state())
  if(bodyMode===11||bodyMode===12){api.runCommand('unit.stop',{},progress,undefined,true).then(v=>{child='v:'+hex(v);log.push('reply:stop')},e=>{child=error(e);log.push('reply:stop-error')});await turn();frames.push(state())}
  if(bodyMode===5||bodyMode===6||bodyMode===12||bodyMode===22)for(const timer of timers.values())if(timer.active){timer.active=false;log.push('timer:fire');timer.fn()}
  await turn();frames.push(state())
  for(const job of jobs)try{job()}catch(e){log.push('late-throw:'+error(e))}
  await turn();frames.push(state())
  if(caller==='pending'||unhandled)throw Error('unfinished/unhandled fixture')
  process.stdout.write(id+'\t'+frames.join(';')+'\n')
 } finally {globalThis.setTimeout=saved.set;globalThis.clearTimeout=saved.clear;globalThis.Date=saved.date;process.removeListener('unhandledRejection',onUnhandled)}
}
