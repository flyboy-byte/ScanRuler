#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
"""Authored input-only source contract; never derives a fixture from Rust outputs."""
import hashlib
import json
from pathlib import Path
import random
import struct

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT / 'docs/port-plan/fixtures/obj-files'
OUT.mkdir(exist_ok=True)
ROWS = []

def word(n):
    return struct.pack('<I', n)

def blob(b):
    return word(len(b)) + b

def text(s):
    return blob(s.encode('utf-8'))

def add(identity, packet, mode):
    (OUT / (identity + '.bin')).write_bytes(packet)
    ROWS.append(dict(id=identity, mode=mode, bytes=len(packet), sha256=hashlib.sha256(packet).hexdigest()))

def scenario(identity, files, operations):
    packet = b'\1' + word(len(files))
    for name, data in files:
        packet += text(name) + blob(data)
    packet += word(len(operations))
    for op, ids in operations:
        packet += bytes([op]) + word(len(ids)) + b''.join(word(i) for i in ids)
    add(identity, packet, 1)

NAMES = ['', 'stl', 'STL', 'obj', 'thing.obj', '.obj', 'a.b.OBJ', 'x.obj.',
         'x.obj ', ' x.obj', 'folder.obj/file', 'a/b.OBJ', 'a\\b.OBJ', 'model.STEP',
         'model.stp', 'x.ply', 'x.3MF', 'x.MTL', 'x.PNG', 'x.jpg', 'x.JPEG',
         'x.BMP', 'x.WebP', 'x.gif', 'x.svg', 'x.tiff', 'x.bin', 'OBJ\n',
         '模型.OBJ', 'a.ΟΣ', 'x.İ', 'x.K', 'x.ſtl', 'x.ＰＮＧ', 'x.obj\0',
         'e\u0301.PNG', 'é.PNG', 'a.\ufeffobj', 'no extension', 'a..', 'a..stl']
for i, name in enumerate(NAMES):
    add('format-' + str(i), b'\0' + text(name), 0)

F = [('scan.obj', b'v 0 0 0\n'), ('scan.mtl', b'newmtl a\n'), ('skin.png', bytes(range(256))),
     ('notes.txt', b'ignore'), ('other.STL', b'other'), ('scan.obj', b'different identity'),
     ('skin.png', b'different pixels'), ('extra.WEBP', b'raw\0\xff')]
scenarios = [
 ('empty', [], [(1, []), (3, [])]),
 ('fallback', F, [(1, [3, 1, 2]), (2, [3]), (3, [3, 1, 2])]),
 ('model-after-texture', F, [(1, [1, 2, 0]), (2, [0]), (3, [1, 2])]),
 ('first-model', F, [(1, [4, 0, 1, 2]), (2, [4]), (2, [0])]),
 ('same-name-distinct-model', F, [(1, [5, 0, 1, 2]), (2, [0]), (2, [5])]),
 ('same-name-distinct-texture', F, [(1, [0, 2, 6]), (3, [2, 6])]),
 ('duplicate-identities', F, [(1, [0, 0, 2, 2, 1, 1]), (2, [0]), (3, [2, 2, 1, 1])]),
 ('retains-old-association', F, [(1, [0, 1, 2]), (1, [0]), (1, []), (2, [0])]),
 ('replace-association', F, [(1, [0, 1, 2]), (1, [0, 7]), (2, [0]), (3, [7])]),
 ('independent-model-associations', F, [(1, [0, 1, 2]), (1, [4, 6, 7]), (2, [0]), (2, [4])]),
 ('ignored-files', F, [(1, [0, 3, 4, 5]), (2, [0])]),
 ('raw-all-bytes', F, [(3, [2]), (3, [2, 2]), (3, []), (2, [2])]),
 ('texture-without-model', F, [(1, [1, 2, 7]), (2, [1])]),
 ('name-not-normalized', [('A.OBJ ', b'1'), (' A.OBJ', b'2'), ('路径/X.PNG', b'3')], [(1, [0, 2, 1]), (3, [0, 2])]),
]
for identity, files, ops in scenarios:
    scenario(identity, files, ops)
for seed in [1, 7, 99, 1039]:
    rng = random.Random(seed)
    for run in range(8):
        files = [(rng.choice(NAMES), bytes(rng.randrange(256) for _ in range(rng.randrange(64)))) for _ in range(12)]
        ops = []
        for _ in range(12):
            op = rng.randrange(1, 4)
            ids = [rng.randrange(len(files)) for _ in range(1 if op == 2 else rng.randrange(20))]
            ops.append((op, ids))
        scenario(f'seed-{seed}-{run}', files, ops)
manifest = dict(contract='io.obj.files.input.v1', reference_sha='2cd89c6769a10c27ee7754b477ada521c9ef5cb7',
                license='AGPL-3.0-only', seeds=[1, 7, 99, 1039], cases=ROWS)
(OUT / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
# Compiled schedule includes raw authored inputs only, not expected outputs.
lines = ['// SPDX-License-Identifier: AGPL-3.0-only', 'vec![']
for row in ROWS:
    lines.append(f'    Case {{ id: "{row["id"]}", input: include_bytes!("../../../../../docs/port-plan/fixtures/obj-files/{row["id"]}.bin") }},')
lines.append(']')
(ROOT/'rust/crates/port-io/tests/support/obj_files_cases.rs').write_text('\n'.join(lines)+'\n')
print(len(ROWS), 'fixed cases')
