#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
"""Freeze an exact candidate graph for review; not a blanket purity/security proof."""
import argparse,hashlib,json,re,tomllib
from pathlib import Path

def sha(data):return hashlib.sha256(data).hexdigest()
def capture(metadata,wasm_metadata,lock):
    native=json.loads(metadata.read_text());wasm=json.loads(wasm_metadata.read_text())
    checksums={(p['name'],p['version']):p.get('checksum') for p in tomllib.loads(lock.read_text())['package']}
    nodes={n['id']:n for n in native['resolve']['nodes']};packages={p['id']:p for p in native['packages']}
    key=lambda p:p['name']+'@'+p['version']
    rows=[]
    for p in sorted(native['packages'],key=key):
        if not p.get('source'):continue
        folder=Path(p['manifest_path']).parent
        licenses=[f for f in folder.iterdir() if f.is_file() and ('LICENSE' in f.name.upper() or 'COPYING' in f.name.upper())]
        if not licenses:raise ValueError('No shipped license file: '+key(p))
        tree=[];foreign=[];ffi=[]
        for f in sorted(folder.rglob('*')):
            if not f.is_file() or f.name.startswith('.cargo'):continue
            relative=f.relative_to(folder).as_posix();data=f.read_bytes();tree.append([relative,sha(data)])
            if f.suffix.lower() in ['.c','.cc','.cpp','.cxx','.s','.a','.so']:foreign.append(relative)
            if f.suffix=='.rs' and relative.startswith('src/'):
                text=data.decode('utf-8')
                for n,line in enumerate(text.splitlines(),1):
                    if re.search(r'extern\s+"C"|rustc-link-lib|cc::Build|cmake|libloading',line):ffi.append(dict(path=relative,line=n))
        builds=[dict(path=Path(t['src_path']).name,sha256=sha(Path(t['src_path']).read_bytes())) for t in p['targets'] if 'custom-build' in t['kind']]
        rows.append(dict(package=key(p),license=p['license'],repository=p['repository'],source=p['source'],checksum=checksums[(p['name'],p['version'])],features=nodes[p['id']]['features'],dependencies=sorted(key(packages[d['pkg']]) for d in nodes[p['id']]['deps']),license_files=[dict(path=f.name,bytes=f.stat().st_size,sha256=sha(f.read_bytes()))for f in sorted(licenses)],manifest_sha256=sha(Path(p['manifest_path']).read_bytes()),source_tree_sha256=sha(json.dumps(tree,separators=(',',':')).encode()),build_scripts=builds,foreign_code_files=foreign,ffi_scan_locations=ffi))
    wm={n['id']:n for n in wasm['resolve']['nodes']};wp={p['id']:p for p in wasm['packages']}
    native_graph={key(p):sorted(nodes[p['id']]['features'])for p in native['packages']if p.get('source')}
    wasm_graph={key(p):sorted(wm[p['id']]['features'])for p in wasm['packages']if p.get('source')and p['id']in wm}
    return dict(contract='image.audit.dependencies.v1',root='image@0.25.10',root_features=['bmp','jpeg','png','webp'],default_features=False,lock_sha256=sha(lock.read_bytes()),native_graph=native_graph,wasm_graph=wasm_graph,packages=rows,verdict='Selected license/purity/build candidate only; pixel compatibility and product adoption require the separate measured verdict.')
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--metadata',type=Path,required=True);p.add_argument('--wasm-metadata',type=Path,required=True);p.add_argument('--lock',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args()
    result=capture(a.metadata,a.wasm_metadata,a.lock);a.out.write_text(json.dumps(result,indent=2)+'\n');print(len(result['packages']),'exact packages captured')
