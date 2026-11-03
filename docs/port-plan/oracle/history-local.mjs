// SPDX-License-Identifier: AGPL-3.0-only
// Full real original history module and stores; independent authored inputs only.
import {readFileSync} from 'node:fs'
import {pathToFileURL} from 'node:url'
const root=new URL('../../../',import.meta.url)
const bundle=new URL('rust/target/history-local-reference/dist-session/history.mjs',root)
let n=0
for(const line of readFileSync(0,'utf8').trim().split('\n')){
 const [id,mode,m,a,b,g]=line.split('\t'), mask=Number(m), activeFault=Number(a), bodyFault=Number(b),guard=Number(g)
 const api=await import(pathToFileURL(bundle.pathname).href+'?case='+n++)
 const log=[],locals=[]
 api.useShell.getState().setWorkspace('measure')
 api.configureHistory({blocked:()=>guard===1})
 api.useHistory.setState({restoring:guard===2})
 for(let i=0;i<3;i++){
  const local={workspace:['flat','reverse','measure'][i],label:'editor '+i,
   active(){log.push('active:'+i+':'+api.useShell.getState().workspace);if(activeFault===i)throw Error('active-'+i);return !!(mask&(1<<i))},
   canUndo(){throw Error('canUndo must not be called')},canRedo(){throw Error('canRedo must not be called')},
   undo(){log.push('undo:'+i+':'+api.useShell.getState().workspace);if(bodyFault===i)throw Error('body-'+i)},
   redo(){log.push('redo:'+i+':'+api.useShell.getState().workspace);if(bodyFault===i)throw Error('body-'+i)}}
  locals.push(local);api.registerHistory({local})
 }
 let selected='-',error='-'
 try{if(mode==='select'){const local=api.activeLocalHistory();selected=local===null?'none':String(locals.indexOf(local))}else await api[mode]()}catch(e){error=e.message}
 const h=api.useHistory.getState()
 process.stdout.write([id,selected,error,api.useShell.getState().workspace,h.past.length,h.future.length,h.revision,h.restoring?1:0,log.join(',')].join('\t')+'\n')
}
