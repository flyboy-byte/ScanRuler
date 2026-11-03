// SPDX-License-Identifier: AGPL-3.0-only
import {readFileSync} from 'node:fs'
const bundle=new URL('../../../rust/target/execute-reference/dist-session/registry.mjs',import.meta.url),turn=()=>new Promise(setImmediate)
const hex=s=>Array.from({length:s.length},(_,i)=>s.charCodeAt(i).toString(16).padStart(4,'0')).join('');let count=0
for(const line of readFileSync(0,'utf8').trimEnd().split('\n')){
 const [id,...ns]=line.split('\t'),[discard,busy,deferred,waitMode,bodyMode]=ns.map(Number),api=await import(bundle.href+'?case='+count++),log=[],jobs=[],frames=[];let state='pending',doneSeen=0,cleared=false,unhandled=0
 const onUnhandled=()=>unhandled++;process.on('unhandledRejection',onUnhandled)
 try{
 const fault=kind=>Error(kind),wait=kind=>{if(waitMode===4)throw fault('wait');if(waitMode===0||waitMode===5){cleared=true;return Promise.resolve()}if(waitMode===1)return Promise.reject(fault('wait'));return new Promise((resolve,reject)=>jobs.push({done:false,finish(){if(waitMode===2){cleared=true;resolve()}else reject(fault('wait'))}}))}
 api.configureCommands({busy(){log.push('guard');if(cleared)return null;if(busy===3)throw fault('guard');return busy===0?null:busy===1?'':'busy'}})
 api.registerBusy('plugin',()=>{log.push('plugin-check');return 'plugin'},[],done=>{log.push('stop');done.then(()=>{doneSeen++;log.push('done')});return wait('stop')})
 api.registerSettle(()=>{log.push('settle');if(waitMode===5)return;return wait('settle')})
 const result={value:true},command={title:'Test',history:false,run(_args,ctx){log.push('body:'+Number(ctx.deferred===true));if(bodyMode===4)throw fault('body');if(bodyMode===0)return Promise.resolve(result);if(bodyMode===1)return Promise.reject(fault('body'));return new Promise((resolve,reject)=>jobs.push({done:false,finish:()=>bodyMode===2?resolve(result):reject(fault('body'))}))}}
 if(discard!==0)command.discards=()=>{log.push('discards');if(discard===4)throw fault('discard');return discard===1?false:discard===2?true:'truthy'}
 const error=e=>'err:'+hex(e.name)+':'+hex(e.code??'')+':'+hex(e.message)
 const run=api.execute('unit',command,{},undefined,!!deferred);run.then(v=>state='ok:'+Number(v===result),e=>state=error(e))
 const frame=()=>frames.push([state,doneSeen,log.join(','),unhandled].join('|'));frame();await turn();frame()
 for(let pass=0;pass<3;pass++)for(let j=0;j<jobs.length;j++)if(!jobs[j].done){jobs[j].done=true;jobs[j].finish();log.push('finish:'+j);frame();await turn();frame()}
 await turn();frame();if(state==='pending')throw Error('unfinished')
 process.stdout.write(id+'\t'+frames.join(';')+'\n')
 }finally{process.removeListener('unhandledRejection',onUnhandled)}
}
