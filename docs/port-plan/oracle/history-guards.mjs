// SPDX-License-Identifier: AGPL-3.0-only
// Full original history/store/model graph, evaluated on independent typed inputs.
import {readFileSync} from 'node:fs'
const bundle=new URL('../../../rust/target/history-guards-reference/dist-session/history.mjs',import.meta.url)
function draft(code){if(code===0)return null;const d={picks:[],refs:[],selection:[],kind:'plane'};if(code===2)d.editId=0;if(code===3)d.picks=[[0,0,0]];if(code===4)d.refs=[0];if(code===5)d.refs=[null];if(code===6)d.selection=[0];if(code===7)d.editId=undefined;if(code===8)d.editId=NaN;return d}
let count=0
for(const line of readFileSync(0,'utf8').trim().split('\n')){
 const [id,...fields]=line.split('\t'),[mask,a,b,c,d,participants,partFault,hookFault]=fields.map(Number)
 const api=await import(bundle.href+'?case='+count++), log=[]
 const bit=i=>!!(mask&(1<<i)),s=draft(a),f=draft(b),sd=draft(c),fd=draft(d)
 api.useStore.setState({busy:bit(2),draft:s,dimDraft:sd,alignDraft:bit(8)?{}:null,sectionDraft:bit(9)?{}:null})
 api.useFlat.setState({imageBusy:bit(3),draft:f,dimDraft:fd,tool:{kind:bit(10)?'count':'none'}})
 api.useDeviation.setState({nominalBusy:bit(4),alignStatus:bit(5)?'running':'idle',mapStatus:bit(6)?'running':'idle'})
 api.useThickness.setState({status:bit(7)?'running':'idle'})
 let release,pending
 if(bit(1)){
  // historyAsync intentionally ignores busy flags. Guard hooks are installed below.
  pending=api.historyAsync('hold',()=>new Promise(resolve=>{release=resolve}))
 }
 api.useHistory.setState({restoring:bit(0)})
 api.configureHistory({blocked(){log.push('hook');if(hookFault)throw Error('hook');return bit(11)}})
 for(let i=0;i<3;i++)api.registerHistory({blocked(){log.push('participant:'+i);if(partFault===i)throw Error('participant-'+i);return !!(participants&(1<<i))}})
 let blocked='-',error='-'
 try{blocked=api.historyBlocked()?'1':'0'}catch(e){error=e.message}
 process.stdout.write([id,blocked,error,api.draftHolds(s)?1:0,api.flatDraftHolds(f)?1:0,api.dimHolds(sd)?1:0,api.dimHolds(fd)?1:0,log.join(',')].join('\t')+'\n')
 if(pending){release();await pending}
}
