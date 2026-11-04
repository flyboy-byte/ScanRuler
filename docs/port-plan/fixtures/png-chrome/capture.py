#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
"""Regenerate the committed PNG case batches and capture Chrome's results for them.

  python3 docs/port-plan/fixtures/png-chrome/capture.py

Runs the original's decodeImage in actual Chrome through docs/port-plan/oracle/image-decode.mjs
(Node 22 via fnm). Writes <set>.bin.gz (cases) and <set>.chrome.bin.gz (Chrome's results), gzip
with mtime 0, and manifest.json with hashes and tool versions. No Rust output participates.
"""
import gzip, hashlib, json, os, platform, subprocess, sys, zlib

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, '../../../..'))
sys.path.insert(0, HERE)
import generate  # noqa: E402

SETS = [
    ('named', lambda: list(generate.named().values()), {}),
    ('sweep-s1', lambda: generate.sweep(1, 300), {'seed': 1, 'flips_per_base': 300}),
    ('struct-s11', lambda: generate.structural(11, 400), {'seed': 11, 'images': 400}),
]


def batch(cases):
    out = bytearray(len(cases).to_bytes(4, 'little'))
    for c in cases:
        out += len(c).to_bytes(4, 'little') + c
    return bytes(out)


def main():
    manifest = {'contract': 'png.chrome.cases.v1', 'reference_sha': '2cd89c6769a10c27ee7754b477ada521c9ef5cb7',
                'python': platform.python_version(), 'zlib': zlib.ZLIB_RUNTIME_VERSION, 'sets': []}
    names = list(generate.named())
    with open(os.path.join(HERE, 'named.names.json'), 'w') as f:
        json.dump(names, f, indent=0)
        f.write('\n')
    for name, make, params in SETS:
        cases = batch(make())
        run = subprocess.run(['fnm', 'exec', '--using=22', '--', 'node', 'docs/port-plan/oracle/image-decode.mjs'],
                             input=cases, capture_output=True, cwd=ROOT, check=True)
        manifest['browser'] = run.stderr.decode().strip()
        for suffix, data in [('.bin.gz', cases), ('.chrome.bin.gz', run.stdout)]:
            with open(os.path.join(HERE, name + suffix), 'wb') as f:
                f.write(gzip.compress(data, mtime=0))
        manifest['sets'].append({'name': name, **params, 'cases': int.from_bytes(cases[:4], 'little'),
                                 'cases_sha256': hashlib.sha256(cases).hexdigest(),
                                 'chrome_sha256': hashlib.sha256(run.stdout).hexdigest()})
        print(name, manifest['sets'][-1]['cases'], 'cases captured')
    with open(os.path.join(HERE, 'manifest.json'), 'w') as f:
        json.dump(manifest, f, indent=2)
        f.write('\n')


if __name__ == '__main__':
    main()
