// SPDX-License-Identifier: AGPL-3.0-only
// Local-only Artec CC BY 3.0 inputs. Emit summaries, never scan bytes/mesh arrays.
import {readFileSync} from 'node:fs'
import {fileURLToPath} from 'node:url'
const root=fileURLToPath(new URL('../../../',import.meta.url))
if(process.env.SCANRULER_REAL_SCAN!=='1')throw Error('Private scan comparison is opt-in')
const source=await import(new URL('../../../rust/target/scan-session-owned-reference/dist-session/registry.mjs',import.meta.url))
await source.__portWorkerInThisThread()
const snapshot=await source.__portWorkerSnapshotProvider()
const client=new source.MeshWorkerClient(),session=source.createSession({clientRef:{current:client},sceneRef:{current:null}})
const unwatch=session.watch(),unhost=source.installCommandHost({session});source.registerCoreCommands()
const name='vase.'+process.argv[2]
try {
 const opened=await source.runCommand('scan.open',{bytes:Uint8Array.from(readFileSync(root+'fixtures/private/artec-textured-vase/'+name)),name,units:'mm'})
 const graph=snapshot().graph,sets=source.vertexComponents(graph.indices,graph.vertexCount)
 const components=new Set(Array.from({length:graph.vertexCount},(_,i)=>sets.find(i))).size
 const query=await source.runCommand('scan.query',{box:{min:[-130,-130,-84],max:[130,130,-80]},normal:{dir:[0,0,-1],withinDeg:15},mark:true,fields:'count'})
 const fit=await source.runCommand('element.fit_marked',{kind:'plane',marked:true,sigma:0})
 console.log(JSON.stringify({scan:{...opened.scan,components},selection:query,fit}))
} finally { unhost();unwatch();client.rpc.dispose() }
