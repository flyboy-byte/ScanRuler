// SPDX-License-Identifier: AGPL-3.0-only
// The real full history module; typed participant callbacks author only inputs.
import {readFileSync} from 'node:fs'
const bundle=new URL('../../../rust/target/history-restore-reference/dist-session/history.mjs',import.meta.url)
const hex=s=>s===null?'-':Buffer.from(s,'utf16le').toString('hex')
let count=0
for(const line of readFileSync(0,'utf8').trim().split('\n')){
 const [id,direction,n,workspace,fault]=line.split('\t'),records=Number(n)
 const api=await import(bundle.href+'?case='+count++),pool=[0,1,2,10,11,12].map(v=>({v})),log=[]
 let alpha=pool[0],beta=pool[3],enabled=false
 const cell=x=>x===null?'null':pool.indexOf(x)+'/'+x.v
 const snapshot=x=>['scope','scan','alpha','beta'].map(k=>k+'='+cell(x[k])).join(',')
 const patch=x=>Object.keys(x).map(k=>k+'='+cell(x[k])).join(',')
 const view=()=>{const h=api.useHistory.getState(),s=api.useStore.getState();return [h.restoring?1:0,h.past.length,h.future.length,h.revision,api.useShell.getState().workspace,s.busy?1:0,h.importing?1:0].join('/')}
 const event=(name,p,previous)=>{log.push(name+':'+view()+':'+cell(alpha)+','+cell(beta)+(p?':'+patch(p):'')+(previous?':'+snapshot(previous):''))}
 const fail=name=>{if(enabled&&(fault===name||fault==='swallow-'+name))throw Error('fault:'+name+'\ud800')}
 api.configureHistory({getScope(){event('scope');fail('scope');return null},getScan(){event('scan');fail('scan');return null},
  async runRestore(job){event('run:start');fail('run-before');if(enabled&&fault==='skip')return;try{await job()}catch(e){if(!(enabled&&fault.startsWith('swallow-')))throw e}event('run:end');fail('run-after')},
  async beforeRestore(p){event('before-core',p);api.useStore.setState({busy:true});fail('before-core')},
  async afterRestore(p,previous){event('after-core',p,previous);fail('after-core');api.useStore.setState({busy:false})}})
 for(const key of ['alpha','beta'])api.registerHistory({sections:[{key,capture(){event('capture-'+key);fail('capture-'+key);return key==='alpha'?alpha:beta},apply(value){event('apply-'+key);fail('apply-'+key);if(key==='alpha')alpha=value;else beta=value}}],
 async beforeRestore(p){event('before-'+key,p);fail('before-'+key)},async afterRestore(p,previous){event('after-'+key,p,previous);fail('after-'+key)}})
 api.useShell.getState().setWorkspace(workspace)
 for(let i=1;i<=records;i++)api.historyAction('change'+i+'\ud800',()=>{alpha=pool[i];beta=pool[i+3]})
 if(direction==='redo')await api.undo()
 api.useShell.getState().setWorkspace('measure');api.useStore.setState({busy:false,statusText:'initial',errorText:null});log.length=0;enabled=true
 let error='-';try{await api[direction]()}catch(e){error=hex(e.message)}
 const h=api.useHistory.getState(),s=api.useStore.getState()
 const entries=xs=>xs.map(e=>[patch(e.before),patch(e.after),hex(e.label),e.workspace,e.group].join('~')).join(';')
 process.stdout.write([id,error,cell(alpha),cell(beta),entries(h.past),entries(h.future),h.revision,h.restoring?1:0,h.importing?1:0,api.useShell.getState().workspace,s.busy?1:0,hex(s.statusText),hex(s.errorText),log.join(';')].join('\t')+'\n')
}
