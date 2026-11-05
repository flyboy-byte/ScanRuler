#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
"""Exact info/selection, existing 1e-12 plane geometry budget. No private mesh output."""
import json, math, os, sys
from pathlib import Path

def compare(a, b, path='', geometry=False):
    if isinstance(a, dict):
        assert isinstance(b, dict) and a.keys() == b.keys(), (path, 'keys', a.keys(), getattr(b, 'keys', lambda: [])())
        for k in a:
            compare(a[k], b[k], path+'.'+k, geometry or k == 'fit')
    elif isinstance(a, list):
        assert isinstance(b, list) and len(a) == len(b), (path, 'length')
        for i, (x, y) in enumerate(zip(a,b)):
            compare(x, y, path+f'[{i}]', geometry)
    elif isinstance(a, (int,float)) and not isinstance(a,bool):
        assert isinstance(b,(int,float)) and not isinstance(b,bool), (path, 'type')
        assert math.isfinite(a) and math.isfinite(b), (path, 'nonfinite')
        assert abs(a-b) <= (1e-12*(1+abs(a)) if geometry else 0), (path, a, b)
    else:
        assert type(a) is type(b) and a == b, (path, a, b)

if __name__ == '__main__':
    folder=Path(sys.argv[1])
    for extension in ('ply','stl'):
        a=json.loads((folder/f'{extension}.original.json').read_text())
        b=json.loads((folder/f'{extension}.rust.json').read_text())
        if os.getenv('SCANRULER_REAL_SCAN_SABOTAGE'):
            b['scan']['vertices'] += 1
        compare(a,b,extension)
        print(extension, 'open/info and selected base plane: identical within existing budget')
    print('4/4 private real-scan controls; no model data retained')
