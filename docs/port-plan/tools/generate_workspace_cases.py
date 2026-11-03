# SPDX-License-Identifier: AGPL-3.0-only
"""Independent raw strings and original core/default/probe-plugin contexts."""
from pathlib import Path

def generate():
    strings=['elements','measure','deviation','thickness','flat','reverse','mesh-tools','','ELEMENTS','Measure','elements\0','measure\n','unknown','__proto__','💩','\ud800','\udc00\ud800','addon💩\ud800']
    rows=[]
    for mask in range(17):
        for value in strings:
            raw=value.encode('utf-16-le','surrogatepass');units=''.join(f'{int.from_bytes(raw[i:i+2],"little"):04x}'for i in range(0,len(raw),2))
            rows.append(f'case-{len(rows):04}\t{mask}\t{units}\n')
    return ''.join(rows).encode()

if __name__=='__main__':
    folder=Path(__file__).resolve().parents[1]/'fixtures/workspaces';folder.mkdir(exist_ok=True);(folder/'cases.tsv').write_bytes(generate())
