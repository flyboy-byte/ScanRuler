# SPDX-License-Identifier: AGPL-3.0-only
"""Source-only capture; default stays in target, never overwrites frozen evidence."""
import argparse,gzip,hashlib,json,struct,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3];PLAN=ROOT/'docs/port-plan'
def capture(out):
 folder=PLAN/'fixtures/history-record';m=json.loads((folder/'manifest.json').read_text());data=b''
 for c in m['cases']:
  b=(folder/(c['id']+'.bin')).read_bytes();assert hashlib.sha256(b).hexdigest()==c['sha256'];data+=struct.pack('<I',len(b))+b
 r=subprocess.run(['fnm','exec','--using=22','node',str(PLAN/'oracle/history-record.mjs')],input=data,cwd=ROOT,capture_output=True,check=True)
 assert not r.stderr,r.stderr;at=0;rows=[]
 for c in m['cases']:
  n=struct.unpack_from('<I',r.stdout,at)[0];at+=4;b=r.stdout[at:at+n];at+=n
  rows.append(dict(id=c['id'],bytes=n,sha256=hashlib.sha256(b).hexdigest(),hex=b.hex()))
 assert at==len(r.stdout)
 bundle=ROOT/'rust/target/history-reference/dist-session/history.mjs'
 report=dict(bundle_sha256=hashlib.sha256(bundle.read_bytes()).hexdigest(),package_lock_sha256=hashlib.sha256((ROOT/'package-lock.json').read_bytes()).hexdigest(),contract='history.record.source.v1',reference_sha=m['reference_sha'],candidate_sha=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),node=subprocess.check_output(['fnm','exec','--using=22','node','-v'],text=True).strip(),manifest_sha256=hashlib.sha256((folder/'manifest.json').read_bytes()).hexdigest(),source_sha256=hashlib.sha256(subprocess.check_output(['git','show',m['reference_sha']+':src/state/historyStore.ts'],cwd=ROOT)).hexdigest(),cases=rows)
 out.parent.mkdir(parents=True,exist_ok=True);raw=(json.dumps(report,indent=2)+'\n').encode();out.write_bytes(gzip.compress(raw,mtime=0) if out.suffix=='.gz' else raw);print(len(rows),'source states',out)
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--out',type=Path,default=ROOT/'rust/target/history-record-source.json');capture(p.parse_args().out)
