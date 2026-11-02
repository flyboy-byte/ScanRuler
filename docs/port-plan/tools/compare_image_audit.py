#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
"""Candidate audit only: preserve every exact mismatch, never bless its output."""
import argparse,collections,hashlib,json,struct,subprocess,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3];BASE=ROOT/'docs/port-plan'
def sha(b):return hashlib.sha256(b).hexdigest()
def frame(raw):
    if len(raw)<9 or raw[:4]!=b'IMG1':raise ValueError('Missing/invalid decoder frame')
    tag=raw[4]
    if tag==1:
        if len(raw)<17:raise ValueError('Missing decoded image')
        w,h,n=struct.unpack_from('<III',raw,5)
        if not w or not h or n!=w*h*4 or len(raw)!=17+n:raise ValueError('Missing pixels or invalid dimensions')
        return dict(kind='image',width=w,height=h,rgba_hex=raw[17:].hex())
    if tag not in [0,2]:raise ValueError('Unknown frame kind')
    n=struct.unpack_from('<I',raw,5)[0]
    if len(raw)!=9+n or not n:raise ValueError('Missing decoder diagnostic')
    return dict(kind='host-refusal' if tag==2 else 'omission',message=raw[9:].decode('utf-8'))
def verdict(source,candidate):
    if candidate['kind']=='host-refusal':return 'host_refusal'
    if source is None:return 'both_omitted' if candidate['kind']=='omission' else 'presence_mismatch'
    if candidate['kind']!='image':return 'presence_mismatch'
    if (source['width'],source['height'])!=(candidate['width'],candidate['height']):return 'dimensions_mismatch'
    return 'exact_pixels' if source['rgba_hex']==candidate['rgba_hex'] else 'pixel_mismatch'
def compare(binary,out):
    manifest=json.loads((BASE/'fixtures/image-audit/manifest.json').read_text())
    source_path=BASE/'evidence/2026-11-02-image-source.json';source=json.loads(source_path.read_text())
    ids=[c['id']for c in manifest['cases']]
    if len(ids)!=56 or len(set(ids))!=56 or ids!=[c['id']for c in source['cases']]:raise ValueError('Missing/changed source corpus')
    run=ROOT/'rust/target/image-audit'/('run-'+str(time.time_ns()));run.mkdir(parents=True)
    candidate_sha=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
    rows=[]
    for c,s in zip(manifest['cases'],source['cases']):
        input_path=BASE/'fixtures/image-audit'/(c['id']+'.bin');b=input_path.read_bytes()
        if sha(b)!=c['sha256'] or s['input_sha256']!=c['sha256']:raise ValueError('Changed input')
        p=subprocess.run([str(binary),str(input_path)],capture_output=True,timeout=10)
        directory=run/c['id'];directory.mkdir()
        for name,data in [('input.bin',b),('source.json',json.dumps(s).encode()),('candidate.bin',p.stdout),('candidate.stderr',p.stderr)]: (directory/name).write_bytes(data)
        if p.returncode or p.stderr:raise ValueError('Candidate process failure: '+c['id']+' '+p.stderr.decode(errors='replace'))
        observed=frame(p.stdout);status=verdict(s['source'],observed)
        row=dict(id=c['id'],format=c['format'],input_sha256=c['sha256'],source=s['source'],candidate=observed,candidate_frame_hex=p.stdout.hex(),candidate_frame_sha256=sha(p.stdout),verdict=status)
        if status=='pixel_mismatch':
            a=bytes.fromhex(s['source']['rgba_hex']);b=bytes.fromhex(observed['rgba_hex']);bad=[i for i,(x,y)in enumerate(zip(a,b))if x!=y]
            row['different_bytes']=len(bad);row['first_difference']=dict(offset=bad[0],source=a[bad[0]],candidate=b[bad[0]]);row['max_abs_byte_difference']=max(abs(a[i]-b[i])for i in bad)
        rows.append(row)
    report=dict(contract='image.audit.comparison.v1',reference_sha=source['reference_sha'],candidate_sha=candidate_sha,source_capture_sha256=sha(source_path.read_bytes()),candidate_binary_sha256=sha(binary.read_bytes()),coordinator_sha256=sha(Path(__file__).read_bytes()),rustc=subprocess.check_output(['rustc','--version'],text=True).strip(),seeds=manifest['seeds'],comparison='exact presence/dimensions/every RGBA byte; no tolerance or normalization',counts=dict(sorted(collections.Counter(c['verdict']for c in rows).items())),cases=rows,status='completed-candidate-audit-not-a-port')
    out.write_text(json.dumps(report,indent=2)+'\n');(run/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(report['counts']);print(run)
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args();compare(a.binary,a.out)
