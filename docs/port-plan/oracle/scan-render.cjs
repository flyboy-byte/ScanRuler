// SPDX-License-Identifier: AGPL-3.0-only
// Whole frozen modules; TypeScript compiler erases types without body stubs.
const {execFileSync}=require('node:child_process'),fs=require('node:fs'),vm=require('node:vm'),path=require('node:path'),ts=require('typescript')
const pin='2cd89c6769a10c27ee7754b477ada521c9ef5cb7',cache=new Map()
if(Number(process.versions.node.split('.')[0])!==22)throw Error('Pinned original oracle requires Node22')
function load(name){
 if(cache.has(name))return cache.get(name).exports
 if(!name.startsWith('src/core/'))throw Error('Unexpected original import '+name)
 const source=execFileSync('git',['show',pin+':'+name+'.ts'],{encoding:'utf8'})
 const js=ts.transpileModule(source,{compilerOptions:{module:ts.ModuleKind.CommonJS,target:ts.ScriptTarget.ES2022}}).outputText
 const module={exports:{}};cache.set(name,module)
 vm.runInThisContext('(function(require,module,exports){'+js+'\n})',{filename:name+'.ts'})(s=>load(path.posix.normalize(path.posix.join(path.posix.dirname(name),s))),module,module.exports)
 return module.exports
}
const crease=load('src/core/geometry/crease'),wire=load('src/core/geometry/wireSlots'),adj=load('src/core/geometry/adjacency'),normals=load('src/core/geometry/normals')
function mesh(kind,seed){
 let p=[],ix=[];let state=seed>>>0;const random=()=>{state=(Math.imul(state,1664525)+1013904223)>>>0;return state/4294967296}
 if(kind===0){p=[-1,-1,-1,1,-1,-1,1,1,-1,-1,1,-1,-1,-1,1,1,-1,1,1,1,1,-1,1,1];ix=[0,2,1,0,3,2,4,5,6,4,6,7,0,1,5,0,5,4,1,2,6,1,6,5,2,3,7,2,7,6,3,0,4,3,4,7]}
 else if(kind===1||kind===2){p=[0,0,0,1,0,0,0,1,0,1,1,kind===1?0:1];ix=[0,1,2,1,3,2]}
 else if(kind===3||kind===4){const n=kind===3?7:129;p=[0,0,0];for(let k=0;k<n;k++){p.push(k%11,Math.floor(k/11),k%3);ix.push(0,k+1,(k+1)%n+1)}}
 else if(kind===5){p=[0,0,0,1,0,0,0,1,0,2,2,2];ix=[0,0,1,0,1,2,2,1,0,0,1,2]}
 else if(kind===6){p=[0,0,0,1,0,0,0,1,0,0,-1,1,0,1,1];ix=[0,1,2,1,0,3,0,1,4,0,1,2]}
 else if(kind===8){for(let k=0;k<9;k++)p.push(k%3,Math.floor(k/3),k%2);for(let a=0;a<9;a++)for(let b=a+1;b<9;b++)for(let c=b+1;c<9;c++)ix.push(a,b,c)}
 else if(kind===11){for(let t=0;t<40000;t++){const a=t*4,x=t*3;p.push(x,0,0,x+1,0,0,x,1,0,x,0,1);ix.push(a,a+2,a+1,a,a+1,a+3,a,a+3,a+2,a+1,a+2,a+3)}}
 else if(kind===9){}
 else if(kind===10){p=[1,2,3]}
 else {const n=9;for(let y=0;y<n;y++)for(let x=0;x<n;x++)p.push(x,y,(random()-.5)*.2);for(let y=0;y<n-1;y++)for(let x=0;x<n-1;x++){const a=y*n+x;ix.push(a,a+1,a+n,a+1,a+n+1,a+n)}}
 return {positions:Float32Array.from(p),indices:Uint32Array.from(ix)}
}
function observe(row){const [id,k,seed,mode,angle,budget]=row.split('\t');const {positions,indices}=mesh(+k,+seed),n=positions.length/3,{offsets,list}=adj.buildAdjacency(indices,n),slots=wire.wireSlots(offsets,list,n),out=[]
 const u=v=>{const b=Buffer.alloc(4);b.writeUInt32LE(v);out.push(b)},f=v=>{const b=Buffer.alloc(8);b.writeDoubleLE(v);out.push(b)}
 const array=(a,float)=>{u(a.length);for(const v of a){const b=Buffer.alloc(4);float?b.writeFloatLE(v):b.writeUInt32LE(v);out.push(b)}}
 const split=s=>{u(s?1:0);if(s){array(s.positions,true);array(s.indices,false);array(s.normals,true);array(s.wireSlots,false);array(s.copyOf,false)}}
 array(slots,false);u(wire.wireConflicts(indices,slots));f(crease.coplanarShare(positions,indices));u(crease.looksTessellated(positions,indices)?1:0);u(crease.creaseBudget(n));u(crease.creaseBudget(2000000));split(crease.splitCreases(positions,indices,slots,+angle,+budget))
 const result=crease.creaseFor(mode,positions,indices,slots,+angle);split(result.split);u(result.report.added);u([null,'off','scan','budget'].indexOf(result.report.skipped))
 const render=crease.scanRender({positions,indices,normals:normals.computeVertexNormals(positions,indices),adjOffsets:offsets,adjList:list,vertexCount:n,triangleCount:indices.length/3},{mode,angleDeg:+angle});split(render);u(render.crease.added);u([null,'off','scan','budget'].indexOf(render.crease.skipped));return id+'\t'+Buffer.concat(out).toString('hex')
}
for(const row of fs.readFileSync(0,'utf8').trim().split('\n'))console.log(observe(row))
