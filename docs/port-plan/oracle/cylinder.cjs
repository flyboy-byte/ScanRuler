// SPDX-License-Identifier: AGPL-3.0-only
// Pinned cylinder.ts/circle2d.ts and their imports, TypeScript syntax removed only.
// Reads fixtures/cylinder/cases.tsv rows on stdin; prints id<TAB>hex(result).
const {execFileSync}=require('node:child_process'),{readFileSync}=require('node:fs'),path=require('node:path'),vm=require('node:vm'),ts=require('typescript')
const pin='2cd89c6769a10c27ee7754b477ada521c9ef5cb7',cache={}
function load(name){
 if(cache[name])return cache[name].exports
 const raw=execFileSync('git',['show',pin+':'+name],{encoding:'utf8'})
 const out=ts.transpileModule(raw,{fileName:name,compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS}}).outputText
 const module={exports:{}};cache[name]=module
 vm.runInThisContext('(function(exports,require,module){'+out+'\n})',{filename:name})(module.exports,r=>load(path.posix.normalize(path.posix.join(path.posix.dirname(name),r+'.ts'))),module)
 return module.exports
}
const cyl=load('src/core/fit/cylinder.ts'),circle=load('src/core/fit/circle2d.ts')
const f32=h=>{const b=Buffer.from(h,'hex');const a=new Float32Array(b.length/4);for(let i=0;i<a.length;i++)a[i]=b.readFloatLE(i*4);return a}
const num=s=>s==='nan'?NaN:s==='inf'?Infinity:s==='-inf'?-Infinity:Number(s)
const enc=()=>{const parts=[];return{b(x){parts.push(Buffer.from([x]))},f(x){const b=Buffer.alloc(8);b.writeDoubleLE(x);parts.push(b)},u(x){const b=Buffer.alloc(4);b.writeUInt32LE(x);parts.push(b)},hex(){return Buffer.concat(parts).toString('hex')}}}
const cylOut=(e,c)=>{for(const k of ['px','py','pz','ax','ay','az','r'])e.f(c[k])}
const asCyl=a=>({px:a[0],py:a[1],pz:a[2],ax:a[3],ay:a[4],az:a[5],r:a[6]})
for(const line of readFileSync(0,'utf8').trim().split('\n')){
 const [id,op,ptext,ph,nh,itext]=line.split('\t'),params=ptext?ptext.split(',').map(num):[],p=f32(ph),n=f32(nh),idx=Uint32Array.from(itext?itext.split(',').map(Number):[]),e=enc()
 if(op==='clipped'){const r=cyl.fitCylinderClipped(p,idx,asCyl(params),params[7]);if(!r)e.b(0);else{e.b(1);cylOut(e,r.cylinder);for(const x of [r.sigma,r.span,r.min,r.max])e.f(x);e.u(r.used.length);for(const i of r.used)e.u(i)}}
 else if(op==='refine'){cylOut(e,cyl.refineCylinderGeometric(p,idx,asCyl(params),params[7]))}
 else if(op==='onaxis'){const r=cyl.fitCylinderOnAxis(p,idx,params);if(!r)e.b(0);else{e.b(1);cylOut(e,r)}}
 else if(op==='axis'){const r=cyl.axisFromNormals(n,idx);if(!r)e.b(0);else{e.b(1);for(const x of r)e.f(x)}}
 else if(op==='ransac'){const r=cyl.ransacCylinder(p,n,idx,{seed:params[0],iterations:params[1]});if(!r)e.b(0);else{e.b(1);cylOut(e,r.cylinder);e.f(r.sigma);e.u(r.inliers.length);for(const i of r.inliers)e.u(i)}}
 else if(op==='circle'){const pu=[],pv=[];for(const i of idx){pu.push(p[i*3]);pv.push(p[i*3+1])}const r=circle.fitCircle2d(pu,pv);if(!r)e.b(0);else{e.b(1);for(const x of [r.cu,r.cv,r.r])e.f(x)}}
 else throw new Error('unknown op '+op)
 process.stdout.write(id+'\t'+e.hex()+'\n')
}
